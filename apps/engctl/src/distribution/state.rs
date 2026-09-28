use super::{
    Error, fail,
    render::{NativeFile, Rendered, digest},
};
use eng_contracts::{valid_hash, valid_id};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub const STATE_LIMIT: u64 = 16384;
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Installed {
    pub target: String,
    pub release_id: String,
    pub manifest_sha256: String,
    pub content_sha256: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct State {
    pub schema_version: u32,
    pub files: BTreeMap<NativeFile, Installed>,
}
impl State {
    pub fn empty() -> Self {
        Self {
            schema_version: 1,
            files: BTreeMap::new(),
        }
    }
    pub fn parse(text: &str) -> Result<Self, Error> {
        let invalid = || fail("invalid_installation_state");
        if text.len() > STATE_LIMIT as usize {
            return Err(invalid());
        }
        let state: Self = serde_json::from_str(text).map_err(|_| invalid())?;
        if state.schema_version != 1 || state.files.is_empty() || state.files.len() > 2 {
            return Err(invalid());
        }
        for (file, entry) in &state.files {
            if entry.target != file.target()
                || !valid_id(&entry.release_id)
                || !valid_hash(&entry.manifest_sha256)
                || !valid_hash(&entry.content_sha256)
            {
                return Err(invalid());
            }
        }
        Ok(state)
    }
    pub fn insert(&mut self, rendered: &Rendered) {
        self.files.insert(
            rendered.file,
            Installed {
                target: rendered.target.clone(),
                release_id: rendered.release_id.clone(),
                manifest_sha256: rendered.manifest_sha256.clone(),
                content_sha256: digest(rendered.text.as_bytes()),
            },
        );
    }
    pub fn json(&self) -> Result<String, Error> {
        let text = serde_json::to_string(self).map_err(|_| fail("invalid_installation_state"))?;
        Self::parse(&text)?;
        Ok(text)
    }
}
