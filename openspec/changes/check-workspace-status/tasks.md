# Implementation Tasks

## Phase 1: Core Infrastructure

### Task 1: Extend workspace status data structures
- Add `WorkspaceStatusDetails` struct with fields for uncommitted, unpushed, and merged status
- Add fields for storing detailed status information (branch names, merge targets, change summaries)
- Update `Workspace` struct if needed to support new status information
- **Validation**: Unit tests for new data structures
- **Deliverable**: New types in `src/workspace.rs` with documentation

### Task 2: Implement unpushed branch detection
- Add `check_branch_pushed()` function to check if branch exists on any remote
- Use `git branch -r` or `git for-each-ref refs/remotes` to list remote branches
- Handle cases: no remotes, multiple remotes, detached HEAD
- Add caching for remote branch lists per repository (avoid repeated calls)
- **Validation**: Unit tests with mock git repos, integration tests with real repos
- **Deliverable**: Working `check_branch_pushed()` function in `src/workspace.rs`

### Task 3: Implement merged branch detection
- Add `get_default_branch()` function to identify repository default branch
- Add `check_branch_merged()` function using `git merge-base --is-ancestor`
- Handle special cases: workspace is on default branch itself, default branch unknown
- Verify branch is pushed before considering it merged
- **Validation**: Unit tests for merge detection logic, integration tests
- **Deliverable**: Working merge detection functions in `src/workspace.rs`

### Task 4: Implement comprehensive status checking
- Add `get_workspace_status_details()` function that combines all status checks
- Integrate existing `check_workspace_status()` for uncommitted changes
- Add new unpushed and merged checks
- Handle errors gracefully and provide partial results when possible
- **Validation**: Integration tests covering all status dimensions
- **Deliverable**: Unified status checking function with error handling

## Phase 2: Status Command Implementation

### Task 5: Add CLI command structure
- Add `Status` variant to `WorkspaceCommands` enum in `src/main.rs`
- Add command arguments: optional repository filter, `--format` flag
- Add command help text and examples
- **Validation**: Verify command parsing with `cargo test`
- **Deliverable**: CLI command structure ready for implementation

### Task 6: Implement status command handler
- Create handler function for `workspace status` command
- List all workspaces (or filter by repository if specified)
- Call `get_workspace_status_details()` for each workspace
- Aggregate results by status category
- **Validation**: Integration tests for status command
- **Deliverable**: Working status command with repository filtering

### Task 7: Implement text output formatter
- Create formatter for human-readable output
- Group workspaces by status category (uncommitted, unpushed, merged, clean)
- Add section headers and summary counts
- Use colors for interactive terminals (via existing terminal UI code)
- Handle non-interactive mode (strip colors)
- **Validation**: Manual testing in terminal, automated tests for formatting logic
- **Deliverable**: Well-formatted text output for status command

### Task 8: Implement JSON output formatter
- Create JSON serialization for workspace status
- Include all status dimensions in output
- Add summary section with counts
- Ensure valid JSON output
- **Validation**: JSON schema validation, parsing tests
- **Deliverable**: JSON output format for status command

### Task 9: Add progress indication
- Detect long-running operations (>2 seconds)
- Show progress indicator in interactive mode
- Display count of workspaces checked
- Use existing terminal UI components if available
- **Validation**: Manual testing with large number of workspaces
- **Deliverable**: Progress feedback for status command

## Phase 3: Check Command Implementation

### Task 10: Add check command CLI structure
- Add `Check` variant to `WorkspaceCommands` enum
- Add flags: `--uncommitted`, `--unpushed`, `--merged`
- Add `--format` flag and optional repository argument
- Add `--quiet` flag for minimal output
- **Validation**: Verify command parsing
- **Deliverable**: CLI structure for check command

### Task 11: Implement check command handler
- Create handler that processes check flags
- Determine which checks to perform based on flags (default: all)
- Filter workspaces by repository if specified
- Call appropriate status checking functions
- **Validation**: Integration tests for various flag combinations
- **Deliverable**: Working check command with filtering

### Task 12: Implement check output formatters
- Create text formatter for check results
- Create JSON formatter for check results
- Create quiet mode formatter (workspace identifiers only)
- Ensure consistent exit codes (0 for success, 1 for errors)
- **Validation**: Output format tests, exit code tests
- **Deliverable**: Complete output formatting for check command

### Task 13: Optimize performance for targeted checks
- Implement conditional execution of status checks based on flags
- Skip network operations when checking only uncommitted changes
- Add early exit optimizations
- Cache repository-level information (default branches, remote branches)
- **Validation**: Performance benchmarks, verify network calls avoided when appropriate
- **Deliverable**: Optimized check command performance

## Phase 4: Error Handling and Polish

### Task 14: Implement comprehensive error handling
- Add graceful handling for network failures (show warnings, continue)
- Handle corrupted workspaces (report error, continue with others)
- Handle missing default branches (skip merge checks, warn)
- Add timeout handling for network operations
- **Validation**: Error scenario tests, network failure simulation
- **Deliverable**: Robust error handling throughout

### Task 15: Add empty state handling
- Implement helpful messages when no workspaces exist
- Add messages when no workspaces match filters
- Add suggestions (create workspace, check repository name)
- **Validation**: Empty state integration tests
- **Deliverable**: User-friendly empty state messages

### Task 16: Update documentation
- Update command help text with examples
- Add inline code documentation for new functions
- Document exit codes and output formats
- **Validation**: Documentation review, help text clarity
- **Deliverable**: Complete documentation for new commands

## Phase 5: Testing and Validation

### Task 17: Write comprehensive integration tests
- Test status command with various workspace configurations
- Test check command with different flag combinations
- Test repository filtering
- Test output formats (text, JSON, quiet)
- Test error scenarios (network failures, corrupted repos)
- **Validation**: All tests pass, edge cases covered
- **Deliverable**: Comprehensive test suite in `tests/`

### Task 18: Performance testing
- Test with large number of workspaces (50+)
- Measure execution time for various check types
- Verify parallel checking works correctly
- Verify network operations are minimized
- **Validation**: Performance meets requirements (<5s for typical forge)
- **Deliverable**: Performance benchmarks and any necessary optimizations

### Task 19: Manual testing and polish
- Test commands in real-world forge setups
- Verify color output looks good in terminals
- Test with various git configurations (multiple remotes, no remotes, etc.)
- Verify error messages are helpful and actionable
- **Validation**: Manual testing checklist completed
- **Deliverable**: Polished, production-ready commands

## Dependencies

- Tasks 2-4 can be done in parallel after Task 1
- Task 6 depends on Tasks 1-5
- Tasks 7-9 depend on Task 6
- Tasks 11-13 depend on Tasks 1-4, 10
- Task 14 should be integrated throughout but can be finalized after core implementation
- Tasks 17-19 validate all previous tasks

## Estimated Complexity

- **Total tasks**: 19
- **Core infrastructure**: 4 tasks (foundational)
- **Status command**: 5 tasks (comprehensive reporting)
- **Check command**: 4 tasks (targeted checks)
- **Polish and testing**: 6 tasks (quality assurance)

Each task is designed to deliver user-visible progress or testable functionality.
