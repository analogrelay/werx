## ADDED Requirements

### Requirement: Agent Auto-Detection

The system SHALL automatically detect available coding agents by checking the system PATH.

#### Scenario: Detect OpenCode

- **WHEN** the system checks for available agents
- **AND** `opencode` executable is in PATH
- **THEN** OpenCode is registered as an available agent

#### Scenario: Detect Claude Code

- **WHEN** the system checks for available agents
- **AND** `claude` executable is in PATH
- **THEN** Claude Code is registered as an available agent

#### Scenario: Detect GitHub Copilot CLI

- **WHEN** the system checks for available agents
- **AND** `gh` executable is in PATH
- **AND** the `gh copilot` extension is installed
- **THEN** GitHub Copilot CLI is registered as an available agent

#### Scenario: No agents available

- **WHEN** the system checks for available agents
- **AND** no known agent executables are found
- **THEN** the system reports that no agents are available
- **AND** provides instructions for installing supported agents

### Requirement: Default Agent Preference

The system SHALL use OpenCode as the preferred default agent.

#### Scenario: OpenCode as default when available

- **WHEN** user spawns an agent without specifying type
- **AND** OpenCode is installed
- **THEN** OpenCode is used

#### Scenario: Fallback when OpenCode not available

- **WHEN** user spawns an agent without specifying type
- **AND** OpenCode is not installed
- **AND** Claude Code is installed
- **THEN** Claude Code is used as fallback

#### Scenario: Error when no agents available

- **WHEN** user spawns an agent without specifying type
- **AND** no supported agents are installed
- **THEN** the command fails with an error listing supported agents and install instructions

### Requirement: Global Agent Configuration

The system SHALL support global configuration of agent preferences and settings.

#### Scenario: Configure default agent globally

- **WHEN** user sets `agents.default = "claude"` in Forge config
- **AND** spawns an agent without `--agent` flag
- **THEN** Claude Code is used instead of OpenCode

#### Scenario: Configure agent command override

- **WHEN** user sets `agents.providers.opencode.command = "/custom/path/opencode"` in config
- **THEN** the custom path is used when spawning OpenCode agents

#### Scenario: Configure agent with default arguments

- **WHEN** user sets `agents.providers.claude.args = ["--model", "opus"]` in config
- **THEN** those arguments are passed to Claude Code on every spawn

### Requirement: Per-Repository Agent Preference

The system SHALL support per-repository agent preferences without repo-local config files.

#### Scenario: Set preferred agent for repository

- **WHEN** user configures `agents.repos."github.com/company/repo".preferred_agent = "copilot"` in global config
- **AND** spawns an agent for that repository without `--agent` flag
- **THEN** GitHub Copilot CLI is used instead of the global default

#### Scenario: Repository preference overrides global default

- **WHEN** global default is set to "opencode"
- **AND** repository `company/repo` has preferred_agent set to "claude"
- **AND** user spawns an agent for `company/repo`
- **THEN** Claude Code is used

#### Scenario: Explicit flag overrides repository preference

- **WHEN** repository `company/repo` has preferred_agent set to "claude"
- **AND** user runs `forge agent spawn company/repo --agent opencode`
- **THEN** OpenCode is used (explicit flag takes precedence)

### Requirement: Agent Configuration Schema

The system SHALL define a configuration schema for agent settings.

#### Scenario: Minimal configuration

- **WHEN** no agent configuration exists in Forge config
- **THEN** the system uses built-in defaults
- **AND** auto-detects available agents

#### Scenario: Full configuration example

- **WHEN** user wants complete control over agent configuration
- **THEN** the following configuration structure is supported:
```toml
[agents]
default = "opencode"

[agents.providers.opencode]
command = "opencode"
args = []

[agents.providers.claude]
command = "claude"
args = ["--model", "sonnet"]

[agents.providers.copilot]
command = "gh"
args = ["copilot"]

[agents.repos."github.com/work/project"]
preferred_agent = "copilot"

[agents.repos."github.com/personal/oss"]
preferred_agent = "opencode"
```

### Requirement: List Available Agents

The system SHALL provide a way to see which agents are available.

#### Scenario: List detected agents

- **WHEN** user runs `forge agent providers`
- **THEN** the system lists all detected agents
- **AND** indicates which is the current default
- **AND** shows the command that would be executed for each

#### Scenario: Show agent not found

- **WHEN** user runs `forge agent providers`
- **AND** a configured agent is not found in PATH
- **THEN** that agent is listed as "not available"
- **AND** shows the expected command path
