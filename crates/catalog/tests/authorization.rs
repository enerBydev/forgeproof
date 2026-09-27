use eng_catalog::{TrustStore,RevocationSnapshot,VerifiedBundle,verify_integrity,authorize};
use eng_contracts::AuthorizationStatus;
use std::{fs,path::PathBuf,time::{UNIX_EPOCH,Duration}};
use serde_json::json;

fn bundle() -> VerifiedBundle {
    let root=PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../.local/fixtures").canonicalize().unwrap();
    let trust=TrustStore::parse(&fs::read(root.join("trust.json")).unwrap()).unwrap();
    verify_integrity(&root.join("valid"),&trust).unwrap()
}
fn snapshot() -> serde_json::Value { json!({"schema_version":1,"generated_at":100,"expires_at":200,"revoked_release_ids":[],"revoked_manifest_sha256":[],"revoked_signer_ids":[]}) }
fn parse(v: &serde_json::Value) -> RevocationSnapshot { RevocationSnapshot::parse(&serde_json::to_vec(v).unwrap()).unwrap() }

#[test]
fn approves_only_with_current_revocations() {
    let r=authorize(&bundle(),"codex",Some(&parse(&snapshot())),UNIX_EPOCH+Duration::from_secs(150));
    assert_eq!(r.authorization,AuthorizationStatus::Approved); assert_eq!(r.exit_code(),0);
}
#[test]
fn rejects_revoked_release() {
    for field in ["revoked_release_ids","revoked_manifest_sha256","revoked_signer_ids"] {
        let b=bundle(); let mut v=snapshot();
        v[field]=json!([match field {"revoked_release_ids"=>b.manifest().release_id.as_str(),"revoked_manifest_sha256"=>b.manifest_sha256(),_=>b.signer_id()}]);
        let r=authorize(&b,"codex",Some(&parse(&v)),UNIX_EPOCH+Duration::from_secs(150));
        assert_eq!(r.authorization,AuthorizationStatus::Revoked); assert_eq!(r.exit_code(),1);
    }
}
#[test]
fn marks_expired_revocations_unknown() {
    for time in [99,200,201] {
        let r=authorize(&bundle(),"codex",Some(&parse(&snapshot())),UNIX_EPOCH+Duration::from_secs(time));
        assert_eq!(r.authorization,AuthorizationStatus::Unknown); assert_eq!(r.exit_code(),2);
        assert_ne!(r.errors[0].code,"not_implemented");
    }
}
#[test]
fn missing_revocations_never_authorize() {
    let r=authorize(&bundle(),"codex",None,UNIX_EPOCH+Duration::from_secs(150));
    assert_eq!(r.authorization,AuthorizationStatus::Unknown); assert_eq!(r.errors[0].code,"revocations_missing");
}
#[test]
fn rejects_unsupported_target() {
    let r=authorize(&bundle(),"unsupported",Some(&parse(&snapshot())),UNIX_EPOCH+Duration::from_secs(150));
    assert_eq!(r.authorization,AuthorizationStatus::Denied);assert_eq!(r.errors[0].code,"unsupported_target");
}
#[test]
fn rejects_duplicate_revocations_and_invalid_validity() {
    let mut v=snapshot();v["revoked_release_ids"]=json!(["x","x"]);
    assert!(RevocationSnapshot::parse(&serde_json::to_vec(&v).unwrap()).is_err());
    let mut v=snapshot();v["expires_at"]=json!(100);
    assert!(RevocationSnapshot::parse(&serde_json::to_vec(&v).unwrap()).is_err());
}
