# check-workspace-status

## Problem Statement

Developers working with multiple Git worktrees across multiple repositories need visibility into the state of their workspaces to make informed decisions about when work is complete and safe to clean up. Currently, users must manually check each workspace individually using git commands to determine:

- Which workspaces have uncommitted changes
- Which workspaces have branches that exist only locally (not pushed to remote)
- Which workspaces are targeting branches that have been fully merged and pushed

This manual process is time-consuming and error-prone, especially when managing many workspaces across multiple repositories in a Forge.

## Proposed Solution

Add `forge workspace status` and `forge workspace check` commands that provide comprehensive visibility into workspace states across the entire Forge or filtered to a specific repository. These commands will:

1. **Report uncommitted changes**: Identify workspaces with modified, staged, or untracked files
2. **Report unpushed branches**: Identify workspaces whose branches exist only locally
3. **Report merged branches**: Identify workspaces targeting branches that are fully merged to the main branch and pushed

The solution will support:
- Checking all workspaces in the Forge
- Filtering to a specific repository
- Multiple output formats (human-readable and JSON for scripting)
- Clear, actionable information to support pruning workflows

## User Impact

### Positive Impact
- **Improved visibility**: Users can quickly audit their entire workspace state
- **Safer cleanup**: Users can confidently identify which workspaces are safe to remove
- **Reduced cognitive load**: No need to manually track workspace states
- **Better workflow support**: Enables automated pruning and cleanup workflows
- **Scriptable**: JSON output enables integration with other tools

### Potential Concerns
- Performance: Checking many workspaces may be slow (mitigated by showing progress)
- Network calls: Checking remote branch status requires network access (show warnings for network issues)

## Alternatives Considered

### 1. Extend `forge workspace list` with additional status fields
**Rejected**: The list command is for discovery; status checking is a distinct analysis operation with different performance characteristics and output needs.

### 2. Single `forge workspace check` command with flags for different checks
**Rejected**: Having separate semantic concerns (uncommitted changes vs. remote branch status vs. merge status) grouped under one command makes the interface less clear. A `status` subcommand can show all checks together, while individual checks could be flags.

### 3. Automatic cleanup without status checking
**Rejected**: Too dangerous without user visibility and control. Status checking is a prerequisite for safe automated cleanup.

## Implementation Approach

### Phase 1: Core Status Checking (this proposal)
- Implement `workspace status` command showing all status dimensions
- Implement `workspace check` command with optional filters (uncommitted, unpushed, merged)
- Support repository filtering
- Support JSON output format

### Phase 2: Pruning Features (future proposal)
- Interactive pruning workflows
- Automated cleanup based on status criteria
- Safety confirmations and dry-run modes

## Dependencies

- Requires existing `workspace list` functionality (already implemented)
- Uses git commands: `git status`, `git branch -r`, `git merge-base`, `git rev-list`

## Success Criteria

1. Users can quickly identify workspaces with uncommitted changes across all repositories
2. Users can identify workspaces with unpushed branches
3. Users can identify workspaces targeting merged branches
4. Commands complete in reasonable time (< 2s for typical forge with 10 repos and 20 workspaces)
5. JSON output is parseable and complete
6. Commands fail gracefully when network is unavailable or repositories are in unexpected states
