# Local Distribution Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Execution and integration already authorized by the user.

**Goal:** Install deterministic, verified native instructions and recover interrupted project updates without overwriting user changes.

**Architecture:** CLI-local Rust modules reuse the catalog verifier. A bounded journal and OS file lock coordinate native output and installation state; recovery rolls back.

**Tech Stack:** Rust 1.98.0, existing serde/serde_json/sha2 versions, std filesystem APIs, independent Minisign fixtures, GitHub Actions and NixOS VM.

**Spec:** `docs/design/local-distribution.md`, under delivery 2 of `docs/design/architecture.md`.

## Global Constraints

- Linux x86-64 first; frozen registry dependencies and no unsafe code.
- Exact profiles `codex@0.158.0` and `claude-code@2.1.283`; instructions/policy only, unsupported capabilities fail explicitly.
- Output <=32768 bytes, state <=16384 bytes, journal <=524288 bytes. UTF-8, strict schemas and fixed destinations.
- Never overwrite unmanaged or edited files. No network, scripts, client launch or permission expansion.
- Fresh authorization for every install. Installation status is consistency evidence only.
- All distribution operations share the project lock; pending journal blocks readiness and new installs.

## Review Focus

- Interrupted rollback and locally edited pending files: recovery must be repeatable and preserve edits (Task 2 crash harness).
- Invalid yet parseable journal: fixed paths and before/after relationships must be validated (Task 2 tests).
- Two adapters in one project: updating one must preserve the other's output and state (Task 1 CLI test).
- Symlink/hardlink control surfaces and stale temporary files: no redirection or accidental overwrite (Task 2 tests).
- Native file size/format and Codex override: reject incompatibility before altering managed files (Task 1 tests).

### Task 1: Verified deterministic native installation

**Files:** `apps/engctl/src/distribution/{mod,render,state,transaction}.rs`, `apps/engctl/src/distribution_cli.rs`, `apps/engctl/src/main.rs`, `apps/engctl/tests/install_cli.rs`, `scripts/generate-fixtures.py`, `apps/engctl/Cargo.toml`, `Cargo.lock`.

**Interfaces:**
- Consumes `verify_integrity(&Path,&TrustStore)`, `authorize(&VerifiedBundle,&str,Option<&RevocationSnapshot>,SystemTime)` and `read_bounded`.
- Produces `distribution::install(project,bundle,trust,revocations,target) -> Result<&'static str,ValidationError>`, `inspect(project)` and `recover(project)` with the same result. All paths are borrowed `&Path`; revocations is `Option<&Path>`; target is `&str`.
- Render produces a private `Rendered { file: NativeFile, text: String, release_id: String, manifest_sha256: String, target: String }`.

- [ ] Write process tests. Assert literal native filenames and source text, stable bytes/inode on repeat, changed version on update, both clients retained, tampered/revoked/expired/untrusted no output, unmanaged and locally changed files preserved, unsupported target/role rejected, exact size cap and UTF-8/NUL checks. Extend signed test generator with versioned targets and updated/unsupported/non-UTF8/oversize variants.
- [ ] Run CI `cargo test -p engctl --test install_cli --locked`; expected RED because install is absent (`invalid_arguments` instead of installed). Existing verification tests remain green.
- [ ] Implement exact adapters, bounded typed state, verified authorization and serialized transaction. Add only workspace serde/sha2 dependencies; lock update must contain no registry version/hash changes.
- [ ] Run `cargo test --workspace --locked`, `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets --locked -- -D warnings`; expected GREEN. Commit the slice with observed evidence.

### Task 2: Recovery and integration evidence

**Files:** Task 1 transaction module and tests; `scripts/check-installer-recovery.py`, `.github/workflows/verify.yml`, `nix/tests/verifier.nix`, `docs/usage/install.md`, `docs/evidence/{progress,traceability}.md`, README.

**Interfaces:** Consumes Task 1 CLI; produces recoverable journal, readiness check and acceptance documentation. No job runner yet; future managed execution must consume readiness and then reauthorize.

- [ ] Add recovery tests for malformed/oversized journals, symlink/hardlink control files, lock contention, user edits and invalid state relationships. Add a disposable-build interruption harness that injects exits after journal, native output, state and rollback writes; assert pending status, denied new installs, preserved edits, exact restoration and successful retry.
- [ ] Observe failing cases before adding missing recovery behavior. A test that already passes characterizes the Task 1 transaction, and is recorded as such rather than falsely labeled RED.
- [ ] Complete rollback validation and durable updates. Run real process acceptance in VM: install/repeat/update/both clients/drift rejection, plus normal verifier scenarios. Run the interruption harness in CI and retain its log.
- [ ] Run frozen full CI, inspect logs, map requirements to evidence, and obtain one fresh whole-branch review. Fix blocking findings with regression tests, integrate and delete the completed branch under the user's existing authorization.
