## ADDED Requirements

### Requirement: Continuous Integration Pipeline

The system SHALL provide automated continuous integration via GitHub Actions that runs on every push to `main` and on every pull request.

#### Scenario: CI runs on push to main
- **WHEN** a commit is pushed to the `main` branch
- **THEN** the CI workflow is triggered
- **AND** formatting is checked with `cargo fmt --check`
- **AND** linting is performed with `cargo clippy -- -D warnings`
- **AND** tests are run with `cargo test`

#### Scenario: CI runs on pull request
- **WHEN** a pull request is opened or updated
- **THEN** the CI workflow is triggered
- **AND** all checks (format, lint, test) run against the PR branch

#### Scenario: CI matrix covers target platforms
- **WHEN** the CI workflow runs
- **THEN** tests execute on both Linux and macOS runners
- **AND** tests execute on both x86_64 and aarch64 architectures where available

#### Scenario: Warnings treated as errors
- **WHEN** any CI build step compiles Rust code
- **THEN** the build MUST use `RUSTFLAGS="-D warnings"`
- **AND** the build fails if any compiler warnings are emitted

### Requirement: Local Check Script

The system SHALL provide a `script/check` bash script that runs the same validation steps as CI, allowing developers to verify their changes locally before pushing.

#### Scenario: Script exists with proper shebang
- **WHEN** a developer inspects `script/check`
- **THEN** the file exists
- **AND** the file starts with `#!/usr/bin/env bash` shebang
- **AND** the file is executable

#### Scenario: Script runs all validation steps
- **WHEN** a developer runs `script/check`
- **THEN** the script runs `cargo fmt --check`
- **AND** the script runs `cargo clippy -- -D warnings`
- **AND** the script runs `cargo build`
- **AND** the script runs `cargo test`
- **AND** the script uses `RUSTFLAGS="-D warnings"` for compilation steps

#### Scenario: CI uses the check script
- **WHEN** the CI workflow runs validation steps
- **THEN** the CI invokes `script/check` rather than sequencing cargo commands in GitHub Actions YAML
- **AND** the validation logic is centralized in the script

### Requirement: Release Pipeline

The system SHALL provide an automated release pipeline via GitHub Actions that is triggered by manual workflow dispatch.

#### Scenario: Release triggered by workflow dispatch
- **WHEN** a user triggers the release workflow manually
- **AND** provides a version number (e.g., `0.1.0`)
- **THEN** the workflow creates a git tag `v<version>` on the current commit
- **AND** pushes the tag to the repository
- **AND** a GitHub Release is created with the tag name

#### Scenario: Release builds binaries for all targets
- **WHEN** the release workflow runs
- **THEN** release binaries are built for Linux x86_64
- **AND** release binaries are built for Linux aarch64
- **AND** release binaries are built for macOS x86_64
- **AND** release binaries are built for macOS aarch64

#### Scenario: Release assets are uploaded
- **WHEN** release binaries are successfully built
- **THEN** binaries are uploaded as assets to the GitHub Release
- **AND** asset names indicate the target platform and architecture

#### Scenario: Release requires passing tests on all platforms
- **WHEN** the release workflow is triggered
- **THEN** tests MUST pass on Linux x86_64
- **AND** tests MUST pass on Linux aarch64
- **AND** tests MUST pass on macOS x86_64
- **AND** tests MUST pass on macOS aarch64
- **BEFORE** any release artifacts are created or GitHub Release is published
