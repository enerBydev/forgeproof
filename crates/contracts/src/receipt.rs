use crate::ValidationError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IntegrityStatus {
    Verified,
    Failed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuthorizationStatus {
    Approved,
    Denied,
    Revoked,
    Unknown,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VerificationReceipt {
    pub schema_version: u32,
    pub release_id: Option<String>,
    pub manifest_sha256: Option<String>,
    pub signer_id: Option<String>,
    pub target: String,
    pub integrity: IntegrityStatus,
    pub authorization: AuthorizationStatus,
    pub errors: Vec<ValidationError>,
}

impl VerificationReceipt {
    pub fn exit_code(&self) -> u8 {
        match (self.integrity, self.authorization) {
            (IntegrityStatus::Verified, AuthorizationStatus::Approved)
                if self.errors.is_empty() =>
            {
                0
            }
            (IntegrityStatus::Verified, AuthorizationStatus::Unknown) => 2,
            _ => 1,
        }
    }
}
