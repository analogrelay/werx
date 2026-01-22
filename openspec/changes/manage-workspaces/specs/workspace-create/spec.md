# workspace-create Specification

## Purpose

Enable users to create git worktrees from repositories stored in the Forge, providing working directories for development.

## ADDED Requirements

### Requirement: Create Workspace Command

The system SHALL provide a command to create git worktrees from Forge repositories.

#### Scenario: Create workspace with explicit repository and branch

- **WHEN** user runs `forge workspace create owner/repo main`
- **THEN** a git worktree is created at `<forge-root>/[repo-name]@main/`
- **AND** the worktree is linked to the bare repository in `.forge/repos/`
- **AND** the worktree is checked out to the `main` branch

#### Scenario: Create workspace via workspaces subcommand

- **WHEN** user runs `forge workspaces create owner/repo main`
- **THEN** it behaves identically to `forge workspace create owner/repo main`

#### Scenario: Create workspace via alias

- **WHEN** user runs `forge wt create owner/repo main`
- **THEN** it behaves identically to `forge workspace create owner/repo main`

#### Scenario: Create workspace via worktree alias

- **WHEN** user runs `forge worktree create owner/repo main`
- **THEN** it behaves identically to `forge workspace create owner/repo main`

### Requirement: Interactive Repository Selection

The system SHALL provide an interactive repository selector when repository is not specified.

#### Scenario: Show repository selector when repo not specified

- **WHEN** user runs `forge workspace create` without repository argument
- **AND** terminal is interactive (isatty)
- **THEN** an interactive selector displays available repositories
- **AND** user can navigate with arrow keys and select with Enter

#### Scenario: Search filter in repository selector

- **WHEN** repository selector is displayed
- **THEN** user can type to filter repositories by clone URL
- **AND** filter matches substring anywhere in the URL
- **AND** filtered list updates in real-time as user types

#### Scenario: Repository selector shows relevant information

- **WHEN** repository selector is displayed
- **THEN** each repository shows its clone URL
- **AND** default branch is displayed if available
- **AND** invalid repositories are excluded from the list

#### Scenario: Non-interactive context requires repository argument

- **WHEN** user runs `forge workspace create` without repository argument
- **AND** terminal is non-interactive (piped, scripted)
- **THEN** command fails with error indicating repository is required
- **AND** error suggests using `forge workspace create <repo> [<branch>]`

### Requirement: Interactive Workspace Naming

The system SHALL prompt for workspace name with auto-generated default suggestion.

#### Scenario: Interactive name prompt with default

- **WHEN** workspace creation proceeds
- **AND** terminal is interactive
- **THEN** user is prompted for workspace name
- **AND** default suggestion follows format `[repo-name]@[branch]`
- **AND** user can accept default by pressing Enter
- **AND** user can type custom name to override

#### Scenario: Default name generation for main branch

- **WHEN** creating workspace for repository `github.com/owner/repo` on `main` branch
- **THEN** suggested default name is `repo@main`

#### Scenario: Default name generation for feature branch

- **WHEN** creating workspace for repository `github.com/owner/my-project` on `feature/auth`
- **THEN** suggested default name is `my-project@feature/auth`

#### Scenario: Override name with flag in interactive mode

- **WHEN** user runs `forge workspace create owner/repo main --name custom-workspace`
- **THEN** no interactive prompt is shown
- **AND** workspace is created with name `custom-workspace`

#### Scenario: Non-interactive context requires name flag

- **WHEN** user runs `forge workspace create owner/repo main` in non-interactive context
- **AND** `--name` flag is not provided
- **THEN** workspace is created with auto-generated name `[repo-name]@[branch]`

### Requirement: Branch Selection

The system SHALL support creating workspaces for specific branches.

#### Scenario: Create workspace for specified branch

- **WHEN** user runs `forge workspace create owner/repo feature-branch`
- **THEN** worktree is checked out to `feature-branch`

#### Scenario: Create workspace for default branch when not specified

- **WHEN** user runs `forge workspace create owner/repo`
- **AND** repository has default branch `main`
- **THEN** worktree is checked out to `main`

#### Scenario: Create workspace for default branch via interactive selector

- **WHEN** user selects repository interactively without branch argument
- **THEN** worktree is checked out to the repository's default branch

#### Scenario: Handle non-existent branch

- **WHEN** user runs `forge workspace create owner/repo non-existent-branch`
- **THEN** command fails with error indicating branch does not exist
- **AND** lists available branches

### Requirement: Workspace Storage Location

The system SHALL create workspaces at the Forge root level.

#### Scenario: Workspace created at forge root

- **WHEN** workspace is created with name `my-workspace`
- **THEN** directory is created at `<forge-root>/my-workspace/`
- **AND** directory contains working tree files
- **AND** git metadata points to bare repository in `.forge/repos/`

#### Scenario: Workspace name becomes directory name

- **WHEN** user specifies workspace name `project-v2`
- **THEN** workspace directory is `<forge-root>/project-v2/`

### Requirement: Duplicate Workspace Prevention

The system SHALL prevent creating workspaces with names that conflict with existing directories.

#### Scenario: Reject workspace with existing directory name

- **WHEN** user attempts to create workspace named `existing-dir`
- **AND** directory `<forge-root>/existing-dir/` already exists
- **THEN** command fails with error indicating name conflict
- **AND** suggests choosing a different name

#### Scenario: Detect conflict with .forge directory

- **WHEN** user attempts to create workspace named `.forge`
- **THEN** command fails with error indicating reserved name
- **AND** suggests choosing a different name

### Requirement: Repository Resolution

The system SHALL resolve repository specifications to repositories stored in the Forge.

#### Scenario: Resolve by full URL

- **WHEN** user runs `forge workspace create https://github.com/owner/repo.git main`
- **THEN** system finds matching repository in `.forge/repos/`
- **AND** creates worktree from that repository

#### Scenario: Resolve by shorthand

- **WHEN** user runs `forge workspace create owner/repo main`
- **THEN** system resolves using URL resolution rules (see repo-url-resolution spec)
- **AND** finds matching repository in `.forge/repos/`

#### Scenario: Repository not found error

- **WHEN** user runs `forge workspace create owner/non-existent main`
- **AND** no matching repository exists in the Forge
- **THEN** command fails with error indicating repository not found
- **AND** suggests running `forge add owner/non-existent` first

### Requirement: Forge Existence Check

The system SHALL verify a Forge exists before creating workspaces.

#### Scenario: Require initialized Forge

- **WHEN** user runs `forge workspace create` outside a Forge directory
- **AND** no Forge exists at the default location
- **THEN** command fails with error indicating no Forge found
- **AND** suggests running `forge init` first

### Requirement: Git Worktree Creation

The system SHALL use git worktree functionality to create workspaces.

#### Scenario: Create worktree from bare repository

- **WHEN** workspace is created
- **THEN** `git worktree add` is executed on the bare repository
- **AND** worktree is linked to the bare repository in `.forge/repos/`

#### Scenario: Worktree shares git history with repository

- **WHEN** worktree is created
- **THEN** git operations in worktree affect the shared repository
- **AND** fetches/pulls in worktree update the bare repository
- **AND** all branches are accessible from the worktree

### Requirement: Success Feedback

The system SHALL provide clear feedback when workspace is created successfully.

#### Scenario: Display success message

- **WHEN** workspace is created successfully
- **THEN** success message confirms the workspace was created
- **AND** shows the workspace directory path
- **AND** shows the branch that was checked out

#### Scenario: Show next steps

- **WHEN** workspace is created successfully
- **THEN** message suggests `cd <workspace-path>` to enter the workspace

### Requirement: Error Handling

The system SHALL handle git worktree failures gracefully.

#### Scenario: Handle git worktree command failures

- **WHEN** `git worktree add` fails for any reason
- **THEN** command fails with descriptive error message
- **AND** any partially created workspace directory is removed

#### Scenario: Handle permission errors

- **WHEN** workspace creation fails due to permission issues
- **THEN** error message explains the permission problem
- **AND** suggests potential solutions
