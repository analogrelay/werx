# repo-list Specification

## Purpose
TBD - created by archiving change manage-repos. Update Purpose after archive.
## Requirements
### Requirement: List Repositories Command

The system SHALL provide a command to list all repositories in the Forge.

#### Scenario: List all repositories

- **WHEN** user runs `forge repos list`
- **THEN** all repositories in `.forge/repos/` are displayed

#### Scenario: Empty repository list

- **WHEN** user runs `forge repos list`
- **AND** no repositories have been added
- **THEN** message indicates no repositories are present
- **AND** suggests using `forge add` to add repositories

### Requirement: Repository Information Display

The system SHALL display useful information about each repository.

#### Scenario: Show repository clone URL

- **WHEN** repositories are listed
- **THEN** each entry shows the clone URL used to add the repository

#### Scenario: Show repository storage location

- **WHEN** repositories are listed
- **THEN** each entry shows the directory name in `.forge/repos/<name>-<hash>/`

#### Scenario: Show repository metadata

- **WHEN** repositories are listed
- **THEN** each entry includes information that can be retrieved from the bare clone
- **AND** may include default branch or last update time

### Requirement: List Format

The system SHALL present repository list in a clear, readable format.

#### Scenario: Tabular display format

- **WHEN** repositories are listed
- **THEN** information is presented in a structured, aligned format
- **AND** is easy to scan visually

#### Scenario: Machine-readable output option

- **WHEN** user runs `forge repos list --format json`
- **THEN** repository information is output in JSON format
- **AND** can be parsed by other tools

### Requirement: Forge Existence Check

The system SHALL verify a Forge exists before listing repositories.

#### Scenario: Require initialized Forge

- **WHEN** user runs `forge repos list` outside a Forge directory
- **AND** no Forge exists at the default location
- **THEN** the command fails with error indicating no Forge found

#### Scenario: Suggest forge init

- **WHEN** command fails due to missing Forge
- **THEN** error message suggests running `forge init` first

### Requirement: Handle Corrupted Repositories

The system SHALL handle cases where repository directories are corrupted or invalid.

#### Scenario: Skip invalid repository directories

- **WHEN** a directory in `.forge/repos/` is not a valid Git repository
- **THEN** it is marked as invalid in the list
- **OR** skipped with a warning message

#### Scenario: Continue listing despite errors

- **WHEN** some repositories cannot be read
- **THEN** valid repositories are still listed
- **AND** errors are reported separately

