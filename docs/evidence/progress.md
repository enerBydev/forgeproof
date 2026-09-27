# Development evidence

Plan: docs/design/architecture.md. First milestone only; no certification claim.

## Decisions

- Product and public repository name: Forgeproof. User authorized repository creation and Vercel deployment on 2026-09-26.
- Public source contains no credentials or production signing keys. Runtime secrets are external configuration, never grounds for committing them into a private repository.
- Vercel hosts the web interface. NixOS verification and long-running workers remain separate execution processes.
- Contract types feed catalog verification; catalog output feeds authorization; all feed the CLI. No shared-interface conflict found during preflight.
- Pin Rust 1.98.0, confirmed against the official release announcement. Candidate crate versions: serde 1.0.229, serde_json 1.0.151, sha2 0.11.0, minisign-verify 0.2.5. Compatibility remains unverified until compilation.
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

## Official sources consulted

- https://blog.rust-lang.org/2026/08/20/Rust-1.98.0/
- https://docs.rs/serde/1.0.229/serde/
- https://docs.rs/serde_json/1.0.151/serde_json/
- https://docs.rs/sha2/0.11.0/sha2/
- https://docs.rs/minisign-verify/0.2.5/minisign_verify/struct.PublicKey.html
- https://jedisct1.github.io/minisign/

Sources inform specific APIs; this is not a claim to have read all ecosystem documentation.
