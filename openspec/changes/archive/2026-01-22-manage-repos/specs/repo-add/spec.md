# repo-add Specification

## Purpose
Enables adding Git repositories to the Forge by cloning them into the managed repository storage area. Repositories are stored as bare clones and referenced by their deterministic directory name.

## ADDED Requirements

### Requirement: Add Repository Command

The system SHALL provide a command to add repositories to the Forge.

#### Scenario: Add repository via repos subcommand

- **WHEN** user runs `forge repos add <repo-spec>`
- **THEN** the repository is cloned to `.forge/repos/<name>-<hash>/`
- **AND** success message displays the repository location

#### Scenario: Add repository via top-level alias

- **WHEN** user runs `forge add <repo-spec>`
- **THEN** it behaves identically to `forge repos add <repo-spec>`

#### Scenario: Add repository with full URL

- **WHEN** user runs `forge add https://github.com/owner/repo.git`
- **THEN** the repository is cloned using the provided URL

#### Scenario: Add repository with shorthand

- **WHEN** user runs `forge add github:owner/repo`
- **THEN** the repository is cloned using the resolved URL `https://github.com/owner/repo.git`

#### Scenario: Add repository with owner/repo shorthand

- **WHEN** user runs `forge add owner/repo`
- **AND** default provider is `github`
- **THEN** the repository is cloned using `https://github.com/owner/repo.git`

### Requirement: Bare Clone Storage

The system SHALL store repositories as bare Git clones.

#### Scenario: Clone repository as bare

- **WHEN** repository is added to the Forge
- **THEN** it is cloned with `git clone --bare`
- **AND** stored in `.forge/repos/<name>-<hash>/`

#### Scenario: Bare clone contains all Git data

- **WHEN** repository is cloned as bare
- **THEN** all branches, tags, and refs are available
- **AND** no working directory is created

### Requirement: Duplicate Prevention

The system SHALL prevent adding a repository that already exists in the Forge.

#### Scenario: Reject duplicate repository by URL

- **WHEN** user attempts to add a repository
- **AND** a repository with the same normalized URL already exists
- **THEN** the command fails with error message indicating the repository already exists

#### Scenario: Detect duplicate via different URL forms

- **WHEN** user previously added `https://github.com/owner/repo.git`
- **AND** attempts to add `github:owner/repo`
- **THEN** the command fails because they resolve to the same normalized URL

#### Scenario: Show existing location in error

- **WHEN** duplicate repository addition is rejected
- **THEN** error message includes the existing repository location

### Requirement: Forge Existence Check

The system SHALL verify a Forge exists before adding repositories.

#### Scenario: Require initialized Forge

- **WHEN** user runs `forge add <repo>` outside a Forge directory
- **AND** no Forge exists at the default location
- **THEN** the command fails with error indicating no Forge found

#### Scenario: Suggest forge init

- **WHEN** command fails due to missing Forge
- **THEN** error message suggests running `forge init` first

### Requirement: Protocol Preference Prompting

The system SHALL prompt for protocol preference when resolving shorthand URLs if not configured.

#### Scenario: Prompt for protocol on first shorthand add

- **WHEN** user runs `forge add <shorthand-spec>` (e.g., `github:owner/repo` or `owner/repo`)
- **AND** protocol preference is not set in config
- **THEN** user is prompted to choose between SSH and HTTPS
- **AND** choice is saved to `.forge/config`

#### Scenario: Use existing protocol preference

- **WHEN** user runs `forge add <shorthand-spec>`
- **AND** protocol preference is already set in config
- **THEN** no prompt is shown
- **AND** existing preference is used for URL resolution

#### Scenario: No prompt needed for full URLs

- **WHEN** user runs `forge add <full-url>` (e.g., `https://github.com/owner/repo.git`)
- **THEN** no protocol prompt is shown
- **AND** the URL is used as provided

#### Scenario: Protocol preference persists

- **WHEN** protocol preference is set during add operation
- **THEN** it is saved to config
- **AND** applies to all future shorthand URL resolutions

### Requirement: Git Clone Error Handling

The system SHALL handle Git clone failures gracefully.

#### Scenario: Handle invalid repository URL

- **WHEN** user attempts to add a repository with invalid URL
- **THEN** the command fails with error from Git
- **AND** no directory is created in `.forge/repos/`

#### Scenario: Handle name conflicts with existing directories

- **WHEN** user attempts to add a repository
- **AND** a directory with the computed `<name>-<hash>` already exists
- **THEN** the command recognizes it as a duplicate and fails appropriately

#### Scenario: Handle authentication failures

- **WHEN** Git clone fails due to authentication
- **THEN** error message indicates authentication is required
- **AND** suggests checking Git credentials

#### Scenario: Handle network failures

- **WHEN** Git clone fails due to network issues
- **THEN** error message indicates network problem
- **AND** no partial clone is left in `.forge/repos/`

#### Scenario: Clean up failed clones

- **WHEN** Git clone operation fails
- **THEN** any partially created directory in `.forge/repos/` is removed

### Requirement: Success Feedback

The system SHALL provide clear feedback when repository is added successfully.

#### Scenario: Display success message

- **WHEN** repository is added successfully
- **THEN** success message confirms the repository was added
- **AND** shows the repository specification used

#### Scenario: Show repository storage location

- **WHEN** repository is added successfully
- **THEN** message includes the internal storage path `.forge/repos/<name>-<hash>/`

### Requirement: Clone Progress Display

The system SHALL show progress during Git clone operations.

#### Scenario: Show clone progress for large repositories

- **WHEN** repository is being cloned
- **THEN** Git's progress output is displayed to the user
- **AND** user can see clone is in progress
