use serde_json::Value;
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    sync::atomic::{AtomicUsize, Ordering},
};

static NEXT: AtomicUsize = AtomicUsize::new(0);
struct Project(PathBuf);
impl Project {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "forgeproof-install-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
}
impl Drop for Project {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
fn fixtures() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../.local/fixtures")
        .canonicalize()
        .unwrap()
}
fn command(args: &[&str], project: &Path) -> (i32, Value) {
    let out = Command::new(env!("CARGO_BIN_EXE_engctl"))
        .args(args)
        .arg("--project")
        .arg(project)
        .arg("--json")
        .output()
        .unwrap();
    assert!(
        out.stderr.is_empty(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    (
        out.status.code().unwrap(),
        serde_json::from_slice(&out.stdout).unwrap(),
    )
}
fn install(project: &Path, bundle: &str, revocations: &str, target: &str) -> (i32, Value) {
    let root = fixtures();
    command(
        &[
            "install",
            "--bundle",
            root.join(bundle).to_str().unwrap(),
            "--trust",
            root.join("trust.json").to_str().unwrap(),
            "--revocations",
            root.join(revocations).to_str().unwrap(),
            "--target",
            target,
        ],
        project,
    )
}
fn success(result: (i32, Value), status: &str) {
    assert_eq!(result.0, 0, "{}", result.1);
    assert_eq!(result.1["status"], status);
}
fn error(result: (i32, Value), code: &str) {
    assert_eq!(result.0, 1, "{}", result.1);
    assert_eq!(result.1["errors"][0]["code"], code);
}
const CODEX: &str = "codex@0.158.0";
const CLAUDE: &str = "claude-code@2.1.283";

#[test]
fn installs_repeats_updates_and_preserves_other_client() {
    use std::os::unix::fs::MetadataExt;
    let p = Project::new();
    success(install(&p.0, "valid", "current.json", CODEX), "installed");
    let agents = p.0.join("AGENTS.md");
    let before = fs::read(&agents).unwrap();
    let state = fs::read(p.0.join(".forgeproof/state.json")).unwrap();
    let inode = fs::metadata(&agents).unwrap().ino();
    let text = String::from_utf8(before.clone()).unwrap();
    assert!(text.contains("Target: codex@0.158.0\n"));
    assert!(text.ends_with("# Test instructions\nRespect the project lockfiles.\n\n"));
    success(install(&p.0, "valid", "current.json", CODEX), "unchanged");
    assert_eq!(fs::read(&agents).unwrap(), before);
    assert_eq!(fs::metadata(&agents).unwrap().ino(), inode);
    assert_eq!(fs::read(p.0.join(".forgeproof/state.json")).unwrap(), state);
    success(install(&p.0, "valid", "current.json", CLAUDE), "installed");
    let claude = fs::read(p.0.join("CLAUDE.md")).unwrap();
    assert!(
        String::from_utf8(claude.clone())
            .unwrap()
            .contains("Target: claude-code@2.1.283\n")
    );
    success(install(&p.0, "updated", "current.json", CODEX), "installed");
    assert!(
        fs::read_to_string(&agents)
            .unwrap()
            .contains("Run the pinned tests.")
    );
    assert!(
        fs::read_to_string(&agents)
            .unwrap()
            .contains("Release: rust-nix@0.2.0")
    );
    assert_eq!(fs::read(p.0.join("CLAUDE.md")).unwrap(), claude);
    success(command(&["installation-status"], &p.0), "ready");
    success(command(&["recover"], &p.0), "unchanged");
}
#[test]
fn rejects_unauthorized_inputs_without_creating_project_state() {
    let p = Project::new();
    for (bundle, revocations, code) in [
        ("tampered", "current.json", "hash_mismatch"),
        ("untrusted", "current.json", "untrusted_signer"),
        ("valid", "revoked.json", "release_revoked"),
        ("valid", "expired.json", "revocations_expired"),
    ] {
        error(install(&p.0, bundle, revocations, CODEX), code);
        assert_eq!(fs::read_dir(&p.0).unwrap().count(), 0);
    }
    let root = fixtures();
    error(
        command(
            &[
                "install",
                "--bundle",
                root.join("valid").to_str().unwrap(),
                "--trust",
                root.join("trust.json").to_str().unwrap(),
                "--target",
                CODEX,
            ],
            &p.0,
        ),
        "revocations_missing",
    );
    assert_eq!(fs::read_dir(&p.0).unwrap().count(), 0);
}
#[test]
fn preserves_unmanaged_and_locally_modified_files() {
    let p = Project::new();
    let agents = p.0.join("AGENTS.md");
    fs::write(&agents, "My instructions").unwrap();
    error(
        install(&p.0, "valid", "current.json", CODEX),
        "unmanaged_file",
    );
    assert_eq!(fs::read_to_string(&agents).unwrap(), "My instructions");
    fs::remove_file(&agents).unwrap();
    success(install(&p.0, "valid", "current.json", CODEX), "installed");
    fs::write(&agents, "Edited by owner").unwrap();
    error(
        install(&p.0, "updated", "current.json", CODEX),
        "local_modification",
    );
    error(
        command(&["installation-status"], &p.0),
        "local_modification",
    );
    assert_eq!(fs::read_to_string(&agents).unwrap(), "Edited by owner");
    fs::remove_file(&agents).unwrap();
    error(
        install(&p.0, "updated", "current.json", CODEX),
        "local_modification",
    );
}
#[test]
fn rejects_incompatible_profiles_roles_and_text() {
    let p = Project::new();
    error(
        install(&p.0, "valid", "current.json", "codex"),
        "unsupported_adapter",
    );
    error(
        install(&p.0, "unsupported-role", "current.json", CODEX),
        "unsupported_role",
    );
    error(
        install(&p.0, "non-utf8", "current.json", CODEX),
        "invalid_instruction_text",
    );
    error(
        install(&p.0, "nul-text", "current.json", CODEX),
        "invalid_instruction_text",
    );
    error(
        install(&p.0, "over-output-limit", "current.json", CODEX),
        "instructions_too_large",
    );
    assert_eq!(fs::read_dir(&p.0).unwrap().count(), 0);
    success(
        install(&p.0, "at-output-limit", "current.json", CODEX),
        "installed",
    );
    assert_eq!(fs::metadata(p.0.join("AGENTS.md")).unwrap().len(), 32768);
}
#[test]
fn rejects_redirected_paths_and_codex_override() {
    use std::os::unix::fs::symlink;
    let p = Project::new();
    let other = Project::new();
    fs::write(other.0.join("precious"), "preserve me").unwrap();
    symlink(other.0.join("precious"), p.0.join("AGENTS.md")).unwrap();
    error(
        install(&p.0, "valid", "current.json", CODEX),
        "unsafe_install_path",
    );
    assert_eq!(
        fs::read_to_string(other.0.join("precious")).unwrap(),
        "preserve me"
    );
    fs::remove_file(p.0.join("AGENTS.md")).unwrap();
    symlink(&other.0, p.0.join(".forgeproof")).unwrap_or_else(|_| {
        fs::remove_dir_all(p.0.join(".forgeproof")).unwrap();
        symlink(&other.0, p.0.join(".forgeproof")).unwrap();
    });
    error(
        install(&p.0, "valid", "current.json", CODEX),
        "unsafe_install_path",
    );
    fs::remove_file(p.0.join(".forgeproof")).unwrap();
    fs::write(p.0.join("AGENTS.override.md"), "override").unwrap();
    error(
        install(&p.0, "valid", "current.json", CODEX),
        "shadowed_instructions",
    );
    assert!(!p.0.join("AGENTS.md").exists());
}
#[test]
fn strict_distribution_arguments_and_absent_installation() {
    let p = Project::new();
    error(command(&["installation-status"], &p.0), "not_installed");
    error(
        command(&["install", "--wat", "x"], &p.0),
        "invalid_arguments",
    );
    error(
        command(&["recover", "--project", "duplicate"], &p.0),
        "invalid_arguments",
    );
    error(
        command(&["installation-status", "--target", CODEX], &p.0),
        "invalid_arguments",
    );
    assert_eq!(fs::read_dir(&p.0).unwrap().count(), 0);
}

#[test]
fn blocks_competing_operations_without_changing_outputs() {
    let p = Project::new();
    success(install(&p.0, "valid", "current.json", CODEX), "installed");
    let before = fs::read(p.0.join("AGENTS.md")).unwrap();
    let lock = fs::OpenOptions::new().read(true).write(true).open(p.0.join(".forgeproof/install.lock")).unwrap();
    lock.lock().unwrap();
    for result in [install(&p.0, "updated", "current.json", CODEX), command(&["installation-status"], &p.0), command(&["recover"], &p.0)] {
        error(result, "installation_busy");
    }
    assert_eq!(fs::read(p.0.join("AGENTS.md")).unwrap(), before);
}
#[test]
fn rejects_corrupt_state_and_pending_journals() {
    let p = Project::new();
    success(install(&p.0, "valid", "current.json", CODEX), "installed");
    let before = fs::read(p.0.join("AGENTS.md")).unwrap();
    let state = p.0.join(".forgeproof/state.json");
    let original = fs::read(&state).unwrap();
    fs::write(&state, b"{}").unwrap();
    error(command(&["installation-status"], &p.0), "invalid_installation_state");
    error(install(&p.0, "updated", "current.json", CODEX), "invalid_installation_state");
    fs::write(&state, &original).unwrap();
    let journal = p.0.join(".forgeproof/pending.json");
    for content in [b"{".to_vec(), b"{}".to_vec(), vec![b' '; 524289]] {
        fs::write(&journal, &content).unwrap();
        error(command(&["installation-status"], &p.0), "installation_incomplete");
        error(install(&p.0, "updated", "current.json", CODEX), "installation_incomplete");
        error(command(&["recover"], &p.0), "invalid_installation_journal");
        assert_eq!(fs::read(p.0.join("AGENTS.md")).unwrap(), before);
        assert_eq!(fs::read(&state).unwrap(), original);
    }
    // Parseable JSON with an inconsistent state/output relationship is also invalid.
    let forged = serde_json::json!({"schema_version":1,"file":"AGENTS.md","before":String::from_utf8(before.clone()).unwrap(),"after":"unrelated text","state_before":String::from_utf8(original.clone()).unwrap(),"state_after":String::from_utf8(original.clone()).unwrap()});
    fs::write(&journal, serde_json::to_vec(&forged).unwrap()).unwrap();
    error(command(&["recover"], &p.0), "invalid_installation_journal");
    assert_eq!(fs::read(p.0.join("AGENTS.md")).unwrap(), before);
}
#[test]
fn rejects_links_in_control_surfaces_and_unrecognized_control_files() {
    use std::os::unix::fs::symlink;
    for name in ["state.json", "pending.json", "write.tmp", "install.lock"] {
        let p = Project::new();
        let other = Project::new();
        success(install(&p.0, "valid", "current.json", CODEX), "installed");
        let protected = other.0.join("precious");
        fs::write(&protected, "preserved").unwrap();
        let control = p.0.join(".forgeproof").join(name);
        if control.exists() { fs::remove_file(&control).unwrap(); }
        symlink(&protected, &control).unwrap();
        error(command(&["recover"], &p.0), "unsafe_install_path");
        assert_eq!(fs::read_to_string(&protected).unwrap(), "preserved");
    }
    let p = Project::new();
    success(install(&p.0, "valid", "current.json", CODEX), "installed");
    fs::hard_link(p.0.join("AGENTS.md"), p.0.join("copy")).unwrap();
    error(install(&p.0, "updated", "current.json", CODEX), "unsafe_install_path");
    fs::remove_file(p.0.join("copy")).unwrap();
    fs::write(p.0.join(".forgeproof/unknown"), "owned by user").unwrap();
    error(install(&p.0, "updated", "current.json", CODEX), "invalid_installation_state");
    assert_eq!(fs::read_to_string(p.0.join(".forgeproof/unknown")).unwrap(), "owned by user");
}
