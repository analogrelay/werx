# Change: Add Coding Agent Integration

## Why

Developers increasingly use AI coding agents (OpenCode, Claude Code, GitHub Copilot CLI) to assist with software development. Currently, using these agents with a repository requires manual setup: creating worktrees, launching terminal sessions, and managing multiple agent instances. Forge already manages repositories and worktrees; extending it to orchestrate coding agents is a natural evolution that streamlines the developer workflow.

By integrating coding agent management into Forge, users can:

- Spawn isolated agent instances with dedicated worktrees in a single command
- Run multiple agents concurrently on the same or different repositories
- Easily switch between agent sessions without losing context
- Mix and match different agent tools based on preference or requirement

## What Changes

- **New `forge agent` subcommand** with operations: `spawn`, `list`, `attach`, `kill`
- **Agent spawning creates dedicated worktrees** - each spawned agent gets its own isolated worktree for the target repository
- **tmux-based session management** - all agents run in a shared `forge-agents` tmux session, each in its own window
- **Multi-agent support** - OpenCode (default/preferred), Claude Code, and GitHub Copilot CLI
- **Agent auto-detection** - discovers available agents by checking `$PATH`
- **Configurable agent preferences** - global defaults and per-repo overrides (without repo-local config files)
- **Optional initial prompts** - pass a task to the agent at spawn time via CLI or `$EDITOR`

## Impact

- **Affected specs**: None (new capability)
- **New specs**:
  - `agent-spawn` - spawning agents with worktrees
  - `agent-management` - listing, attaching, killing agents
  - `agent-config` - agent detection and configuration
- **Affected code**:
  - `src/main.rs` - new `agent` subcommand
  - New `src/agent/` module for agent management
  - `src/config.rs` - agent configuration schema
  - `src/workspace.rs` - potential reuse for worktree creation
- **Dependencies**:
  - Requires `tmux` installed on the system
  - No new Rust crate dependencies anticipated (shell out to tmux)
