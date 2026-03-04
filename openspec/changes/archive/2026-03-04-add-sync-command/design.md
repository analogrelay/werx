## Context

Werx stores every managed repository as a **bare clone** under `.werx/repos/<name>/`. Workspaces are `git worktree` instances hanging off these bare repos. This architecture means:

- Standard `git rebase` is unavailable inside a bare repo (no working tree); a temporary worktree must be created for any rebase operation.
- Before touching any branch, werx must check `git worktree list` to see if a workspace is currently checked out on it.
- Fast-forward can be done entirely via ref manipulation (`git update-ref`) without a working tree.

The existing codebase uses `crossterm` (already a dependency) and spawns `git` as subprocesses. There is no existing TUI layer; `ratatui` is not yet a dependency.

## Goals / Non-Goals

**Goals:**
- Plan → Confirm → Execute workflow with a safe, inspectable plan before any mutation
- `--dry-run` stops after presenting the plan (no changes, no confirmation prompt)
- `--no-confirm` skips confirmation, goes straight to execution
- Parallelize fetch and analysis operations across repos; within a single repo, operations are sequential
- Live animated TUI progress during execution; graceful degradation to plain output in non-TTY contexts
- Rebase with guaranteed rollback: if a rebase hits conflicts, abort and restore the branch to its exact pre-rebase state
- Shared `branch_trash()` utility used for all auto-cleanup of branches throughout werx

**Non-Goals:**
- Resolving merge/rebase conflicts interactively (out of scope; just flag and skip)
- Pushing new branches with no upstream (user must set `-u` manually)
- Garbage-collecting the `werx/trash/` namespace (deferred to a future `werx trash gc` command)
- Any GUI beyond the terminal

---

## Decisions

### 1. Plan data model

A `SyncPlan` is built during the read-only planning phase and passed through confirmation into execution.

```
SyncPlan
  └─ Vec<RepoPlan>
       ├─ repo: String (dir_name)
       └─ Vec<BranchAction>
            ├─ FastForward { branch, from_sha, to_sha }
            ├─ Rebase      { branch, onto_sha }
            ├─ Push        { branch, remote }
            ├─ Trash       { branch, reason }   // remote tracking gone
            └─ Skip        { branch, reason }   // worktree active / would force-push / conflict
```

`Skip` is the universal "flag to user" action. It is always shown in both the plan and the post-execution summary. All other actions are mutations.

**Rationale**: Separating the plan from execution lets `--dry-run` and `--no-confirm` share identical planning code. The plan is also what gets displayed in the confirmation screen.

---

### 2. Parallelism strategy — `rayon`

The planning phase fetches multiple repos. Each fetch is an independent blocking `git fetch` subprocess call. `rayon`'s parallel iterator (`par_iter`) is the right tool: it spreads blocking calls across a thread pool with no async machinery.

**Execution phase** runs each repo's actions sequentially within that repo (fetch must precede rebase; rebase must precede trash), but repos themselves are independent — `rayon` is used here too.

**Not `tokio`**: Async I/O adds significant complexity and a large dependency; it's not warranted when all I/O is subprocess-based. If async HTTP is needed in the future (e.g., checking CI status), tokio can be added then.

---

### 3. TUI progress display — `ratatui`

Add `ratatui` as a new dependency (it already depends on `crossterm`, which is already present).

The progress display is a mini panel rendered during the Execute phase:

```
Syncing 4 repositories...

  myrepo          ████████░░  fetching...
  other-repo      ██████████  ✓ 3 updated, 1 trashed
  third-repo      ██████████  ✓ up to date
  conflict-repo   ██████████  ⚠ 1 branch needs attention
```

- Each repo row shows a spinner / progress bar while its operations are in flight.
- When a repo completes, the row is replaced with a summary line.
- After all repos complete, the panel is replaced with a structured text summary (same format as today's `println!`-style output elsewhere in the codebase).

**Non-TTY / piped output**: If `stdout` is not a terminal (`!std::io::stdout().is_terminal()`), skip the TUI entirely and emit simple structured text lines as operations complete. This matches the pattern already used in `workspace.rs`.

---

### 4. Rebase in a bare repo — temporary worktree

`git rebase` requires a working tree. The approach:

1. `git worktree add --detach <temp-dir> <branch-sha>` — create a throwaway worktree at the branch tip
2. Inside that worktree, `git rebase <upstream-ref>`
3. **On success**: read the new HEAD sha, clean up the worktree (`git worktree remove --force <temp-dir>`), then `git update-ref refs/heads/<branch> <new-sha>` in the bare repo
4. **On conflict**: `git rebase --abort` inside the worktree, then `git worktree remove --force <temp-dir>` — the bare repo's branch ref is untouched

The temp dir is created under the system temp directory (via `tempfile::TempDir`), not inside the werx directory, to avoid confusing `git worktree list` output.

---

### 5. Fast-forward via `git update-ref`

For branches that can fast-forward (i.e., the local tip is an ancestor of the remote tip):

1. Verify with `git merge-base --is-ancestor <local-sha> <remote-sha>`
2. Update with `git update-ref refs/heads/<branch> <remote-sha>`

No working tree needed. Safe for bare repos.

---

### 6. Active-worktree guard

Before any mutation (FF, rebase, or trash), check `git worktree list --porcelain` in the bare repo. If the branch is checked out in any worktree, emit a `Skip` action with reason `"active worktree"`. No mutation occurs.

For rebase/trash: skip if checked out, regardless of clean/dirty state.
For FF: also skip if the worktree is dirty (has uncommitted changes), since updating the branch ref under a dirty worktree would cause confusion. FF is permitted if the worktree is clean.

---

### 7. `branch_trash()` utility — shared code in `src/trash.rs`

```rust
pub fn branch_trash(repo_path: &Path, branch: &str, date: &str) -> Result<String> {
    // Returns the new trash branch name
    let trash_name = format!("werx/trash/{}/{}", branch, date);
    // git branch -m <branch> <trash_name>  (or update-ref dance for bare repos)
    ...
    Ok(trash_name)
}
```

Branch names with slashes are preserved verbatim in the trash path (git supports hierarchical ref names), e.g., `feature/foo` → `werx/trash/feature/foo/20260227`.

The `date` parameter is always `YYYYMMDD` format, passed in by the caller (makes it testable without mocking system time).

---

### 8. Config schema

New section in `.werx/config.toml`:

```toml
[sync]
remotes = ["origin", "upstream"]   # optional; these are the defaults
```

If the key is absent, defaults apply. If a listed remote doesn't exist in a given repo, it is silently skipped.

---

### 9. Command flags

```
werx sync [<repospec>]
    --dry-run        Build and display the plan; take no action, no confirmation prompt
    --no-confirm     Skip the confirmation prompt; execute immediately after planning
```

With neither flag: plan → show plan → prompt "Proceed? [y/N]" → execute.

---

## Risks / Trade-offs

| Risk | Mitigation |
|------|-----------|
| Temporary worktree creation fails (disk full, permissions) | Treat as a `Skip` with reason "could not create temp worktree"; log the underlying error |
| Rebase abort fails after conflict (leaves dangling worktree) | Log a prominent warning with the temp path so the user can clean up manually |
| `rayon` thread count exhausts OS process limits when many repos | Rayon respects `RAYON_NUM_THREADS`; default thread count is number of CPUs, which is safe in practice |
| TUI output corrupted if a subprocess writes directly to stdout | All git subprocess output is captured (not inherited); only werx writes to the terminal |
| Branch name with characters illegal in some filesystem paths (temp worktree path) | Use a hash or sanitized name for the temp directory path, not the branch name directly |

## Open Questions

- Should `werx sync` also update `HEAD` in the bare repo if the default branch advances (e.g., `main` was rebased)? Currently not planned — `HEAD` in a bare repo is mainly used by `git clone` to pick the default branch, not operationally by werx. Deferred.
- Should the confirmation prompt show a diff-like summary (N branches to update, M to trash) or the full per-branch plan? Lean toward full plan for transparency, but this is a UX call for implementation.
