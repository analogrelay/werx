## Context

Werx is a CLI workspace management tool that currently serves two roles: (1) managing git repositories as bare clones and their worktrees, and (2) managing AI coding agent sessions via tmux. The agent management layer (`src/agent/`, ~700 lines across 6 files plus `src/namedata.rs`) handles spawning agents in tmux windows, detecting installed agent providers, generating random agent names, and tracking agent lifecycle.

The agent layer depends on the workspace layer (it creates worktrees for agents), but the workspace layer has no dependency on agents. This makes removal clean — the agent code is a leaf in the dependency graph.

The codebase has 16 OpenSpec specs, 3 of which are agent-specific (`agent-spawn`, `agent-management`, `agent-config`). The remaining 13 cover init, repos, workspaces, and shell integration.

## Goals / Non-Goals

**Goals:**
- Remove all agent session management code, commands, config, and specs
- Remove dependencies that exist solely for agent management (`exec`, `rand`)
- Produce a clean, compilable codebase with no dead code or dangling references to agents
- Preserve all repository and workspace management functionality exactly as-is

**Non-Goals:**
- Renaming "forge" references in specs to "werx" (separate change)
- Adding new features or capabilities
- Changing the workspace or repo management behavior in any way
- Restructuring the remaining code (e.g., breaking `main.rs` into smaller modules)

## Decisions

### 1. Delete agent module entirely rather than feature-gating it

**Decision**: Remove `src/agent/` and `src/namedata.rs` outright.

**Rationale**: Feature-gating would preserve dead code for a capability we're explicitly deciding doesn't belong in werx. The agent layer has no non-agent consumers. A clean delete is simpler, easier to verify, and avoids conditional compilation complexity.

**Alternative considered**: Cargo feature flag (`--features agents`) to optionally include agent support. Rejected because the goal is a philosophical refocus, not a temporary toggle.

### 2. Remove only `exec` and `rand` from Cargo.toml

**Decision**: Remove `exec` and `rand` crates. Keep `skim`, `fuzzy-matcher`, `crossterm`, and `dialoguer`.

**Rationale**: Code analysis confirms `exec` is used only in `src/agent/tmux.rs` (for `exec::Command` to replace the process with tmux attach) and `rand` is used only in `src/agent/names.rs` (for random agent name generation). The other crates — `skim` and `fuzzy-matcher` (fuzzy workspace search in `werx go`), `crossterm` (terminal raw mode for tab-completion in branch prompts), and `dialoguer` (interactive prompts throughout) — are all used by non-agent code.

### 3. Silently ignore unknown config sections rather than warning

**Decision**: When an existing `.werx/config.toml` contains an `[agents]` section, silently ignore it (serde's default behavior with `deny_unknown_fields` not set).

**Rationale**: TOML deserialization with serde already ignores unknown fields by default. Users who have agent config will simply have an inert section in their config file. A warning would add complexity for a temporary situation — users will naturally clean up their config, or we can document it in the changelog.

**Alternative considered**: Emit a deprecation warning on startup if `[agents]` section exists. Rejected as over-engineered for a pre-1.0 tool with a small user base.

### 4. Delete agent specs from openspec/specs/

**Decision**: Remove the 3 agent spec directories (`agent-spawn/`, `agent-management/`, `agent-config/`) from `openspec/specs/`.

**Rationale**: Specs describe required behavior. If the behavior no longer exists, the specs are misleading to keep. Keeping them under a "deprecated" marker adds noise. Git history preserves them if ever needed.

### 5. Strip agent references from `forge-init` spec

**Decision**: Edit the `forge-init` spec to remove any agent-related initialization requirements (e.g., default agent config setup). Keep all repo/workspace init behavior unchanged.

**Rationale**: The init spec currently may reference setting up agent defaults as part of initialization. Those requirements no longer apply and should be removed to keep the spec accurate.

## Risks / Trade-offs

**[Risk: Users depending on `werx agent spawn`]** → The worktree creation that agents used is still available via `werx work create`. Users can launch agents manually in those worktrees. Document the migration path in CHANGELOG.md.

**[Risk: Stale agent config in existing installations]** → Silently ignored by serde. No runtime impact. Could confuse users reading their config file. → Mitigated by changelog entry noting the removal.

**[Risk: Missing an agent reference somewhere in the codebase]** → The Rust compiler will catch any dangling references to deleted modules/types at build time. Run `cargo build` and `cargo test` as verification. Also grep for "agent" strings in remaining code to catch comments or string literals.

**[Trade-off: No feature flag for gradual migration]** → Accepted. This is a pre-1.0 tool. Clean removal is preferred over maintaining optional dead code paths.
