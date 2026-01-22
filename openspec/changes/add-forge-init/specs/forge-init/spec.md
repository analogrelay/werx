# Capability: Forge Initialization

## ADDED Requirements

### Requirement: Default Forge Location

The system SHALL initialize a Forge at `~/forge` by default when no explicit location is provided.

#### Scenario: Initialize with default location

- **WHEN** user runs `forge init` without specifying a location
- **THEN** the Forge is created at `~/forge`

#### Scenario: Default location already exists as empty directory

- **WHEN** user runs `forge init` and `~/forge` exists but is empty
- **THEN** the Forge is initialized in the existing directory

### Requirement: Custom Forge Location via Environment Variable

The system SHALL allow users to specify a custom Forge location using the `FORGE_DIR` environment variable.

#### Scenario: Initialize with FORGE_DIR environment variable

- **WHEN** user sets `FORGE_DIR=/custom/path` and runs `forge init`
- **THEN** the Forge is created at `/custom/path`

#### Scenario: FORGE_DIR overrides default location

- **WHEN** user sets `FORGE_DIR=/custom/path` and runs `forge init` without arguments
- **THEN** the Forge is created at `/custom/path`, not `~/forge`

### Requirement: Custom Forge Location via Command Argument

The system SHALL allow users to specify a custom Forge location as a command-line argument.

#### Scenario: Initialize with explicit path argument

- **WHEN** user runs `forge init /custom/path`
- **THEN** the Forge is created at `/custom/path`

#### Scenario: Command argument overrides environment variable

- **WHEN** user sets `FORGE_DIR=/env/path` and runs `forge init /arg/path`
- **THEN** the Forge is created at `/arg/path`, not `/env/path`

#### Scenario: Command argument overrides default location

- **WHEN** user runs `forge init /custom/path`
- **THEN** the Forge is created at `/custom/path`, not `~/forge`

### Requirement: Forge Directory Structure Creation

The system SHALL create the necessary directory structure when initializing a Forge.

#### Scenario: Create standard Forge directories

- **WHEN** user runs `forge init`
- **THEN** the following directories are created:
  - `<forge-root>/repos` for repositories
  - `<forge-root>/workspaces` for workspaces

### Requirement: Existing Forge Detection

The system SHALL detect when a Forge already exists at the target location and handle it appropriately.

#### Scenario: Prevent re-initialization of existing Forge

- **WHEN** user runs `forge init` and a Forge already exists at the target location
- **THEN** the command fails with an error message indicating the Forge already exists

#### Scenario: Provide override option for re-initialization

- **WHEN** user runs `forge init --force` and a Forge already exists
- **THEN** the existing Forge configuration is updated without destroying existing content

### Requirement: Path Validation

The system SHALL validate that the target path is suitable for a Forge.

#### Scenario: Reject invalid paths

- **WHEN** user runs `forge init` with a path that cannot be created
- **THEN** the command fails with a descriptive error message

#### Scenario: Reject paths that are regular files

- **WHEN** user runs `forge init /path/to/file` and the path points to an existing regular file
- **THEN** the command fails indicating the path must be a directory

#### Scenario: Create parent directories if needed

- **WHEN** user runs `forge init /parent/child/forge` and `/parent/child` does not exist
- **THEN** parent directories are created before initializing the Forge

### Requirement: Success Feedback

The system SHALL provide clear feedback when initialization succeeds.

#### Scenario: Display success message with Forge location

- **WHEN** user runs `forge init` successfully
- **THEN** a success message is displayed showing the initialized Forge location

#### Scenario: Display next steps after initialization

- **WHEN** user runs `forge init` successfully
- **THEN** helpful next steps are displayed (e.g., "Run 'forge add' to add repositories")

### Requirement: Error Handling

The system SHALL provide clear error messages for common failure scenarios.

#### Scenario: Handle permission denied errors

- **WHEN** user runs `forge init /restricted/path` without appropriate permissions
- **THEN** an error message explains the permission issue

#### Scenario: Handle filesystem errors gracefully

- **WHEN** initialization fails due to filesystem errors (disk full, I/O error, etc.)
- **THEN** a descriptive error message is displayed and partial initialization is cleaned up
