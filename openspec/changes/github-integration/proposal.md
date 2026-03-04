## Why

Werx manages repos and worktrees but has no awareness of GitHub or fork relationships. Developers working in forks need to track upstream remotes, keep branches in sync with upstream, and create worktrees directly from GitHub issue/PR references rather than manually specifying branch names.

## What Changes

- Werx detects whether a managed repo is a GitHub fork and tracks its upstream repository
- A managed `upstream` remote is established for fork repos, pointing at the parent repo
- `werx sync` gains upstream-aware branch sync: branches that exist in `upstream` are fast-forwarded from upstream before being pushed to `origin`
- `werx wt create` accepts GitHub issue/PR references (`#1234`) in addition to branch names
  - Issues: finds or creates a work branch for the issue, then creates a worktree
  - PRs: checks out the PR's HEAD branch into a worktree
- Branch naming follows a configurable strategy; the default pattern is `<username>/[<issue#>-]<topic>`
- GitHub username is detected from the GitHub API (via token or `gh` CLI) and cached in werx config

## Capabilities

### New Capabilities

- `fork-tracking`: Detect if a managed repo is a GitHub fork; ensure an `upstream` remote exists pointing at the parent repo; surface fork metadata (parent repo URL, default branch) for use by other capabilities
- `branch-naming`: Configurable branch naming strategy stored in werx config; default pattern is `<username>/[<issue#>-]<topic>`; provides a naming service used when creating branches for issues

### Modified Capabilities

- `repo-sync`: When a repo has an `upstream` remote (fork repos), branches that also exist in upstream SHALL be fast-forwarded from upstream before the normal push-to-origin step
- `workspace-create`: `werx wt create` SHALL accept a `#<number>` argument in addition to branch names; resolve the number as a GitHub issue or PR and create the worktree accordingly

## Impact

- **New dependency**: GitHub API client (likely via `octocrab` crate or `gh` CLI subprocess)
- **Config**: `werx.toml` gains `[github]` table with `username` field and `[branch-naming]` table with `pattern` field
- **Code**: `src/commands/sync.rs` — upstream sync logic; `src/commands/workspace/create.rs` — issue/PR resolution; new module `src/github/` for API client and fork detection
- **Existing specs affected**: `repo-sync`, `workspace-create`
