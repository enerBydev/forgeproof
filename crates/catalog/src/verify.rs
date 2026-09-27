use eng_contracts::{Manifest, ValidationError, MAX_MANIFEST_BYTES, parse_manifest, valid_id};
use minisign_verify::{PublicKey, Signature};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::{collections::BTreeSet, fs, io::Read, path::{Component, Path}};
pub type VerificationError = ValidationError;

#[derive(Debug)]
pub struct TrustStore { keys: Vec<(String, PublicKey)> }

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct TrustDocument { schema_version: u32, keys: Vec<TrustKey> }
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct TrustKey { id: String, public_key: String }

impl TrustStore {
    pub fn parse(bytes: &[u8]) -> Result<Self, VerificationError> {
        let invalid = || VerificationError::new("invalid_trust",None);
        if bytes.len() > 65536 { return Err(invalid()); }
        let doc: TrustDocument = serde_json::from_slice(bytes).map_err(|_| invalid())?;
        if doc.schema_version != 1 || doc.keys.is_empty() || doc.keys.len() > 128 { return Err(invalid()); }
        let mut ids = BTreeSet::new();
        let mut keys: Vec<(String,PublicKey)> = Vec::new();
        for item in doc.keys {
            if !valid_id(&item.id) || !ids.insert(item.id.clone()) { return Err(invalid()); }
            let key = PublicKey::from_base64(&item.public_key).map_err(|_| invalid())?;
            if keys.iter().any(|(_,k)| *k == key) { return Err(invalid()); }
            keys.push((item.id,key));
        }
        Ok(Self { keys })
    }
}

#[derive(Debug)]
pub struct VerifiedBundle {
    pub(crate) manifest: Manifest,
    pub(crate) manifest_sha256: String,
    pub(crate) signer_id: String,
}
impl VerifiedBundle {
    pub fn manifest(&self) -> &Manifest { &self.manifest }
    pub fn manifest_sha256(&self) -> &str { &self.manifest_sha256 }
    pub fn signer_id(&self) -> &str { &self.signer_id }
}

/// The caller must isolate the staging directory from concurrent writers.
/// This verifies current bytes, never grants a reusable execution capability.
pub fn verify_integrity(root: &Path, trust: &TrustStore) -> Result<VerifiedBundle, VerificationError> {
    check_components(root)?;
    if !root.is_dir() { return Err(VerificationError::new("invalid_bundle",None)); }
    let bytes = read_bounded(&root.join("manifest.json"),MAX_MANIFEST_BYTES as u64)?;
    let sig_bytes = read_bounded(&root.join("manifest.json.minisig"),16384)?;
    let sig_text = std::str::from_utf8(&sig_bytes).map_err(|_| VerificationError::new("invalid_signature",None))?;
    let signature = Signature::decode(sig_text).map_err(|_| VerificationError::new("invalid_signature",None))?;
    let mut signer = None;
    let mut matched_key = false;
    for (id,key) in &trust.keys {
        match key.verify(&bytes,&signature,false) {
            Ok(()) => { signer = Some(id.clone()); break; },
            Err(minisign_verify::Error::UnexpectedKeyId) => {},
            Err(_) => { matched_key = true; }
        }
    }
    let signer_id = signer.ok_or_else(|| VerificationError::new(if matched_key {"invalid_signature"} else {"untrusted_signer"},None))?;
    let manifest = parse_manifest(&bytes)?;
    let expected: BTreeSet<_> = manifest.files.iter().map(|f| f.path.as_str()).collect();
    let mut directories = BTreeSet::new();
    for path in &expected { for (i,_) in path.match_indices('/') { directories.insert(&path[..i]); } }
    let mut seen = BTreeSet::new();
    enumerate(root,root,&expected,&directories,&mut seen)?;
    for item in &manifest.files {
        if !seen.contains(item.path.as_str()) { return Err(VerificationError::new("missing_file",Some(&item.path))); }
        let payload = read_bounded(&root.join(&item.path),item.size_bytes)?;
        if payload.len() as u64 != item.size_bytes { return Err(VerificationError::new("size_mismatch",Some(&item.path))); }
        if sha256(&payload) != item.sha256 { return Err(VerificationError::new("hash_mismatch",Some(&item.path))); }
    }
    Ok(VerifiedBundle { manifest, manifest_sha256: sha256(&bytes), signer_id })
}

fn sha256(bytes: &[u8]) -> String {
    Sha256::digest(bytes).iter().map(|byte| format!("{byte:02x}")).collect()
}

fn enumerate(root: &Path, dir: &Path, expected: &BTreeSet<&str>, directories: &BTreeSet<&str>, seen: &mut BTreeSet<String>) -> Result<(),VerificationError> {
    for entry in fs::read_dir(dir).map_err(|_| VerificationError::new("io_error",None))? {
        let entry = entry.map_err(|_| VerificationError::new("io_error",None))?;
        let path = entry.path();
        let relative = path.strip_prefix(root).ok().and_then(Path::to_str).ok_or_else(|| VerificationError::new("invalid_path",None))?;
        let kind = entry.file_type().map_err(|_| VerificationError::new("io_error",Some(relative)))?;
        if kind.is_symlink() { return Err(VerificationError::new("symlink",Some(relative))); }
        if kind.is_dir() {
            if !directories.contains(relative) { return Err(VerificationError::new("extra_directory",Some(relative))); }
            enumerate(root,&path,expected,directories,seen)?;
        } else if kind.is_file() {
            if ["manifest.json","manifest.json.minisig"].contains(&relative) { continue; }
            if !expected.contains(relative) { return Err(VerificationError::new("extra_file",Some(relative))); }
            seen.insert(relative.to_owned());
        } else { return Err(VerificationError::new("unsupported_file_type",Some(relative))); }
    }
    Ok(())
}

fn check_components(path: &Path) -> Result<(),VerificationError> {
    let mut prefix = std::path::PathBuf::new();
    for component in path.components() {
        if component == Component::ParentDir { return Err(VerificationError::new("invalid_path",None)); }
        prefix.push(component);
        let metadata = fs::symlink_metadata(&prefix).map_err(|_| VerificationError::new("io_error",None))?;
        if metadata.file_type().is_symlink() { return Err(VerificationError::new("symlink",None)); }
    }
    Ok(())
}

/// Bounded regular-file reader shared by the CLI's trusted local inputs.
pub fn read_bounded(path: &Path, limit: u64) -> Result<Vec<u8>,VerificationError> {
    check_components(path)?;
    let metadata=fs::symlink_metadata(path).map_err(|_| VerificationError::new("io_error",None))?;
    if !metadata.is_file() { return Err(VerificationError::new("unsupported_file_type",None)); }
    if metadata.len()>limit { return Err(VerificationError::new("size_mismatch",None)); }
    let mut bytes=Vec::new();
    fs::File::open(path).map_err(|_| VerificationError::new("io_error",None))?.take(limit.saturating_add(1)).read_to_end(&mut bytes).map_err(|_| VerificationError::new("io_error",None))?;
    if bytes.len() as u64>limit { return Err(VerificationError::new("size_mismatch",None)); }
    Ok(bytes)
}
