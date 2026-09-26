use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

pub const MAX_MANIFEST_BYTES: usize = 1024 * 1024;
pub const MAX_FILES: usize = 1024;
pub const MAX_FILE_BYTES: u64 = 16 * 1024 * 1024;
pub const MAX_BUNDLE_BYTES: u64 = 256 * 1024 * 1024;

pub fn valid_id(value: &str) -> bool {
    !value.is_empty() && value.len() <= 128
        && value.bytes().all(|b| b.is_ascii_alphanumeric() || b"._-@".contains(&b))
}

pub fn valid_hash(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

pub fn valid_path(path: &str) -> bool {
    !path.is_empty() && path.len() <= 4096 && !path.contains(['\\', ':', '\0'])
        && !path.chars().any(char::is_control)
        && path.split('/').count() <= 32
        && path.split('/').all(|p| !p.is_empty() && p != "." && p != ".." && p.len() <= 255)
        && !["manifest.json", "manifest.json.minisig"].contains(&path)
}

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

pub fn parse_manifest(bytes: &[u8]) -> Result<Manifest, ValidationError> {
    if bytes.len() > MAX_MANIFEST_BYTES {
        return Err(ValidationError::new("manifest_too_large", None));
    }
    let manifest: Manifest = serde_json::from_slice(bytes).map_err(|_| ValidationError::new("invalid_manifest", None))?;
    if manifest.schema_version != 1 { return Err(ValidationError::new("unsupported_schema", None)); }
    if !valid_id(&manifest.release_id) { return Err(ValidationError::new("invalid_release_id", None)); }
    let targets: BTreeSet<_> = manifest.supported_targets.iter().collect();
    if targets.is_empty() || targets.len() > 64 || targets.len() != manifest.supported_targets.len()
        || !targets.iter().all(|t| valid_id(t)) {
        return Err(ValidationError::new("invalid_targets", None));
    }
    if manifest.files.is_empty() || manifest.files.len() > MAX_FILES { return Err(ValidationError::new("file_count", None)); }
    let mut paths = BTreeSet::new();
    let mut total = 0u64;
    for file in &manifest.files {
        let err = |code| ValidationError::new(code, Some(&file.path));
        if !valid_path(&file.path) { return Err(err("invalid_path")); }
        if !paths.insert(file.path.as_str()) { return Err(err("duplicate_path")); }
        if !valid_hash(&file.sha256) { return Err(err("invalid_hash")); }
        if file.size_bytes > MAX_FILE_BYTES { return Err(err("file_too_large")); }
        total += file.size_bytes;
        if total > MAX_BUNDLE_BYTES { return Err(err("bundle_too_large")); }
    }
    for path in &paths {
        for (i,_) in path.match_indices('/') {
            if paths.contains(&path[..i]) || ["manifest.json","manifest.json.minisig"].contains(&&path[..i]) {
                return Err(ValidationError::new("path_collision", Some(path)));
            }
        }
    }
    Ok(manifest)
}
