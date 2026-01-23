# repo-add Specification

## Purpose
TBD - created by archiving change manage-repos. Update Purpose after archive.
## Requirements
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

The system SHALL store repositories as bare Git clones using human-readable directory names.

#### Scenario: Clone repository with simple name

- **WHEN** repository is added to the Forge with no name conflicts
- **THEN** it is cloned with `git clone --bare`
- **AND** stored in `.forge/repos/<name>/`
- **AND** no hash suffix is included

#### Scenario: Clone repository with owner-qualified name on conflict

- **WHEN** repository is added to the Forge
- **AND** simple name conflicts with different repository
- **THEN** it is cloned with `git clone --bare`
- **AND** stored in `.forge/repos/<owner>-<name>/`

#### Scenario: Clone repository with hash-qualified name on double conflict

- **WHEN** repository is added to the Forge
- **AND** simple name and owner-qualified name both conflict
- **THEN** it is cloned with `git clone --bare`
- **AND** stored in `.forge/repos/<owner>-<name>-<hash>/`
- **AND** hash is 6 hexadecimal characters

#### Scenario: Bare clone contains all Git data

- **WHEN** repository is cloned as bare
- **THEN** all branches, tags, and refs are available
- **AND** no working directory is created
- **AND** storage location uses progressive qualification

### Requirement: Duplicate Prevention

The system SHALL prevent adding a repository that already exists in the Forge by checking for conflicts during directory name resolution.

#### Scenario: Reject duplicate repository by simple name match

- **WHEN** user attempts to add a repository
- **AND** simple name directory exists with matching normalized URL
- **THEN** the command fails with error message indicating the repository already exists
- **AND** shows existing repository location

#### Scenario: Reject duplicate repository by owner-qualified name match

- **WHEN** user attempts to add a repository
- **AND** owner-qualified name directory exists with matching normalized URL
- **THEN** the command fails with error message indicating the repository already exists
- **AND** shows existing repository location

#### Scenario: Reject duplicate repository by hash-qualified name match

- **WHEN** user attempts to add a repository
- **AND** hash-qualified name directory exists with matching normalized URL
- **THEN** the command fails with error message indicating the repository already exists
- **AND** shows existing repository location

#### Scenario: Allow same name for different repositories

- **WHEN** user adds repository `torvalds/linux`
- **AND** later adds repository `greg/linux`
- **THEN** both are added successfully
- **AND** first uses simple name `linux/` or `torvalds-linux/`
- **AND** second uses owner-qualified name distinguishing it from the first

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

The system SHALL handle Git clone failures gracefully and clean up conflicted directory names.

#### Scenario: Handle name conflicts with existing directories

- **WHEN** user attempts to add a repository
- **AND** directory with computed name already exists with different normalized URL
- **THEN** progressive qualification advances to next level
- **AND** checks owner-qualified name, then hash-qualified name
- **AND** clone proceeds with non-conflicting name

#### Scenario: Clean up failed clones

- **WHEN** Git clone operation fails
- **THEN** any partially created directory in `.forge/repos/` is removed
- **AND** includes directories created with any qualification level

### Requirement: Success Feedback

The system SHALL provide clear feedback when repository is added successfully, showing the human-readable directory name used.

#### Scenario: Display success message with simple name

- **WHEN** repository is added successfully using simple name
- **THEN** success message confirms the repository was added
- **AND** shows repository specification used
- **AND** shows storage location as `.forge/repos/<name>/`

#### Scenario: Display success message with qualified name

- **WHEN** repository is added successfully using owner-qualified or hash-qualified name
- **THEN** success message confirms the repository was added
- **AND** shows repository specification used
- **AND** shows storage location with qualification (e.g., `.forge/repos/<owner>-<name>/`)
- **AND** explains why qualification was necessary ("name conflict")

#### Scenario: Show repository storage location

- **WHEN** repository is added successfully
- **THEN** message includes the internal storage path
- **AND** path uses human-readable directory name without unnecessary hash suffixes

### Requirement: Clone Progress Display

The system SHALL show progress during Git clone operations.

#### Scenario: Show clone progress for large repositories

- **WHEN** repository is being cloned
- **THEN** Git's progress output is displayed to the user
- **AND** user can see clone is in progress

