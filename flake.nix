{
  description = "Werx - A tool for managing code repositories and workspaces";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };

        # Read toolchain from rust-toolchain.toml
        rustToolchain =
          pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;

        cargoToml = builtins.fromTOML (builtins.readFile ./Cargo.toml);

        werx = pkgs.rustPlatform.buildRustPackage {
          pname = "werx";
          version = cargoToml.package.version;

          src = ./.;

          cargoLock = { lockFile = ./Cargo.lock; };

          nativeCheckInputs = [ pkgs.git ];

          # Configure git for tests that create commits
          preCheck = ''
            export HOME=$(mktemp -d)
            git config --global user.email "test@example.com"
            git config --global user.name "Test User"
          '';

          meta = with pkgs.lib; {
            description =
              "A tool for managing code repositories and workspaces";
            homepage = "https://github.com/analogrelay/werx";
            license = licenses.mit;
            maintainers = [ ];
            mainProgram = "werx";
          };
        };
      in {
        packages = {
          default = werx;
          werx = werx;
        };

        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            rustToolchain
            rust-analyzer
            cargo-watch
            cargo-edit
          ];

          RUST_SRC_PATH = "${rustToolchain}/lib/rustlib/src/rust/library";
        };
      });
}
