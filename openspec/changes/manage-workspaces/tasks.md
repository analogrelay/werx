# Tasks: manage-workspaces

## Implementation Tasks

### 1. Add workspace module foundation
- Create `src/workspace.rs` module
- Add workspace module to `src/lib.rs`
- Define `Workspace` struct with fields: name, path, repository, branch, status
- Implement `Forge::workspaces_dir()` helper (returns `self.root`)
- Add unit tests for workspace struct

### 2. Implement workspace discovery
- Add `list_workspaces(&Forge) -> Result<Vec<Workspace>>` function
- Iterate through repositories in `.forge/repos/`
- Execute `git worktree list --porcelain` for each repository
- Parse worktree output to extract path, branch, and status
- Filter worktrees to only include those under Forge root
- Handle git command failures gracefully
- Add unit tests for discovery logic
- Validate: `cargo test` passes

### 3. Add workspace list command
- Add `Workspace` / `Workspaces` command variants to CLI enum
- Add `List` subcommand under workspace commands
- Implement `cmd_workspace_list(format: String)` handler
- Support `--format text|json` flag
- Display workspace name, repository, branch, and status
- Show empty state message when no workspaces exist
- Handle invalid/missing workspaces in listing
- Strip colors in non-interactive contexts
- Validate: `cargo build` succeeds
- Validate: `forge workspace list` displays workspaces correctly

### 4. Implement repository resolution for workspaces
- Add `find_repository(&Forge, &str) -> Result<RepoInfo>` function
- Resolve by full URL, shorthand, or owner/repo format
- Reuse existing URL resolution logic from `repo_spec.rs`
- Return error with helpful message if repository not found
- Add unit tests for resolution logic
- Validate: `cargo test` passes

### 4b. Implement context-aware repository detection
- Add `detect_current_workspace(current_dir: &Path, forge: &Forge) -> Result<Option<RepoInfo>>` function
- Check if current directory is within a workspace under Forge root
- Query git worktree list for each repository to find matching workspace
- Return repository info if workspace is found
- Return None if not in a workspace
- Add unit tests for workspace detection
- Validate: `cargo test` passes

### 5. Implement interactive repository selector
- Add `inquire` or `dialoguer` crate for interactive prompts (check terminal UI requirements)
- Implement `select_repository(&Forge) -> Result<RepoInfo>` function
- Display list of repositories with clone URLs
- Add search/filter capability for clone URLs
- Show default branch information
- Detect interactive vs non-interactive context (check `isatty`)
- Return error in non-interactive contexts
- Validate: `cargo build` succeeds
- Validate: Selector displays and filters correctly in terminal

### 6. Implement workspace path generation and prompting
- Add `generate_workspace_path(repo_name: &str, branch: &str) -> PathBuf` function
- Format: `[repo-name]/[branch]` (hierarchical structure)
- Add `prompt_workspace_name(default: &str) -> Result<String>` function
- Default workspace name is branch name
- Use interactive prompt with default suggestion
- Support `--name` flag to skip prompt
- Auto-use branch name in non-interactive contexts
- Add unit tests for path generation
- Validate: `cargo test` passes

### 7. Implement git worktree creation with hierarchical structure
- Add `create_worktree(&Forge, &RepoInfo, &str, &str) -> Result<PathBuf>` function
- Parameters: forge, repository, workspace name, branch
- Create repository directory `<forge-root>/[repo-name]/` if needed
- Check for workspace path conflicts at `<forge-root>/[repo-name]/[workspace-name]/`
- Execute `git worktree add <path> <branch>` on bare repository
- Handle git command failures
- Clean up partially created directories on failure
- Add unit tests with temporary directories
- Validate: `cargo test` passes

### 8. Add workspace create command with context awareness
- Add `Create` subcommand with optional `repo` and `branch` arguments
- Add `--name` flag for custom workspace names
- Implement `cmd_workspace_create(repo: Option<String>, branch: Option<String>, name: Option<String>)` handler
- Detect current workspace and use its repository if repo not provided
- Call interactive selector if repo not provided and not detected (interactive only)
- Use repository's default branch if branch not specified
- Call name prompt if name not provided (interactive only, default to branch name)
- Call `create_worktree` to perform creation with hierarchical path
- Display success message with full hierarchical path and next steps
- Handle all error cases with helpful messages
- Validate: `cargo build` succeeds
- Validate: `forge workspace create` works end-to-end
- Validate: Context detection works when running from within a workspace

### 9. Add workspace confirmation and status checking
- Add `check_workspace_status(path: &Path) -> Result<WorkspaceStatus>` function
- Check for uncommitted changes using `git status --porcelain`
- Return enum with Clean, Modified, or Untracked states
- Add `confirm_workspace_removal(name: &str, status: WorkspaceStatus) -> Result<bool>` function
- Prompt with workspace details and change warnings
- Support `--force` flag to skip confirmation
- Handle non-interactive contexts
- Validate: `cargo test` passes

### 10. Implement workspace removal with hierarchical cleanup
- Add `remove_workspace(&Forge, path: &str) -> Result<()>` function
- Parse workspace path (support both `repo/workspace` and `workspace` with context)
- Find workspace by path in discovered workspaces
- Handle ambiguous workspace names (multiple repos with same workspace name)
- Execute `git worktree remove <path>` on parent repository
- Remove workspace directory
- Clean up empty repository directories after removal
- Handle orphaned metadata (directory missing but git metadata exists)
- Prune with `git worktree prune` if needed
- Add unit tests for removal logic
- Validate: `cargo test` passes

### 11. Add workspace remove command
- Add `Remove` / `Rm` subcommand with `workspace` argument
- Add `--force` flag
- Implement `cmd_workspace_remove(workspace: String, force: bool)` handler
- Check workspace status before removal
- Call confirmation prompt unless `--force` or non-interactive
- Call `remove_workspace` to perform removal
- Display success message
- Handle all error cases
- Validate: `cargo build` succeeds
- Validate: `forge workspace remove` works end-to-end

### 12. Add command aliases
- Add `wt` and `worktree` as top-level command aliases for `workspace`
- Add `workspaces` as alternative to `workspace`
- Update CLI help text
- Validate: `forge wt create`, `forge worktree list`, etc. work correctly

### 13. Integration testing
- Write integration tests in `tests/workspace_integration.rs`
- Test workspace creation with various repository specs
- Test workspace listing output formats
- Test workspace removal with and without force
- Test error cases (missing repo, invalid branch, etc.)
- Test non-interactive behavior
- Validate: `cargo test --test workspace_integration` passes

### 14. Documentation and polish
- Update help text for all workspace commands
- Ensure error messages are clear and actionable
- Test all commands manually
- Verify graceful degradation in non-interactive contexts
- Run `cargo fmt` and `cargo clippy`
- Validate: All clippy warnings addressed

## Validation Checklist

- [ ] All unit tests pass (`cargo test`)
- [ ] All integration tests pass
- [ ] Commands work in interactive terminals
- [ ] Commands work in non-interactive contexts (pipes, scripts)
- [ ] Error messages are helpful and actionable
- [ ] Success messages provide clear feedback
- [ ] Code follows project conventions (rustfmt, clippy)
- [ ] No regression in existing functionality
- [ ] Performance is acceptable (no noticeable lag in UI)

## Dependencies

- Task 2 depends on Task 1
- Task 3 depends on Task 2
- Task 4b depends on Task 2
- Task 5 depends on Task 1
- Task 6 depends on Task 1
- Task 7 depends on Task 4
- Task 8 depends on Tasks 4, 4b, 5, 6, 7
- Task 9 depends on Task 1
- Task 10 depends on Task 9
- Task 11 depends on Task 10
- Task 13 depends on Tasks 8, 11
- Task 14 depends on all previous tasks

## Parallel Work Opportunities

- Tasks 2, 4, 5, 6 can be worked on in parallel after Task 1
- Task 4b can be worked on in parallel with Tasks 5, 6
- Tasks 9 and 7 can be worked on in parallel after Task 4
- Tasks 3 and 8 can be tested independently
