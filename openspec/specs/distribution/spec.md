# distribution Specification

## Purpose
Defines how the application is distributed to users, including Nix flake packaging and pre-built binary downloads.

## Requirements

### Requirement: Nix Flake Distribution

The system SHALL be installable via Nix flake for users of Nix, NixDarwin, and NixOS.

#### Scenario: Install via nix profile
- **WHEN** a user runs `nix profile install github:analogrelay/werx`
- **THEN** the werx binary is installed to their Nix profile
- **AND** the binary is functional

#### Scenario: Run without installing
- **WHEN** a user runs `nix run github:analogrelay/werx`
- **THEN** the werx binary executes directly
- **AND** no permanent installation is made

#### Scenario: Development shell available
- **WHEN** a user runs `nix develop` in the repository
- **THEN** a development shell is provided with all required build tools
- **AND** the Rust toolchain is available

### Requirement: Binary Distribution

The system SHALL provide pre-built binaries for direct download from GitHub Releases.

#### Scenario: Download binary for Linux x86_64
- **WHEN** a user downloads the Linux x86_64 binary from a GitHub Release
- **THEN** the binary is executable on Linux x86_64 systems
- **AND** no additional runtime dependencies are required

#### Scenario: Download binary for macOS aarch64
- **WHEN** a user downloads the macOS aarch64 binary from a GitHub Release
- **THEN** the binary is executable on Apple Silicon Macs
- **AND** no additional runtime dependencies are required
