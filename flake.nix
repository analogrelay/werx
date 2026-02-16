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
    let
      # Package builder that works with any nixpkgs instance.
      # Used by both the per-system packages and the overlay.
      mkWerx = pkgs:
        pkgs.rustPlatform.buildRustPackage {
          pname = "werx";
          version = (builtins.fromTOML
            (builtins.readFile ./Cargo.toml)).package.version;

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
            mainProgram = "werx";
          };
        };
    in {
      # Overlay for consuming werx from NixOS/nix-darwin configurations.
      # Usage: nixpkgs.overlays = [ werx.overlays.default ];
      #        environment.systemPackages = [ pkgs.werx ];
      overlays.default = final: _prev: { werx = mkWerx final; };
    } // flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };

        # Read toolchain from rust-toolchain.toml
        rustToolchain =
          pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
      in {
        packages = {
          default = mkWerx pkgs;
          werx = mkWerx pkgs;
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
