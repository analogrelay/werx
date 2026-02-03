## 1. Local Check Script

- [ ] 1.1 Create `script/check` with `#!/usr/bin/env bash` shebang
- [ ] 1.2 Add `cargo fmt --check` step
- [ ] 1.3 Add `cargo clippy -- -D warnings` step
- [ ] 1.4 Add `cargo build` step with `RUSTFLAGS="-D warnings"`
- [ ] 1.5 Add `cargo test` step with `RUSTFLAGS="-D warnings"`
- [ ] 1.6 Make the script executable

## 2. CI Pipeline

- [ ] 2.1 Create `.github/workflows/ci.yml` workflow
- [ ] 2.2 Configure workflow to run on push to `main` and on pull requests
- [ ] 2.3 Configure workflow to invoke `script/check`
- [ ] 2.4 Add matrix build for Linux and macOS (x86_64 and aarch64 where available)

## 3. Release Pipeline

- [ ] 3.1 Create `.github/workflows/release.yml` workflow
- [ ] 3.2 Configure workflow to trigger on manual workflow dispatch with version input
- [ ] 3.3 Add step to run `script/check` on all target platforms before release
- [ ] 3.4 Add step to create and push git tag from version input
- [ ] 3.5 Add job to create GitHub Release with the tag
- [ ] 3.6 Add job to build release binaries for Linux (x86_64, aarch64)
- [ ] 3.7 Add job to build release binaries for macOS (x86_64, aarch64)
- [ ] 3.8 Upload binaries as release assets

## 4. Nix Flake Packaging

- [ ] 4.1 Create `flake.nix` with basic structure
- [ ] 4.2 Define package derivation using `rustPlatform.buildRustPackage`
- [ ] 4.3 Add devShell for development environment
- [ ] 4.4 Add `flake.lock` to repository
- [ ] 4.5 Test `nix build` and `nix run` locally
- [ ] 4.6 Document Nix installation in README

## 5. Documentation

- [ ] 5.1 Update README with installation instructions for all distribution methods
- [ ] 5.2 Add badges for CI status
