## ADDED Requirements

### Requirement: Sync command invocation
The `werx sync` command SHALL accept an optional `<repospec>` positional argument. When a repospec is provided, sync SHALL operate on that single repository. When no repospec is provided, sync SHALL operate on all repositories in the Werx.

#### Scenario: Sync a single repository
- **WHEN** the user runs `werx sync <repospec>`
- **THEN** sync executes only for the matching repository

#### Scenario: Sync all repositories
- **WHEN** the user runs `werx sync` with no arguments
- **THEN** sync executes for every repository in the Werx

#### Scenario: Unknown repospec
- **WHEN** the user provides a repospec that matches no managed repository
- **THEN** werx exits with a clear error message and a non-zero exit code

---

### Requirement: Plan phase is read-only
The sync command SHALL perform a read-only **Plan phase** before making any changes. The Plan phase SHALL fetch remote tracking refs from all configured remotes and analyze all local branches to determine what actions are required. No branch refs, files, or configuration SHALL be modified during the Plan phase.

#### Scenario: Plan phase completes without mutations
- **WHEN** the Plan phase runs
- **THEN** no local branch refs are updated, no pushes occur, and no branches are trashed

#### Scenario: Plan is presented to the user
- **WHEN** the Plan phase completes
- **THEN** the full plan is displayed, listing every planned action per repository and branch

---

### Requirement: Dry-run mode
When `--dry-run` is specified, the sync command SHALL present the plan and then exit without prompting for confirmation and without executing any actions.

#### Scenario: Dry-run shows plan and exits
- **WHEN** the user runs `werx sync --dry-run`
- **THEN** the plan is displayed and the process exits with code 0, with no mutations applied

#### Scenario: Dry-run with no planned actions
- **WHEN** all repositories are already up to date
- **THEN** dry-run reports "nothing to do" and exits cleanly

---

### Requirement: Confirmation prompt
After presenting the plan, sync SHALL prompt the user to confirm before executing any actions. If the user declines, sync SHALL exit without applying any changes.

#### Scenario: User confirms
- **WHEN** the user responds affirmatively to the confirmation prompt
- **THEN** sync proceeds to the Execute phase

#### Scenario: User declines
- **WHEN** the user responds negatively to the confirmation prompt
- **THEN** sync exits with code 0 and no changes are applied

#### Scenario: No-confirm flag skips prompt
- **WHEN** the user runs `werx sync --no-confirm`
- **THEN** the confirmation prompt is skipped and sync proceeds directly to execution after presenting the plan

#### Scenario: Non-interactive context skips prompt
- **WHEN** stdout is not a TTY (e.g., piped output or CI environment)
- **THEN** sync treats the context as `--no-confirm` and proceeds without prompting

---

### Requirement: Remote fetching
The Plan phase SHALL fetch from all configured remotes for each repository. Remotes that do not exist in a given repository SHALL be silently skipped. Tags SHALL be fetched along with refs. Local tags SHALL NOT be pruned automatically.

#### Scenario: Fetch from default remotes
- **WHEN** no `[sync]` configuration is present
- **THEN** sync fetches from `origin` and `upstream` for each repo

#### Scenario: Fetch from configured remotes
- **WHEN** `[sync] remotes = ["origin", "myupstream"]` is set in config
- **THEN** sync fetches from `origin` and `myupstream` for each repo

#### Scenario: Missing remote is skipped silently
- **WHEN** a configured remote does not exist in a repository
- **THEN** sync skips it without error or warning and continues with the remaining remotes

#### Scenario: New tags are fetched
- **WHEN** a remote has tags not present locally
- **THEN** the new tags are fetched and available locally after sync

---

### Requirement: Configurable sync remotes
The Werx configuration SHALL support a `[sync]` table with an optional `remotes` key listing remote names to fetch from. The default value when absent SHALL be `["origin", "upstream"]`.

#### Scenario: Default remotes apply when key absent
- **WHEN** the config file has no `[sync]` table
- **THEN** sync uses `["origin", "upstream"]` as the remote list

#### Scenario: Custom remote list overrides defaults
- **WHEN** `[sync] remotes = ["origin"]` is set
- **THEN** sync fetches only from `origin` and does not attempt `upstream`

---

### Requirement: Branch fast-forward
For each local branch that has a remote tracking branch, the Plan phase SHALL determine whether the local branch can be fast-forwarded (i.e., the local tip is an ancestor of the remote tip). If so, the plan action SHALL be FastForward and the Execute phase SHALL advance the branch ref to the remote tip without creating a working tree.

#### Scenario: Branch fast-forwarded successfully
- **WHEN** the local branch tip is an ancestor of the upstream tip
- **THEN** the branch ref is advanced to the upstream tip

#### Scenario: Fast-forward skipped for active worktree with uncommitted changes
- **WHEN** the branch is checked out in a worktree that has uncommitted changes
- **THEN** the branch is recorded as Skipped with reason "active worktree (dirty)" and no update is applied

#### Scenario: Fast-forward allowed for active worktree that is clean
- **WHEN** the branch is checked out in a clean worktree
- **THEN** the fast-forward is applied normally

---

### Requirement: Branch rebase on diverged history
When a local branch cannot be fast-forwarded but has a remote tracking branch, the Plan phase SHALL record a Rebase action. The Execute phase SHALL create a temporary working tree, perform `git rebase` inside it, and on success advance the bare-repo branch ref. On conflict, the rebase SHALL be aborted and the branch ref SHALL be left in its exact pre-rebase state. A conflicted branch SHALL be recorded as Skipped in the execution summary.

#### Scenario: Rebase succeeds
- **WHEN** a branch cannot fast-forward but rebases cleanly
- **THEN** the branch ref is updated to the rebased tip

#### Scenario: Rebase aborted on conflict
- **WHEN** a rebase produces conflicts
- **THEN** the rebase is aborted, the branch ref is unchanged, and the branch is flagged as "needs manual rebase"

#### Scenario: Branch with active worktree skipped before rebase
- **WHEN** the branch is checked out in any worktree (clean or dirty)
- **THEN** no rebase is attempted and the branch is Skipped with reason "active worktree"

#### Scenario: Temporary worktree is cleaned up on success
- **WHEN** a rebase succeeds
- **THEN** the temporary worktree directory is removed before sync completes

#### Scenario: Temporary worktree is cleaned up on abort
- **WHEN** a rebase is aborted due to conflicts
- **THEN** the temporary worktree directory is removed before sync completes

#### Scenario: Temporary worktree creation fails
- **WHEN** the system cannot create a temporary worktree (e.g., disk full)
- **THEN** the branch is Skipped with an error reason and sync continues with other branches

---

### Requirement: Push local branches
For each local branch that has a configured remote tracking branch, the Execute phase SHALL push the branch to its tracking remote. If the push would require `--force`, the branch SHALL be flagged as Skipped with reason "force-push required" and SHALL NOT be pushed.

#### Scenario: Branch pushed successfully
- **WHEN** a local branch is ahead of its remote tracking branch
- **THEN** the branch is pushed to the tracking remote

#### Scenario: Up-to-date branch not pushed
- **WHEN** a local branch is already at the same commit as its remote tracking branch
- **THEN** no push is performed

#### Scenario: Force-push candidate is flagged and skipped
- **WHEN** a push would require `--force` (remote has diverged)
- **THEN** the branch is Skipped with reason "force-push required" and is included in the attention summary

#### Scenario: Branch with no remote tracking branch is not pushed
- **WHEN** a local branch has no configured upstream remote tracking branch
- **THEN** no push is attempted for that branch

---

### Requirement: Trash stale local branches
When a local branch has a remote tracking branch reference that no longer exists on the remote (deleted upstream), the Execute phase SHALL trash the local branch using the shared `branch_trash()` utility before removing the local branch ref.

#### Scenario: Stale branch is trashed
- **WHEN** a local branch's remote tracking branch has been deleted from the remote
- **THEN** the branch is moved to `werx/trash/<original>/<YYYYMMDD>` and the original branch name is removed

#### Scenario: Active worktree prevents trash
- **WHEN** a stale branch is checked out in any worktree
- **THEN** the branch is Skipped with reason "active worktree" and not trashed

---

### Requirement: Live TUI progress during execution
During the Execute phase, when stdout is a TTY, sync SHALL display an animated mini TUI progress panel showing per-repository status in real time. Each repository row SHALL display a spinner or progress indicator while its operations are in flight and a summary line when complete.

#### Scenario: Progress panel shown in TTY
- **WHEN** stdout is a TTY and execution begins
- **THEN** an animated progress panel is rendered, updating as each repo's operations complete

#### Scenario: Plain text output in non-TTY
- **WHEN** stdout is not a TTY
- **THEN** sync emits plain structured text lines as operations complete, with no ANSI escape codes or interactive elements

#### Scenario: Progress panel replaced by summary on completion
- **WHEN** all repositories have finished
- **THEN** the animated panel is replaced by a final structured text summary of all actions taken and items needing attention

---

### Requirement: Parallel fetch and analysis
The Plan phase SHALL fetch and analyze repositories in parallel to minimize wall-clock time. Operations within a single repository SHALL remain sequential (fetch before analysis, analysis before action planning).

#### Scenario: Multiple repos fetched concurrently
- **WHEN** syncing multiple repositories
- **THEN** fetch operations for different repos run concurrently rather than sequentially

#### Scenario: Per-repo operations remain ordered
- **WHEN** fetching and analyzing a single repository
- **THEN** analysis begins only after the fetch for that repo completes

---

### Requirement: Attention summary
After execution, sync SHALL print a summary section listing all Skipped actions with their reasons, so the user knows exactly what needs manual follow-up.

#### Scenario: Skipped branches reported
- **WHEN** one or more branches were skipped for any reason
- **THEN** the summary lists each skipped branch with its repository, branch name, and reason

#### Scenario: No attention needed
- **WHEN** no branches were skipped
- **THEN** the summary reports that everything is up to date with no items requiring attention
