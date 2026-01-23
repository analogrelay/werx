# workspace-status Specification

## Purpose

Provide a comprehensive status report of all workspaces in the Forge, showing uncommitted changes, unpushed branches, merged branches, and clean workspaces. This enables users to quickly audit their workspace state and identify work that needs attention or is ready for cleanup.

## ADDED Requirements

### Requirement: Workspace Status Command

The system SHALL provide a command to display comprehensive status information for all workspaces.

#### Scenario: Display status via workspace status command

- **WHEN** user runs `forge workspace status`
- **THEN** system displays status report for all workspaces in the Forge
- **AND** groups workspaces by status category (uncommitted changes, unpushed branches, merged branches, clean)
- **AND** provides count summary for each category

#### Scenario: Display status via work status alias

- **WHEN** user runs `forge work status`
- **THEN** it behaves identically to `forge workspace status`

#### Scenario: Display status via wt status alias

- **WHEN** user runs `forge wt status`
- **THEN** it behaves identically to `forge workspace status`

### Requirement: Repository Filtering

The system SHALL support filtering status checks to a specific repository.

#### Scenario: Check status for specific repository by name

- **WHEN** user runs `forge workspace status myrepo`
- **THEN** only workspaces belonging to repository 'myrepo' are included in status report
- **AND** status report header indicates filtering is active

#### Scenario: Check status for specific repository by URL

- **WHEN** user runs `forge workspace status github.com/user/myrepo`
- **THEN** only workspaces belonging to that repository are included
- **AND** repository is resolved using standard repository specification parsing

#### Scenario: Error when repository not found

- **WHEN** user specifies a repository that doesn't exist in the Forge
- **THEN** command fails with error message indicating repository not found
- **AND** suggests running `forge repos list` to see available repositories

### Requirement: Uncommitted Changes Detection

The system SHALL identify workspaces with uncommitted changes.

#### Scenario: Report workspaces with modified files

- **WHEN** workspace has modified tracked files
- **THEN** workspace appears in "Uncommitted Changes" section
- **AND** shows summary of modified files

#### Scenario: Report workspaces with staged changes

- **WHEN** workspace has changes staged for commit
- **THEN** workspace appears in "Uncommitted Changes" section
- **AND** indicates staged changes exist

#### Scenario: Report workspaces with untracked files

- **WHEN** workspace has untracked files
- **THEN** workspace appears in "Uncommitted Changes" section
- **AND** shows untracked file indicators

#### Scenario: Clean workspaces excluded from uncommitted section

- **WHEN** workspace has no uncommitted changes
- **THEN** workspace does not appear in "Uncommitted Changes" section
- **AND** may appear in "Clean Workspaces" section if no other issues exist

### Requirement: Unpushed Branch Detection

The system SHALL identify workspaces whose branches exist only locally.

#### Scenario: Report branches not on any remote

- **WHEN** workspace branch does not exist on any configured remote
- **THEN** workspace appears in "Unpushed Branches" section
- **AND** indicates branch name not found on remote

#### Scenario: Report when no remotes configured

- **WHEN** repository has no remotes configured
- **THEN** all workspace branches are considered unpushed
- **AND** status report indicates no remotes available

#### Scenario: Branch pushed to any remote counts as pushed

- **WHEN** workspace branch exists on at least one remote
- **THEN** branch is considered pushed
- **AND** workspace does not appear in "Unpushed Branches" section

#### Scenario: Handle detached HEAD state

- **WHEN** workspace is in detached HEAD state
- **THEN** workspace is reported separately as "detached"
- **AND** does not appear in unpushed branches section

### Requirement: Merged Branch Detection

The system SHALL identify workspaces targeting branches that are fully merged to the default branch.

#### Scenario: Report fully merged branches

- **WHEN** workspace branch is fully merged to default branch (e.g., main)
- **AND** both local and remote are up to date
- **THEN** workspace appears in "Merged Branches" section
- **AND** indicates which branch it's merged into

#### Scenario: Exclude default branch itself

- **WHEN** workspace is on the default branch (main, master, develop)
- **THEN** it never appears in "Merged Branches" section
- **AND** may appear in "Clean Workspaces" if no other issues

#### Scenario: Handle branches not fully merged

- **WHEN** workspace branch has commits not in default branch
- **THEN** workspace does not appear in "Merged Branches" section

#### Scenario: Skip merge check when default branch unknown

- **WHEN** default branch cannot be determined
- **THEN** merge status check is skipped for that repository
- **AND** warning is displayed in output

### Requirement: Clean Workspaces Reporting

The system SHALL identify and report workspaces with no status issues.

#### Scenario: Report clean workspaces

- **WHEN** workspace has no uncommitted changes
- **AND** branch is pushed to remote
- **AND** branch is not fully merged (or is the default branch)
- **THEN** workspace appears in "Clean Workspaces" section

#### Scenario: Collapsed clean workspace display

- **WHEN** displaying clean workspaces
- **THEN** list is displayed in compact format
- **AND** shows count of clean workspaces
- **AND** optionally shows full list with `--verbose` flag

### Requirement: Output Format Options

The system SHALL support multiple output formats for status reports.

#### Scenario: Human-readable text format

- **WHEN** user runs `forge workspace status`
- **THEN** output is formatted as human-readable text
- **AND** uses section headers for each status category
- **AND** uses colors for readability (in interactive terminals)
- **AND** includes summary counts

#### Scenario: JSON format for scripting

- **WHEN** user runs `forge workspace status --format json`
- **THEN** output is valid JSON
- **AND** includes array of workspace objects with status fields
- **AND** includes summary object with counts
- **AND** is suitable for parsing by scripts

#### Scenario: Non-interactive output strips colors

- **WHEN** running in non-interactive context (pipe, redirect)
- **THEN** output contains no ANSI color codes
- **AND** formatting is plain text

### Requirement: Performance and Progress

The system SHALL provide reasonable performance and progress feedback.

#### Scenario: Show progress for long operations

- **WHEN** checking status takes more than 2 seconds
- **THEN** progress indicator is shown (in interactive mode)
- **AND** indicates number of workspaces checked

#### Scenario: Parallel status checking

- **WHEN** checking multiple workspaces
- **THEN** status checks are performed concurrently where possible
- **AND** results are aggregated before display

#### Scenario: Reasonable performance

- **WHEN** checking typical Forge (10 repos, 20 workspaces)
- **THEN** command completes in under 5 seconds
- **AND** does not require network operations for uncommitted changes check

### Requirement: Error Handling

The system SHALL handle errors gracefully during status checking.

#### Scenario: Handle network failures

- **WHEN** network operations fail (checking remote branches)
- **THEN** command continues checking other status dimensions
- **AND** displays warning about network-dependent checks
- **AND** completes with partial results

#### Scenario: Handle corrupted workspace

- **WHEN** workspace directory or git metadata is corrupted
- **THEN** workspace is marked with error status
- **AND** error details are shown
- **AND** other workspaces are still checked

#### Scenario: Handle missing workspace directories

- **WHEN** workspace exists in git metadata but directory is missing
- **THEN** workspace is reported as "prunable"
- **AND** other status checks are skipped for that workspace

### Requirement: Empty State Handling

The system SHALL provide helpful messaging when no workspaces exist.

#### Scenario: No workspaces in Forge

- **WHEN** user runs `forge workspace status` and no workspaces exist
- **THEN** message indicates no workspaces found
- **AND** suggests running `forge workspace create` to create one

#### Scenario: No workspaces for specified repository

- **WHEN** user filters to a repository with no workspaces
- **THEN** message indicates no workspaces for that repository
- **AND** confirms repository exists in Forge
