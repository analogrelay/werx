{
  description = "Werx - A tool for managing code repositories and workspaces";

  inputs = {
    nixpkgs.url = "github:cachix/devenv-nixpkgs/rolling";
    devenv.url = "github:cachix/devenv";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  nixConfig = {
    extra-trusted-public-keys =
      "devenv.cachix.org-1:w1cLUi8dv3hnoSPGAuibQv+f9TZLr6cv/Hm9XgU50cw=";
    extra-substituters = "https://devenv.cachix.org";
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay, devenv }@inputs:
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

        devShells.default = devenv.lib.mkShell {
          inherit inputs pkgs;
          modules = [
            ({ ... }: {
              devcontainer.enable = true;
              languages.rust.enable = true;
            })
          ];
        };
      });
}
