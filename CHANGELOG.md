# Release History

## 0.1.0 (Unreleased)

- Initial Release

### Features Added

- Added `werx config list/get/set/delete` commands for dotted-path config read/write (PR TBD)
- Added `werx status` shorthand command that dispatches to `werx workspace status` (PR TBD)
- Redesigned `werx workspace status` to display repos as top-level with fork annotations and per-workspace branch status (PR TBD)
- Fixed branch naming to strip issue-number prefix from AI-generated slugs, preventing duplication like `user/1234-1234-fix-bug` (PR TBD)
- Added `werx go #N` / `werx go N` to navigate directly to a workspace by issue or PR number (PR TBD)
- Added `werx on [REPO] #N` command to navigate to or create a workspace for a GitHub issue or PR in one step (PR TBD)
- Added `--build` and `--release` modes to `script/check` for flexible CI validation

### Features Added

- Added global `--verbose` / `-v` flag that enables `werx=debug` tracing and verbose git output (PR TBD)
- Added animated progress spinners (via `indicatif`) during `werx add` and `werx sync`, showing scrolling git output in TTY mode and plain lines in non-TTY/piped mode (PR TBD)
- Added colored plan and summary output to `werx sync` using `console::style()`: repo names in cyan/bold, fast-forwards with green `^`, pushes with blue `->`, trash with yellow, skips dimmed (PR TBD)
- Added colored success message to `werx add` (green bold header, cyan spec/location) (PR TBD)

- Fixed namedata module structure by converting to a single Rust file
- Resolved clippy warnings and formatting issues
- Added `werx sync [<repospec>]` command with Plan → Confirm → Execute workflow: fetch remotes, fast-forward/rebase tracking branches, push local branches, and trash stale branches; `--dry-run` and `--no-confirm` flags; live progress display; parallelized across repos (PR TBD)
- Added shared `branch_trash()` utility in `src/trash.rs` for safe branch removal to `werx/trash/<original>/<YYYYMMDD>` (PR TBD)
- Added GitHub fork tracking: on `werx add`, detect if the cloned repo is a GitHub fork via the `gh` CLI, persist fork metadata to `werx-repo.toml` beside the bare clone, and automatically configure an `upstream` remote pointing to the parent repo (PR TBD)
- Added upstream-aware sync: `werx sync` now fast-forwards fork branches from `upstream/<branch>` before pushing to `origin`; branches that have diverged from upstream are marked as skipped with a "needs manual rebase" note (PR TBD)
- Added `werx wt create <repo> #<N>` to create worktrees directly from GitHub issue or PR numbers; PR references check out the PR HEAD branch, issue references generate a branch name via the configured naming pattern and optionally invoke a coding agent for the slug (PR TBD)
- `werx wt create <fork-repo> #<N>` now searches both the fork and its upstream repo for issues; if found in both, the user is prompted to choose; if found only in upstream, it is used automatically (PR TBD)
- Fixed `gh repo view` parsing: `parent` JSON now uses `name`/`owner.login` fields instead of the missing `nameWithOwner` field (PR TBD)
- Added branch naming service (`src/branch_naming.rs`): `username/[N-]topic` pattern, GitHub username auto-detection via `gh api user` with caching in `werx.toml`, and AI-assisted slug generation via Claude or GitHub Copilot CLI (PR TBD)

### Breaking Changes

- Removed `werx agent` command group and all subcommands (`spawn`, `list`, `status`, `attach`, `kill`, `providers`) (PR TBD)
- Removed `[agents]` configuration section from `.werx/config.toml`; existing configs with this section will have it silently ignored (PR TBD)
- Removed `exec` and `rand` dependencies that were only used by agent management (PR TBD)

### Other Changes

- Added `tracing`/`tracing-subscriber` for structured diagnostics; enable with `WERX_LOG=debug` (git commands) or `WERX_LOG=trace` (full stdout/stderr); defaults to silent (PR TBD)
- Added Nix overlay output (`overlays.default`) for easy integration with NixOS and nix-darwin configurations (PR TBD)
- Added crates.io publishing step to release workflow
- Added pre-release validation for duplicate GitHub Releases, crates.io versions, and CARGO_REGISTRY_TOKEN

### Bugs Fixed

- Fixed shell hook hanging by replacing stdout-scraping directive protocol with a temp file (`WERX_DIRECTIVE_FILE`) (PR TBD)
- Fixed infinite recursion in `werx` shell function caused by function self-invocation instead of calling the binary (PR TBD)
- Added `rust-toolchain.toml` for consistent Rust version management
- Added `TestContext` for isolated test environments with pre-configured git
- Extracted reusable GitHub workflows (`_build.yml`, `_nix-build.yml`) to reduce CI duplication
- Added Nix build job to CI and release workflows
- Updated AGENTS.md with changelog maintenance instructions
