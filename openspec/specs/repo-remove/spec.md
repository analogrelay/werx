# repo-remove Specification

## Purpose
TBD - created by archiving change manage-repos. Update Purpose after archive.
## Requirements
### Requirement: Remove Repository Command

The system SHALL provide a command to remove repositories from the Forge.

#### Scenario: Remove repository by URL

- **WHEN** user runs `forge repos remove <repo-spec>`
- **THEN** the repository matching that specification is removed from `.forge/repos/`

#### Scenario: Remove repository with full URL

- **WHEN** user runs `forge repos remove https://github.com/owner/repo.git`
- **THEN** the repository cloned from that URL is removed

#### Scenario: Remove repository with shorthand

- **WHEN** user runs `forge repos remove github:owner/repo`
- **THEN** the repository is removed using the resolved URL

#### Scenario: Remove repository with owner/repo shorthand

- **WHEN** user runs `forge repos remove owner/repo`
- **AND** default provider is `github`
- **THEN** the repository for `https://github.com/owner/repo.git` is removed

### Requirement: Repository Resolution

The system SHALL resolve repository specifications to find the matching repository.

#### Scenario: Find repository by normalized URL

- **WHEN** user specifies a repository to remove
- **THEN** the URL is normalized using the same process as `repo add`
- **AND** the matching directory name is computed (`<name>-<hash>`)
- **AND** repository at `.forge/repos/<name>-<hash>/` is removed

#### Scenario: Handle URL variations consistently

- **WHEN** repository was added as `https://github.com/owner/repo.git`
- **AND** user removes it as `github:owner/repo`
- **THEN** the repository is found and removed successfully

### Requirement: Non-existent Repository Handling

The system SHALL handle attempts to remove repositories that don't exist.

#### Scenario: Repository not found error

- **WHEN** user attempts to remove a repository
- **AND** no repository with matching normalized URL exists
- **THEN** the command fails with error indicating repository not found

#### Scenario: Suggest listing repositories

- **WHEN** repository removal fails because it's not found
- **THEN** error message suggests running `forge repos list` to see available repositories

### Requirement: Confirmation Before Removal

The system SHALL request confirmation before removing a repository.

#### Scenario: Prompt for confirmation

- **WHEN** user runs `forge repos remove <repo-spec>`
- **THEN** a confirmation prompt displays the repository to be removed
- **AND** asks user to confirm the action

#### Scenario: Skip confirmation with flag

- **WHEN** user runs `forge repos remove <repo-spec> --force`
- **THEN** repository is removed without confirmation prompt

#### Scenario: Cancel on negative confirmation

- **WHEN** user is prompted to confirm removal
- **AND** responds negatively
- **THEN** repository is not removed
- **AND** message indicates operation was cancelled

### Requirement: Complete Removal

The system SHALL completely remove the repository and its storage directory.

#### Scenario: Remove repository directory

- **WHEN** repository is removed
- **THEN** the entire `.forge/repos/<name>-<hash>/` directory is deleted

#### Scenario: Remove all repository data

- **WHEN** repository is removed
- **THEN** all Git objects, refs, and metadata are deleted
- **AND** no traces of the repository remain in `.forge/repos/`

### Requirement: Forge Existence Check

The system SHALL verify a Forge exists before removing repositories.

#### Scenario: Require initialized Forge

- **WHEN** user runs `forge repos remove <repo>` outside a Forge directory
- **AND** no Forge exists at the default location
- **THEN** the command fails with error indicating no Forge found

### Requirement: Success Feedback

The system SHALL provide clear feedback when repository is removed.

#### Scenario: Display success message

- **WHEN** repository is removed successfully
- **THEN** success message confirms the repository was removed
- **AND** shows the repository specification that was removed

### Requirement: Error Handling

The system SHALL handle filesystem errors during removal.

#### Scenario: Handle permission errors

- **WHEN** repository directory cannot be removed due to permissions
- **THEN** error message explains the permission issue
- **AND** suggests how to resolve it

#### Scenario: Handle filesystem errors

- **WHEN** removal fails due to filesystem error
- **THEN** descriptive error message is displayed
- **AND** user is informed if partial removal occurred

