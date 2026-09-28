use super::{Error, fail};
use eng_catalog::{VerifiedBundle, read_bounded};
use eng_contracts::FileRole;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::Path;

pub const OUTPUT_LIMIT: usize = 32768;
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum NativeFile {
    #[serde(rename = "AGENTS.md")]
    Codex,
    #[serde(rename = "CLAUDE.md")]
    Claude,
}
impl NativeFile {
    pub fn name(self) -> &'static str {
        match self {
            Self::Codex => "AGENTS.md",
            Self::Claude => "CLAUDE.md",
        }
    }
    pub fn target(self) -> &'static str {
        match self {
            Self::Codex => "codex@0.158.0",
            Self::Claude => "claude-code@2.1.283",
        }
    }
    pub fn for_target(target: &str) -> Result<Self, Error> {
        match target {
            "codex@0.158.0" => Ok(Self::Codex),
            "claude-code@2.1.283" => Ok(Self::Claude),
            _ => Err(fail("unsupported_adapter")),
        }
    }
}
pub fn digest(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}
pub struct Rendered {
    pub file: NativeFile,
    pub text: String,
    pub release_id: String,
    pub manifest_sha256: String,
    pub target: String,
}
pub fn render(root: &Path, bundle: &VerifiedBundle, target: &str) -> Result<Rendered, Error> {
    let file = NativeFile::for_target(target)?;
    let manifest = bundle.manifest();
    let mut text = format!(
        "# Forgeproof managed instructions\n\nRelease: {}\nManifest: {}\nTarget: {target}\n\n",
        manifest.release_id,
        bundle.manifest_sha256()
    );
    let mut entries: Vec<_> = manifest.files.iter().collect();
    entries.sort_by_key(|f| &f.path);
    for entry in entries {
        if !matches!(entry.role, FileRole::Instructions | FileRole::Policy) {
            return Err(Error::new("unsupported_role", Some(&entry.path)));
        }
        if entry.size_bytes > OUTPUT_LIMIT as u64 {
            return Err(fail("instructions_too_large"));
        }
        let bytes = read_bounded(&root.join(&entry.path), entry.size_bytes)?;
        if bytes.len() as u64 != entry.size_bytes || digest(&bytes) != entry.sha256 {
            return Err(fail("source_changed"));
        }
        let content = std::str::from_utf8(&bytes).map_err(|_| fail("invalid_instruction_text"))?;
        if content.contains('\0') {
            return Err(fail("invalid_instruction_text"));
        }
        text.push_str(&format!("## Source: {}\n\n", entry.path));
        text.push_str(content);
        text.push('\n');
        if text.len() > OUTPUT_LIMIT {
            return Err(fail("instructions_too_large"));
        }
    }
    Ok(Rendered {
        file,
        text,
        release_id: manifest.release_id.clone(),
        manifest_sha256: bundle.manifest_sha256().into(),
        target: target.into(),
    })
}
