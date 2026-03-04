## ADDED Requirements

### Requirement: Default branch naming pattern
The system SHALL provide a branch naming service that produces branch names following the pattern `<username>/[<issue#>-]<topic>`, where `<username>` is the GitHub username, `<issue#>` is an optional issue number, and `<topic>` is a brief slug describing the work.

#### Scenario: Branch name with issue number
- **WHEN** the branch naming service is called with username `"ashley"`, issue number `42`, and topic `"update-auth-service"`
- **THEN** the produced branch name is `"ashley/42-update-auth-service"`

#### Scenario: Branch name without issue number
- **WHEN** the branch naming service is called with username `"ashley"`, no issue number, and topic `"update-auth-service"`
- **THEN** the produced branch name is `"ashley/update-auth-service"`

#### Scenario: Topic slug normalisation
- **WHEN** the topic contains uppercase letters, spaces, or special characters
- **THEN** the slug is lowercased and non-alphanumeric characters (except hyphens) are replaced with hyphens
- **AND** consecutive hyphens are collapsed into one
- **AND** leading/trailing hyphens are removed

---

### Requirement: GitHub username resolution
The system SHALL resolve the GitHub username for use in branch naming. On first use, the username SHALL be fetched via `gh api user --jq '.login'` and cached in `werx.toml` under `[github] username`. Subsequent calls SHALL use the cached value.

#### Scenario: Username fetched and cached on first use
- **WHEN** branch naming is invoked and no username is cached in `werx.toml`
- **AND** `gh` is available
- **THEN** `gh api user --jq '.login'` is executed
- **AND** the result is written to `[github] username` in `werx.toml`
- **AND** the fetched username is used for the branch name

#### Scenario: Cached username used on subsequent calls
- **WHEN** `[github] username` is already set in `werx.toml`
- **THEN** no API call is made
- **AND** the cached value is used

#### Scenario: Username manually set in config
- **WHEN** `[github] username = "custom"` is set in `werx.toml`
- **THEN** `"custom"` is used as the username without any API call

#### Scenario: `gh` unavailable and no cached username
- **WHEN** `gh` is not in `$PATH` and no username is cached
- **THEN** the system prompts the user to enter their GitHub username
- **AND** the entered value is cached in `werx.toml` for future use

---

### Requirement: Coding agent configuration
The system SHALL read a `[agent]` table from `werx.toml` to configure the coding agent integration. The `agent` field specifies the agent CLI to use. Supported values are `"claude"` (default) and `"copilot"`. If absent, no agent integration is active.

#### Scenario: Claude configured
- **WHEN** `[agent] agent = "claude"` is set in `werx.toml`
- **THEN** the system invokes the `claude` CLI for agent-assisted operations

#### Scenario: Copilot configured
- **WHEN** `[agent] agent = "copilot"` is set in `werx.toml`
- **THEN** the system invokes the `gh copilot` CLI for agent-assisted operations

#### Scenario: No agent configured
- **WHEN** `werx.toml` has no `[agent]` table
- **THEN** agent-assisted features are skipped and fallback behaviour applies

#### Scenario: Configured agent CLI not found in PATH
- **WHEN** an agent is configured but its CLI binary is not found in `$PATH`
- **THEN** a warning is emitted: `"Configured agent '<name>' not found in PATH — skipping agent assistance"`
- **AND** fallback behaviour applies for the operation

---

### Requirement: AI-assisted topic slug generation
When a coding agent is configured and available, the branch naming service SHALL write a prompt containing the issue title and body to a temporary file and invoke the agent CLI, capturing stdout. The agent is asked to return a slug of at most 4 words wrapped in a `<branch-slug>…</branch-slug>` tag. The system SHALL extract the slug by scanning stdout for the first match of `<branch-slug>([^<]+)</branch-slug>` and normalizing the captured group. If no tag is found or the agent exits non-zero, the system SHALL fall back to the slugified issue title.

#### Scenario: Claude agent invoked
- **WHEN** `[agent] agent = "claude"` is configured and `claude` is in `$PATH`
- **THEN** the prompt is written to a temp file
- **AND** the command `cat <prompt-file> | claude --print` is executed as a subprocess
- **AND** stdout is captured for slug extraction

#### Scenario: Copilot agent invoked
- **WHEN** `[agent] agent = "copilot"` is configured and `copilot` is in `$PATH`
- **THEN** the prompt is written to a temp file
- **AND** the command `copilot -p $(<prompt-file>)` is executed as a subprocess
- **AND** stdout is captured for slug extraction

#### Scenario: Agent returns tagged slug
- **WHEN** agent stdout contains `<branch-slug>fix-null-pointer-crash</branch-slug>`
- **THEN** `"fix-null-pointer-crash"` is extracted and used as the topic slug after normalization

#### Scenario: Agent returns slug with surrounding commentary
- **WHEN** agent stdout contains prose text plus `<branch-slug>update-auth-flow</branch-slug>`
- **THEN** only the content inside the tag is used; surrounding text is ignored

#### Scenario: Agent stdout contains no tag
- **WHEN** agent exits zero but stdout has no `<branch-slug>…</branch-slug>` match
- **THEN** a warning is emitted: `"Agent returned no slug tag — using issue title as fallback"`
- **AND** the slugified issue title is used

#### Scenario: Agent invocation fails — fallback to title slug
- **WHEN** the agent CLI exits with a non-zero status
- **THEN** a warning is emitted: `"Agent slug generation failed — using issue title as fallback"`
- **AND** the slugified issue title is used instead

#### Scenario: Agent not configured — title slug used directly
- **WHEN** no `[agent]` table is present in `werx.toml`
- **THEN** no agent is invoked
- **AND** the slugified issue title is used as the default topic slug

---

### Requirement: Branch naming pattern configuration
The system SHALL read a `[github] branch_pattern` field from `werx.toml`. The default value when absent is `"username-issue-topic"`. If an unrecognized value is set, a warning is emitted and the default pattern is used.

#### Scenario: Default pattern used when config absent
- **WHEN** `werx.toml` has no `[github] branch_pattern` key
- **THEN** the `username-issue-topic` pattern is applied

#### Scenario: Configured pattern matches default
- **WHEN** `[github] branch_pattern = "username-issue-topic"` is set
- **THEN** branch naming behaves identically to the default

#### Scenario: Unrecognized pattern triggers warning
- **WHEN** `[github] branch_pattern = "unknown-pattern"` is set
- **THEN** a warning is printed: `"Unknown branch pattern 'unknown-pattern', using default"`
- **AND** the `username-issue-topic` pattern is applied
