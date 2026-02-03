## Context

Forge is a CLI tool for managing repositories and workspaces. This change adds the ability to spawn and manage AI coding agents (OpenCode, Claude Code, GitHub Copilot CLI) within isolated worktrees, orchestrated via tmux.

**Stakeholders**: Developers using AI coding agents who want streamlined multi-agent workflows.

**Constraints**:

- Must work with existing worktree infrastructure
- tmux is a required external dependency (for now)
- Agents are terminal applications invoked via CLI
- No repo-local config files (team buy-in concern)

## Goals / Non-Goals

**Goals**:

- Spawn coding agents in isolated worktrees with a single command
- Manage multiple concurrent agents across repos
- Support OpenCode, Claude Code, and GitHub Copilot CLI
- Auto-detect available agents from `$PATH`
- Allow per-repo agent preferences without repo-local config
- Enable optional initial prompts at spawn time

**Non-Goals** (for this iteration):

- Live monitoring dashboard or TUI for viewing agent activity
- Automated task distribution across agents
- Agent-to-agent communication
- Support for non-terminal agents (IDE plugins, etc.)
- Alternative session managers (screen, zellij) - tmux only for now

## Decisions

### Decision 1: tmux for Session Management

**Choice**: Use tmux with a single shared session (`forge-agents`) containing one window per agent.

**Rationale**:

- tmux is ubiquitous on developer machines
- Provides process isolation - agents survive Forge process exit
- Easy window switching for users already familiar with tmux
- Scriptable via `tmux` CLI for status checks and control

**Alternatives considered**:

- **Direct process spawning**: Would require Forge to stay running as supervisor; loses isolation
- **Screen**: Less common, fewer features
- **Zellij**: Newer, less ubiquitous
- **Custom PTY management**: High complexity for little benefit

### Decision 2: One Worktree Per Agent Instance

**Choice**: Each `forge agent spawn` creates a new, unique worktree for that agent.

**Rationale**:

- Provides complete isolation - agents can make changes without conflicts
- Enables parallel work on different features/branches
- Aligns with Forge's worktree-centric model
- Easy cleanup when agent work is complete

**Worktree naming**: `<repo>/<agent-name>/` where `<agent-name>` is the Docker-style human-readable name (e.g., `my-project/happy-ferret/`).

### Decision 3: Agent Detection via PATH

**Choice**: Auto-detect agents by checking for known executables in `$PATH`.

**Rationale**:

- Zero-config for common setups
- Users don't need to manually register agents
- Graceful degradation if agent not installed

**Known agents**:

| Agent | Executable | Detection |
|-------|------------|-----------|
| OpenCode | `opencode` | `which opencode` |
| Claude Code | `claude` | `which claude` |
| GitHub Copilot CLI | `gh` | `which gh` + check for copilot extension |

### Decision 4: Configuration Without Repo-Local Files

**Choice**: Store per-repo agent preferences in global Forge config, keyed by repository identifier.

**Rationale**:

- No team buy-in required to add `.forge.toml` to repos
- Config travels with the user across machines (if synced)
- Follows existing Forge config patterns

**Schema sketch**:

```toml
[agents]
default = "opencode"  # Global default

[agents.providers.opencode]
command = "opencode"
# Optional overrides

[agents.providers.claude]
command = "claude"

[agents.repos."github.com/company/private-repo"]
preferred_agent = "copilot"
```

### Decision 5: Initial Prompt Delivery

**Choice**: Support two modes for initial prompts:

1. `--prompt "message"` / `-P` - inline on command line
2. `--edit-prompt` / `-p` - opens `$EDITOR` for composing prompt

**Rationale**:

- Inline is fast for simple tasks
- Editor mode supports complex, multi-line prompts
- Follows Unix conventions (`git commit -m` vs `git commit`)

**Delivery to agent**: Write prompt to a temp file, pass as argument or pipe to stdin depending on agent capabilities.

### Decision 6: Agent Identification and Naming

**Choice**: Each agent instance gets:

1. A random hash-based ID for internal tracking and uniqueness
2. A human-readable name generated Docker-style by combining elements from two word lists (adjective + noun)

**Examples**: `happy-ferret`, `brave-dolphin`, `cosmic-penguin`

**Rationale**:

- Human-readable names are memorable and easy to type
- Docker has proven this naming pattern works well for ephemeral resources
- Random hash ensures uniqueness even if name generation has collisions
- Names are used for display, worktree paths, and tmux window names

**Implementation notes**:

- A curated dataset of adjectives and nouns will be provided during implementation
- If a generated name collides with an existing agent, regenerate until unique
- The hash ID is stored internally but rarely shown to users

## Risks / Trade-offs

| Risk | Mitigation |
|------|------------|
| tmux not installed | Check at spawn time, provide helpful error with install instructions |
| Agent CLI changes | Wrap agent invocation in adapter layer; isolate agent-specific code |
| Worktree proliferation | Add `forge agent cleanup` later; document cleanup workflow |
| tmux session conflicts | Use unique session name `forge-agents`; check for existing session |
| Agent startup failures | Capture exit codes; report errors in `forge agent list` |

## Open Questions

1. **Worktree cleanup policy**: Should `forge agent kill` also remove the worktree? Or leave it for manual cleanup? (Suggest: leave by default, add `--cleanup` flag)

2. **Agent restart**: If an agent exits (user quit, crash), should `forge agent spawn` with same parameters reuse the existing worktree? (Suggest: no, always create fresh; can add `--reuse` later)

3. **Prompt file format**: For agents that accept file input, what format? Plain text? Markdown? (Suggest: plain text for maximum compatibility)

4. **GitHub Copilot CLI invocation**: `gh copilot` is a subcommand, need to verify exact invocation for chat mode.
