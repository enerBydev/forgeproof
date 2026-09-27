use eng_catalog::{TrustStore, verify_integrity};
use std::{fs, path::PathBuf};

fn fixtures() -> PathBuf { PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../.local/fixtures").canonicalize().unwrap() }
fn trust() -> TrustStore { TrustStore::parse(&fs::read(fixtures().join("trust.json")).unwrap()).unwrap() }

#[test]
fn accepts_trusted_signature() {
    let bundle = verify_integrity(&fixtures().join("valid"), &trust()).unwrap();
    assert_eq!(bundle.signer_id(), "test-publisher");
    assert_eq!(bundle.manifest().release_id,"rust-nix@0.1.0");
}

#[test]
fn rejects_valid_untrusted_signer() { reject("untrusted", "untrusted_signer"); }
#[test]
fn rejects_tampered_file() { reject("tampered", "hash_mismatch"); }
#[test]
fn rejects_extra_file() { reject("extra", "extra_file"); }
#[test]
fn rejects_symlink_component() { reject("symlink", "symlink"); }
#[test]
fn rejects_size_mismatch() { reject("size-mismatch", "size_mismatch"); }
#[test]
fn rejects_missing_file() { reject("missing", "missing_file"); }
#[test]
fn rejects_corrupt_signature() { reject("bad-signature", "invalid_signature"); }
#[test]
fn signature_covers_original_manifest_bytes() { reject("changed-manifest", "invalid_signature"); }
#[test]
fn rejects_extra_directory() { reject("extra-directory", "extra_directory"); }

fn reject(name: &str, code: &str) {
    assert_eq!(verify_integrity(&fixtures().join(name),&trust()).unwrap_err().code,code);
}

#[test]
fn rejects_ambiguous_trust() {
    let data = fs::read(fixtures().join("trust.json")).unwrap();
    let mut value: serde_json::Value=serde_json::from_slice(&data).unwrap();
    let key = value["keys"][0].clone();
    value["keys"].as_array_mut().unwrap().push(key);
    assert_eq!(TrustStore::parse(&serde_json::to_vec(&value).unwrap()).unwrap_err().code,"invalid_trust");
}

#[test]
fn detects_changed_content_on_reverification() {
    let unique = format!("forgeproof-revalidation-{}-{}", std::process::id(),std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos());
    let root=std::env::temp_dir().join(unique);
    fs::create_dir_all(root.join("instructions")).unwrap();
    for name in ["manifest.json","manifest.json.minisig","instructions/rust.md"] {
        fs::copy(fixtures().join("valid").join(name),root.join(name)).unwrap();
    }
    assert!(verify_integrity(&root,&trust()).is_ok());
    let path=root.join("instructions/rust.md"); let mut bytes=fs::read(&path).unwrap(); bytes[0] ^= 1; fs::write(path,bytes).unwrap();
    let result=verify_integrity(&root,&trust());
    fs::remove_dir_all(root).unwrap();
    assert_eq!(result.unwrap_err().code,"hash_mismatch");
}
