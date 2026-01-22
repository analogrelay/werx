# repo-url-resolution Specification

## Purpose
Provides flexible repository specification through URL resolution, allowing users to specify repositories using full clone URLs or convenient shorthand notation with configurable provider defaults.

## ADDED Requirements

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

### Requirement: Deterministic Repository Directory

The system SHALL generate a unique, deterministic directory name from the normalized clone URL using the format `<name>-<hash>`.

#### Scenario: Generate name-hash directory format

- **WHEN** repository URL is normalized
- **THEN** the repository base name is extracted from the URL path
- **AND** a deterministic hash is computed from the normalized URL
- **AND** the directory name follows the format `<name>-<hash>` in `.forge/repos/`

#### Scenario: Extract repository base name

- **WHEN** repository URL is `https://github.com/owner/myproject.git`
- **THEN** the base name is `myproject`
- **AND** the directory name is `myproject-<hash>`

#### Scenario: Same URL produces same directory

- **WHEN** the same repository URL is resolved multiple times
- **THEN** it always produces the same directory name

#### Scenario: Different URLs produce different directories

- **WHEN** two different repository URLs are resolved
- **THEN** they produce different directory names with different hashes

#### Scenario: Same base name with different URLs

- **WHEN** two different repositories have the same base name (e.g., both named `utils`)
- **THEN** they produce different directory names due to different hash suffixes

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

- **WHEN** user adds `https://github.com/owner/repo.git`
- **AND** later adds `git@github.com:owner/repo.git`
- **THEN** both are treated as separate repositories with different directory names

#### Scenario: Different hostnames treated as distinct

- **WHEN** user adds `https://github.com/owner/repo.git`
- **AND** later adds `https://www.github.com/owner/repo.git`
- **THEN** both are treated as separate repositories after normalization

#### Scenario: No cross-protocol deduplication

- **WHEN** determining if a repository already exists
- **THEN** only exact normalized URL matches are detected as duplicates
- **AND** no provider-specific equivalence rules are applied
