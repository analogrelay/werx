## Why

Werx currently tries to do two things: manage git repositories/worktrees and manage AI agent sessions (spawning agents in tmux, detecting providers, tracking agent lifecycle). The agent management layer adds significant complexity — tmux orchestration, provider detection, agent naming, per-repo agent preferences — for something that is better handled by the agents and their tooling themselves. By removing agent session management, werx becomes a sharper, more focused tool: a workspace manager that handles bare clones, worktrees, and the status/cleanup tasks that developers (and agents) actually need from a workspace layer.

## What Changes

- **BREAKING**: Remove the `werx agent` command and all subcommands (`spawn`, `list`, `status`, `attach`, `kill`, `providers`)
- **BREAKING**: Remove agent-related configuration (`agents.default`, `agents.providers.*`, `agents.repos.*`) from `config.toml`
- **BREAKING**: Remove the `exec`, `skim`, `fuzzy-matcher`, and `rand` dependencies that exist solely for agent management (confirm which are agent-only before removing)
- Remove the entire `src/agent/` module tree (`mod.rs`, `spawn.rs`, `manage.rs`, `providers.rs`, `tmux.rs`, `names.rs`) and `src/namedata.rs`
- Remove the 3 agent-related OpenSpec specs (`agent-spawn`, `agent-management`, `agent-config`)
- Retain all repository management: `werx init`, `werx add`, `werx create`, `werx repos *`
- Retain all workspace/worktree management: `werx work create/list/remove/go/status/check`
- Retain shell integration (`werx go`, `werx shell init`)
- Retain the core workspace status and cleanup capabilities (uncommitted changes, unpushed branches, merged branch detection)

## Capabilities

### New Capabilities

_(None — this change is a removal/simplification, not a feature addition.)_

### Modified Capabilities

- `forge-init`: Remove agent-related config initialization (protocol prompting and directory setup remain unchanged)
- `shell-integration`: No functional change, but verify no agent-specific directive handling exists

## Impact

- **Code**: The entire `src/agent/` directory (~5 files, ~500+ lines) and `src/namedata.rs` (~200 lines) are deleted. `src/main.rs` loses the `agent` command tree and all agent handler functions. `src/config.rs` loses `AgentConfig`, `ProviderConfig`, and related fields.
- **CLI**: The `werx agent` / `werx agents` command group is removed entirely. All other commands remain.
- **Config**: The `[agents]` section of `.werx/config.toml` is no longer recognized. Existing configs with this section will have it silently ignored (or warned).
- **Dependencies**: `exec`, `rand`, and potentially `skim`/`fuzzy-matcher` crates may be removable (need to verify `skim`/`fuzzy-matcher` aren't used by `werx go` fuzzy search before removing).
- **Specs**: 3 specs removed (`agent-spawn`, `agent-management`, `agent-config`). 13 specs remain. Specs still reference "forge" naming from the pre-rename era — this change does not address that.
- **Users**: Anyone relying on `werx agent spawn` to launch coding agents will need to manage agent sessions independently. The worktree creation that agents need is still available via `werx work create`.
