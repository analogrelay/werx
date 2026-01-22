# Implementation Tasks

## 1. Project Setup

- [x] 1.1 Initialize Cargo project with binary crate
- [x] 1.2 Add CLI framework dependency (clap)
- [x] 1.3 Add error handling dependency (anyhow)
- [x] 1.4 Add configuration management dependencies (serde, toml)
- [x] 1.5 Add path manipulation and filesystem dependencies
- [x] 1.6 Set up basic project structure (main.rs, lib.rs, modules)

## 2. Path Resolution Logic

- [x] 2.1 Implement default path resolution (`~/forge`)
- [x] 2.2 Implement environment variable resolution (`FORGE_DIR`)
- [x] 2.3 Implement command-line argument parsing for custom path
- [x] 2.4 Implement priority order (CLI arg > env var > default)
- [x] 2.5 Add path expansion logic (handle `~` expansion)
- [x] 2.6 Add unit tests for path resolution logic

## 3. Path Validation

- [x] 3.1 Implement validation for path writability
- [x] 3.2 Implement check for existing regular files at target path
- [x] 3.3 Implement check for existing Forge at target path
- [x] 3.4 Add parent directory creation logic
- [x] 3.5 Add unit tests for path validation

## 4. Forge Initialization

- [x] 4.1 Implement Forge directory structure creation (`repos/`, `workspaces/`)
- [x] 4.2 Implement Forge marker file creation (to detect existing Forges)
- [x] 4.3 Add `--force` flag implementation for re-initialization
- [x] 4.4 Add cleanup logic for failed initialization
- [x] 4.5 Add integration tests for initialization flow

## 5. CLI Command Implementation

- [x] 5.1 Implement `forge init` command with clap
- [x] 5.2 Add optional path argument to init command
- [x] 5.3 Add `--force` flag to init command
- [x] 5.4 Add success message output with Forge location
- [x] 5.5 Add helpful next steps message
- [x] 5.6 Add integration tests for CLI command

## 6. Error Handling

- [x] 6.1 Use `anyhow::Result` for all fallible operations
- [x] 6.2 Add context to errors using `.context()` for permission denied scenarios
- [x] 6.3 Add context to errors for filesystem operations (disk full, I/O errors)
- [x] 6.4 Add context to errors for existing Forge scenarios
- [x] 6.5 Ensure all error messages are user-friendly and actionable
- [x] 6.6 Add tests for error scenarios and verify error messages

## 7. Documentation

- [x] 7.1 Add inline code documentation
- [x] 7.2 Add CLI help text for `forge init` command
- [x] 7.3 Add examples to help text
- [x] 7.4 Update README with installation and usage instructions

## 8. Testing and Validation

- [x] 8.1 Run all unit tests and ensure they pass
- [x] 8.2 Run all integration tests and ensure they pass
- [x] 8.3 Test on macOS (primary platform)
- [x] 8.4 Run `cargo fmt` and `cargo clippy`
- [x] 8.5 Manually test all scenarios from spec
- [x] 8.6 Verify error messages are helpful and clear

## 9. Build and Polish

- [x] 9.1 Verify binary builds successfully
- [x] 9.2 Check binary size is reasonable
- [x] 9.3 Test command-line experience (speed, responsiveness)
- [x] 9.4 Final review of all code changes
