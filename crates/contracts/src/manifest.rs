use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    pub schema_version: u32,
    pub release_id: String,
    pub supported_targets: Vec<String>,
    pub files: Vec<FileEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FileEntry {
    pub path: String,
    pub sha256: String,
    pub size_bytes: u64,
    pub role: FileRole,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FileRole {
    Instructions,
    Skill,
    Policy,
    SourceSpec,
    TestDefinition,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ValidationError {
    pub code: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
}

impl ValidationError {
    pub fn new(code: &str, path: Option<&str>) -> Self {
        Self { code: code.into(), path: path.map(String::from) }
    }
}

// Deliberately incomplete in the first test commit: CI must demonstrate rejection gaps.
pub fn parse_manifest(bytes: &[u8]) -> Result<Manifest, ValidationError> {
    serde_json::from_slice(bytes).map_err(|_| ValidationError::new("invalid_manifest", None))
}
