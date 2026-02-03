## 1. Foundation

- [x] 1.1 Add `tmux` dependency check utility function
- [x] 1.2 Create `src/agent/mod.rs` module structure
- [x] 1.3 Define `Agent` struct (id, name, type, repo, worktree_path, status)
- [x] 1.4 Define `AgentType` enum (OpenCode, ClaudeCode, Copilot)
- [x] 1.5 Add agent-related types to `src/lib.rs` exports
- [x] 1.6 Create name generation module with adjective/noun word lists (dataset TBD)
- [x] 1.7 Implement Docker-style name generator (adjective-noun combination)

## 2. Agent Detection & Configuration

- [x] 2.1 Implement PATH-based agent detection (`which opencode`, `which claude`, etc.)
- [x] 2.2 Implement GitHub Copilot extension detection (`gh extension list`)
- [x] 2.3 Extend `Config` struct with `AgentConfig` section
- [x] 2.4 Implement per-repo preference lookup (keyed by normalized URL)
- [x] 2.5 Implement default agent resolution (explicit > repo-pref > global > auto-detect)
- [x] 2.6 Add `forge agent providers` command to list detected agents

## 3. tmux Session Management

- [x] 3.1 Implement `tmux_session_exists()` check
- [x] 3.2 Implement `tmux_create_session()` for initial session creation
- [x] 3.3 Implement `tmux_create_window()` for adding agent windows
- [x] 3.4 Implement `tmux_list_windows()` to enumerate agents
- [x] 3.5 Implement `tmux_select_window()` for attach with specific agent
- [x] 3.6 Implement `tmux_kill_window()` for agent termination
- [x] 3.7 Implement `tmux_attach()` for interactive attachment

## 4. Agent Spawning

- [x] 4.1 Implement agent worktree creation (reuse workspace.rs logic)
- [x] 4.2 Implement unique agent ID generation (random hash)
- [x] 4.3 Implement human-readable name generation with collision detection
- [x] 4.4 Implement agent command construction per agent type
- [x] 4.5 Implement initial prompt handling (inline `--prompt`)
- [x] 4.6 Implement editor-based prompt (`--edit-prompt` / `-e`)
- [x] 4.7 Implement `forge agent spawn` command handler
- [x] 4.8 Add context detection for repository (from current workspace)

## 5. Agent Management Commands

- [x] 5.1 Implement `forge agent list` command
- [x] 5.2 Implement `forge agent status` command (detailed view)
- [x] 5.3 Implement `forge agent attach` command (with optional agent-name)
- [x] 5.4 Implement `forge agent kill` command
- [x] 5.5 Implement `--cleanup` flag for kill (remove worktree)
- [x] 5.6 Add `forge agents` alias for `forge agent list`

## 6. CLI Integration

- [x] 6.1 Add `AgentCommands` enum to `src/main.rs`
- [x] 6.2 Wire up `agent` subcommand with all operations
- [x] 6.3 Add `--agent` flag to spawn command
- [x] 6.4 Add `--branch` flag to spawn command
- [x] 6.5 Add `--prompt` and `--edit-prompt` flags
- [x] 6.6 Add `--format json` flag to list/status commands
- [x] 6.7 Implement interactive agent selection (using dialoguer)

## 7. Error Handling & UX

- [x] 7.1 Add helpful error for missing tmux
- [x] 7.2 Add helpful error for no agents available
- [x] 7.3 Add helpful error for repository not in Forge
- [x] 7.4 Implement success messages with next-step hints
- [x] 7.5 Handle non-interactive terminal gracefully

## 8. Testing

- [ ] 8.1 Unit tests for agent detection logic
- [ ] 8.2 Unit tests for agent ID and name generation
- [ ] 8.3 Unit tests for name collision handling
- [ ] 8.4 Unit tests for config parsing (agent preferences)
- [ ] 8.5 Integration tests for spawn (mock tmux)
- [ ] 8.6 Integration tests for list/status/kill commands

## 9. Documentation

- [x] 9.1 Update CLI help text for all agent commands
- [x] 9.2 Add examples to command descriptions
