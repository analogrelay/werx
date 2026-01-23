# workspace-remove Specification

## Purpose
TBD - created by archiving change manage-workspaces. Update Purpose after archive.
## Requirements
### Requirement: Remove Workspace Command

The system SHALL provide a command to remove workspaces from the Forge.

#### Scenario: Remove workspace via workspace subcommand

- **WHEN** user runs `forge workspace remove my-workspace`
- **THEN** the workspace is removed from the Forge

#### Scenario: Remove workspace via workspaces subcommand

- **WHEN** user runs `forge workspaces remove my-workspace`
- **THEN** it behaves identically to `forge workspace remove my-workspace`

#### Scenario: Remove workspace via alias

- **WHEN** user runs `forge wt remove my-workspace`
- **THEN** it behaves identically to `forge workspace remove my-workspace`

#### Scenario: Remove workspace via worktree alias

- **WHEN** user runs `forge worktree remove my-workspace`
- **THEN** it behaves identically to `forge workspace remove my-workspace`

#### Scenario: Remove workspace via rm subcommand

- **WHEN** user runs `forge workspace rm my-workspace`
- **THEN** it behaves identically to `forge workspace remove my-workspace`

### Requirement: Workspace Identification

The system SHALL identify workspaces by hierarchical path or workspace name with context.

#### Scenario: Remove by full hierarchical path

- **WHEN** user runs `forge workspace remove project/main`
- **AND** workspace exists at `<forge-root>/project/main/`
- **THEN** that workspace is removed

#### Scenario: Remove by workspace name when in repository directory

- **WHEN** user is in `<forge-root>/project/` or any subdirectory
- **AND** runs `forge workspace remove main`
- **AND** workspace exists at `<forge-root>/project/main/`
- **THEN** that workspace is removed

#### Scenario: Workspace path is case-sensitive

- **WHEN** user runs `forge workspace remove MyProject/Main`
- **AND** workspace exists as `<forge-root>/myproject/main/`
- **THEN** command fails indicating workspace not found
- **AND** suggests closest matches if available

#### Scenario: Require full path when ambiguous

- **WHEN** user runs `forge workspace remove main` from outside any workspace
- **AND** multiple repositories have a `main` workspace
- **THEN** command fails with error indicating ambiguous workspace name
- **AND** lists all matching workspaces
- **AND** suggests using full path like `repo-name/main`

#### Scenario: Handle workspace not found

- **WHEN** user runs `forge workspace remove non-existent`
- **AND** no workspace with that name exists
- **THEN** command fails with error indicating workspace not found
- **AND** suggests running `forge workspace list` to see available workspaces

### Requirement: Confirmation Prompt

The system SHALL prompt for confirmation before removing a workspace.

#### Scenario: Prompt for confirmation in interactive mode

- **WHEN** user runs `forge workspace remove my-workspace`
- **AND** terminal is interactive
- **THEN** user is prompted to confirm the removal
- **AND** workspace path is shown in the prompt
- **AND** prompt indicates if workspace has uncommitted changes

#### Scenario: Accept confirmation to proceed

- **WHEN** user confirms the removal prompt
- **THEN** workspace is removed
- **AND** success message is displayed

#### Scenario: Reject confirmation to cancel

- **WHEN** user rejects the removal prompt
- **THEN** workspace is not removed
- **AND** message indicates operation was cancelled

#### Scenario: Skip confirmation with force flag

- **WHEN** user runs `forge workspace remove my-workspace --force`
- **THEN** no confirmation prompt is shown
- **AND** workspace is removed immediately

#### Scenario: Non-interactive context requires force flag

- **WHEN** user runs `forge workspace remove my-workspace` in non-interactive context
- **AND** `--force` flag is not provided
- **THEN** command fails with error indicating confirmation required
- **AND** suggests using `--force` flag for non-interactive usage

### Requirement: Uncommitted Changes Warning

The system SHALL warn users about uncommitted changes before removal.

#### Scenario: Warn about uncommitted changes

- **WHEN** workspace has uncommitted changes
- **AND** user attempts to remove it
- **THEN** confirmation prompt highlights uncommitted changes
- **AND** warns that changes will be lost

#### Scenario: Show clean status when no changes

- **WHEN** workspace has no uncommitted changes
- **THEN** confirmation prompt indicates workspace is clean

#### Scenario: Force removal removes uncommitted changes

- **WHEN** user runs `forge workspace remove my-workspace --force`
- **AND** workspace has uncommitted changes
- **THEN** workspace is removed including uncommitted changes
- **AND** no data is preserved

### Requirement: Git Worktree Cleanup

The system SHALL properly clean up git worktree metadata when removing workspaces.

#### Scenario: Remove worktree from git metadata

- **WHEN** workspace is removed
- **THEN** `git worktree remove` is executed on the parent repository
- **AND** worktree metadata is cleaned from `.git/worktrees/`

#### Scenario: Remove workspace directory

- **WHEN** workspace is removed
- **THEN** workspace directory at `<forge-root>/[repo-name]/[workspace-name]/` is deleted
- **AND** all files in the workspace are removed

#### Scenario: Clean up empty repository directories

- **WHEN** last workspace for a repository is removed
- **AND** repository directory `<forge-root>/[repo-name]/` is empty
- **THEN** the empty repository directory is removed
- **AND** success message indicates cleanup was performed

#### Scenario: Preserve bare repository

- **WHEN** workspace is removed
- **THEN** bare repository in `.forge/repos/` is preserved
- **AND** repository remains available for creating new workspaces

### Requirement: Forge Existence Check

The system SHALL verify a Forge exists before removing workspaces.

#### Scenario: Require initialized Forge

- **WHEN** user runs `forge workspace remove` outside a Forge directory
- **AND** no Forge exists at the default location
- **THEN** command fails with error indicating no Forge found

### Requirement: Success Feedback

The system SHALL provide clear feedback when workspace is removed successfully.

#### Scenario: Display success message

- **WHEN** workspace is removed successfully
- **THEN** success message confirms the workspace was removed
- **AND** shows the workspace name that was removed

#### Scenario: Show what was preserved

- **WHEN** workspace is removed successfully
- **THEN** message indicates the repository still exists
- **AND** suggests running `forge workspace create` to create a new workspace if needed

### Requirement: Error Handling

The system SHALL handle workspace removal failures gracefully.

#### Scenario: Handle git worktree removal failures

- **WHEN** `git worktree remove` fails
- **THEN** command fails with descriptive error message
- **AND** workspace directory may still exist if git cleanup failed

#### Scenario: Handle missing directory with existing git metadata

- **WHEN** git worktree metadata exists but directory is missing
- **THEN** command proceeds to clean up git metadata
- **AND** success message indicates metadata was cleaned up

#### Scenario: Handle permission errors

- **WHEN** workspace removal fails due to permission issues
- **THEN** error message explains the permission problem
- **AND** suggests potential solutions

#### Scenario: Prune orphaned worktree metadata

- **WHEN** workspace directory doesn't exist but git metadata remains
- **AND** user runs `forge workspace remove [name]`
- **THEN** orphaned git metadata is pruned
- **AND** success message indicates cleanup was performed

