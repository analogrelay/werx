# shell-integration Specification

## Purpose
TBD - created by archiving change workspace-navigation. Update Purpose after archive.
## Requirements
### Requirement: Shell Init Command

The system SHALL provide a `forge shell` command to output shell integration code.

#### Scenario: Initialize bash integration

- **WHEN** user runs `forge shell init bash`
- **THEN** bash shell function code is output to stdout
- **AND** code defines `forge()` function that wraps binary
- **AND** code handles directive parsing
- **AND** code is valid bash syntax

#### Scenario: Initialize zsh integration

- **WHEN** user runs `forge shell init zsh`
- **THEN** zsh shell function code is output to stdout
- **AND** code defines `forge()` function that wraps binary
- **AND** code handles directive parsing
- **AND** code is valid zsh syntax

#### Scenario: Reject unsupported shell

- **WHEN** user runs `forge shell init <shell>`
- **AND** `<shell>` is not `bash` or `zsh`
- **THEN** command fails with error
- **AND** error lists supported shells
- **AND** command exits with non-zero code

#### Scenario: Shell init with auto-detection

- **WHEN** user runs `forge shell init` with no shell argument
- **THEN** command detects shell from environment variables
- **AND** bash integration code is output if BASH environment detected
- **AND** zsh integration code is output if ZSH environment detected
- **AND** command succeeds with detected shell

#### Scenario: Shell init with no shell and detection fails

- **WHEN** user runs `forge shell init` with no shell argument
- **AND** shell cannot be detected from environment variables
- **THEN** command fails with error
- **AND** error indicates shell could not be detected
- **AND** error lists supported shells
- **AND** command exits with non-zero code

### Requirement: Shell Wrapper Function

The system SHALL generate shell wrapper that intercepts forge commands.

#### Scenario: Wrapper calls real binary

- **WHEN** shell wrapper function is invoked
- **THEN** real `forge` binary is located via PATH
- **AND** all arguments are forwarded to binary
- **AND** binary exit code is preserved
- **AND** binary stdout is displayed to user

#### Scenario: Wrapper uses FORGE_BIN variable

- **WHEN** `FORGE_BIN` environment variable is set
- **THEN** wrapper uses `$FORGE_BIN` instead of `forge` from PATH
- **AND** allows testing with custom binary location
- **AND** defaults to `forge` if `FORGE_BIN` not set

#### Scenario: Wrapper captures directives

- **WHEN** binary outputs directives to stderr
- **THEN** wrapper extracts lines starting with `@forge:`
- **AND** wrapper filters directives from user-visible output
- **AND** wrapper processes each directive
- **AND** wrapper displays non-directive stderr normally

#### Scenario: Wrapper preserves normal behavior

- **WHEN** binary outputs no directives
- **THEN** wrapper behaves like direct binary invocation
- **AND** all output is displayed normally
- **AND** exit code is preserved
- **AND** no overhead is noticeable

### Requirement: Directive Protocol

The system SHALL define protocol for binary-to-shell communication.

#### Scenario: Change directory directive format

- **WHEN** binary needs to change shell directory
- **THEN** directive format is `@forge:change_directory:<path>`
- **AND** `<path>` is absolute path to target directory
- **AND** directive is output to stderr
- **AND** directive is on single line

#### Scenario: Directive naming convention

- **WHEN** directive is defined
- **THEN** directive name contains only lowercase letters and underscores
- **AND** format is `@forge:<name>:<argument>`
- **AND** directive name is matched exactly
- **AND** argument can contain any text except newlines

#### Scenario: Multiple directives in output

- **WHEN** binary outputs multiple directives
- **THEN** each directive is on separate line
- **AND** wrapper processes directives in order
- **AND** all directives are executed

### Requirement: Change Directory Processing

The system SHALL execute directory changes in user's shell.

#### Scenario: Process change directory directive

- **WHEN** wrapper receives `@forge:change_directory:<path>` directive
- **THEN** wrapper extracts `<path>` from directive
- **AND** wrapper verifies `<path>` is directory
- **AND** wrapper executes `cd <path>` in current shell
- **AND** current working directory changes

#### Scenario: Handle missing directory

- **WHEN** change directory directive references non-existent path
- **THEN** wrapper prints error message to stderr
- **AND** error indicates directory does not exist
- **AND** no directory change occurs
- **AND** shell remains in current directory

#### Scenario: Handle directory permission error

- **WHEN** change directory directive references inaccessible path
- **THEN** `cd` command fails
- **AND** error is displayed to user
- **AND** shell remains in current directory
- **AND** wrapper continues processing (no crash)

### Requirement: Shell Integration Installation

The system SHALL support user installation of shell integration.

#### Scenario: Eval initialization in shell config

- **WHEN** user adds `eval "$(forge shell init bash)"` to `.bashrc`
- **THEN** shell wrapper is defined on shell startup
- **AND** `forge` command uses wrapper for all invocations
- **AND** user can call forge commands normally

#### Scenario: Uninstall shell integration

- **WHEN** user removes eval line from shell config
- **THEN** `forge` command reverts to direct binary invocation
- **AND** directives are no longer processed
- **AND** binary continues to work (directives ignored)

#### Scenario: Test with FORGE_BIN

- **WHEN** user sets `FORGE_BIN=/path/to/test/forge`
- **AND** shell integration is active
- **THEN** wrapper uses test binary instead of PATH binary
- **AND** allows testing without installing to PATH
- **AND** normal users don't need to set FORGE_BIN

### Requirement: Error Handling in Wrapper

The system SHALL handle errors gracefully in shell wrapper.

#### Scenario: Binary not found

- **WHEN** wrapper cannot locate forge binary
- **AND** `FORGE_BIN` is not set or invalid
- **THEN** shell displays command not found error
- **AND** wrapper returns non-zero exit code
- **AND** no directory change occurs

#### Scenario: Binary exits with error

- **WHEN** forge binary exits with non-zero code
- **THEN** wrapper preserves exit code
- **AND** wrapper processes any directives emitted before error
- **AND** error messages are displayed to user

#### Scenario: Malformed directive

- **WHEN** binary outputs line starting with `@forge:`
- **AND** directive format is invalid or unknown
- **THEN** wrapper ignores directive
- **AND** no error is displayed (forward compatibility)
- **AND** wrapper continues processing other directives

### Requirement: Shell Compatibility

The system SHALL support common shell versions.

#### Scenario: Bash 3.2 compatibility (macOS default)

- **WHEN** shell integration used in bash 3.2
- **THEN** wrapper function works correctly
- **AND** regex matching works (BASH_REMATCH)
- **AND** directory changes execute successfully

#### Scenario: Bash 4.0+ compatibility

- **WHEN** shell integration used in bash 4.0 or later
- **THEN** wrapper function works correctly
- **AND** all features operate normally

#### Scenario: Zsh 5.0+ compatibility

- **WHEN** shell integration used in zsh 5.0 or later
- **THEN** wrapper function works correctly
- **AND** regex matching works (match array)
- **AND** directory changes execute successfully

### Requirement: Shell Integration Documentation

The system SHALL provide clear integration instructions.

#### Scenario: Help text for shell command

- **WHEN** user runs `forge shell --help`
- **THEN** help text explains shell integration purpose
- **AND** shows supported shells
- **AND** provides installation example
- **AND** mentions FORGE_BIN variable

#### Scenario: Post-init suggestion

- **WHEN** user successfully runs `forge init`
- **THEN** success message suggests shell integration
- **AND** shows example: `eval "$(forge shell init bash)"`
- **AND** indicates adding to shell config file
- **AND** suggests appropriate config file for detected shell

