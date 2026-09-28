mod filesystem;
mod render;
mod state;
mod transaction;
use eng_catalog::{RevocationSnapshot, TrustStore, authorize, read_bounded, verify_integrity};
use eng_contracts::{AuthorizationStatus, ValidationError};
use filesystem::LockedProject;
use std::{path::Path, time::SystemTime};
type Error = ValidationError;
fn fail(code: &str) -> Error {
    Error::new(code, None)
}

pub fn install(
    project: &Path,
    bundle: &Path,
    trust: &Path,
    revocations: Option<&Path>,
    target: &str,
) -> Result<&'static str, Error> {
    render::NativeFile::for_target(target)?;
    let trust = TrustStore::parse(&read_bounded(trust, 65536)?)?;
    let verified = verify_integrity(bundle, &trust)?;
    let revocations = revocations
        .map(|p| read_bounded(p, 1024 * 1024).and_then(|b| RevocationSnapshot::parse(&b)))
        .transpose()?;
    let receipt = authorize(&verified, target, revocations.as_ref(), SystemTime::now());
    if receipt.authorization != AuthorizationStatus::Approved {
        return Err(receipt
            .errors
            .into_iter()
            .next()
            .unwrap_or_else(|| fail("installation_not_authorized")));
    }
    let rendered = render::render(bundle, &verified, target)?;
    let locked =
        LockedProject::open(project, true)?.ok_or_else(|| fail("installation_io_error"))?;
    transaction::apply(&locked, rendered)
}
pub fn inspect(project: &Path) -> Result<&'static str, Error> {
    let locked = LockedProject::open(project, false)?.ok_or_else(|| fail("not_installed"))?;
    transaction::ensure_complete(&locked)?;
    let (_, state) = transaction::read_state(&locked)?;
    if state.files.is_empty() {
        return Err(fail("not_installed"));
    }
    transaction::check_files(&locked, &state)?;
    Ok("ready")
}
pub fn recover(project: &Path) -> Result<&'static str, Error> {
    let Some(locked) = LockedProject::open(project, false)? else {
        return Ok("unchanged");
    };
    transaction::rollback(&locked)
}
