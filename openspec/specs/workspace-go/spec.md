# workspace-go Specification

## Purpose
TBD - created by archiving change workspace-navigation. Update Purpose after archive.
## Requirements
### Requirement: Go Command

The system SHALL provide a `forge go` command to navigate to workspaces using fuzzy search.

#### Scenario: Launch interactive fuzzy search

- **WHEN** user runs `forge go` with no arguments
- **THEN** interactive fuzzy search interface is launched
- **AND** all workspaces in the forge are available for selection
- **AND** user can type to filter workspaces
- **AND** user can navigate with arrow keys
- **AND** user can select with Enter key
- **AND** user can cancel with Escape key

#### Scenario: Pre-fill fuzzy search query

- **WHEN** user runs `forge go <query>`
- **THEN** interactive fuzzy search interface is launched
- **AND** query field is pre-filled with `<query>`
- **AND** results are filtered by `<query>` initially
- **AND** user can backspace to modify query
- **AND** user can continue typing to refine query

#### Scenario: Direct navigation for single match

- **WHEN** user runs `forge go <query>`
- **AND** exactly one workspace matches `<query>`
- **THEN** fuzzy search interface is NOT launched
- **AND** navigation directive is emitted immediately
- **AND** command completes without user interaction

#### Scenario: Handle no forge initialized

- **WHEN** user runs `forge go` in directory without initialized forge
- **THEN** command fails with error message
- **AND** error indicates forge must be initialized
- **AND** suggests running `forge init`

### Requirement: Fuzzy Workspace Matching

The system SHALL use fuzzy matching to filter workspaces by user query.

#### Scenario: Match repository and workspace name

- **WHEN** user types query in fuzzy search
- **THEN** system matches against `<repository>/<workspace>` format
- **AND** matches are ranked by relevance
- **AND** substring matches are prioritized

#### Scenario: Display workspace context

- **WHEN** workspace is displayed in fuzzy search results
- **THEN** primary text shows `<repository>/<workspace>`
- **AND** secondary text shows branch name
- **AND** secondary text shows full path to workspace
- **AND** format is: `"branch: <branch> | <full-path>"`

#### Scenario: Case-insensitive matching

- **WHEN** user types query in any case combination
- **THEN** matching is case-insensitive
- **AND** `"Main"` matches workspace `"main"`
- **AND** `"FEAT"` matches workspace `"feature-branch"`

### Requirement: Navigation Directive Emission

The system SHALL emit navigation directives to communicate with shell wrapper.

#### Scenario: Emit change directory directive on selection

- **WHEN** user selects workspace in fuzzy search
- **THEN** directive `@forge:change_directory:<path>` is emitted
- **AND** `<path>` is absolute path to workspace
- **AND** directive is output to stderr
- **AND** command exits with code 0

#### Scenario: Emit change directory directive on direct match

- **WHEN** single workspace matches query
- **THEN** directive `@forge:change_directory:<path>` is emitted
- **AND** no fuzzy search UI is shown
- **AND** command exits with code 0

#### Scenario: No directive on cancellation

- **WHEN** user cancels fuzzy search with Escape
- **THEN** no directive is emitted
- **AND** command exits with code 0
- **AND** shell directory remains unchanged

#### Scenario: No directive on error

- **WHEN** command encounters error (no forge, no workspaces)
- **THEN** no directive is emitted
- **AND** error message is printed to stderr
- **AND** command exits with non-zero code

### Requirement: Non-Interactive Behavior

The system SHALL handle non-interactive contexts gracefully.

#### Scenario: Detect non-TTY environment

- **WHEN** `forge go` runs in non-interactive context (pipe, script)
- **AND** query has single match
- **THEN** directive is emitted without launching fuzzy search
- **AND** command succeeds

#### Scenario: Fail gracefully without TTY

- **WHEN** `forge go` runs in non-interactive context
- **AND** query has zero or multiple matches
- **THEN** command fails with error
- **AND** error indicates interactive terminal required
- **AND** suggests running in interactive shell

### Requirement: Fuzzy Search User Experience

The system SHALL provide intuitive fuzzy search interface.

#### Scenario: Display keyboard hints

- **WHEN** fuzzy search interface is active
- **THEN** help text shows keyboard controls
- **AND** indicates Enter to select
- **AND** indicates Escape to cancel
- **AND** indicates arrows to navigate

#### Scenario: Show result count

- **WHEN** query filters workspaces
- **THEN** interface shows count of matching results
- **AND** format is: `"<count> / <total>"`
- **AND** updates in real-time as user types

#### Scenario: Handle empty workspace list

- **WHEN** forge has no workspaces
- **THEN** `forge go` fails with error
- **AND** error indicates no workspaces exist
- **AND** suggests creating workspace with `forge workspace create`

### Requirement: Go Command Alias

The system SHALL support command aliases for convenience.

#### Scenario: Use go via workspace subcommand

- **WHEN** user runs `forge workspace go`
- **THEN** it behaves identically to `forge go`

#### Scenario: Use go via workspaces subcommand

- **WHEN** user runs `forge workspaces go`
- **THEN** it behaves identically to `forge go`

#### Scenario: Use go via wt alias

- **WHEN** user runs `forge wt go`
- **THEN** it behaves identically to `forge go`

