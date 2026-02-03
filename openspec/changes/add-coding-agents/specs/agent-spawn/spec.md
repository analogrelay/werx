## ADDED Requirements

### Requirement: Agent Naming

The system SHALL assign each agent a unique identifier consisting of a random hash and a human-readable name.

#### Scenario: Generate unique agent ID

- **WHEN** an agent is spawned
- **THEN** the system generates a random hash-based ID for internal tracking
- **AND** the ID is unique across all agents

#### Scenario: Generate human-readable name

- **WHEN** an agent is spawned
- **THEN** the system generates a human-readable name in Docker style
- **AND** the name is composed by combining random elements from two word lists (adjective + noun)
- **AND** the name is used for display, worktree paths, and tmux window names

#### Scenario: Name uniqueness

- **WHEN** generating a name for a new agent
- **AND** the generated name conflicts with an existing agent
- **THEN** the system regenerates until a unique name is found

#### Scenario: Name dataset

- **WHEN** the system generates agent names
- **THEN** it uses a curated dataset of name elements (to be provided during implementation)
- **AND** the dataset contains adjectives and nouns suitable for agent naming

### Requirement: Spawn Agent Command

The system SHALL provide a command to spawn a coding agent in an isolated worktree.

#### Scenario: Spawn agent with explicit repository and branch

- **WHEN** user runs `forge agent spawn owner/repo --branch main`
- **THEN** a new worktree is created for the repository at a unique path
- **AND** the specified agent is launched in a new tmux window within the `forge-agents` session
- **AND** the agent's working directory is set to the new worktree
- **AND** the command returns immediately without attaching to the session

#### Scenario: Spawn agent with default branch

- **WHEN** user runs `forge agent spawn owner/repo`
- **AND** no `--branch` flag is specified
- **THEN** the worktree is created from the repository's default branch

#### Scenario: Spawn agent with context detection

- **WHEN** user runs `forge agent spawn` from within an existing workspace
- **AND** no repository argument is provided
- **THEN** the system detects the repository from the current workspace
- **AND** spawns an agent for that repository

#### Scenario: Spawn with specific agent type

- **WHEN** user runs `forge agent spawn owner/repo --agent claude`
- **THEN** Claude Code is launched instead of the default agent
- **AND** the agent type is reflected in the worktree name and tmux window name

#### Scenario: Spawn with default agent preference

- **WHEN** user runs `forge agent spawn owner/repo`
- **AND** no `--agent` flag is specified
- **AND** no per-repo preference is configured
- **THEN** the system uses OpenCode if available
- **AND** falls back to other available agents if OpenCode is not installed

### Requirement: Initial Prompt Support

The system SHALL support providing an initial prompt to the spawned agent.

#### Scenario: Inline prompt via command line

- **WHEN** user runs `forge agent spawn owner/repo --prompt "Fix the login bug"`
- **THEN** the agent is started with the provided prompt as its initial task

#### Scenario: Prompt via editor

- **WHEN** user runs `forge agent spawn owner/repo --edit-prompt`
- **THEN** the system opens `$EDITOR` for the user to compose a prompt
- **AND** waits for the editor to close
- **AND** uses the edited content as the initial prompt

#### Scenario: Prompt via editor with short flag

- **WHEN** user runs `forge agent spawn owner/repo -e`
- **THEN** it behaves identically to `--edit-prompt`

#### Scenario: No prompt provided

- **WHEN** user runs `forge agent spawn owner/repo` without prompt flags
- **THEN** the agent starts without an initial prompt
- **AND** the user can interact with it after attaching

### Requirement: Worktree Creation for Agents

The system SHALL create a unique worktree for each spawned agent.

#### Scenario: Unique worktree per spawn

- **WHEN** user spawns an agent for `owner/repo` on branch `main`
- **THEN** a new worktree is created at `<forge-root>/<repo-name>/<agent-name>/`
- **AND** the worktree is checked out to the `main` branch
- **AND** the worktree is isolated from other workspaces

#### Scenario: Multiple agents on same repo

- **WHEN** user spawns two agents for the same repository
- **THEN** each agent gets its own unique worktree
- **AND** the agents can work independently without conflicts

#### Scenario: Worktree naming uses agent name

- **WHEN** user spawns an OpenCode agent for `my-project` on `main`
- **THEN** the worktree path uses the agent's human-readable name (e.g., `my-project/happy-ferret/`)
- **AND** the path is unique across multiple spawns

### Requirement: tmux Session Management

The system SHALL manage agents within a shared tmux session.

#### Scenario: Create tmux session if not exists

- **WHEN** user spawns an agent
- **AND** the `forge-agents` tmux session does not exist
- **THEN** the system creates the `forge-agents` session
- **AND** creates a window for the agent within that session

#### Scenario: Add window to existing session

- **WHEN** user spawns an agent
- **AND** the `forge-agents` tmux session already exists
- **THEN** the system creates a new window in the existing session
- **AND** does not disturb other running agents

#### Scenario: Window naming

- **WHEN** an agent is spawned
- **THEN** the tmux window is named with the agent's human-readable name (e.g., `happy-ferret`)
- **AND** the window name is visible when listing windows or attaching

#### Scenario: Non-blocking spawn

- **WHEN** user runs `forge agent spawn`
- **THEN** the command returns immediately after launching the agent
- **AND** does not attach to the tmux session
- **AND** prints the agent ID and instructions for attaching

### Requirement: Spawn Feedback

The system SHALL provide clear feedback when an agent is spawned.

#### Scenario: Display success message

- **WHEN** an agent is successfully spawned
- **THEN** the system displays the agent ID
- **AND** shows the worktree path
- **AND** shows how to attach to the agent (e.g., `forge agent attach <id>`)

#### Scenario: Display error when agent not available

- **WHEN** user requests a specific agent that is not installed
- **THEN** the command fails with a descriptive error
- **AND** lists available agents on the system

#### Scenario: Display error when tmux not installed

- **WHEN** user attempts to spawn an agent
- **AND** tmux is not installed
- **THEN** the command fails with an error explaining tmux is required
- **AND** suggests how to install tmux

### Requirement: Repository Resolution

The system SHALL resolve repository specifications to Forge repositories.

#### Scenario: Resolve by shorthand

- **WHEN** user runs `forge agent spawn owner/repo`
- **THEN** the system resolves the repository using Forge's URL resolution
- **AND** spawns the agent in a worktree of that repository

#### Scenario: Repository not in Forge

- **WHEN** user attempts to spawn an agent for a repository not in the Forge
- **THEN** the command fails with an error indicating the repository is not found
- **AND** suggests running `forge add owner/repo` first

### Requirement: Forge Existence Check

The system SHALL verify a Forge exists before spawning agents.

#### Scenario: Require initialized Forge

- **WHEN** user runs `forge agent spawn` outside a Forge directory
- **AND** no Forge exists at the default location
- **THEN** the command fails with an error indicating no Forge found
- **AND** suggests running `forge init` first
