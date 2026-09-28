use eng_contracts::parse_manifest;
use serde_json::{Value, json};

fn valid() -> Value {
    json!({"schema_version":1,"release_id":"rust-nix@0.1.0", "supported_targets":["codex"],"files":[{"path":"instructions/rust.md","sha256":"e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855","size_bytes":0,"role":"instructions"}]})
}
fn check(value: &Value) -> Result<eng_contracts::Manifest, eng_contracts::ValidationError> {
    parse_manifest(&serde_json::to_vec(value).unwrap())
}
fn reject(value: &Value, code: &str) {
    assert_eq!(check(value).unwrap_err().code, code);
}

#[test]
fn accepts_valid_manifest() {
    assert!(check(&valid()).is_ok());
}

#[test]
fn rejects_duplicate_paths() {
    let mut v = valid();
    let entry = v["files"][0].clone();
    v["files"].as_array_mut().unwrap().push(entry);
    reject(&v, "duplicate_path");
}

#[test]
fn rejects_escape_paths() {
    for path in [
        "../a",
        "/a",
        "a/../b",
        "a/./b",
        "a//b",
        "a\\b",
        "a\0b",
        "",
        "C:/a",
        "manifest.json",
        "manifest.json.minisig",
    ] {
        let mut v = valid();
        v["files"][0]["path"] = json!(path);
        reject(&v, "invalid_path");
    }
}

#[test]
fn rejects_unknown_schema() {
    let mut v = valid();
    v["schema_version"] = json!(2);
    reject(&v, "unsupported_schema");
}

#[test]
fn enforces_size_limits() {
    let mut v = valid();
    v["files"][0]["size_bytes"] = json!(16 * 1024 * 1024);
    assert!(check(&v).is_ok());
    v["files"][0]["size_bytes"] = json!(16 * 1024 * 1024 + 1);
    reject(&v, "file_too_large");
    let entry = valid()["files"][0].clone();
    v["files"] = json!(
        (0..1024)
            .map(|i| {
                let mut e = entry.clone();
                e["path"] = json!(format!("f{i}"));
                e
            })
            .collect::<Vec<_>>()
    );
    assert!(check(&v).is_ok());
    v["files"].as_array_mut().unwrap().push(entry.clone());
    reject(&v, "file_count");
    v["files"] = json!([]);
    reject(&v, "file_count");
    v["files"] = json!(
        (0..16)
            .map(|i| {
                let mut e = entry.clone();
                e["path"] = json!(format!("f{i}"));
                e["size_bytes"] = json!(16 * 1024 * 1024);
                e
            })
            .collect::<Vec<_>>()
    );
    assert!(check(&v).is_ok());
    v["files"]
        .as_array_mut()
        .unwrap()
        .push(json!({"size_bytes":1,"path":"overflow", "sha256":entry["sha256"],"role":"policy"}));
    reject(&v, "bundle_too_large");
    assert_eq!(
        parse_manifest(&vec![b' '; 1024 * 1024 + 1])
            .unwrap_err()
            .code,
        "manifest_too_large"
    );
}

#[test]
fn accepts_manifest_at_exact_byte_limit() {
    let mut bytes = serde_json::to_vec(&valid()).unwrap();
    bytes.resize(1024 * 1024, b' ');
    assert!(parse_manifest(&bytes).is_ok());
}

#[test]
fn rejects_ambiguous_metadata() {
    let mut v = valid();
    v["files"][0]["sha256"] = json!("bad");
    reject(&v, "invalid_hash");
    let mut v = valid();
    v["supported_targets"] = json!(["codex", "codex"]);
    reject(&v, "invalid_targets");
    let mut v = valid();
    v["supported_targets"] = json!([]);
    reject(&v, "invalid_targets");
    let mut v = valid();
    v["release_id"] = json!("");
    reject(&v, "invalid_release_id");
    let mut v = valid();
    v["unexpected"] = json!(true);
    reject(&v, "invalid_manifest");
    let duplicate = String::from_utf8(serde_json::to_vec(&valid()).unwrap())
        .unwrap()
        .replacen("{", "{\"schema_version\":1,", 1);
    assert_eq!(
        parse_manifest(duplicate.as_bytes()).unwrap_err().code,
        "invalid_manifest"
    );
}

#[test]
fn rejects_file_directory_collision() {
    let mut v = valid();
    let mut entry = v["files"][0].clone();
    entry["path"] = json!("instructions");
    v["files"].as_array_mut().unwrap().push(entry);
    reject(&v, "path_collision");
}
