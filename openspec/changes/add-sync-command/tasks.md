## 1. Foundation & Dependencies

- [x] 1.1 Add `rayon` to `[dependencies]` in `Cargo.toml`
- [x] 1.2 Add `ratatui` to `[dependencies]` in `Cargo.toml` (crossterm already present)
- [x] 1.3 Create `src/trash.rs` with module stub
- [x] 1.4 Create `src/sync.rs` with module stub
- [x] 1.5 Register `trash` and `sync` modules in `src/lib.rs` and export their public APIs

## 2. Branch Trash Utility (`src/trash.rs`)

- [x] 2.1 Define `branch_trash(repo_path: &Path, branch: &str, date: &str) -> Result<String>` — `date` is `YYYYMMDD` string supplied by caller
- [x] 2.2 Implement trash branch name computation: `werx/trash/<original>/<date>`, preserving slashes in the original name
- [x] 2.3 Implement collision detection loop: if the computed trash name already exists, append `-2`, `-3`, etc. until unique
- [x] 2.4 Implement the rename via `git update-ref` (create trash ref pointing to original sha) + `git branch -d` (remove original ref) — works in bare repos
- [x] 2.5 Return an error if the source branch does not exist
- [x] 2.6 Write unit tests: simple branch name, branch with slashes, single collision, multiple collisions, missing source branch

## 3. Config Extension

- [x] 3.1 Add `SyncConfig` struct to `src/config.rs` with `remotes: Option<Vec<String>>`
- [x] 3.2 Add `pub sync: SyncConfig` field to `Config` (default `remotes = None`, meaning use `["origin", "upstream"]`)
- [x] 3.3 Add `Config::sync_remotes() -> &[String]` accessor that returns configured list or the default `["origin", "upstream"]`
- [x] 3.4 Write tests: serialize/deserialize `[sync] remotes`, absent key yields defaults

## 4. Sync Plan Data Model (`src/sync.rs`)

- [x] 4.1 Define `BranchAction` enum with variants: `FastForward { branch, from_sha, to_sha }`, `Rebase { branch, onto_sha }`, `Push { branch, remote }`, `Trash { branch, reason }`, `Skip { branch, reason }`
- [x] 4.2 Define `RepoPlan { repo: String, actions: Vec<BranchAction> }`
- [x] 4.3 Define `SyncPlan { repos: Vec<RepoPlan> }` with `has_mutations() -> bool` and `skipped_actions() -> Vec<(&str, &BranchAction)>` helpers
- [x] 4.4 Implement a `format_plan()` function that renders the `SyncPlan` as a human-readable multi-line string for display in the confirm step

## 5. Plan Phase — Fetch

- [x] 5.1 Implement `fetch_repo(repo_path: &Path, remotes: &[String]) -> Result<()>`: runs `git fetch --tags <remote>` per remote; detects "no such remote" exit and silently skips; propagates other errors
- [x] 5.2 Implement `list_worktrees(repo_path: &Path) -> Result<Vec<WorktreeInfo>>` using `git worktree list --porcelain`; record branch name and whether the worktree has uncommitted changes (`git status --porcelain` inside the worktree path)

## 6. Plan Phase — Branch Analysis

- [x] 6.1 Implement `list_branches_with_upstreams(repo_path: &Path) -> Result<Vec<BranchInfo>>`: for each local branch, record its upstream remote tracking ref (if any), local sha, and upstream sha
- [x] 6.2 Implement fast-forward detection: `git merge-base --is-ancestor <local-sha> <upstream-sha>`
- [x] 6.3 Implement stale-upstream detection: upstream ref was present before fetch but no longer resolves after fetch (remote branch was deleted)
- [x] 6.4 Implement push-needed detection: local sha != upstream sha and local is ahead of upstream
- [x] 6.5 Implement force-push detection: local sha != upstream sha and local is NOT a descendant of upstream (diverged)
- [x] 6.6 Implement active-worktree guard: cross-reference `list_worktrees()` result; a branch checked out anywhere blocks rebase and trash; dirty-worktree blocks even FF
- [x] 6.7 Implement `build_repo_plan(repo_path: &Path, remotes: &[String]) -> Result<RepoPlan>` composing all of the above into a `RepoPlan`

## 7. Execute Phase — Fast Forward

- [x] 7.1 Implement `apply_fast_forward(repo_path: &Path, branch: &str, to_sha: &str) -> Result<()>` via `git update-ref refs/heads/<branch> <to_sha>`
- [x] 7.2 Write unit test verifying the ref is updated and the original commit is unchanged

## 8. Execute Phase — Rebase via Temporary Worktree

- [x] 8.1 Implement RAII `TempWorktree` struct that calls `git worktree add --detach <temp-dir> <branch-sha>` on construction and `git worktree remove --force <temp-dir>` on `Drop`; uses `tempfile::TempDir` for the directory
- [x] 8.2 Implement `apply_rebase(repo_path: &Path, branch: &str, onto_sha: &str) -> Result<RebaseOutcome>` where `RebaseOutcome` is `Success(new_sha)` or `Conflict`
- [x] 8.3 On `Success`: advance bare-repo branch ref via `git update-ref refs/heads/<branch> <new_sha>`
- [x] 8.4 On `Conflict`: run `git rebase --abort` inside temp worktree; leave bare-repo branch ref unchanged
- [x] 8.5 On temp worktree creation failure: return error (caller converts to `Skip`)
- [x] 8.6 Write tests: clean rebase advances ref; conflicting rebase leaves ref unchanged; `TempWorktree` is removed on both paths

## 9. Execute Phase — Push

- [x] 9.1 Define `PushOutcome` enum: `Pushed`, `UpToDate`, `ForcePushRequired`, `NoUpstream`
- [x] 9.2 Implement `push_branch(repo_path: &Path, branch: &str, remote: &str) -> Result<PushOutcome>`: attempt `git push <remote> <branch>`, interpret exit code and stderr to distinguish outcomes; never pass `--force`
- [x] 9.3 Write tests: successful push, already up-to-date, force-push-required detection

## 10. Execute Phase — Trash Stale Branches

- [x] 10.1 Implement `trash_stale_branch(repo_path: &Path, branch: &str, date: &str) -> Result<String>` calling `branch_trash()` from `src/trash.rs`
- [x] 10.2 Write integration test: fetch removes remote tracking ref, branch is correctly identified as stale and moved to trash name

## 11. Plan → Confirm → Execute Orchestration

- [x] 11.1 Implement `run_sync(werx: &Werx, repospec: Option<&str>, dry_run: bool, no_confirm: bool) -> Result<()>` as the top-level sync entry point
- [x] 11.2 Plan phase: resolve repos from `repospec`, call `build_repo_plan()` for each (parallelized in task 12), collect into `SyncPlan`
- [x] 11.3 Present plan using `format_plan()`
- [x] 11.4 Implement `--dry-run` early exit: print plan and return `Ok(())` without prompting
- [x] 11.5 Implement confirmation prompt using `dialoguer::Confirm`; bypass if `--no-confirm` or `!stdout.is_terminal()`
- [x] 11.6 Execute phase: call action executors per plan entry; collect outcomes and skips
- [x] 11.7 Print final summary: actions taken + attention section listing all skips with reasons

## 12. Parallelization

- [x] 12.1 Wrap per-repo `build_repo_plan()` calls in `rayon::par_iter` during Plan phase; collect errors separately so one failing repo doesn't block others
- [x] 12.2 Wrap per-repo action execution in `rayon::par_iter` during Execute phase; actions within a single repo remain sequential
- [x] 12.3 Verify thread-safe error handling: repo errors are accumulated and displayed in summary rather than propagated immediately

## 13. TUI Progress Display

- [x] 13.1 Implement `SyncProgressDisplay` that owns the `ratatui` terminal handle; initializes only when `stdout.is_terminal()`
- [x] 13.2 Implement per-repo row rendering: spinner animation while in-flight, replaced by a one-line summary (✓ / ⚠) on completion
- [x] 13.3 Implement non-TTY fallback: plain `println!`-style lines as each repo completes
- [x] 13.4 Implement final summary panel (replace animated view): list all actions taken and attention items (skipped branches with reasons)
- [x] 13.5 Ensure terminal is restored cleanly on error/panic (use `ratatui`'s built-in restore mechanism)

## 14. Command Integration (`src/main.rs`)

- [x] 14.1 Add `Sync` variant to `Commands` enum with `repospec: Option<String>`, `--dry-run: bool`, `--no-confirm: bool` clap fields
- [x] 14.2 Add match arm for `Commands::Sync` that calls `cmd_sync()`
- [x] 14.3 Implement `cmd_sync(repospec, dry_run, no_confirm)` that resolves the werx and delegates to `run_sync()`

## 15. Tests

- [x] 15.1 Integration test: sync a repo with a fast-forwardable branch; verify branch ref advances
- [x] 15.2 Integration test: `--dry-run` prints plan and makes no changes to any branch ref
- [x] 15.3 Integration test: conflicting rebase leaves branch ref at original sha
- [x] 15.4 Integration test: branch with deleted remote tracking ref is trashed to `werx/trash/...` name
- [x] 15.5 Integration test: branch with active worktree (dirty) is skipped and included in attention summary
- [x] 15.6 Integration test: force-push candidate is skipped and included in attention summary

## 16. Changelog

- [x] 16.1 Add `werx sync [<repospec>]` entry to the `Features Added` section of `CHANGELOG.md` (PR TBD)
