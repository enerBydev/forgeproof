#!/usr/bin/env python3
"""Produce test-only signed bundles with the independent Minisign executable.

Private keys exist only inside TemporaryDirectory and are never published.
Bundle semantics and content hashes are reproducible; random keys/signatures vary.
"""
import hashlib
import json
import pathlib
import shutil
import subprocess
import sys
import tempfile
import time


def run(*args):
    subprocess.run(args, check=True, stdout=subprocess.DEVNULL, stderr=subprocess.PIPE)


def generate(output):
    output.mkdir(parents=True, exist_ok=False)
    now = int(time.time())
    snapshot = {"schema_version": 1, "generated_at": now - 60, "expires_at": now + 3600, "revoked_release_ids": [], "revoked_manifest_sha256": [], "revoked_signer_ids": []}
    (output / "current.json").write_text(json.dumps(snapshot))
    (output / "revoked.json").write_text(json.dumps(dict(snapshot, revoked_release_ids=["rust-nix@0.1.0"])))
    (output / "expired.json").write_text(json.dumps(dict(snapshot, generated_at=now - 120, expires_at=now - 1)))
    with tempfile.TemporaryDirectory(prefix="forgeproof-test-keys-") as temp:
        keys = pathlib.Path(temp)
        for signer in ("trusted", "other"):
            run("minisign", "-G", "-W", "-p", str(keys / f"{signer}.pub"), "-s", str(keys / f"{signer}.sec"))
        pub = (keys / "trusted.pub").read_text()
        (output / "trusted.pub").write_text(pub)
        (output / "trust.json").write_text(json.dumps({"schema_version": 1, "keys": [{"id": "test-publisher", "public_key": pub.splitlines()[1]}]}))
        content = b"# Test instructions\nRespect the project lockfiles.\n"
        manifest = {"schema_version": 1, "release_id": "rust-nix@0.1.0", "supported_targets": ["codex", "claude-code"], "files": [{"path": "instructions/rust.md", "sha256": hashlib.sha256(content).hexdigest(), "size_bytes": len(content), "role": "instructions"}]}
        def bundle(name, signer="trusted", document=None):
            root = output / name
            (root / "instructions").mkdir(parents=True)
            (root / "instructions/rust.md").write_bytes(content)
            (root / "manifest.json").write_text(json.dumps(document or manifest, sort_keys=True, separators=(",", ":")))
            run("minisign", "-S", "-s", str(keys / f"{signer}.sec"), "-m", str(root / "manifest.json"), "-t", "forgeproof-test-fixture")
            return root
        valid = bundle("valid")
        run("minisign", "-V", "-p", str(keys / "trusted.pub"), "-m", str(valid / "manifest.json"))
        bundle("untrusted", "other")
        tampered = bundle("tampered")
        (tampered / "instructions/rust.md").write_bytes(b"!" + content[1:])
        extra = bundle("extra"); (extra / "extra.txt").write_text("unexpected")
        missing = bundle("missing"); (missing / "instructions/rust.md").unlink()
        altered_size = json.loads(json.dumps(manifest)); altered_size["files"][0]["size_bytes"] += 1
        bundle("size-mismatch", document=altered_size)
        symlink = bundle("symlink")
        shutil.rmtree(symlink / "instructions")
        (symlink / "instructions").symlink_to(valid / "instructions", target_is_directory=True)
        bad_signature = bundle("bad-signature")
        (bad_signature / "manifest.json.minisig").write_text("not a signature\n")
        changed_manifest = bundle("changed-manifest")
        with (changed_manifest / "manifest.json").open("a") as f: f.write("\n")
        unexpected_dir = bundle("extra-directory"); (unexpected_dir / "hidden").mkdir()
    print(f"Test fixtures generated in {output}; independent Minisign verification passed.")


if __name__ == "__main__":
    generate(pathlib.Path(sys.argv[1] if len(sys.argv) > 1 else ".local/fixtures").resolve())
