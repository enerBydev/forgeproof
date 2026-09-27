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
  '';
}
