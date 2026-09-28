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


def signature_oracle(root, public_key, expected):
    result = subprocess.run(
        ["minisign", "-V", "-p", str(public_key), "-m", str(root / "manifest.json")],
        stdout=subprocess.PIPE, stderr=subprocess.PIPE,
    )
    accepted = result.returncode == 0
    if accepted != expected:
        raise AssertionError(f"Minisign oracle disagrees for {root.name}: expected {expected}, exit {result.returncode}")
    print(f"Independent Minisign: {root.name} / {public_key.name}: {'accept' if accepted else 'reject'}")


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
        manifest = {"schema_version": 1, "release_id": "rust-nix@0.1.0", "supported_targets": ["codex", "claude-code", "codex@0.158.0", "claude-code@2.1.283"], "files": [{"path": "instructions/rust.md", "sha256": hashlib.sha256(content).hexdigest(), "size_bytes": len(content), "role": "instructions"}]}
        def bundle(name, signer="trusted", document=None, payload=content):
            root = output / name
            (root / "instructions").mkdir(parents=True)
            (root / "instructions/rust.md").write_bytes(payload)
            (root / "manifest.json").write_text(json.dumps(document or manifest, sort_keys=True, separators=(",", ":")))
            run("minisign", "-S", "-s", str(keys / f"{signer}.sec"), "-m", str(root / "manifest.json"), "-t", "forgeproof-test-fixture")
            return root
        valid = bundle("valid")
        untrusted = bundle("untrusted", "other")
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
        def native_variant(name, payload, role="instructions", release="rust-nix@0.1.0"):
            document = json.loads(json.dumps(manifest))
            document["release_id"] = release
            document["files"][0].update(sha256=hashlib.sha256(payload).hexdigest(), size_bytes=len(payload), role=role)
            return bundle(name, document=document, payload=payload)
        native_variant("updated", b"# Updated instructions\nRun the pinned tests.\n", release="rust-nix@0.2.0")
        native_variant("unsupported-role", content, role="skill")
        native_variant("non-utf8", b"\xff\xfe")
        native_variant("nul-text", b"Do not load\x00hidden text")
        # Hand-specified output envelope: manifest digest always occupies 64 bytes.
        envelope = "# Forgeproof managed instructions\n\nRelease: rust-nix@0.1.0\nManifest: " + "0" * 64 + "\nTarget: codex@0.158.0\n\n## Source: instructions/rust.md\n\n"
        limit_payload = b"A" * (32768 - len(envelope.encode()) - 1)
        native_variant("at-output-limit", limit_payload)
        native_variant("over-output-limit", limit_payload + b"A")
        policy = b"# Test policy\nReview changes before publishing.\n"
        combined_doc = json.loads(json.dumps(manifest))
        combined_doc["files"].insert(0, {"path": "policy.md", "sha256": hashlib.sha256(policy).hexdigest(), "size_bytes": len(policy), "role": "policy"})
        combined = bundle("combined", document=combined_doc)
        (combined / "policy.md").write_bytes(policy)
        signature_oracle(valid, keys / "trusted.pub", True)
        signature_oracle(untrusted, keys / "other.pub", True)
        signature_oracle(untrusted, keys / "trusted.pub", False)
        signature_oracle(bad_signature, keys / "trusted.pub", False)
        signature_oracle(changed_manifest, keys / "trusted.pub", False)
        # Minisign authenticates the manifest; payload hashes are a separate check.
        signature_oracle(tampered, keys / "trusted.pub", True)
    print(f"Test fixtures generated in {output}; independent Minisign verification passed.")


if __name__ == "__main__":
    generate(pathlib.Path(sys.argv[1] if len(sys.argv) > 1 else ".local/fixtures").resolve())
