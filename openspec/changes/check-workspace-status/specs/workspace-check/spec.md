# workspace-check Specification

## Purpose

Provide targeted status checks for specific workspace conditions. This command is optimized for scripting and automation, allowing users to check for specific issues (uncommitted changes, unpushed branches, merged branches) without performing all status checks.

## Relationships

- **Complements**: `workspace-status` - while status provides comprehensive reports, check provides targeted queries
- **Uses**: Same underlying status checking functions as `workspace-status`

## ADDED Requirements

### Requirement: Workspace Check Command

The system SHALL provide a command to check workspaces for specific conditions.

#### Scenario: Check all conditions by default

- **WHEN** user runs `forge workspace check`
- **THEN** all status checks are performed (uncommitted, unpushed, merged)
- **AND** results are displayed grouped by check type
- **AND** behavior is similar to `forge workspace status`

#### Scenario: Check via work alias

- **WHEN** user runs `forge work check`
- **THEN** it behaves identically to `forge workspace check`

#### Scenario: Check via wt alias

- **WHEN** user runs `forge wt check`
- **THEN** it behaves identically to `forge workspace check`

### Requirement: Targeted Uncommitted Changes Check

The system SHALL support checking only for uncommitted changes.

#### Scenario: Check for uncommitted changes only

- **WHEN** user runs `forge workspace check --uncommitted`
- **THEN** only uncommitted changes are checked
- **AND** unpushed and merged checks are skipped
- **AND** output shows only workspaces with uncommitted changes

#### Scenario: List workspaces with uncommitted changes

- **WHEN** checking for uncommitted changes
- **THEN** each workspace with uncommitted changes is listed
- **AND** shows workspace path and repository
- **AND** shows summary of changes (modified, staged, untracked)

#### Scenario: No uncommitted changes found

- **WHEN** checking for uncommitted changes
- **AND** no workspaces have uncommitted changes
- **THEN** message indicates all workspaces are clean
- **AND** command exits successfully

### Requirement: Targeted Unpushed Branch Check

The system SHALL support checking only for unpushed branches.

#### Scenario: Check for unpushed branches only

- **WHEN** user runs `forge workspace check --unpushed`
- **THEN** only unpushed branch status is checked
- **AND** uncommitted and merged checks are skipped
- **AND** output shows only workspaces with unpushed branches

#### Scenario: List workspaces with unpushed branches

- **WHEN** checking for unpushed branches
- **THEN** each workspace with unpushed branch is listed
- **AND** shows workspace path, repository, and branch name
- **AND** indicates branch not found on remote

#### Scenario: No unpushed branches found

- **WHEN** checking for unpushed branches
- **AND** all workspace branches are pushed
- **THEN** message indicates all branches are pushed
- **AND** command exits successfully

### Requirement: Targeted Merged Branch Check

The system SHALL support checking only for merged branches.

#### Scenario: Check for merged branches only

- **WHEN** user runs `forge workspace check --merged`
- **THEN** only merged branch status is checked
- **AND** uncommitted and unpushed checks are skipped
- **AND** output shows only workspaces with merged branches

#### Scenario: List workspaces with merged branches

- **WHEN** checking for merged branches
- **THEN** each workspace with merged branch is listed
- **AND** shows workspace path, repository, and branch name
- **AND** indicates which branch it's merged into (e.g., main)

#### Scenario: No merged branches found

- **WHEN** checking for merged branches
- **AND** no workspace branches are merged
- **THEN** message indicates no merged branches
- **AND** command exits successfully

### Requirement: Combined Check Filters

The system SHALL support combining multiple check filters.

#### Scenario: Check for uncommitted and unpushed

- **WHEN** user runs `forge workspace check --uncommitted --unpushed`
- **THEN** both uncommitted and unpushed checks are performed
- **AND** merged check is skipped
- **AND** output shows both categories

#### Scenario: Check all three conditions explicitly

- **WHEN** user runs `forge workspace check --uncommitted --unpushed --merged`
- **THEN** all three checks are performed
- **AND** behavior is identical to `forge workspace check` with no flags

### Requirement: Repository Filtering

The system SHALL support filtering checks to a specific repository.

#### Scenario: Check specific repository for all conditions

- **WHEN** user runs `forge workspace check myrepo`
- **THEN** checks are performed only for workspaces in repository 'myrepo'
- **AND** all status checks are performed (unless filtered by flags)

#### Scenario: Check specific repository for specific condition

- **WHEN** user runs `forge workspace check --uncommitted myrepo`
- **THEN** only uncommitted changes check is performed
- **AND** only for workspaces in repository 'myrepo'

#### Scenario: Repository not found error

- **WHEN** user specifies non-existent repository
- **THEN** command fails with error
- **AND** suggests running `forge repos list` to see available repositories

### Requirement: Exit Status

The system SHALL provide meaningful exit codes for scripting.

#### Scenario: Exit code 0 when no issues found

- **WHEN** check completes successfully
- **AND** no workspaces match the checked conditions
- **THEN** command exits with code 0

#### Scenario: Exit code 0 when issues found

- **WHEN** check completes successfully
- **AND** workspaces match the checked conditions
- **THEN** command exits with code 0
- **AND** matching workspaces are displayed

#### Scenario: Exit code 1 on error

- **WHEN** check encounters an error (invalid repo, forge not found, etc.)
- **THEN** command exits with code 1
- **AND** error message is written to stderr

#### Scenario: Note on exit code design

- **NOTE** Exit code 0 is used even when issues are found because finding issues is a successful outcome of the check operation
- **NOTE** Scripting workflows can parse output or use `--format json` to determine if action is needed
- **NOTE** Only errors in running the check itself result in non-zero exit

### Requirement: Output Formats

The system SHALL support multiple output formats.

#### Scenario: Human-readable text format

- **WHEN** user runs `forge workspace check`
- **THEN** output is formatted as human-readable text
- **AND** uses section headers for each check type
- **AND** uses colors for readability (in interactive terminals)

#### Scenario: JSON format for scripting

- **WHEN** user runs `forge workspace check --format json`
- **THEN** output is valid JSON
- **AND** includes array of matching workspaces
- **AND** includes metadata about which checks were performed
- **AND** is suitable for parsing by scripts

#### Scenario: Quiet mode for scripting

- **WHEN** user runs `forge workspace check --quiet`
- **THEN** only workspace identifiers are printed (one per line)
- **AND** no headers or formatting are included
- **AND** suitable for piping to other commands

### Requirement: Performance Optimization

The system SHALL optimize performance when only specific checks are requested.

#### Scenario: Skip expensive operations for targeted checks

- **WHEN** user runs `forge workspace check --uncommitted`
- **THEN** only local git status is checked (no network operations)
- **AND** remote branch queries are not performed
- **AND** command completes quickly

#### Scenario: Network operations only when needed

- **WHEN** user runs `forge workspace check --uncommitted`
- **THEN** no network operations are performed
- **WHEN** user runs `forge workspace check --unpushed` or `--merged`
- **THEN** network operations may be performed to query remotes

### Requirement: Error Handling

The system SHALL handle errors gracefully during checks.

#### Scenario: Handle network failures for remote checks

- **WHEN** network is unavailable
- **AND** user runs `forge workspace check --unpushed`
- **THEN** command displays warning about network issues
- **AND** reports which checks could not be completed
- **AND** exits with code 1

#### Scenario: Continue on individual workspace errors

- **WHEN** one workspace has corrupted git metadata
- **THEN** error is reported for that workspace
- **AND** other workspaces are still checked
- **AND** command completes with partial results

#### Scenario: Handle missing default branch for merge checks

- **WHEN** repository default branch cannot be determined
- **AND** user runs `forge workspace check --merged`
- **THEN** merge check is skipped for that repository
- **AND** warning is displayed
- **AND** other checks continue

### Requirement: Empty State Handling

The system SHALL provide helpful messaging when no workspaces exist or match.

#### Scenario: No workspaces in Forge

- **WHEN** user runs `forge workspace check` and no workspaces exist
- **THEN** message indicates no workspaces found
- **AND** suggests running `forge workspace create` to create one
- **AND** exits with code 0

#### Scenario: No workspaces match condition

- **WHEN** checking for specific condition
- **AND** no workspaces match that condition
- **THEN** message indicates no matches found
- **AND** exits with code 0

#### Scenario: No workspaces for specified repository

- **WHEN** user filters to repository with no workspaces
- **THEN** message indicates no workspaces for that repository
- **AND** confirms repository exists in Forge
- **AND** exits with code 0
