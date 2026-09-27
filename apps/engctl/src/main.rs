use eng_catalog::{
    RevocationSnapshot, TrustStore, VerifiedBundle, authorize, read_bounded, verify_integrity,
};
use eng_contracts::{
    AuthorizationStatus, IntegrityStatus, ValidationError, VerificationReceipt, valid_id,
};
use std::{collections::BTreeMap, io::Write, path::PathBuf, process::ExitCode, time::SystemTime};

struct Options {
    bundle: PathBuf,
    trust: PathBuf,
    revocations: Option<PathBuf>,
    target: String,
}
fn parse_args(args: Vec<String>) -> Result<Options, ValidationError> {
    let invalid = || ValidationError::new("invalid_arguments", None);
    if args.first().map(String::as_str) != Some("verify") {
        return Err(invalid());
    }
    let mut flags = BTreeMap::new();
    let mut json = false;
    let mut rest = args.into_iter().skip(1);
    while let Some(flag) = rest.next() {
        if flag == "--json" {
            if json {
                return Err(invalid());
            }
            json = true;
            continue;
        }
        if !["--bundle", "--trust", "--revocations", "--target"].contains(&flag.as_str()) {
            return Err(invalid());
        }
        let value = rest
            .next()
            .filter(|v| !v.is_empty() && !v.starts_with("--"))
            .ok_or_else(invalid)?;
        if flags.insert(flag, value).is_some() {
            return Err(invalid());
        }
    }
    let target = flags.remove("--target").ok_or_else(invalid)?;
    if !valid_id(&target) {
        return Err(invalid());
    }
    Ok(Options {
        bundle: flags.remove("--bundle").ok_or_else(invalid)?.into(),
        trust: flags.remove("--trust").ok_or_else(invalid)?.into(),
        revocations: flags.remove("--revocations").map(PathBuf::from),
        target,
    })
}
fn denied(
    target: &str,
    error: ValidationError,
    bundle: Option<&VerifiedBundle>,
) -> VerificationReceipt {
    VerificationReceipt {
        schema_version: 1,
        release_id: bundle.map(|b| b.manifest().release_id.clone()),
        manifest_sha256: bundle.map(|b| b.manifest_sha256().into()),
        signer_id: bundle.map(|b| b.signer_id().into()),
        target: target.into(),
        integrity: if bundle.is_some() {
            IntegrityStatus::Verified
        } else {
            IntegrityStatus::Failed
        },
        authorization: AuthorizationStatus::Denied,
        errors: vec![error],
    }
}
fn verify(options: Options) -> VerificationReceipt {
    let trust = match read_bounded(&options.trust, 65536).and_then(|b| TrustStore::parse(&b)) {
        Ok(t) => t,
        Err(e) => return denied(&options.target, e, None),
    };
    let bundle = match verify_integrity(&options.bundle, &trust) {
        Ok(b) => b,
        Err(e) => return denied(&options.target, e, None),
    };
    let revocations = match options.revocations {
        Some(path) => {
            match read_bounded(&path, 1024 * 1024).and_then(|b| RevocationSnapshot::parse(&b)) {
                Ok(r) => Some(r),
                Err(e) => return denied(&options.target, e, Some(&bundle)),
            }
        }
        None => None,
    };
    authorize(
        &bundle,
        &options.target,
        revocations.as_ref(),
        SystemTime::now(),
    )
}
fn main() -> ExitCode {
    let args: Result<Vec<_>, _> = std::env::args_os()
        .skip(1)
        .map(|s| s.into_string())
        .collect();
    if matches!(&args,Ok(v) if v==&["--help"]) {
        println!(
            "engctl verify --bundle DIR --trust FILE [--revocations FILE] --target ID [--json]\nAlways emits a JSON receipt. Exit: 0 approved; 1 denied/invalid; 2 unknown validity."
        );
        return ExitCode::SUCCESS;
    }
    let receipt = match args
        .map_err(|_| ValidationError::new("invalid_arguments", None))
        .and_then(parse_args)
    {
        Ok(options) => verify(options),
        Err(error) => denied("unspecified", error, None),
    };
    let code = receipt.exit_code();
    let Ok(mut bytes) = serde_json::to_vec(&receipt) else {
        return ExitCode::FAILURE;
    };
    bytes.push(b'\n');
    if std::io::stdout().lock().write_all(&bytes).is_err() {
        return ExitCode::FAILURE;
    }
    ExitCode::from(code)
}
