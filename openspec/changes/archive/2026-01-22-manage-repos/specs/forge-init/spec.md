# forge-init Specification

## Purpose
(Existing purpose from base spec - this delta adds protocol preference prompting)

## MODIFIED Requirements

### Requirement: Success Feedback

The system SHALL provide clear feedback when initialization succeeds.

#### Scenario: Prompt for protocol preference during init

- **WHEN** user runs `forge init` successfully
- **THEN** user is prompted to choose their preferred Git protocol (SSH or HTTPS)
- **AND** choice is saved to `.forge/config`

#### Scenario: Skip protocol prompt with flag

- **WHEN** user runs `forge init --protocol <ssh|https>`
- **THEN** the specified protocol is saved to config without prompting

#### Scenario: Protocol preference applies to future operations

- **WHEN** protocol preference is set during init
- **THEN** it is used for all future shorthand repository URL resolutions
- **AND** user will not be prompted again during `forge add` operations

#### Scenario: Config file contains protocol preference

- **WHEN** Forge is initialized
- **THEN** `.forge/config` includes the protocol preference
- **AND** serves as both config and marker file
