# Release History

## 0.1.0 (Unreleased)

- Initial Release
- Added `--build` and `--release` modes to `script/check` for flexible CI validation (#1)

### Features Added

- Fixed namedata module structure by converting to a single Rust file (#1)
- Resolved clippy warnings and formatting issues (#1)

### Other Changes

- Added crates.io publishing step to release workflow (PR TBD)

### Bugs Fixed

- Added `rust-toolchain.toml` for consistent Rust version management (#1)
- Added `TestContext` for isolated test environments with pre-configured git (#1)
- Extracted reusable GitHub workflows (`_build.yml`, `_nix-build.yml`) to reduce CI duplication (#1)
- Added Nix build job to CI and release workflows (#1)
- Installed OpenCode in CI for agent detection tests (#1)
- Updated AGENTS.md with changelog maintenance instructions

