{ pkgs, engctl }:
pkgs.testers.runNixOSTest {
  name = "forgeproof-verifier";
  nodes.machine = { ... }: {
    environment.systemPackages = [ engctl pkgs.minisign pkgs.python3 ];
    virtualisation.memorySize = 1024;
  };
  testScript = ''
    import json
    machine.start()
    machine.wait_for_unit("multi-user.target")
    machine.succeed("python3 ${../../scripts/generate-fixtures.py} /tmp/fixtures")
    def verify(bundle, target, revocations, expected_code, expected_state):
        command = "engctl verify --bundle /tmp/fixtures/" + bundle + " --trust /tmp/fixtures/trust.json --target " + target + " --json"
        if revocations:
            command += " --revocations /tmp/fixtures/" + revocations
        code, output = machine.execute(command)
        assert code == expected_code, (code, output)
        assert json.loads(output)["authorization"] == expected_state
    verify("valid", "codex", "current.json", 0, "approved")
    verify("tampered", "codex", "current.json", 1, "denied")
    verify("untrusted", "codex", "current.json", 1, "denied")
    verify("valid", "codex", "revoked.json", 1, "revoked")
    verify("valid", "unsupported", "current.json", 1, "denied")
    verify("valid", "codex", "expired.json", 2, "unknown")
    verify("valid", "codex", None, 2, "unknown")

    machine.succeed("mkdir /tmp/project")
    def distribution(operation, expected_status=None, expected_error=None, bundle="valid", target="codex@0.158.0"):
        command = "engctl " + operation + " --project /tmp/project --json"
        if operation == "install":
            command += " --bundle /tmp/fixtures/" + bundle + " --trust /tmp/fixtures/trust.json --revocations /tmp/fixtures/current.json --target " + target
        code, output = machine.execute(command)
        result = json.loads(output)
        if expected_error:
            assert code == 1 and result["errors"][0]["code"] == expected_error, (code, result)
        else:
            assert code == 0 and result["status"] == expected_status, (code, result)
    distribution("install", "installed")
    initial = machine.succeed("cat /tmp/project/AGENTS.md")
    distribution("install", "unchanged")
    assert machine.succeed("cat /tmp/project/AGENTS.md") == initial
    distribution("install", "installed", target="claude-code@2.1.283")
    claude = machine.succeed("cat /tmp/project/CLAUDE.md")
    distribution("install", "installed", bundle="updated")
    updated = machine.succeed("cat /tmp/project/AGENTS.md")
    assert "rust-nix@0.2.0" in updated and "Run the pinned tests." in updated
    assert machine.succeed("cat /tmp/project/CLAUDE.md") == claude
    distribution("installation-status", "ready")
    machine.succeed("cp /tmp/project/AGENTS.md /tmp/agents-backup")
    machine.succeed("echo owner-edits > /tmp/project/AGENTS.md")
    distribution("install", expected_error="local_modification")
    distribution("installation-status", expected_error="local_modification")
    assert machine.succeed("cat /tmp/project/AGENTS.md").strip() == "owner-edits"
    machine.succeed("cp /tmp/agents-backup /tmp/project/AGENTS.md")
    distribution("installation-status", "ready")
    distribution("recover", "unchanged")
  '';
}
