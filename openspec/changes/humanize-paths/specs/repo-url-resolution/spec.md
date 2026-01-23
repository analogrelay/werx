# Spec Delta: repo-url-resolution

## MODIFIED Requirements

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

## REMOVED Requirements

### Requirement: Deterministic Repository Directory

~~The system SHALL generate a unique, deterministic directory name from the normalized clone URL using the format `<name>-<hash>`.~~

**Rationale**: Replaced by human-readable directory name generation with progressive qualification.

The following scenarios are removed:
- ~~Generate name-hash directory format~~
- ~~Extract repository base name~~
- ~~Same URL produces same directory~~
- ~~Different URLs produce different directories~~
- ~~Same base name with different URLs~~

**Note**: Directory name generation is now context-dependent (checks for conflicts), not purely deterministic from URL alone.
