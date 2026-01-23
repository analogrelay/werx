# workspace-create Specification

## Purpose
TBD - created by archiving change manage-workspaces. Update Purpose after archive.
## Requirements
### Requirement: Create Workspace Command

The system SHALL provide a command to create git worktrees from Forge repositories.

#### Scenario: Create workspace with explicit repository and branch

- **WHEN** user runs `forge workspace create owner/repo main`
- **THEN** a git worktree is created at `<forge-root>/[repo-name]/main/`
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

### Requirement: Context-Aware Repository Detection

The system SHALL automatically detect the repository when running from within a workspace.

#### Scenario: Detect repository from current workspace

- **WHEN** user runs `forge workspace create` from within an existing workspace
- **THEN** the system detects the repository associated with the current workspace
- **AND** uses that repository without prompting for selection

#### Scenario: Detect repository with branch argument only

- **WHEN** user runs `forge workspace create feature-branch` from within a workspace
- **THEN** the system uses the current workspace's repository
- **AND** creates a new workspace for `feature-branch`

#### Scenario: Quick worktree creation for new branch

- **WHEN** user is in workspace `~/forge/my-project/main/`
- **AND** runs `forge workspace create feature/auth`
- **THEN** new workspace is created at `~/forge/my-project/feature/auth/`
- **AND** uses same repository as current workspace

#### Scenario: Override detected repository with explicit repo argument

- **WHEN** user runs `forge workspace create other/repo main` from within a workspace
- **THEN** the system uses `other/repo` instead of detected repository
- **AND** creates workspace for the specified repository

#### Scenario: No repository detection outside workspaces

- **WHEN** user runs `forge workspace create` from outside a workspace
- **THEN** no automatic repository detection occurs
- **AND** repository selector is shown (if interactive)

### Requirement: Interactive Repository Selection

The system SHALL provide an interactive repository selector when repository is not specified and cannot be detected.

#### Scenario: Show repository selector when repo not specified and not detected

- **WHEN** user runs `forge workspace create` without repository argument
- **AND** not running from within a workspace
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

#### Scenario: Non-interactive context requires repository argument or detection

- **WHEN** user runs `forge workspace create` without repository argument
- **AND** not running from within a workspace
- **AND** terminal is non-interactive (piped, scripted)
- **THEN** command fails with error indicating repository is required
- **AND** error suggests using `forge workspace create <repo> [<branch>]` or running from within a workspace

### Requirement: Interactive Workspace Naming

The system SHALL prompt for workspace name with auto-generated default suggestion.

#### Scenario: Interactive name prompt with default

- **WHEN** workspace creation proceeds
- **AND** terminal is interactive
- **THEN** user is prompted for workspace name
- **AND** default suggestion follows format `[repo-name]@[branch]`
- **AND** user can accept default by pressing Enter
- **AND** user can type custom name to override

#### Scenario: Default name is branch name

- **WHEN** creating workspace for repository `github.com/owner/repo` on `main` branch
- **THEN** suggested default name is `main`
- **AND** workspace will be created at `<forge-root>/repo/main/`

#### Scenario: Default name for feature branch

- **WHEN** creating workspace for repository `github.com/owner/my-project` on `feature/auth`
- **THEN** suggested default name is `feature/auth`
- **AND** workspace will be created at `<forge-root>/my-project/feature/auth/`

#### Scenario: Override name with flag in interactive mode

- **WHEN** user runs `forge workspace create owner/repo main --name custom-workspace`
- **THEN** no interactive prompt is shown
- **AND** workspace is created with name `custom-workspace`

#### Scenario: Non-interactive context uses default name

- **WHEN** user runs `forge workspace create owner/repo main` in non-interactive context
- **AND** `--name` flag is not provided
- **THEN** workspace is created with default name equal to branch name
- **AND** workspace is created at `<forge-root>/[repo-name]/main/`

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

### Requirement: Hierarchical Workspace Storage

The system SHALL create workspaces in a hierarchical structure grouped by repository using human-readable directory names.

#### Scenario: Workspace created in repository directory with simple name

- **WHEN** workspace is created for repository stored with simple name `linux`
- **AND** workspace name is `main`
- **THEN** directory is created at `<forge-root>/linux/main/`
- **AND** directory contains working tree files
- **AND** git metadata points to bare repository in `.forge/repos/linux/`

#### Scenario: Workspace created in repository directory with owner-qualified name

- **WHEN** workspace is created for repository stored with owner-qualified name `torvalds-linux`
- **AND** workspace name is `main`
- **THEN** directory is created at `<forge-root>/torvalds-linux/main/`
- **AND** git metadata points to bare repository in `.forge/repos/torvalds-linux/`

#### Scenario: Workspace created in repository directory with hash-qualified name

- **WHEN** workspace is created for repository stored with hash-qualified name `torvalds-linux-a1b2c3`
- **AND** workspace name is `feat/new-feature`
- **THEN** directory is created at `<forge-root>/torvalds-linux-a1b2c3/feat/new-feature/`
- **AND** git metadata points to bare repository in `.forge/repos/torvalds-linux-a1b2c3/`

#### Scenario: Repository directory created if needed

- **WHEN** creating first workspace for a repository
- **THEN** repository directory `<forge-root>/<repo-dir-name>/` is created
- **AND** repo-dir-name matches the directory name in `.forge/repos/`
- **AND** workspace is created within that directory

#### Scenario: Multiple workspaces share repository directory

- **WHEN** creating multiple workspaces for same repository
- **THEN** all workspaces are created under `<forge-root>/<repo-dir-name>/`
- **AND** each workspace has its own subdirectory
- **AND** repo-dir-name is the human-readable directory name (simple or qualified)

#### Scenario: Workspace path uses human-readable repository name

- **WHEN** user creates workspace named `feature` for repository with simple name `project`
- **THEN** workspace directory is `<forge-root>/project/feature/`
- **AND** path is immediately recognizable and navigable

### Requirement: Duplicate Workspace Prevention

The system SHALL prevent creating workspaces with names that conflict with existing directories.

#### Scenario: Reject workspace with existing workspace name

- **WHEN** user attempts to create workspace named `main` for repository `my-repo`
- **AND** directory `<forge-root>/my-repo/main/` already exists
- **THEN** command fails with error indicating workspace already exists
- **AND** suggests choosing a different name

#### Scenario: Allow same workspace name across different repositories

- **WHEN** user creates workspace named `main` for repository `repo-a`
- **AND** later creates workspace named `main` for repository `repo-b`
- **THEN** both workspaces are created successfully
- **AND** exist as `<forge-root>/repo-a/main/` and `<forge-root>/repo-b/main/`

#### Scenario: Detect conflict with repository directory

- **WHEN** user attempts to create workspace with name matching an existing repository directory
- **AND** that repository directory contains other workspaces
- **THEN** command fails with error indicating name conflict
- **AND** explains the hierarchical structure

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

The system SHALL provide clear feedback when workspace is created successfully, showing human-readable paths.

#### Scenario: Display success message with simple repository name

- **WHEN** workspace is created successfully for repository with simple name
- **THEN** success message confirms the workspace was created
- **AND** shows workspace directory path as `<forge-root>/<name>/<workspace>/`
- **AND** shows the branch that was checked out

#### Scenario: Display success message with qualified repository name

- **WHEN** workspace is created successfully for repository with owner-qualified name
- **THEN** success message confirms the workspace was created
- **AND** shows workspace directory path as `<forge-root>/<owner>-<name>/<workspace>/`
- **AND** path clearly indicates both owner and repository name

#### Scenario: Show next steps with readable path

- **WHEN** workspace is created successfully
- **THEN** message suggests `cd <workspace-path>` to enter the workspace
- **AND** path is human-readable without hash suffixes in common cases

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

