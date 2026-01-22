# Change: Add forge init command

## Why

Users need a way to initialize their Forge - a centralized root directory that will hold all their workspaces. Workspaces live at the root level for easy access, while repositories and internal data are stored in a hidden `.forge/` directory. This is the foundational command that sets up the Forge structure and configuration, enabling all other Forge operations.

## What Changes

- Add `forge init` command to create and configure a new Forge
- Support default location (`~/forge`) with customization via:
  - `FORGE_DIR` environment variable
- Create necessary directory structure for the Forge
- Initialize configuration with user's chosen Forge location
- Validate that the target directory is suitable for a Forge
- Handle cases where a Forge already exists at the target location

## Impact

- Affected specs: `forge-init` (new capability)
- Affected code:
  - New CLI command implementation (`forge init`)
  - Configuration management system (for storing Forge location)
  - Directory structure initialization logic
  - Validation and error handling for initialization scenarios
