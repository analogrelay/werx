# Proposal: manage-workspaces

## Overview

This change introduces workspace management commands to the Forge CLI, enabling users to create and manage git worktrees from repositories that have been cloned into the Forge.

## Problem

Currently, Forge can add, list, and remove repositories, storing them as bare clones in `.forge/repos/`. However, users cannot actually work with these repositories - they need a way to create working directories (worktrees) where they can make changes, switch branches, and perform development work. Without this functionality, the repositories in the Forge are essentially inaccessible for day-to-day development.

## Solution

Add workspace management commands that:
1. Create git worktrees from repositories stored in the Forge
2. List existing workspaces and their associated repositories
3. Remove workspaces when they're no longer needed
4. Provide an interactive repository selector with search filtering for easy workspace creation

Workspaces will be created as directories at the Forge root level (e.g., `~/forge/my-workspace/`), while the underlying bare repository remains in `.forge/repos/`. This separation keeps working directories clean and discoverable while maintaining efficient git storage.

## Scope

### In Scope
- `forge workspace create` command to create git worktrees
- `forge workspace list` command to display existing workspaces
- `forge workspace remove` command to delete workspaces
- Interactive repository selector with search/filter capability (using ratatui)
- Auto-generated workspace names with format `[repo-name]@[branch]`
- Interactive name prompt with suggested default
- Support for specifying branch when creating workspace
- Graceful degradation for non-interactive contexts (pipes, scripts)

### Out of Scope
- Scratch directories (non-git workspaces) - deferred to future change
- Workspace switching/activation commands
- Integration with shell prompts or environment variables
- Migration of existing non-worktree directories into Forge
- Workspace templates or initialization scripts

## Dependencies

### Upstream Dependencies
None - this change builds on existing repository management functionality.

### Downstream Implications
This change lays the groundwork for future workspace features:
- Scratch directory support
- Workspace metadata and tags
- Workspace-aware command shortcuts

## User Impact

### New Commands
- `forge workspace create [<repo>] [<branch>]` - Create a new worktree workspace
- `forge workspaces list` / `forge workspace list` - List all workspaces
- `forge workspace remove <workspace>` - Remove a workspace
- `forge worktree` / `forge wt` - Alias for `forge workspace`

### Breaking Changes
None - this is purely additive functionality.

### Migration Path
Not applicable - no existing data to migrate.

## Alternatives Considered

### Alternative 1: Full git clone instead of worktrees
**Rejected** because:
- Wastes disk space with duplicate repository data
- Doesn't share refs/tags between working directories
- Goes against the bare repository design already in place

### Alternative 2: Auto-generate workspace names without user input
**Rejected** because:
- Users often want specific names for their workspaces
- Interactive prompt provides better UX while allowing automation via flags
- Default suggestions make it quick to accept generated names

### Alternative 3: Require workspace names as arguments
**Rejected** because:
- Less discoverable for new users
- Adds friction to the common case of creating a workspace for a branch
- Interactive flow is more consistent with modern CLI best practices

## Open Questions

None - design decisions clarified during scoping.
