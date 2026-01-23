# Design: Workspace Status Checking

## Overview

This design introduces workspace status checking commands that provide visibility into workspace states across the Forge. The design focuses on three key status dimensions: uncommitted changes, unpushed branches, and merged branches.

## Architecture

### Command Structure

Two complementary commands will be added:

1. **`forge workspace status [REPO]`**: Comprehensive status report showing all status dimensions for all workspaces (optionally filtered to a repository)

2. **`forge workspace check [--uncommitted] [--unpushed] [--merged] [REPO]`**: Targeted checks for specific conditions, useful for scripting and focused audits

### Status Dimensions

#### 1. Uncommitted Changes
- **What it checks**: Modified files, staged changes, untracked files
- **Git operation**: `git status --porcelain`
- **Use case**: Identify workspaces with work in progress
- **Already implemented**: Yes, in `check_workspace_status()` function

#### 2. Unpushed Branches
- **What it checks**: Whether the current branch exists on any remote
- **Git operations**:
  - `git branch -r` - list remote branches
  - `git for-each-ref refs/remotes` - get remote branch references
  - Compare with current branch from worktree
- **Use case**: Identify work that hasn't been backed up to remote
- **Edge cases**:
  - No remotes configured (treat as unpushed)
  - Detached HEAD state (skip or report separately)
  - Multiple remotes (branch pushed to any remote counts as pushed)

#### 3. Merged Branches
- **What it checks**: Whether the branch is fully merged to the main/default branch
- **Git operations**:
  - Identify default branch: check repository config or use `git symbolic-ref refs/remotes/origin/HEAD`
  - Check if merged: `git merge-base --is-ancestor <branch> <default-branch>`
  - Verify pushed: compare local and remote branch refs
- **Use case**: Identify completed work that's safe to clean up
- **Edge cases**:
  - Branch is main/default itself (never report as merged)
  - Main branch not yet pushed (skip merge check)
  - No remote tracking (can't verify if pushed)

### Data Flow

```
1. List all workspaces (or filter by repo)
   ↓
2. For each workspace:
   - Get current branch
   - Check uncommitted changes (existing function)
   - Check if branch exists on remote (new)
   - Check if branch is merged (new)
   ↓
3. Aggregate results by status dimension
   ↓
4. Format output (text or JSON)
   ↓
5. Display to user
```

### Output Formats

#### Text Format (Human-Readable)

```
Workspace Status for Forge

Uncommitted Changes (2 workspaces):
  myrepo/feature-auth      M src/auth.rs, ?? tests/
  otherrepo/bugfix-123     M README.md

Unpushed Branches (3 workspaces):
  myrepo/feature-wip       Branch 'feature-wip' not on any remote
  myrepo/local-test        Branch 'local-test' not on any remote
  otherrepo/experiment     Branch 'experiment' not on any remote

Merged Branches (1 workspace):
  myrepo/old-feature       Branch 'old-feature' merged to main

Clean Workspaces (5 workspaces):
  myrepo/main
  myrepo/feature-new
  otherrepo/main
  otherrepo/develop
  thirdrepo/main
```

#### JSON Format (Machine-Readable)

```json
{
  "workspaces": [
    {
      "name": "feature-auth",
      "path": "/home/user/forge/myrepo/feature-auth",
      "repository": "myrepo",
      "branch": "feature-auth",
      "uncommitted_changes": true,
      "unpushed_branch": false,
      "merged_branch": false,
      "status_details": {
        "modified_files": ["src/auth.rs"],
        "untracked_files": ["tests/auth_test.rs"]
      }
    },
    {
      "name": "old-feature",
      "path": "/home/user/forge/myrepo/old-feature",
      "repository": "myrepo",
      "branch": "old-feature",
      "uncommitted_changes": false,
      "unpushed_branch": false,
      "merged_branch": true,
      "merge_details": {
        "merged_into": "main",
        "remote_tracking": "origin/old-feature"
      }
    }
  ],
  "summary": {
    "total": 11,
    "uncommitted": 2,
    "unpushed": 3,
    "merged": 1,
    "clean": 5
  }
}
```

## New Functions

### In `src/workspace.rs`

```rust
/// Extended workspace status with remote tracking information
pub struct WorkspaceStatusDetails {
    pub uncommitted_changes: bool,
    pub unpushed_branch: bool,
    pub merged_branch: bool,
    pub branch_name: Option<String>,
    pub default_branch: Option<String>,
}

/// Check if a workspace's branch exists on any remote
pub fn check_branch_pushed(workspace_path: &Path, branch: &str) -> Result<bool>

/// Check if a workspace's branch is merged to the default branch
pub fn check_branch_merged(
    workspace_path: &Path,
    branch: &str,
    default_branch: &str
) -> Result<bool>

/// Get the default branch for a repository
pub fn get_default_branch(repo_path: &Path) -> Result<String>

/// Get comprehensive status for a workspace
pub fn get_workspace_status_details(workspace: &Workspace) -> Result<WorkspaceStatusDetails>
```

## Performance Considerations

1. **Parallelization**: Check multiple workspaces concurrently using thread pool
2. **Caching**: Cache remote branch lists per repository (avoid repeated `git ls-remote` calls)
3. **Progress indication**: Show progress for long-running operations
4. **Timeout**: Set reasonable timeouts for network operations
5. **Early exit**: For `check` command with specific filters, skip unnecessary checks

## Error Handling

- Network failures: Report warnings but continue checking other workspaces
- Missing branches: Handle detached HEAD and missing default branches gracefully
- Git command failures: Log errors but don't halt entire operation
- Permission issues: Report and skip inaccessible workspaces

## Testing Strategy

1. **Unit tests**: Test individual status checking functions with mock git repos
2. **Integration tests**: Test full status command with real git repositories
3. **Edge case tests**: Test detached HEAD, no remotes, corrupted repos
4. **Performance tests**: Verify reasonable performance with many workspaces

## Future Extensions

1. **Filtering**: `--uncommitted-only`, `--unpushed-only`, `--merged-only` flags
2. **Watch mode**: Continuous monitoring of workspace status
3. **Notifications**: Alert when branches become merged
4. **Integration**: Export status to other tools or dashboards
