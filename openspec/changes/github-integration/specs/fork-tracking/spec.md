## ADDED Requirements

### Requirement: Detect fork status at repo add time
When a repository is added to the Werx, the system SHALL attempt to determine whether the repository is a GitHub fork by querying the GitHub API via the `gh` CLI. The result SHALL be persisted in a `werx-repo.toml` file inside the bare repository directory.

#### Scenario: Fork detection succeeds
- **WHEN** a repository is added and `gh` is available in `$PATH`
- **AND** the repository URL points to a GitHub repository
- **AND** the GitHub API reports the repository is a fork
- **THEN** `werx-repo.toml` is written with `is_fork = true`, `upstream_owner`, `upstream_repo`, and `upstream_default_branch` fields set from the parent repository

#### Scenario: Non-fork detection succeeds
- **WHEN** a repository is added and `gh` is available
- **AND** the GitHub API reports the repository is not a fork
- **THEN** `werx-repo.toml` is written with `is_fork = false` and no upstream fields

#### Scenario: `gh` CLI not available
- **WHEN** a repository is added and `gh` is not found in `$PATH`
- **THEN** a warning is printed indicating GitHub integration is unavailable
- **AND** `werx-repo.toml` is not written
- **AND** repo add completes successfully

#### Scenario: Non-GitHub remote
- **WHEN** a repository is added with a remote URL that is not a GitHub URL
- **THEN** fork detection is skipped silently
- **AND** `werx-repo.toml` is not written

#### Scenario: GitHub API call fails
- **WHEN** `gh repo view` exits with a non-zero status
- **THEN** a warning is printed indicating fork status could not be determined
- **AND** `werx-repo.toml` is not written
- **AND** repo add completes successfully

---

### Requirement: Persist fork metadata in werx-repo.toml
The system SHALL store per-repo GitHub metadata in `.werx/repos/<name>/werx-repo.toml`. This file SHALL use a `[github]` table.

#### Scenario: Fork repo metadata file contents
- **WHEN** fork detection identifies a fork
- **THEN** `werx-repo.toml` contains a `[github]` table with fields: `owner`, `repo`, `is_fork = true`, `upstream_owner`, `upstream_repo`, `default_branch`, `upstream_default_branch`

#### Scenario: Non-fork repo metadata file contents
- **WHEN** fork detection identifies a non-fork
- **THEN** `werx-repo.toml` contains a `[github]` table with fields: `owner`, `repo`, `is_fork = false`, `default_branch`

#### Scenario: Missing werx-repo.toml treated as unknown
- **WHEN** a repo directory has no `werx-repo.toml`
- **THEN** the system behaves as if no GitHub metadata is available
- **AND** no errors are produced

---

### Requirement: Manage upstream remote for fork repos
For repositories whose `werx-repo.toml` indicates `is_fork = true`, the system SHALL ensure an `upstream` remote exists in the bare repository pointing at the parent repository's clone URL. The remote SHALL use the same protocol (SSH or HTTPS) as the `origin` remote.

#### Scenario: upstream remote added when missing
- **WHEN** a fork repo is detected at add time and no `upstream` remote exists
- **THEN** an `upstream` remote is added pointing at the parent repository URL

#### Scenario: upstream remote already correct
- **WHEN** an `upstream` remote already exists with the correct URL
- **THEN** no changes are made

#### Scenario: upstream remote URL is wrong
- **WHEN** an `upstream` remote exists but points at a different URL than the parent repo
- **THEN** the remote URL is updated to match the parent repository

#### Scenario: Non-fork repo not modified
- **WHEN** a repo has `is_fork = false` in `werx-repo.toml`
- **THEN** no `upstream` remote is added or modified

#### Scenario: No metadata available
- **WHEN** a repo has no `werx-repo.toml`
- **THEN** no upstream remote management is performed
