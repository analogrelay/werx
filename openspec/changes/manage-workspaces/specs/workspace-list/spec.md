# workspace-list Specification

## Purpose

Enable users to view and manage existing workspaces in the Forge.

## ADDED Requirements

### Requirement: List Workspaces Command

The system SHALL provide a command to list all workspaces in the Forge.

#### Scenario: List workspaces via workspace subcommand

- **WHEN** user runs `forge workspace list`
- **THEN** all workspaces in the Forge are displayed

#### Scenario: List workspaces via workspaces subcommand

- **WHEN** user runs `forge workspaces list`
- **THEN** it behaves identically to `forge workspace list`

#### Scenario: List workspaces via workspace alias

- **WHEN** user runs `forge wt list`
- **THEN** it behaves identically to `forge workspace list`

#### Scenario: List workspaces via worktree alias

- **WHEN** user runs `forge worktree list`
- **THEN** it behaves identically to `forge workspace list`

### Requirement: Workspace Discovery

The system SHALL discover workspaces by querying git worktrees for each repository.

#### Scenario: Discover worktrees from bare repositories

- **WHEN** listing workspaces
- **THEN** system iterates through repositories in `.forge/repos/`
- **AND** runs `git worktree list` on each bare repository
- **AND** identifies worktrees located at `<forge-root>/` level

#### Scenario: Filter worktrees to forge root only

- **WHEN** discovering workspaces
- **THEN** only worktrees under `<forge-root>/` are included
- **AND** worktrees outside the Forge are excluded

#### Scenario: Handle repositories with no worktrees

- **WHEN** repository has no worktrees
- **THEN** it is not included in workspace listing

### Requirement: Workspace Information Display

The system SHALL display relevant information about each workspace.

#### Scenario: Show workspace name and location

- **WHEN** workspaces are listed
- **THEN** each workspace shows its directory name
- **AND** shows the full path relative to Forge root

#### Scenario: Show associated repository

- **WHEN** workspaces are listed
- **THEN** each workspace shows which repository it's linked to
- **AND** displays the repository's clone URL

#### Scenario: Show current branch

- **WHEN** workspaces are listed
- **THEN** each workspace shows its currently checked-out branch

#### Scenario: Show workspace status

- **WHEN** workspaces are listed
- **THEN** each workspace indicates if it has uncommitted changes
- **AND** shows if workspace directory is missing or invalid

### Requirement: Empty State Handling

The system SHALL provide helpful messaging when no workspaces exist.

#### Scenario: Show empty state message

- **WHEN** user runs `forge workspace list` and no workspaces exist
- **THEN** message indicates no workspaces found
- **AND** suggests running `forge workspace create` to create one

### Requirement: Output Format Options

The system SHALL support multiple output formats for workspace listings.

#### Scenario: Default text format

- **WHEN** user runs `forge workspace list`
- **THEN** output is formatted as human-readable text
- **AND** uses colors and formatting for readability

#### Scenario: JSON format for scripting

- **WHEN** user runs `forge workspace list --format json`
- **THEN** output is valid JSON
- **AND** includes all workspace metadata
- **AND** suitable for parsing by scripts

#### Scenario: Non-interactive output strips colors

- **WHEN** user runs `forge workspace list` in non-interactive context
- **THEN** output contains no ANSI color codes
- **AND** formatting is plain text

### Requirement: Forge Existence Check

The system SHALL verify a Forge exists before listing workspaces.

#### Scenario: Require initialized Forge

- **WHEN** user runs `forge workspace list` outside a Forge directory
- **AND** no Forge exists at the default location
- **THEN** command fails with error indicating no Forge found
- **AND** suggests running `forge init` first

### Requirement: Error Handling

The system SHALL handle workspace discovery failures gracefully.

#### Scenario: Handle corrupted worktree metadata

- **WHEN** git worktree metadata is corrupted or invalid
- **THEN** workspace is marked as invalid in listing
- **AND** error details are shown
- **AND** other workspaces are still listed

#### Scenario: Handle missing workspace directories

- **WHEN** worktree exists in git metadata but directory is missing
- **THEN** workspace is marked as missing in listing
- **AND** suggests running `forge workspace remove` to clean up
- **AND** other workspaces are still listed

#### Scenario: Handle git command failures

- **WHEN** `git worktree list` fails for a repository
- **THEN** error is logged but doesn't halt entire listing
- **AND** other repositories' workspaces are still shown
