# Proposal: Humanize Repository and Workspace Paths

## Problem Statement

Currently, repository storage uses a hash-based directory naming scheme (`<name>-<hash>`) which creates workspace paths like `~/forge/linux-a1b2c3d4e5f6/main/`. This hash suffix:

1. Makes paths less readable and harder to remember
2. Appears unnecessarily in user-facing workspace paths
3. Creates a poor user experience when navigating the filesystem
4. Makes it difficult to know which repository a workspace belongs to at a glance

The hash was introduced to handle repository name conflicts (e.g., different repositories both named "linux"), but conflicts are rare in practice and the current approach optimizes for the edge case at the expense of the common case.

## Proposed Solution

Change the repository and workspace naming to prioritize human-readable paths, with automatic conflict resolution only when actually needed:

### Primary Approach: Simple Names

- **Repository storage**: `.forge/repos/<name>/` (e.g., `.forge/repos/linux/`)
- **Workspace paths**: `<forge-root>/<name>/<workspace>/` (e.g., `~/forge/linux/main/`)
- **No hash suffix** for the common case where names don't conflict

### Conflict Resolution: Owner Qualification

When a repository name conflicts with an existing repository:

- **Repository storage**: `.forge/repos/<owner>-<name>/` (e.g., `.forge/repos/torvalds-linux/`, `.forge/repos/greg-linux/`)
- **Workspace paths**: `<forge-root>/<owner>-<name>/<workspace>/` (e.g., `~/forge/torvalds-linux/main/`)
- **Owner extracted from clone URL** (GitHub: `github.com/owner/repo`, GitLab: `gitlab.com/owner/repo`)

### Fallback: Hash-Based Qualification

If owner-qualified name still conflicts (extremely rare), fall back to hash:

- **Repository storage**: `.forge/repos/<owner>-<name>-<short-hash>/`
- **Short hash**: 6 characters (sufficient for disambiguation within same owner/name combination)

### Fork Handling

Forks are handled explicitly as remotes rather than implicit name conflicts:

- Same underlying repository = same directory
- Different clone URLs for forks = additional git remotes
- This requires future work to detect and manage forks as remotes
- For this proposal: forks are treated as separate repositories (conflict resolution applies)

## Benefits

1. **Human-readable paths**: `~/forge/linux/main` instead of `~/forge/linux-a1b2c3d4e5f6/main`
2. **Common case optimization**: 95%+ of users won't see hashes in their paths
3. **Clear ownership**: When conflicts occur, owner prefix provides clear context
4. **Reasonable path lengths**: Owner prefix is shorter than full hash for most providers
5. **Backwards incompatible but necessary**: This is a fundamental UX improvement worth the breaking change

## Trade-offs

1. **Breaking change**: Existing repositories will need migration or re-cloning
2. **Conflict detection complexity**: Requires checking existing repos and extracting owners
3. **Fork detection deferred**: True fork handling (same repo, multiple remotes) is future work
4. **Owner extraction**: Assumes standard provider URL formats (GitHub, GitLab, etc.)

## Scope

This proposal covers:

1. Modifying repository directory name generation (`RepoSpec::dir_name()`)
2. Implementing conflict detection on `forge add`
3. Extracting owner information from clone URLs
4. Updating workspace path generation to use new repository directory names
5. Updating all specs that reference directory naming

Out of scope (future work):

1. Automatic migration of existing repositories
2. Fork detection and remote consolidation
3. Custom owner aliases or renaming

## Implementation Strategy

1. Extend `RepoSpec` to include owner extraction
2. Modify `dir_name()` to generate simple names by default
3. Implement conflict detection in `add_repo()` that checks existing directories
4. Apply progressive qualification (simple → owner-qualified → hash-qualified)
5. Update workspace path generation to use new directory names
6. Update spec deltas for affected capabilities

## Migration Path

For existing forges (currently single user):

**Fresh start only**: Re-initialize forge and re-clone repositories with new naming scheme.

Since the project is in early development with a single user, automated migration tooling is not needed. Users can simply:
1. Backup any work from existing workspaces
2. Run `forge init` in a new or cleared directory
3. Re-add repositories (which will use new naming)
4. Recreate needed workspaces

This approach is acceptable given the current user base and development stage.
