#!/usr/bin/env python3
"""Prove the added acceptance checks reject deliberate faults in a disposable copy."""
import os
from pathlib import Path
import shutil
import subprocess
import tempfile


def run_test(root, env, package, suite, test, failure=None):
    command = ["cargo", "test", "--locked", "-p", package, "--test", suite, test, "--", "--exact"]
    result = subprocess.run(command, cwd=root, env=env, text=True, capture_output=True)
    output = result.stdout + result.stderr
    if failure is None:
        assert result.returncode == 0, output[-8000:]
        assert "1 passed" in output, output[-8000:]
    else:
        assert result.returncode != 0, f"Mutation survived: {failure}"
        assert "1 failed" in output and failure in output, output[-8000:]


def main():
    source = Path(__file__).resolve().parents[1]
    env = dict(os.environ, CARGO_TARGET_DIR=str(source / "target/mutation-checks"))
    with tempfile.TemporaryDirectory(prefix="forgeproof-mutations-") as temp:
        root = Path(temp) / "repo"
        shutil.copytree(source, root, ignore=shutil.ignore_patterns(
            ".git", ".local", "target", "node_modules", "dist", ".superpowers", ".vercel", "__pycache__"))
        subprocess.run(["python3", "scripts/generate-fixtures.py"], cwd=root, check=True)
        boundary = ("eng-contracts", "manifest", "accepts_manifest_at_exact_byte_limit")
        readonly = ("engctl", "verify_cli", "approved_bundle_exits_zero")
        run_test(root, env, *boundary)
        run_test(root, env, *readonly)
        parser = root / "crates/contracts/src/manifest.rs"
        original = parser.read_text()
        needle = "bytes.len() > MAX_MANIFEST_BYTES"
        assert original.count(needle) == 1
        parser.write_text(original.replace(needle, "bytes.len() >= MAX_MANIFEST_BYTES"))
        run_test(root, env, *boundary, failure="parse_manifest(&bytes).is_ok()")
        print("DETECTED: rejection of a valid manifest at exactly 1 MiB", flush=True)
        parser.write_text(original)
        cli = root / "apps/engctl/src/main.rs"
        original_cli = cli.read_text()
        needle = "ExitCode::from(code)"
        assert original_cli.count(needle) == 1
        # Writes happen after receipt creation, so only the input-preservation assertion detects them.
        for target in ["valid/manifest.json", "valid/manifest.json.minisig", "valid/instructions/rust.md", "trust.json", "current.json"]:
            path = root / ".local/fixtures" / target
            before = path.read_bytes()
            append = f'std::fs::OpenOptions::new().append(true).open(concat!(env!("CARGO_MANIFEST_DIR"), "/../../.local/fixtures/{target}")).unwrap().write_all(b" ").unwrap();\n    '
            cli.write_text(original_cli.replace(needle, append + needle))
            run_test(root, env, *readonly, failure="verification modified its input tree")
            path.write_bytes(before)
            print(f"DETECTED: verifier modified {target}", flush=True)
        cli.write_text(original_cli)
        run_test(root, env, *boundary)
        run_test(root, env, *readonly)
        print("Mutation evidence: all 6 injected faults detected; restored checks pass.", flush=True)


if __name__ == "__main__":
    main()
