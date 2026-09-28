#!/usr/bin/env python3
"""Exercise interrupted installs with real binaries in disposable copies.

Crash injection exists only in the disposable source; shipped code has no
fault-injection environment variable or termination hooks.
"""
import json
import os
import pathlib
import shutil
import subprocess
import tempfile

ROOT = pathlib.Path(__file__).resolve().parents[1]
TARGET = "codex@0.158.0"


def run(binary, project, fixtures, operation, bundle="valid", crash=None):
    args = [str(binary), operation, "--project", str(project), "--json"]
    if operation == "install":
        args += ["--bundle", str(fixtures / bundle), "--trust", str(fixtures / "trust.json"), "--revocations", str(fixtures / "current.json"), "--target", TARGET]
    env = os.environ.copy()
    env.pop("FORGEPROOF_TEST_CRASH", None)
    if crash:
        env["FORGEPROOF_TEST_CRASH"] = crash
    result = subprocess.run(args, capture_output=True, text=True, env=env, timeout=20)
    if result.returncode == 77:
        return 77, None
    assert not result.stderr, result.stderr
    return result.returncode, json.loads(result.stdout)


def ok(result, status):
    assert result[0] == 0 and result[1]["status"] == status, result


def error(result, code):
    assert result[0] == 1 and result[1]["errors"][0]["code"] == code, result


def snapshot(project):
    return {name: (project / name).read_bytes() if (project / name).exists() else None for name in ("AGENTS.md", "CLAUDE.md", ".forgeproof/state.json")}


def build(repo, destination, env):
    result = subprocess.run(["cargo", "build", "--locked", "-p", "engctl"], cwd=repo, env=env, capture_output=True, text=True)
    assert result.returncode == 0, result.stdout + result.stderr
    shutil.copy2(pathlib.Path(env["CARGO_TARGET_DIR"]) / "debug/engctl", destination)


def main():
    with tempfile.TemporaryDirectory(prefix="forgeproof-recovery-") as temporary:
        root = pathlib.Path(temporary)
        repo = root / "repo"
        shutil.copytree(ROOT, repo, ignore=shutil.ignore_patterns(".git", ".local", "target", "node_modules", "dist", ".superpowers", ".vercel", "__pycache__"))
        fixtures = root / "fixtures"
        subprocess.run(["python3", str(repo / "scripts/generate-fixtures.py"), str(fixtures)], check=True)
        env = dict(os.environ, CARGO_TARGET_DIR=str(ROOT / "target/recovery-checks"))
        original, interrupted = root / "original", root / "interrupted"
        build(repo, original, env)
        source = repo / "apps/engctl/src/distribution/transaction.rs"
        code = source.read_text()
        for point in ("journal", "native", "state"):
            marker = f"// Durable checkpoint: {point}."
            assert code.count(marker) == 1, marker
            code = code.replace(marker, marker + '\n    if std::env::var("FORGEPROOF_TEST_CRASH").as_deref() == Ok("' + point + '") { std::process::exit(77); }')
        source.write_text(code)
        build(repo, interrupted, env)
        for mode in ("initial", "update"):
            for point in ("journal", "native", "state"):
                project = root / f"{mode}-{point}"
                project.mkdir()
                if mode == "update":
                    ok(run(original, project, fixtures, "install"), "installed")
                before = snapshot(project)
                candidate = "updated" if mode == "update" else "valid"
                assert run(interrupted, project, fixtures, "install", candidate, point)[0] == 77
                assert (project / ".forgeproof/pending.json").is_file()
                error(run(original, project, fixtures, "installation-status"), "installation_incomplete")
                error(run(original, project, fixtures, "install", candidate), "installation_incomplete")
                ok(run(original, project, fixtures, "recover"), "recovered")
                assert snapshot(project) == before, (mode, point)
                assert not (project / ".forgeproof/pending.json").exists()
                ok(run(original, project, fixtures, "recover"), "unchanged")
                ok(run(original, project, fixtures, "install", candidate), "installed")
                ok(run(original, project, fixtures, "installation-status"), "ready")
                print(f"RECOVERED: {mode} interrupted after {point}", flush=True)
        project = root / "edited-pending"
        project.mkdir()
        assert run(interrupted, project, fixtures, "install", crash="native")[0] == 77
        generated = (project / "AGENTS.md").read_bytes()
        (project / "AGENTS.md").write_text("User edits during interruption")
        before = snapshot(project)
        journal = (project / ".forgeproof/pending.json").read_bytes()
        error(run(original, project, fixtures, "recover"), "local_modification")
        assert snapshot(project) == before
        assert (project / ".forgeproof/pending.json").read_bytes() == journal
        (project / "AGENTS.md").write_bytes(generated)
        ok(run(original, project, fixtures, "recover"), "recovered")
        print("PRESERVED: local edits block rollback without writes", flush=True)
        print("Recovery evidence: all 6 interruption points recovered and local edits preserved.", flush=True)


if __name__ == "__main__":
    main()
