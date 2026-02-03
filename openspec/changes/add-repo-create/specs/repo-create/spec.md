## ADDED Requirements

### Requirement: Create Repository Command

The system SHALL provide a command to create new repositories from scratch within the Forge.

#### Scenario: Create repository via repos subcommand

- **WHEN** user runs `forge repos create owner/repo`
- **THEN** a new bare repository is created in `.forge/repos/<name>/`
- **AND** a worktree is created at `<forge-root>/<name>/main/`
- **AND** success message displays both locations

#### Scenario: Create repository via top-level alias

- **WHEN** user runs `forge create owner/repo`
- **THEN** it behaves identically to `forge repos create owner/repo`

#### Scenario: Reject invalid repository specification

- **WHEN** user runs `forge create invalid-spec` (missing owner or repo)
- **THEN** command fails with error indicating valid format is `owner/repo`

### Requirement: Repository Specification Format

The system SHALL accept repository specifications in `owner/repo` format to establish naming for future provider integration.

#### Scenario: Parse owner and repo from specification

- **WHEN** user runs `forge create mycompany/awesome-project`
- **THEN** owner is parsed as `mycompany`
- **AND** repo is parsed as `awesome-project`
- **AND** these are used for directory naming and future remote configuration

#### Scenario: Validate owner format

- **WHEN** user runs `forge create invalid@owner/repo`
- **THEN** command fails with error indicating invalid owner format
- **AND** explains valid characters (alphanumeric, hyphens)

#### Scenario: Validate repo format

- **WHEN** user runs `forge create owner/invalid repo`
- **THEN** command fails with error indicating invalid repository name format

### Requirement: Bare Repository Creation

The system SHALL create a bare Git repository following existing Forge storage conventions.

#### Scenario: Create bare repository with simple name

- **WHEN** user runs `forge create owner/repo`
- **AND** no naming conflicts exist
- **THEN** bare repository is created at `.forge/repos/repo/`
- **AND** repository is initialized with `git init --bare`

#### Scenario: Create bare repository with owner-qualified name on conflict

- **WHEN** user runs `forge create alice/tools`
- **AND** `.forge/repos/tools/` already exists for different owner
- **THEN** bare repository is created at `.forge/repos/alice-tools/`

#### Scenario: Create bare repository with hash-qualified name on double conflict

- **WHEN** user runs `forge create alice/tools`
- **AND** both `tools/` and `alice-tools/` directories exist for different repositories
- **THEN** bare repository is created at `.forge/repos/alice-tools-<hash>/`
- **AND** hash is 6 hexadecimal characters

### Requirement: Main Branch Initialization

The system SHALL initialize the `main` branch with an empty commit to establish a valid branch state.

#### Scenario: Initialize main branch with empty commit

- **WHEN** bare repository is created
- **THEN** `main` branch is created with an empty commit
- **AND** empty commit has message "Initial commit"
- **AND** `HEAD` points to `main` branch

#### Scenario: Main branch is valid for worktree creation

- **WHEN** main branch is initialized
- **THEN** `git worktree add` can successfully create a worktree on `main`
- **AND** worktree contains no files except git metadata

### Requirement: Automatic Worktree Creation

The system SHALL automatically create a worktree on the `main` branch after repository creation.

#### Scenario: Create worktree in repository directory

- **WHEN** repository `owner/repo` is created successfully
- **AND** bare repository is stored at `.forge/repos/repo/`
- **THEN** worktree is created at `<forge-root>/repo/main/`
- **AND** worktree is checked out to `main` branch

#### Scenario: Worktree creation with qualified repository name

- **WHEN** repository is created with owner-qualified name `alice-tools`
- **THEN** worktree is created at `<forge-root>/alice-tools/main/`

#### Scenario: Worktree linked to bare repository

- **WHEN** worktree is created
- **THEN** git metadata points to bare repository in `.forge/repos/`
- **AND** git operations in worktree affect the bare repository

### Requirement: Forge Existence Check

The system SHALL verify a Forge exists before creating repositories.

#### Scenario: Require initialized Forge

- **WHEN** user runs `forge create owner/repo` outside a Forge directory
- **AND** no Forge exists at the default location
- **THEN** command fails with error indicating no Forge found

#### Scenario: Suggest forge init

- **WHEN** command fails due to missing Forge
- **THEN** error message suggests running `forge init` first

### Requirement: Duplicate Prevention

The system SHALL prevent creating repositories that conflict with existing repositories.

#### Scenario: Reject duplicate by normalized URL match

- **WHEN** user runs `forge create owner/repo`
- **AND** repository with matching owner/repo already exists in Forge
- **THEN** command fails with error indicating repository already exists
- **AND** shows existing repository location

#### Scenario: Allow same repo name for different owners

- **WHEN** user creates `alice/utils`
- **AND** later creates `bob/utils`
- **THEN** both are created successfully with appropriate name qualification

### Requirement: Success Feedback

The system SHALL provide clear feedback when repository is created successfully.

#### Scenario: Display success with locations

- **WHEN** repository is created successfully
- **THEN** success message shows bare repository location in `.forge/repos/`
- **AND** shows worktree location at `<forge-root>/<name>/main/`
- **AND** indicates the repository is ready for use

#### Scenario: Show next steps guidance

- **WHEN** repository is created successfully
- **THEN** message suggests `cd <worktree-path>` to start working
- **AND** mentions future `forge publish` command for creating remote (informational only)

### Requirement: Error Handling

The system SHALL handle creation failures gracefully and clean up partial state.

#### Scenario: Clean up on bare repository creation failure

- **WHEN** `git init --bare` fails
- **THEN** any partially created directory in `.forge/repos/` is removed
- **AND** error message explains the failure

#### Scenario: Clean up on worktree creation failure

- **WHEN** worktree creation fails after bare repository is created
- **THEN** the bare repository is removed
- **AND** any partially created worktree directory is removed
- **AND** error message explains the failure

#### Scenario: Handle permission errors

- **WHEN** creation fails due to permission issues
- **THEN** error message explains the permission problem
- **AND** partial state is cleaned up
