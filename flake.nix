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

        rustToolchain = pkgs.rust-bin.stable.latest.default;

        werx = pkgs.rustPlatform.buildRustPackage {
          pname = "werx";
          version = "0.1.0";

          src = ./.;

          cargoLock = { lockFile = ./Cargo.lock; };

          nativeCheckInputs = [ pkgs.git ];

          # Tests currently use `cargo run` instead of the pre-built binary,
          # which doesn't work well in the nix sandbox. The fix is to update
          # tests/common/mod.rs to use env!("CARGO_BIN_EXE_werx") instead.
          # TODO: Enable tests once test infrastructure is updated.
          doCheck = false;

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
