## 1. CI Pipeline

- [ ] 1.1 Create `.github/workflows/ci.yml` workflow
- [ ] 1.2 Configure workflow to run on push to `main` and on pull requests
- [ ] 1.3 Add job for `cargo fmt --check`
- [ ] 1.4 Add job for `cargo clippy -- -D warnings`
- [ ] 1.5 Add job for `cargo test`
- [ ] 1.6 Add matrix build for Linux and macOS

## 2. Release Pipeline

- [ ] 2.1 Create `.github/workflows/release.yml` workflow
- [ ] 2.2 Configure workflow to trigger on manual workflow dispatch with version input
- [ ] 2.3 Add step to create and push git tag from version input
- [ ] 2.4 Add job to create GitHub Release with the tag
- [ ] 2.5 Add job to build release binaries for Linux (x86_64, aarch64)
- [ ] 2.6 Add job to build release binaries for macOS (x86_64, aarch64)
- [ ] 2.7 Upload binaries as release assets
- [ ] 2.8 Add job to publish to crates.io

## 3. Nix Flake Packaging

- [ ] 3.1 Create `flake.nix` with basic structure
- [ ] 3.2 Define package derivation using `rustPlatform.buildRustPackage`
- [ ] 3.3 Add devShell for development environment
- [ ] 3.4 Add `flake.lock` to repository
- [ ] 3.5 Test `nix build` and `nix run` locally
- [ ] 3.6 Document Nix installation in README

## 4. Crates.io Publishing

- [ ] 4.1 Update `Cargo.toml` with required metadata (description, license, repository, etc.)
- [ ] 4.2 Verify package with `cargo publish --dry-run`
- [ ] 4.3 Configure release workflow to publish on release
- [ ] 4.4 Document `cargo install` in README

## 5. Documentation

- [ ] 5.1 Update README with installation instructions for all distribution methods
- [ ] 5.2 Add badges for CI status and crates.io version
