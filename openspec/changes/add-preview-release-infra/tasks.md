## 1. Local Check Script

- [ ] 1.1 Create `script/check` with `#!/usr/bin/env bash` shebang
- [ ] 1.2 Add `cargo fmt --check` step
- [ ] 1.3 Add `cargo clippy -- -D warnings` step
- [ ] 1.4 Add `cargo build` step with `RUSTFLAGS="-D warnings"`
- [ ] 1.5 Add `cargo test` step with `RUSTFLAGS="-D warnings"`
- [ ] 1.6 Make the script executable

## 2. Release Helper Scripts

- [ ] 2.1 Create `script/helpers/mark-release` with `#!/usr/bin/env bash` shebang
- [ ] 2.2 Implement date replacement (replace "Unreleased" with YYYY-MM-DD date)
- [ ] 2.3 Implement empty section removal
- [ ] 2.4 Implement new unreleased entry creation (next patch version)
- [ ] 2.5 Implement `--check` flag for validation mode
- [ ] 2.6 Accept optional date argument
- [ ] 2.7 Create `script/helpers/bump-version` with `#!/usr/bin/env bash` shebang
- [ ] 2.8 Implement Cargo.toml version increment (patch version bump)
- [ ] 2.9 Make both scripts executable

## 3. CI Pipeline

- [ ] 3.1 Create `.github/workflows/ci.yml` workflow
- [ ] 3.2 Configure workflow to run on push to `main` and on pull requests
- [ ] 3.3 Configure workflow to invoke `script/check`
- [ ] 3.4 Add matrix build for Linux and macOS (x86_64 and aarch64 where available)

## 4. Release Pipeline

- [ ] 4.1 Create `.github/workflows/release.yml` workflow
- [ ] 4.2 Configure workflow to trigger on manual workflow dispatch with version input
- [ ] 4.3 Add step to validate changelog with `script/helpers/mark-release --check`
- [ ] 4.4 Add step to run `script/check` on all target platforms before release
- [ ] 4.5 Add step to create and push git tag from version input
- [ ] 4.6 Add job to create GitHub Release with the tag
- [ ] 4.7 Add job to build release binaries for Linux (x86_64, aarch64)
- [ ] 4.8 Add job to build release binaries for macOS (x86_64, aarch64)
- [ ] 4.9 Upload binaries as release assets
- [ ] 4.10 Add step to run `script/helpers/bump-version` after release
- [ ] 4.11 Add step to open PR with version bump changes

## 5. Nix Flake Packaging

- [ ] 5.1 Create `flake.nix` with basic structure
- [ ] 5.2 Define package derivation using `rustPlatform.buildRustPackage`
- [ ] 5.3 Add devShell for development environment
- [ ] 5.4 Add `flake.lock` to repository
- [ ] 5.5 Test `nix build` and `nix run` locally
- [ ] 5.6 Document Nix installation in README

## 6. Documentation

- [ ] 6.1 Update README with installation instructions for all distribution methods
- [ ] 6.2 Add badges for CI status
