## 1. Local Check Script

- [x] 1.1 Create `script/check` with `#!/usr/bin/env bash` shebang
- [x] 1.2 Add `cargo fmt --check` step
- [x] 1.3 Add `cargo clippy -- -D warnings` step
- [x] 1.4 Add `cargo build` step with `RUSTFLAGS="-D warnings"`
- [x] 1.5 Add `cargo test` step with `RUSTFLAGS="-D warnings"`
- [x] 1.6 Make the script executable

## 2. Release Helper Scripts

- [x] 2.1 Create `script/helpers/mark-release` with `#!/usr/bin/env bash` shebang
- [x] 2.2 Implement date replacement (replace "Unreleased" with YYYY-MM-DD date)
- [x] 2.3 Implement empty section removal
- [x] 2.4 Implement new unreleased entry creation (next patch version)
- [x] 2.5 Implement `--check` flag for validation mode
- [x] 2.6 Accept optional date argument
- [x] 2.7 Create `script/helpers/bump-version` with `#!/usr/bin/env bash` shebang
- [x] 2.8 Implement Cargo.toml version increment (patch version bump)
- [x] 2.9 Make both scripts executable

## 3. CI Pipeline

- [x] 3.1 Create `.github/workflows/ci.yml` workflow
- [x] 3.2 Configure workflow to run on push to `main` and on pull requests
- [x] 3.3 Configure workflow to invoke `script/check`
- [x] 3.4 Add matrix build for Linux and macOS (x86_64 and aarch64 where available)

## 4. Release Pipeline

- [x] 4.1 Create `.github/workflows/release.yml` workflow
- [x] 4.2 Configure workflow to trigger on manual workflow dispatch with version input
- [x] 4.3 Add step to validate changelog with `script/helpers/mark-release --check`
- [x] 4.4 Add step to run `script/check` on all target platforms before release
- [x] 4.5 Add step to create and push git tag from version input
- [x] 4.6 Add job to create GitHub Release with the tag
- [x] 4.7 Add job to build release binaries for Linux (x86_64, aarch64)
- [x] 4.8 Add job to build release binaries for macOS (x86_64, aarch64)
- [x] 4.9 Upload binaries as release assets
- [x] 4.10 Add step to run `script/helpers/bump-version` after release
- [x] 4.11 Add step to open PR with version bump changes

## 5. Nix Flake Packaging

- [x] 5.1 Create `flake.nix` with basic structure
- [x] 5.2 Define package derivation using `rustPlatform.buildRustPackage`
- [x] 5.3 Add devShell for development environment
- [x] 5.4 Add `flake.lock` to repository
- [x] 5.5 Test `nix build` and `nix run` locally
- [x] 5.6 Document Nix installation in README

## 6. Documentation

- [x] 6.1 Update README with installation instructions for all distribution methods
- [x] 6.2 Add badges for CI status
