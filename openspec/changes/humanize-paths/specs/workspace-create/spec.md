# Spec Delta: workspace-create

## MODIFIED Requirements

### Requirement: Hierarchical Workspace Storage

The system SHALL create workspaces in a hierarchical structure grouped by repository using human-readable directory names.

#### Scenario: Workspace created in repository directory with simple name

- **WHEN** workspace is created for repository stored with simple name `linux`
- **AND** workspace name is `main`
- **THEN** directory is created at `<forge-root>/linux/main/`
- **AND** directory contains working tree files
- **AND** git metadata points to bare repository in `.forge/repos/linux/`

#### Scenario: Workspace created in repository directory with owner-qualified name

- **WHEN** workspace is created for repository stored with owner-qualified name `torvalds-linux`
- **AND** workspace name is `main`
- **THEN** directory is created at `<forge-root>/torvalds-linux/main/`
- **AND** git metadata points to bare repository in `.forge/repos/torvalds-linux/`

#### Scenario: Workspace created in repository directory with hash-qualified name

- **WHEN** workspace is created for repository stored with hash-qualified name `torvalds-linux-a1b2c3`
- **AND** workspace name is `feat/new-feature`
- **THEN** directory is created at `<forge-root>/torvalds-linux-a1b2c3/feat/new-feature/`
- **AND** git metadata points to bare repository in `.forge/repos/torvalds-linux-a1b2c3/`

#### Scenario: Repository directory created if needed

- **WHEN** creating first workspace for a repository
- **THEN** repository directory `<forge-root>/<repo-dir-name>/` is created
- **AND** repo-dir-name matches the directory name in `.forge/repos/`
- **AND** workspace is created within that directory

#### Scenario: Multiple workspaces share repository directory

- **WHEN** creating multiple workspaces for same repository
- **THEN** all workspaces are created under `<forge-root>/<repo-dir-name>/`
- **AND** each workspace has its own subdirectory
- **AND** repo-dir-name is the human-readable directory name (simple or qualified)

#### Scenario: Workspace path uses human-readable repository name

- **WHEN** user creates workspace named `feature` for repository with simple name `project`
- **THEN** workspace directory is `<forge-root>/project/feature/`
- **AND** path is immediately recognizable and navigable

### Requirement: Success Feedback

The system SHALL provide clear feedback when workspace is created successfully, showing human-readable paths.

#### Scenario: Display success message with simple repository name

- **WHEN** workspace is created successfully for repository with simple name
- **THEN** success message confirms the workspace was created
- **AND** shows workspace directory path as `<forge-root>/<name>/<workspace>/`
- **AND** shows the branch that was checked out

#### Scenario: Display success message with qualified repository name

- **WHEN** workspace is created successfully for repository with owner-qualified name
- **THEN** success message confirms the workspace was created
- **AND** shows workspace directory path as `<forge-root>/<owner>-<name>/<workspace>/`
- **AND** path clearly indicates both owner and repository name

#### Scenario: Show next steps with readable path

- **WHEN** workspace is created successfully
- **THEN** message suggests `cd <workspace-path>` to enter the workspace
- **AND** path is human-readable without hash suffixes in common cases
