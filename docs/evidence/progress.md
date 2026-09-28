# Development evidence

Plan: docs/design/architecture.md. First milestone only; no certification claim.

## Decisions

- Product and public repository name: Forgeproof. User authorized repository creation and Vercel deployment on 2026-09-26.
- Public source contains no credentials or production signing keys. Runtime secrets are external configuration, never grounds for committing them into a private repository.
- Vercel hosts the web interface. NixOS verification and long-running workers remain separate execution processes.
- Contract types feed catalog verification; catalog output feeds authorization; all feed the CLI. No shared-interface conflict found during preflight.
- Pin Rust 1.98.0, confirmed against the official release announcement. Crate versions: serde 1.0.229, serde_json 1.0.151, sha2 0.11.0, minisign-verify 0.2.5. Compilation and NixOS acceptance were subsequently verified in the runs below.
- Bootstrap CI resolves Cargo.lock once as a candidate artifact. Normal CI must use --locked after the reviewed lock is committed. No hand-written dependency hashes.

## Environment

Local host: Ubuntu; Git, Node.js and Python available. Rust, Cargo, Nix and Minisign unavailable. Requests to Rust, npm and GitHub download endpoints timed out. GitHub and Vercel connected tools work for read access. Repository created through authenticated GitHub web UI.

## Test ledger

- CI run 36260561745: manifest suite RED, 1 passed / 6 failed, due to accepting invalid inputs in the initial deserialization-only implementation.
- CI run 36260835816: manifest suite GREEN, 7 passed. Rust 1.98.0 and candidate crate versions compiled on Ubuntu 24.04.
- Same run: independently generated Minisign fixtures validated. Catalog suite RED, 12 failed against the explicit not_implemented stub. No package-verification success claimed from this run.
- Local web receipt tests RED, 4 passed / 4 failed against a reject-all stub.
- Resumed 2026-09-27 after account-limit interruption. Remote branch remained at a39755758f163e223a2d4613e3a6c99c94b54401. Vercel project did not yet exist.
- Ruling: reject parent-directory components in the trusted staging path. Tests resolve their fixture paths before invoking verification. Staging remains the caller's responsibility; concurrent hostile writers are outside the v1 guarantee.
- CI run 36338804247 caught a compile-time sha2 0.11 API difference: digest arrays do not implement LowerHex. Format digest bytes explicitly; do not downgrade or float the dependency.
- CI run 36339230526: 7 manifest + 12 integrity tests GREEN. Authorization (6) and CLI (6) tests RED against their explicit stubs; these failures were observed before implementing their behavior.
- Local receipt inspector: 8 tests GREEN, static build passed on Node 24.19.0. Imported receipts are explicitly unauthenticated claims; no file is uploaded to a backend.
- Ruling: this milestone's Vercel UI is a dependency-free static inspector. A framework and server API are deferred until the shared catalog requires them; Rust remains the verification authority. This reduces initial surface area without claiming MCP or agent installation is implemented.
- CI run 36339502522: invalid bootstrap YAML prevented jobs from starting. Removed a duplicate artifact option; this run supplied no test evidence.
- CI run 36339818642: all 31 Rust tests, Clippy and the 8 initial web tests passed. Nix evaluation rejected a repeated dynamic attribute; grouped both package outputs under one system attribute. Nix generated the candidate flake lock before the evaluation failure.
- Independent read-only review found a web coercion bug: an array authorization value bypassed strict state comparisons, and a specially shaped object could throw. Observed regression RED (8 passed, 1 failed), then GREEN (9 passed). Require string before the property lookup; include both array and object regressions. Reviewer found no other blocking Rust issues under the documented threat model and deferred runtime verification to CI.
- [CI run 36339981281](https://github.com/enerBydev/forgeproof/actions/runs/36339981281), revision `3068acbd5755e12c0d6a292e39207623f3fc005d`: 31 Rust tests, 9 web tests, Clippy, formatting and web build GREEN using committed dependency locks. `nix flake check --no-update-lock-file` built the package, ran its tests, booted the NixOS VM and completed all 7 CLI acceptance scenarios (VM script 22.73 seconds). This is the first complete system acceptance evidence.
- Replace bootstrap dependency resolution with normal CI for pushes and pull requests: frozen locks, exact action commits, Rust workspace tests and NixOS VM. No ordinary workflow regenerates dependency locks.
- [Final branch CI 36340263488](https://github.com/enerBydev/forgeproof/actions/runs/36340263488), revision `22f3232f13474f33a266e6ecdc28d5ed688526f0`: Rust/web and NixOS VM jobs GREEN with unchanged dependency locks. Independent recheck confirmed the web finding closed.
- PR #1 merged to `main` as `54c941925abc908d10f37081ccae6ccb00c832f4`. [Main CI 36340377977](https://github.com/enerBydev/forgeproof/actions/runs/36340377977) also GREEN.

## First deployment acceptance — 2026-09-28 UTC

- Public application: https://forgeproof-two.vercel.app/ (project `forgeproof` in the authorized Vercel team). Production deployment `dpl_DHiZDKEWW21LJ999VS8PCjywZUbm` reached `READY`, source revision `54c941925abc908d10f37081ccae6ccb00c832f4`.
- `/status.json` returned HTTP 200 and the exact source revision, with `capability = receipt-inspection` and `authenticated_verification = false`.
- Browser acceptance: the supplied example renders unknown validity; malformed JSON hides the previous result; a local JSON file renders its declared approval with the unauthenticated-receipt warning; an array authorization value is rejected. The tests used explicitly synthetic demo data, not real authorization evidence.
- No console error from the application origin was observed during these interactions. The browser extension emitted unrelated metadata errors; they are not attributed to the application.
- Deployed headers confirmed CSP with `connect-src 'none'`, `frame-ancestors 'none'`, `object-src 'none'` and `form-action 'none'`, plus nosniff, no-referrer and disabled camera/microphone/geolocation permissions.
- Public source review found no production credentials or signing keys. Vercel required no secret environment variables for this static interface.
- First milestone delivered: verifier, contracts, CLI, frozen environment/CI, NixOS acceptance and receipt inspector. Native client installation, MCP serving and documentation-update automation remain future milestones; no compatibility certification is claimed.

## Verifier evidence closure — 2026-09-28 UTC

- User authorized closing the audit gaps, updating plan traceability, deleting the integrated branch and synchronizing main before continuing the next milestone.
- CI 36374905582 at `2bbf8d077a6db5e1209b4f23671c947879702353`: Rust suite 32/32, web 9/9, Clippy and formatting pass. Six injected faults were detected in disposable repository copies; restored checks pass. No production verifier behavior changed. NixOS VM acceptance passed.
- External Minisign oracle confirms valid/trusted and other/own-key acceptance, other/trusted-key rejection, corrupt-signature rejection and changed-manifest rejection. Tampered payload still has a valid manifest signature and is rejected by the independent content-hash layer.
- Ruling: retain the approved fixture generator and consolidated integrity suite, documenting their mapping to proposed paths in `traceability.md`; this changes test organization, not the acceptance requirements.
- Ruling: use the existing dedicated checkout on a correction branch. No unrelated user changes were present; native Rust/Nix checks remain on CI because those tools are unavailable locally.
- The original source plan remains historical outside the repository; the maintained repository copy links the current traceability and explicitly records subsequent milestone state.

## Official sources consulted (initial milestone)

- https://blog.rust-lang.org/2026/08/20/Rust-1.98.0/
- https://docs.rs/serde/1.0.229/serde/
- https://docs.rs/serde_json/1.0.151/serde_json/
- https://docs.rs/sha2/0.11.0/sha2/
- https://docs.rs/minisign-verify/0.2.5/minisign_verify/struct.PublicKey.html
- https://jedisct1.github.io/minisign/

Sources inform specific APIs; this is not a claim to have read all ecosystem documentation.
