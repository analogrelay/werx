# agent-management Specification

## Purpose
TBD - created by archiving change add-coding-agents. Update Purpose after archive.
## Requirements
### Requirement: List Agents Command

The system SHALL provide a command to list all running and recently exited agents.

#### Scenario: List running agents

- **WHEN** user runs `forge agent list`
- **THEN** the system displays all agents in the `forge-agents` tmux session
- **AND** shows each agent's name, status, repository, agent type, and worktree path

#### Scenario: List with no agents

- **WHEN** user runs `forge agent list`
- **AND** no agents have been spawned (no `forge-agents` session exists)
- **THEN** the system displays a message indicating no agents are running

#### Scenario: Show agent status

- **WHEN** an agent is listed
- **THEN** the status indicates whether the agent process is running or has exited
- **AND** if exited, shows whether it exited successfully or with an error

#### Scenario: List agents via alias

- **WHEN** user runs `forge agents`
- **THEN** it behaves identically to `forge agent list`

### Requirement: Attach to Agent Command

The system SHALL provide a command to attach to the agent tmux session.

#### Scenario: Attach to session

- **WHEN** user runs `forge agent attach`
- **THEN** the system attaches to the `forge-agents` tmux session
- **AND** the user can interact with agents via tmux

#### Scenario: Attach with agent name

- **WHEN** user runs `forge agent attach <agent-name>`
- **THEN** the system attaches to the `forge-agents` session
- **AND** selects the window corresponding to the specified agent name

#### Scenario: Attach when no session exists

- **WHEN** user runs `forge agent attach`
- **AND** the `forge-agents` session does not exist
- **THEN** the command fails with an error indicating no agents are running
- **AND** suggests spawning an agent first

#### Scenario: Attach with invalid agent name

- **WHEN** user runs `forge agent attach <invalid-name>`
- **AND** no agent with that name exists
- **THEN** the command fails with an error indicating the agent was not found
- **AND** lists available agent names

### Requirement: Kill Agent Command

The system SHALL provide a command to terminate a running agent.

#### Scenario: Kill specific agent

- **WHEN** user runs `forge agent kill <agent-name>`
- **THEN** the system terminates the agent process
- **AND** closes the tmux window for that agent
- **AND** the worktree is left intact for review

#### Scenario: Kill agent with cleanup

- **WHEN** user runs `forge agent kill <agent-name> --cleanup`
- **THEN** the system terminates the agent process
- **AND** closes the tmux window
- **AND** removes the worktree created for that agent

#### Scenario: Kill non-existent agent

- **WHEN** user runs `forge agent kill <invalid-name>`
- **THEN** the command fails with an error indicating the agent was not found

#### Scenario: Kill last agent closes session

- **WHEN** user kills the last remaining agent in the session
- **THEN** the `forge-agents` tmux session is also terminated
- **AND** the system reports that the session was closed

### Requirement: Agent Status Command

The system SHALL provide a command to show detailed status of agents.

#### Scenario: Show all agent statuses

- **WHEN** user runs `forge agent status`
- **THEN** the system displays detailed status for all agents
- **AND** includes: agent name, type, repository, branch, worktree path, process status, uptime

#### Scenario: Show specific agent status

- **WHEN** user runs `forge agent status <agent-name>`
- **THEN** the system displays detailed status for the specified agent only

#### Scenario: Status output formats

- **WHEN** user runs `forge agent status --format json`
- **THEN** the output is formatted as JSON for scripting

### Requirement: Interactive Agent Selection

The system SHALL provide interactive selection when agent ID is ambiguous.

#### Scenario: Interactive selection for attach

- **WHEN** user runs `forge agent attach` without an agent ID
- **AND** multiple agents are running
- **AND** terminal is interactive
- **THEN** an interactive selector displays available agents
- **AND** user can select which agent to attach to

#### Scenario: Interactive selection for kill

- **WHEN** user runs `forge agent kill` without an agent ID
- **AND** multiple agents are running
- **AND** terminal is interactive
- **THEN** an interactive selector displays available agents
- **AND** user can select which agent to kill

#### Scenario: Non-interactive requires explicit name

- **WHEN** user runs `forge agent attach` without an agent name
- **AND** terminal is non-interactive
- **THEN** the command fails with an error requiring an explicit agent name

