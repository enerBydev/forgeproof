{
  description = "Forgeproof: verifiable specialist bundles";
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/5e2305d577ca00acbba631b05cb1094d172b29f3";
    rust-overlay.url = "github:oxalica/rust-overlay/edea2d6f98b1e5f64ce5ac21e19a96c603c20b8c";
    rust-overlay.inputs.nixpkgs.follows = "nixpkgs";
  };
  outputs = { self, nixpkgs, rust-overlay }: let
    system = "x86_64-linux";
    pkgs = import nixpkgs { inherit system; overlays = [ rust-overlay.overlays.default ]; };
    toolchain = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
    platform = pkgs.makeRustPlatform { cargo = toolchain; rustc = toolchain; };
    engctl = platform.buildRustPackage {
      pname = "forgeproof-engctl";
      version = "0.1.0";
      src = pkgs.lib.cleanSource ./.;
      cargoLock.lockFile = ./Cargo.lock;
      nativeCheckInputs = [ pkgs.python3 pkgs.minisign ];
      preCheck = "python3 scripts/generate-fixtures.py";
      doCheck = true;
    };
  in {
    packages.${system}.default = engctl;
    packages.${system}.engctl = engctl;
    devShells.${system}.default = pkgs.mkShell {
      packages = [ toolchain pkgs.minisign pkgs.python3 pkgs.nodejs_24 pkgs.git ];
    };
    checks.${system}.verifier = import ./nix/tests/verifier.nix { inherit pkgs engctl; };
  };
}
