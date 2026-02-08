# Release History

## 0.1.0 (Unreleased)

- Initial Release
- Added `--build` and `--release` modes to `script/check` for flexible CI validation

### Features Added

- Fixed namedata module structure by converting to a single Rust file
- Resolved clippy warnings and formatting issues

### Other Changes

- Added crates.io publishing step to release workflow
- Added pre-release validation for duplicate GitHub Releases, crates.io versions, and CARGO_REGISTRY_TOKEN

### Bugs Fixed

- Added `rust-toolchain.toml` for consistent Rust version management
- Added `TestContext` for isolated test environments with pre-configured git
- Extracted reusable GitHub workflows (`_build.yml`, `_nix-build.yml`) to reduce CI duplication
- Added Nix build job to CI and release workflows
- Installed OpenCode in CI for agent detection tests
- Updated AGENTS.md with changelog maintenance instructions
