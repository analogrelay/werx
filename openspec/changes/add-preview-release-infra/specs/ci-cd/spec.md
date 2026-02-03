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
