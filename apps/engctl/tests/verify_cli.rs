use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    process::Command,
};

#[derive(Debug, PartialEq, Eq)]
enum InputEntry {
    Directory,
    File(Vec<u8>),
    Symlink(PathBuf),
}

fn input_tree(root: &Path) -> BTreeMap<PathBuf, InputEntry> {
    fn visit(root: &Path, dir: &Path, result: &mut BTreeMap<PathBuf, InputEntry>) {
        for entry in fs::read_dir(dir).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            let kind = entry.file_type().unwrap();
            let value = if kind.is_symlink() {
                InputEntry::Symlink(fs::read_link(&path).unwrap())
            } else if kind.is_dir() {
                visit(root, &path, result);
                InputEntry::Directory
            } else {
                assert!(kind.is_file());
                InputEntry::File(fs::read(&path).unwrap())
            };
            result.insert(path.strip_prefix(root).unwrap().to_owned(), value);
        }
    }
    let mut result = BTreeMap::new();
    visit(root, root, &mut result);
    result
}
fn fixtures() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../.local/fixtures")
        .canonicalize()
        .unwrap()
}
fn run(bundle: &str, target: &str, revocations: Option<&str>) -> (i32, Value) {
    let root = fixtures();
    let before = input_tree(&root);
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_engctl"));
    cmd.args(["verify", "--bundle"])
        .arg(root.join(bundle))
        .arg("--trust")
        .arg(root.join("trust.json"))
        .args(["--target", target, "--json"]);
    if let Some(file) = revocations {
        cmd.arg("--revocations").arg(root.join(file));
    }
    let output = cmd.output().unwrap();
    assert!(
        output.stderr.is_empty(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        before,
        input_tree(&root),
        "verification modified its input tree"
    );
    (
        output.status.code().unwrap(),
        serde_json::from_slice(&output.stdout).unwrap(),
    )
}
#[test]
fn approved_bundle_exits_zero() {
    let (code, r) = run("valid", "codex", Some("current.json"));
    assert_eq!(code, 0);
    assert_eq!(r["authorization"], "approved");
    assert_eq!(r["release_id"], "rust-nix@0.1.0");
}
#[test]
fn invalid_bundles_exit_one() {
    for (bundle, error) in [
        ("tampered", "hash_mismatch"),
        ("untrusted", "untrusted_signer"),
        ("bad-signature", "invalid_signature"),
    ] {
        let (code, r) = run(bundle, "codex", Some("current.json"));
        assert_eq!(code, 1);
        assert_eq!(r["errors"][0]["code"], error);
    }
}
#[test]
fn revoked_release_exits_one() {
    let (code, r) = run("valid", "codex", Some("revoked.json"));
    assert_eq!(code, 1);
    assert_eq!(r["authorization"], "revoked");
}
#[test]
fn unsupported_target_exits_one() {
    let (code, r) = run("valid", "other", Some("current.json"));
    assert_eq!(code, 1);
    assert_eq!(r["errors"][0]["code"], "unsupported_target");
}
#[test]
fn expired_or_missing_revocations_exit_two() {
    for rev in [None, Some("expired.json")] {
        let (code, r) = run("valid", "codex", rev);
        assert_eq!(code, 2);
        assert_eq!(r["authorization"], "unknown");
        assert_eq!(r["integrity"], "verified");
    }
}
#[test]
fn invalid_arguments_are_json_errors() {
    for args in [
        vec!["verify", "--json"],
        vec!["verify", "--unknown", "x", "--json"],
    ] {
        let out = Command::new(env!("CARGO_BIN_EXE_engctl"))
            .args(args)
            .output()
            .unwrap();
        assert_eq!(out.status.code(), Some(1));
        let r: Value = serde_json::from_slice(&out.stdout).unwrap();
        assert_eq!(r["errors"][0]["code"], json!("invalid_arguments"));
    }
}
