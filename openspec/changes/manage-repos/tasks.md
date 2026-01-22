# Implementation Tasks: manage-repos

## Task List

### 1. Implement configuration file handling
- Create config module for reading/writing `.forge/config`
- Support default provider setting with fallback to "github"
- Support protocol preference setting (SSH or HTTPS, no default)
- Update `Forge` struct to include config access methods
- Migrate from `.forge/marker` to config-based Forge detection
- **Validation**: Unit tests for config read/write, default values, protocol preference
- **Dependencies**: None - foundational work
- **User-visible**: Enables provider and protocol configuration for shorthand URLs

### 2. Implement URL resolution and normalization
- Create `RepoSpec` type to represent repository specifications
- Implement parsing logic for full URLs, provider:owner/repo, and owner/repo formats
- Implement protocol-aware URL resolution (SSH vs HTTPS based on config)
- Implement URL normalization (lowercase hostname, add .git suffix, etc.)
- Implement deterministic hash generation from normalized URLs
- Implement `<name>-<hash>` directory naming format
- Support common providers (github, gitlab) with both SSH and HTTPS formats
- **Validation**: Unit tests for all URL formats, protocols, and normalization rules
- **Dependencies**: Task 1 (needs config for default provider and protocol preference)
- **User-visible**: Users can specify repos in flexible formats with their preferred protocol

### 3. Implement `forge repos add` command
- Add CLI command definition for `forge repos add <repo-spec>`
- Add top-level `forge add <repo-spec>` alias
- Implement repository spec parsing and URL resolution
- Implement protocol preference prompting if not set in config
- Save protocol preference to config after prompting
- Implement duplicate detection using normalized URL hash
- Implement bare git clone to `.forge/repos/<name>-<hash>/`
- Handle clone failures and clean up partial clones
- Display clone progress and success feedback
- **Validation**: Integration tests for adding repos, duplicate detection, protocol prompting
- **Dependencies**: Task 2 (needs URL resolution)
- **User-visible**: Users can add repositories to the Forge and set protocol preference

### 4. Implement `forge repos list` command
- Add CLI command definition for `forge repos list`
- Enumerate directories in `.forge/repos/`
- Read repository metadata from bare clones (remote URL, default branch)
- Format output in readable tabular format
- Implement `--format json` option for machine-readable output
- Handle missing/corrupted repository directories gracefully
- **Validation**: Integration tests for listing repos, empty list, JSON format
- **Dependencies**: Task 3 (need repos to list)
- **User-visible**: Users can see what repositories are in their Forge
- **Parallelizable**: Can implement alongside Task 5

### 5. Implement `forge repos remove` command
- Add CLI command definition for `forge repos remove <repo-spec>`
- Implement repository lookup using URL resolution and hash
- Add confirmation prompt before removal
- Implement `--force` flag to skip confirmation
- Remove repository directory completely from `.forge/repos/`
- Handle not-found and permission errors
- Display success feedback
- **Validation**: Integration tests for removing repos, confirmation, errors
- **Dependencies**: Task 2 (needs URL resolution)
- **User-visible**: Users can remove repositories from the Forge
- **Parallelizable**: Can implement alongside Task 4

### 6. Update `forge init` with protocol prompting
- Add protocol preference prompt during `forge init`
- Add `--protocol <ssh|https>` flag to skip prompt
- Save protocol preference to `.forge/config`
- Update success message to mention protocol preference
- **Validation**: Integration tests for init with protocol prompting
- **Dependencies**: Task 1 (config handling)
- **User-visible**: Users set protocol preference when initializing Forge

### 7. Update documentation and help text
- Update command help text for all new commands
- Update `forge init` help text to mention protocol preference
- Update success messages to mention `forge add`
- Add examples to command help showing both SSH and HTTPS usage
- **Validation**: Manual review of help output
- **Dependencies**: Tasks 3, 4, 5, 6 (commands must exist)
- **User-visible**: Users have clear guidance on using repo commands

### 8. Integration testing
- Create end-to-end test scenarios covering typical workflows
- Test adding repos with various URL formats
- Test listing repos after adding multiple
- Test removing repos and verifying they're gone
- Test error cases (no forge, invalid URLs, duplicates)
- Test config file creation, provider defaults, and protocol preference
- Test protocol prompting during init and add operations
- Test both SSH and HTTPS URL generation
- **Validation**: All integration tests pass
- **Dependencies**: All previous tasks
- **User-visible**: Ensures reliable, bug-free experience

## Task Relationships

**Sequential Dependencies**:
- Task 2 depends on Task 1 (config for default provider and protocol)
- Task 3 depends on Task 2 (URL resolution)
- Task 5 depends on Task 2 (URL resolution)
- Task 6 depends on Task 1 (config handling for protocol)
- Task 7 depends on Tasks 3, 4, 5, 6 (commands exist)
- Task 8 depends on all previous tasks

**Parallel Opportunities**:
- Tasks 4 and 5 can be implemented in parallel after Task 2 completes
- Task 6 can be implemented in parallel with Tasks 3, 4, 5 (only depends on Task 1)

## Milestone Progress

After each task, users can:
1. ✓ Configure default provider and protocol preference
2. ✓ Use flexible repo URL formats with protocol awareness
3. ✓ Add repositories with automatic protocol prompting
4. ✓ See what repositories are available
5. ✓ Remove repositories they no longer need
6. ✓ Set protocol preference during Forge initialization
7. ✓ Get help on using repo commands
8. ✓ Rely on thoroughly tested repo management
