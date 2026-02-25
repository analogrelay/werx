# Release History

## 0.1.0 (Unreleased)

- Initial Release
- Added `--build` and `--release` modes to `script/check` for flexible CI validation

### Features Added

- Fixed namedata module structure by converting to a single Rust file
- Resolved clippy warnings and formatting issues

### Breaking Changes

- Removed `werx agent` command group and all subcommands (`spawn`, `list`, `status`, `attach`, `kill`, `providers`) (PR TBD)
- Removed `[agents]` configuration section from `.werx/config.toml`; existing configs with this section will have it silently ignored (PR TBD)
- Removed `exec` and `rand` dependencies that were only used by agent management (PR TBD)

### Other Changes

- Added Nix overlay output (`overlays.default`) for easy integration with NixOS and nix-darwin configurations (PR TBD)
- Added crates.io publishing step to release workflow
- Added pre-release validation for duplicate GitHub Releases, crates.io versions, and CARGO_REGISTRY_TOKEN

### Bugs Fixed

- Fixed shell hook hanging by replacing stdout-scraping directive protocol with a temp file (`WERX_DIRECTIVE_FILE`) (PR TBD)
- Fixed infinite recursion in `werx` shell function caused by function self-invocation instead of calling the binary (PR TBD)
- Added `rust-toolchain.toml` for consistent Rust version management
- Added `TestContext` for isolated test environments with pre-configured git
- Extracted reusable GitHub workflows (`_build.yml`, `_nix-build.yml`) to reduce CI duplication
- Added Nix build job to CI and release workflows
- Updated AGENTS.md with changelog maintenance instructions
