use eng_contracts::{Manifest, ValidationError};
use std::path::Path;
pub type VerificationError = ValidationError;

#[derive(Debug)]
pub struct TrustStore;
impl TrustStore {
    pub fn parse(_bytes: &[u8]) -> Result<Self, VerificationError> { Ok(Self) }
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

pub fn verify_integrity(_root: &Path, _trust: &TrustStore) -> Result<VerifiedBundle, VerificationError> {
    Err(VerificationError::new("not_implemented",None))
}
