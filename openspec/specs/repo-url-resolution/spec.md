# repo-url-resolution Specification

## Purpose
TBD - created by archiving change manage-repos. Update Purpose after archive.
## Requirements
### Requirement: Full URL Recognition

The system SHALL recognize and accept full Git clone URLs without modification.

#### Scenario: Accept HTTPS clone URL

- **WHEN** user provides `https://github.com/owner/repo.git` as repository specification
- **THEN** the URL is used as-is for cloning

#### Scenario: Accept SSH clone URL

- **WHEN** user provides `git@github.com:owner/repo.git` as repository specification
- **THEN** the URL is used as-is for cloning

#### Scenario: Detect full URL by protocol

- **WHEN** repository specification contains `://`
- **THEN** it is treated as a full clone URL without further transformation

### Requirement: Provider-Prefixed Shorthand

The system SHALL support shorthand notation with explicit provider prefix.

#### Scenario: Resolve GitHub shorthand with HTTPS

- **WHEN** user provides `github:owner/repo` as repository specification
- **AND** protocol preference is HTTPS
- **THEN** it is resolved to `https://github.com/owner/repo.git`

#### Scenario: Resolve GitHub shorthand with SSH

- **WHEN** user provides `github:owner/repo` as repository specification
- **AND** protocol preference is SSH
- **THEN** it is resolved to `git@github.com:owner/repo.git`

#### Scenario: Resolve GitLab shorthand with HTTPS

- **WHEN** user provides `gitlab:owner/repo` as repository specification
- **AND** protocol preference is HTTPS
- **THEN** it is resolved to `https://gitlab.com/owner/repo.git`

#### Scenario: Resolve GitLab shorthand with SSH

- **WHEN** user provides `gitlab:owner/repo` as repository specification
- **AND** protocol preference is SSH
- **THEN** it is resolved to `git@gitlab.com:owner/repo.git`

#### Scenario: Detect provider prefix format

- **WHEN** repository specification contains `:` but not `://`
- **THEN** it is treated as provider-prefixed shorthand

### Requirement: Default Provider Shorthand

The system SHALL support shorthand notation without provider prefix using a configured default.

#### Scenario: Resolve using default provider with HTTPS

- **WHEN** user provides `owner/repo` as repository specification
- **AND** default provider is `github`
- **AND** protocol preference is HTTPS
- **THEN** it is resolved to `https://github.com/owner/repo.git`

#### Scenario: Resolve using default provider with SSH

- **WHEN** user provides `owner/repo` as repository specification
- **AND** default provider is `github`
- **AND** protocol preference is SSH
- **THEN** it is resolved to `git@github.com:owner/repo.git`

#### Scenario: Detect owner/repo format

- **WHEN** repository specification contains no `:` character
- **THEN** it is treated as owner/repo shorthand and default provider is used

### Requirement: Default Provider Configuration

The system SHALL store and retrieve the default provider from Forge configuration.

#### Scenario: Use GitHub as default provider

- **WHEN** no configuration exists or default provider is not set
- **THEN** `github` is used as the default provider

#### Scenario: Read default provider from config

- **WHEN** Forge configuration specifies a default provider
- **THEN** that provider is used for owner/repo shorthand resolution

#### Scenario: Configuration stored in .forge directory

- **WHEN** default provider is configured
- **THEN** it is stored in a configuration file at `<forge-root>/.forge/config`

#### Scenario: Config file serves as Forge marker

- **WHEN** checking if a directory is a Forge
- **THEN** the presence of `.forge/config` indicates a valid Forge
- **AND** replaces the need for a separate `.forge/marker` file

### Requirement: Protocol Preference Configuration

The system SHALL store and retrieve the protocol preference from Forge configuration.

#### Scenario: Read protocol preference from config

- **WHEN** resolving a shorthand repository specification
- **THEN** the protocol preference (SSH or HTTPS) is read from config
- **AND** used to determine the clone URL format

#### Scenario: No default protocol preference

- **WHEN** protocol preference is not set in config
- **THEN** the system cannot complete URL resolution
- **AND** must prompt the user or fail with appropriate error

#### Scenario: Protocol preference stored in config file

- **WHEN** protocol preference is configured
- **THEN** it is stored in `.forge/config`
- **AND** applies to all future shorthand URL resolutions

#### Scenario: HTTPS protocol generates HTTPS URLs

- **WHEN** protocol preference is HTTPS
- **THEN** shorthand URLs are resolved to HTTPS format
- **AND** example: `github:owner/repo` becomes `https://github.com/owner/repo.git`

#### Scenario: SSH protocol generates SSH URLs

- **WHEN** protocol preference is SSH
- **THEN** shorthand URLs are resolved to SSH format
- **AND** example: `github:owner/repo` becomes `git@github.com:owner/repo.git`

### Requirement: URL Normalization

The system SHALL normalize repository URLs to a canonical form for deduplication.

#### Scenario: Normalize GitHub HTTPS URL

- **WHEN** user provides `https://github.com/owner/repo`
- **THEN** it is normalized to `https://github.com/owner/repo.git`

#### Scenario: Normalize case in hostname

- **WHEN** user provides `https://GitHub.com/owner/repo.git`
- **THEN** hostname is normalized to lowercase `github.com`

#### Scenario: Consistent normalization for deduplication

- **WHEN** multiple equivalent URLs are provided
- **THEN** they all normalize to the same canonical form

### Requirement: Supported Git Providers

The system SHALL support common Git hosting providers through shorthand notation.

#### Scenario: Support GitHub provider

- **WHEN** provider is `github`
- **THEN** shorthand `github:owner/repo` resolves based on protocol preference
- **AND** HTTPS: `https://github.com/owner/repo.git`
- **AND** SSH: `git@github.com:owner/repo.git`

#### Scenario: Support GitLab provider

- **WHEN** provider is `gitlab`
- **THEN** shorthand `gitlab:owner/repo` resolves based on protocol preference
- **AND** HTTPS: `https://gitlab.com/owner/repo.git`
- **AND** SSH: `git@gitlab.com:owner/repo.git`

#### Scenario: Extensible provider support

- **WHEN** a recognized provider prefix is used
- **THEN** it can be expanded to the appropriate clone URL format

### Requirement: Multiple URL Limitation

The system SHALL treat different normalized URLs as distinct repositories, even if they reference the same underlying Git repository.

#### Scenario: HTTPS and SSH URLs treated as distinct

- **WHEN** user adds `https://github.com/torvalds/linux.git` as `linux/`
- **AND** later adds `git@github.com:torvalds/linux.git`
- **THEN** second repository uses owner-qualified name `torvalds-linux/`
- **AND** both are treated as separate repositories

#### Scenario: Different hostnames treated as distinct

- **WHEN** user adds `https://github.com/owner/repo.git`
- **AND** later adds `https://gitlab.com/owner/repo.git`
- **THEN** both may use simple name if no conflicts, or progressive qualification applies

### Requirement: Owner Extraction from Clone URL

The system SHALL extract the repository owner from the clone URL when available.

#### Scenario: Extract owner from GitHub HTTPS URL

- **WHEN** repository URL is `https://github.com/torvalds/linux.git`
- **THEN** owner is extracted as `torvalds`
- **AND** name is extracted as `linux`

#### Scenario: Extract owner from GitHub SSH URL

- **WHEN** repository URL is `git@github.com:torvalds/linux.git`
- **THEN** owner is extracted as `torvalds`
- **AND** name is extracted as `linux`

#### Scenario: Extract owner from GitLab HTTPS URL

- **WHEN** repository URL is `https://gitlab.com/gitlab-org/gitlab.git`
- **THEN** owner is extracted as `gitlab-org`
- **AND** name is extracted as `gitlab`

#### Scenario: Extract owner from GitLab SSH URL

- **WHEN** repository URL is `git@gitlab.com:gitlab-org/gitlab.git`
- **THEN** owner is extracted as `gitlab-org`
- **AND** name is extracted as `gitlab`

#### Scenario: No owner extractable from non-standard URL

- **WHEN** repository URL is `https://git.company.internal/repo.git`
- **AND** URL does not follow standard provider path format
- **THEN** owner is `None`
- **AND** name is still extracted as `repo`

#### Scenario: Normalize owner to lowercase

- **WHEN** repository URL contains `github.com/Torvalds/Linux.git`
- **THEN** owner is normalized to `torvalds`
- **AND** name is normalized to `linux`

### Requirement: Human-Readable Repository Directory Names

The system SHALL generate human-readable directory names without hash suffixes when possible, using progressive qualification on conflicts.

#### Scenario: Simple name when no conflict exists

- **WHEN** adding repository `torvalds/linux`
- **AND** no directory named `linux` exists in `.forge/repos/`
- **THEN** directory name is `linux`
- **AND** repository is stored at `.forge/repos/linux/`

#### Scenario: Owner-qualified name when simple name conflicts

- **WHEN** adding repository `torvalds/linux`
- **AND** directory `.forge/repos/linux/` already exists
- **AND** existing repository has different normalized URL
- **THEN** directory name is `torvalds-linux`
- **AND** repository is stored at `.forge/repos/torvalds-linux/`

#### Scenario: Hash-qualified name when owner-qualified name conflicts

- **WHEN** adding repository with normalized URL `https://github.com/torvalds/linux.git`
- **AND** directory `.forge/repos/linux/` exists (different repo)
- **AND** directory `.forge/repos/torvalds-linux/` exists (different repo)
- **THEN** directory name is `torvalds-linux-{hash}`
- **AND** hash is 6 hexadecimal characters
- **AND** repository is stored at `.forge/repos/torvalds-linux-{hash}/`

#### Scenario: Detect duplicate on simple name match

- **WHEN** adding repository `torvalds/linux`
- **AND** directory `.forge/repos/linux/` exists
- **AND** existing repository has same normalized URL
- **THEN** addition fails with duplicate repository error

#### Scenario: Detect duplicate on owner-qualified name match

- **WHEN** adding repository `torvalds/linux`
- **AND** directory `.forge/repos/torvalds-linux/` exists
- **AND** existing repository has same normalized URL
- **THEN** addition fails with duplicate repository error

#### Scenario: No owner fallback to hash on conflict

- **WHEN** adding repository with no extractable owner
- **AND** simple name conflicts with different repository
- **THEN** directory name is `{name}-{hash}`
- **AND** hash is 6 hexadecimal characters

### Requirement: Shortened Hash for Qualification

The system SHALL generate 6-character hexadecimal hashes for repository disambiguation when needed.

#### Scenario: Hash truncated to 6 characters

- **WHEN** hash is generated from normalized URL
- **THEN** SHA-256 hash is computed
- **AND** truncated to first 6 hexadecimal characters
- **AND** used for hash-qualified directory names

#### Scenario: Same URL produces same hash

- **WHEN** the same repository URL is resolved multiple times
- **THEN** it always produces the same 6-character hash

#### Scenario: Different URLs produce different hashes

- **WHEN** two different repository URLs are resolved
- **THEN** they produce different 6-character hashes

