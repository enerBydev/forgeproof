use crate::distribution;
use eng_contracts::ValidationError;
use serde_json::json;
use std::{collections::BTreeMap, io::Write, path::Path, process::ExitCode};

fn execute(args: &[String]) -> Result<&'static str, ValidationError> {
    let invalid = || ValidationError::new("invalid_arguments", None);
    let operation = args.first().ok_or_else(invalid)?;
    let allowed = if operation == "install" {
        vec![
            "--project",
            "--bundle",
            "--trust",
            "--revocations",
            "--target",
        ]
    } else {
        vec!["--project"]
    };
    let mut flags = BTreeMap::new();
    let mut json = false;
    let mut rest = args.iter().skip(1);
    while let Some(flag) = rest.next() {
        if flag == "--json" {
            if json {
                return Err(invalid());
            }
            json = true;
            continue;
        }
        if !allowed.contains(&flag.as_str()) {
            return Err(invalid());
        }
        let value = rest
            .next()
            .filter(|s| !s.is_empty() && !s.starts_with("--"))
            .ok_or_else(invalid)?;
        if flags.insert(flag.as_str(), value.as_str()).is_some() {
            return Err(invalid());
        }
    }
    let project = Path::new(flags.get("--project").ok_or_else(invalid)?);
    match operation.as_str() {
        "install" => distribution::install(
            project,
            Path::new(flags.get("--bundle").ok_or_else(invalid)?),
            Path::new(flags.get("--trust").ok_or_else(invalid)?),
            flags.get("--revocations").map(Path::new),
            flags.get("--target").ok_or_else(invalid)?,
        ),
        "installation-status" => distribution::inspect(project),
        "recover" => distribution::recover(project),
        _ => Err(invalid()),
    }
}
pub fn run(args: &[String]) -> ExitCode {
    let (code, value) = match execute(args) {
        Ok(status) => (
            0,
            json!({"schema_version":1,"operation":args[0],"status":status,"errors":[]}),
        ),
        Err(error) => (
            1,
            json!({"schema_version":1,"operation":args[0],"status":"error","errors":[error]}),
        ),
    };
    let Ok(mut bytes) = serde_json::to_vec(&value) else {
        return ExitCode::FAILURE;
    };
    bytes.push(b'\n');
    if std::io::stdout().lock().write_all(&bytes).is_err() {
        return ExitCode::FAILURE;
    }
    ExitCode::from(code)
}
