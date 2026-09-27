use crate::{VerifiedBundle,VerificationError};
use eng_contracts::{VerificationReceipt,IntegrityStatus,AuthorizationStatus};
use serde::Deserialize;
use std::time::SystemTime;

#[derive(Debug,Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RevocationSnapshot {
    pub schema_version: u32,
    pub generated_at: u64,
    pub expires_at: u64,
    pub revoked_release_ids: Vec<String>,
    pub revoked_manifest_sha256: Vec<String>,
    pub revoked_signer_ids: Vec<String>,
}
impl RevocationSnapshot {
    pub fn parse(bytes: &[u8]) -> Result<Self,VerificationError> {
        serde_json::from_slice(bytes).map_err(|_| VerificationError::new("invalid_revocations",None))
    }
}
pub fn authorize(bundle: &VerifiedBundle,target: &str,_revocations: Option<&RevocationSnapshot>,_now: SystemTime) -> VerificationReceipt {
    VerificationReceipt {schema_version:1,release_id:Some(bundle.manifest().release_id.clone()),manifest_sha256:Some(bundle.manifest_sha256().into()),signer_id:Some(bundle.signer_id().into()),target:target.into(),integrity:IntegrityStatus::Verified,authorization:AuthorizationStatus::Unknown,errors:vec![VerificationError::new("not_implemented",None)]}
}
