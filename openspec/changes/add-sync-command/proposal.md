## Why

Developers using werx accumulate many repositories over time, and keeping them all current requires tedious manual `git fetch`/`git pull` operations repo-by-repo. A `werx sync` command gives users a single, safe, non-destructive operation that keeps every managed repository's branches up to date with their remotes, pushes local work that has a tracking branch, and prunes branches that have been deleted upstream.

## What Changes

- New `werx sync [<repospec>]` command that operates on one or all managed repos
- Structured **Plan → Confirm → Execute** workflow:
  1. **Plan phase**: Perform safe read-only operations (fetch remote tracking refs, inspect branches) and build a complete action plan
  2. **Confirm phase**: Present the plan to the user for review; skip with `--no-confirm`
  3. **Execute phase**: Carry out all planned actions with a live animated TUI progress display
  - `--dry-run` stops after the Plan phase — presents the plan but takes no action and does not prompt for confirmation
- New configurable list of remotes to fetch (defaulting to `origin` and `upstream`); missing remotes are silently skipped
- Fetch from all configured remotes, including tags (new tags fetched; no auto-pruning of local tags)
- Update local tracking branches using a strict three-step process:
  1. Attempt **fast-forward** — if possible, apply it
  2. If fast-forward is not possible, attempt **rebase** onto the upstream
  3. If rebase produces conflicts, **abort the rebase** and restore the branch to its exact pre-rebase state; flag the branch to the user as needing manual attention
- Skip and flag any branch that has an active worktree with uncommitted changes before attempting rebase or cleanup
- Push branches that have a remote tracking branch; flag (but do not push) any that require `--force`
- Clean up local branches whose remote tracking branch has been deleted; use the shared trash mechanism to preserve them as `werx/trash/<original>/<YYYYMMDD>` before removal
- **Parallelization**: Fetch and per-branch analysis operations are parallelized where safe; sequential ordering is preserved for operations with dependencies (e.g., rebase after fetch)
- Live animated mini TUI progress display during the Execute phase showing per-repo and per-branch status in real time
- New shared `branch_trash()` utility that **all** future auto-cleaning operations in werx must use for safe branch removal

## Capabilities

### New Capabilities

- `repo-sync`: The `werx sync [<repospec>]` command — Plan/Confirm/Execute workflow with fetch, update/rebase tracking branches, push, and prune; `--dry-run` and `--no-confirm` flags; live TUI progress; parallelized operations
- `branch-trash`: Shared utility for non-destructive branch removal; moves branches to `werx/trash/<original>/<YYYYMMDD>` instead of deleting them outright

### Modified Capabilities

<!-- None — no existing spec-level behavior changes -->

## Impact

- **New source files**: `src/sync.rs` (or `src/commands/sync.rs` depending on refactor), plus `src/trash.rs` for the trash utility
- **Modified**: `src/main.rs` (add `Sync` subcommand), `src/config.rs` (add `sync.remotes` config field), `src/lib.rs` (export new public APIs)
- **Config**: New `[sync]` table in `.werx/config.toml` with optional `remotes` list
- **Dependencies**: Will use `ratatui` for the live progress TUI; parallelism via `rayon` or `tokio` (to be decided in design)
- **No breaking changes** to existing commands or APIs
