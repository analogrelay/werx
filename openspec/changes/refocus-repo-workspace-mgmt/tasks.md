## Tasks

### 1. Remove agent module and name data

- [x] Delete `src/agent/` directory (mod.rs, spawn.rs, manage.rs, providers.rs, tmux.rs, names.rs)
- [x] Delete `src/namedata.rs`

### 2. Remove agent declarations and re-exports from `src/lib.rs`

- [x] Remove `pub mod agent;` declaration (line 1)
- [x] Remove `pub use agent::{...};` re-export block (lines 15-19)

### 3. Remove agent CLI definitions from `src/main.rs`

- [x] Remove agent-specific symbols from the `use werx::{...}` import block (lines 5-13): `AgentType`, `SpawnOptions`, `attach_to_agent`, `detect_providers`, `find_agent`, `get_default_provider`, `kill_agent`, `list_agents`, `spawn_agent`
- [x] Remove the `Agent(AgentCommands)` variant from the `Commands` enum (lines 101-103)
- [x] Remove the entire `enum AgentCommands` definition (lines 124-204)
- [x] Remove the `Commands::Agent(subcmd) => match subcmd { ... }` dispatch block (lines 401-426)

### 4. Remove agent handler functions from `src/main.rs`

- [x] Remove `// Agent Commands` comment section (line 1328-1330)
- [x] Remove `fn cmd_agent_spawn(...)` (lines 1332-1424)
- [x] Remove `fn cmd_agent_list(...)` (lines 1426-1475)
- [x] Remove `fn cmd_agent_status(...)` (lines 1477-1514)
- [x] Remove `fn cmd_agent_attach(...)` (lines 1516-1556)
- [x] Remove `fn cmd_agent_kill(...)` (lines 1558-1625)
- [x] Remove `fn cmd_agent_providers()` (lines 1627-1679)
- [x] Remove `fn get_prompt_from_editor()` (lines 1681-1725)

### 5. Remove agent config types from `src/config.rs`

- [x] Remove `pub struct AgentProviderConfig` (lines 67-77)
- [x] Remove `pub struct RepoAgentConfig` (lines 79-85)
- [x] Remove `pub struct AgentConfig` (lines 87-101)
- [x] Remove `pub agents: Option<AgentConfig>` field from `Config` struct (lines 110-112)
- [x] Remove `fn default_agent()` method (lines 166-169)
- [x] Remove `fn preferred_agent_for_repo()` method (lines 171-178)
- [x] Remove `fn agent_provider_config()` method (lines 180-185)

### 6. Remove agent-only dependencies from `Cargo.toml`

- [x] Remove `rand = "0.8"` (line 24)
- [x] Remove `exec = "0.3"` (line 25)

### 7. Remove agent OpenSpec specs

- [x] Delete `openspec/specs/agent-spawn/` directory
- [x] Delete `openspec/specs/agent-management/` directory
- [x] Delete `openspec/specs/agent-config/` directory

### 8. Clean up stale agent references

- [x] Grep remaining source files for "agent" string literals, comments, and documentation references; remove or update any that are found
- [x] Review `README.md` and remove agent-related documentation sections

### 9. Update CHANGELOG.md

- [x] Add breaking change entry for removal of `werx agent` command group
- [x] Add breaking change entry for removal of `[agents]` config section
- [x] Add entry noting removed dependencies (`exec`, `rand`)

### 10. Verify

- [x] Run `cargo build` and confirm clean compilation with no errors or warnings
- [x] Run `cargo test` and confirm all remaining tests pass
- [x] Run `cargo clippy` and confirm no new warnings
