use crate::{VerificationError, VerifiedBundle};
use eng_contracts::{
    AuthorizationStatus, IntegrityStatus, VerificationReceipt, valid_hash, valid_id,
};
use serde::Deserialize;
use std::{
    collections::BTreeSet,
    time::{SystemTime, UNIX_EPOCH},
};

#[derive(Debug)]
pub struct RevocationSnapshot {
    document: RevocationDocument,
}
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RevocationDocument {
    schema_version: u32,
    generated_at: u64,
    expires_at: u64,
    revoked_release_ids: Vec<String>,
    revoked_manifest_sha256: Vec<String>,
    revoked_signer_ids: Vec<String>,
}
impl RevocationSnapshot {
    pub fn parse(bytes: &[u8]) -> Result<Self, VerificationError> {
        let invalid = || VerificationError::new("invalid_revocations", None);
        if bytes.len() > 1024 * 1024 {
            return Err(invalid());
        }
        let document: RevocationDocument = serde_json::from_slice(bytes).map_err(|_| invalid())?;
        if document.schema_version != 1 || document.generated_at >= document.expires_at {
            return Err(invalid());
        }
        let lists = [
            &document.revoked_release_ids,
            &document.revoked_manifest_sha256,
            &document.revoked_signer_ids,
        ];
        if lists.iter().map(|v| v.len()).sum::<usize>() > 10000 {
            return Err(invalid());
        }
        for (i, list) in lists.iter().enumerate() {
            let unique: BTreeSet<_> = list.iter().collect();
            if unique.len() != list.len()
                || !list
                    .iter()
                    .all(|s| if i == 1 { valid_hash(s) } else { valid_id(s) })
            {
                return Err(invalid());
            }
        }
        Ok(Self { document })
    }
}
pub fn authorize(
    bundle: &VerifiedBundle,
    target: &str,
    revocations: Option<&RevocationSnapshot>,
    now: SystemTime,
) -> VerificationReceipt {
    let mut receipt = VerificationReceipt {
        schema_version: 1,
        release_id: Some(bundle.manifest().release_id.clone()),
        manifest_sha256: Some(bundle.manifest_sha256().into()),
        signer_id: Some(bundle.signer_id().into()),
        target: target.into(),
        integrity: IntegrityStatus::Verified,
        authorization: AuthorizationStatus::Unknown,
        errors: vec![],
    };
    let decision = if !bundle
        .manifest()
        .supported_targets
        .iter()
        .any(|t| t == target)
    {
        (AuthorizationStatus::Denied, Some("unsupported_target"))
    } else if let Some(snapshot) = revocations {
        let d = &snapshot.document;
        if d.revoked_release_ids
            .contains(&bundle.manifest().release_id)
            || d.revoked_manifest_sha256
                .iter()
                .any(|v| v == bundle.manifest_sha256())
            || d.revoked_signer_ids.iter().any(|v| v == bundle.signer_id())
        {
            (AuthorizationStatus::Revoked, Some("release_revoked"))
        } else if let Ok(time) = now.duration_since(UNIX_EPOCH) {
            if time.as_secs() < d.generated_at {
                (
                    AuthorizationStatus::Unknown,
                    Some("revocations_not_yet_valid"),
                )
            } else if time.as_secs() >= d.expires_at {
                (AuthorizationStatus::Unknown, Some("revocations_expired"))
            } else {
                (AuthorizationStatus::Approved, None)
            }
        } else {
            (AuthorizationStatus::Unknown, Some("clock_invalid"))
        }
    } else {
        (AuthorizationStatus::Unknown, Some("revocations_missing"))
    };
    receipt.authorization = decision.0;
    if let Some(code) = decision.1 {
        receipt.errors.push(VerificationError::new(code, None));
    }
    receipt
}
