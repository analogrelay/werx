# Spec Delta: forge-init

## MODIFIED Requirements

### Requirement: Success Feedback

#### Scenario: Config file contains protocol preference

- **WHEN** Forge is initialized
- **THEN** `.forge/config` includes the protocol preference
- **AND** serves as both config and marker file
- **AND** config contains no agent-related sections

### Requirement: Forge Directory Structure Creation

#### Scenario: Create standard Forge directories

- **WHEN** user runs `forge init`
- **THEN** the following directories are created:
  - `<forge-root>/.forge/` for internal data (marker file, repositories, configuration)
  - `<forge-root>/.forge/repos/` for storing repository clones
  - Workspaces are created as non-hidden directories directly in `<forge-root>/`
- **AND** no agent session infrastructure is created
