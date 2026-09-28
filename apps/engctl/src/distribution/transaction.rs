use super::{Error, fail, filesystem::{LockedProject, read_optional, regular}, render::{NativeFile, OUTPUT_LIMIT, Rendered, digest}, state::{STATE_LIMIT, State}};
use serde::{Deserialize, Serialize};

pub const JOURNAL_LIMIT: u64 = 524288;
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Journal {
    schema_version: u32,
    file: NativeFile,
    before: Option<String>,
    after: String,
    state_before: Option<String>,
    state_after: String,
}
pub fn read_state(project: &LockedProject) -> Result<(Option<String>, State), Error> {
    let text = read_optional(&project.control.join("state.json"), STATE_LIMIT, "invalid_installation_state")?;
    let state = match &text { Some(t) => State::parse(t)?, None => State::empty() };
    Ok((text, state))
}
pub fn ensure_complete(project: &LockedProject) -> Result<(), Error> {
    if regular(&project.control.join("pending.json"))? { return Err(fail("installation_incomplete")); }
    Ok(())
}
pub fn check_files(project: &LockedProject, state: &State) -> Result<(), Error> {
    for (file, entry) in &state.files {
        let text = read_optional(&project.project.join(file.name()), OUTPUT_LIMIT as u64, "local_modification")?;
        if text.as_ref().map(|t| digest(t.as_bytes())).as_ref() != Some(&entry.content_sha256) { return Err(fail("local_modification")); }
        if *file == NativeFile::Codex && project.project.join("AGENTS.override.md").symlink_metadata().is_ok() { return Err(fail("shadowed_instructions")); }
    }
    Ok(())
}
pub fn apply(project: &LockedProject, rendered: Rendered) -> Result<&'static str, Error> {
    ensure_complete(project)?;
    let (state_before, mut state) = read_state(project)?;
    check_files(project, &state)?;
    let destination = project.project.join(rendered.file.name());
    let before = read_optional(&destination, OUTPUT_LIMIT as u64, "local_modification")?;
    if !state.files.contains_key(&rendered.file) && before.is_some() { return Err(fail("unmanaged_file")); }
    if rendered.file == NativeFile::Codex && project.project.join("AGENTS.override.md").symlink_metadata().is_ok() { return Err(fail("shadowed_instructions")); }
    let previous = state.clone();
    state.insert(&rendered);
    if state == previous && before.as_deref() == Some(&rendered.text) { return Ok("unchanged"); }
    let journal = Journal { schema_version: 1, file: rendered.file, before, after: rendered.text, state_before, state_after: state.json()? };
    let encoded = serde_json::to_string(&journal).map_err(|_| fail("invalid_installation_journal"))?;
    if encoded.len() > JOURNAL_LIMIT as usize { return Err(fail("invalid_installation_journal")); }
    project.replace(&project.control.join("pending.json"), Some(&encoded))?;
    // Durable checkpoint: journal.
    project.replace(&destination, Some(&journal.after))?;
    // Durable checkpoint: native.
    project.replace(&project.control.join("state.json"), Some(&journal.state_after))?;
    // Durable checkpoint: state.
    project.replace(&project.control.join("pending.json"), None)?;
    Ok("installed")
}
pub fn rollback(project: &LockedProject) -> Result<&'static str, Error> {
    if regular(&project.control.join("pending.json"))? { return Err(fail("recovery_not_implemented")); }
    Ok("unchanged")
}
