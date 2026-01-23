# Design: Humanize Repository and Workspace Paths

## Architecture

### Current Implementation

```
RepoSpec {
    name: "linux",           // Extracted from URL
    hash: "a1b2c3d4e5f6",   // SHA-256(normalized_url)[..12]
}
-> dir_name() = "linux-a1b2c3d4e5f6"
-> workspace path = "~/forge/linux-a1b2c3d4e5f6/main/"
```

### New Implementation

```
RepoSpec {
    name: "linux",
    owner: "torvalds",       // NEW: Extracted from URL
    hash: "a1b2c3",         // Shortened to 6 chars
}
-> dir_name(existing_repos) = "linux" | "torvalds-linux" | "torvalds-linux-a1b2c3"
-> workspace path = "~/forge/linux/main/" (or qualified variants)
```

## Owner Extraction

### Supported URL Patterns

**GitHub/GitLab HTTPS:**
- `https://github.com/torvalds/linux.git` → owner: `torvalds`
- `https://gitlab.com/gitlab-org/gitlab.git` → owner: `gitlab-org`

**GitHub/GitLab SSH:**
- `git@github.com:torvalds/linux.git` → owner: `torvalds`
- `git@gitlab.com:gitlab-org/gitlab.git` → owner: `gitlab-org`

**Generic Pattern:**
- Extract path after hostname
- Split by `/`
- Take first component as owner
- Take second component as name (already extracted)

**Fallback:**
- If owner cannot be extracted → treat as no-owner
- Still allows simple name, but hash fallback if conflicts

### Owner Normalization

- Lowercase for consistency
- Preserve hyphens and underscores (valid in usernames)
- Reject invalid filesystem characters

## Conflict Detection Algorithm

When adding a new repository with clone URL resolving to `(owner, name, hash)`:

```
1. Compute simple_name = name
2. Check if .forge/repos/{simple_name}/ exists
   - NO → use simple_name
   - YES → proceed to step 3

3. Load existing repository info at {simple_name}/
4. Compare normalized URLs:
   - SAME normalized URL → duplicate repository error
   - DIFFERENT normalized URL → proceed to step 5

5. Compute qualified_name = "{owner}-{name}"
6. Check if .forge/repos/{qualified_name}/ exists
   - NO → use qualified_name
   - YES → proceed to step 7

7. Load existing repository info at {qualified_name}/
8. Compare normalized URLs:
   - SAME normalized URL → duplicate repository error
   - DIFFERENT normalized URL → proceed to step 9

9. Compute hash_qualified_name = "{owner}-{name}-{hash}"
10. Check if .forge/repos/{hash_qualified_name}/ exists
    - NO → use hash_qualified_name
    - YES → load and compare (should never happen with 6-char hash collision probability)

11. If all names conflict with different URLs:
    - Return error (extremely unlikely hash collision)
```

## Directory Name Selection

### Progressive Qualification

| Attempt | Format | Example | When Used |
|---------|--------|---------|-----------|
| 1 | `<name>` | `linux` | No existing directory with this name |
| 2 | `<owner>-<name>` | `torvalds-linux` | Simple name exists but belongs to different repo |
| 3 | `<owner>-<name>-<hash>` | `torvalds-linux-a1b2c3` | Owner-qualified name exists but belongs to different repo |

### Hash Length Rationale

- **Current**: 12 characters from SHA-256
- **Proposed**: 6 characters from SHA-256
- **Collision probability**:
  - 6 hex chars = 16^6 = 16,777,216 possibilities
  - Collision only matters within same `<owner>-<name>` space
  - Birthday paradox: ~4,000 repos with same owner+name before 1% collision probability
  - Acceptable for this use case (users unlikely to have 4,000 forks of torvalds/linux)

## Data Structure Changes

### RepoSpec Extension

```rust
pub struct RepoSpec {
    pub original: String,
    pub clone_url: String,
    pub normalized_url: String,
    pub hash: String,        // Now 6 chars
    pub name: String,
    pub owner: Option<String>,  // NEW
}
```

### New Methods

```rust
impl RepoSpec {
    // NEW: Extract owner from clone URL
    fn extract_owner(url: &str) -> Option<String>;

    // MODIFIED: Takes existing repos to check conflicts
    pub fn dir_name(&self, existing_repos: &[RepoInfo]) -> String;

    // MODIFIED: Generate 6-char hash instead of 12
    fn generate_hash(url: &str) -> String;
}
```

### Forge Extension

```rust
impl Forge {
    // NEW: Helper to get directory name for a repo spec with conflict detection
    pub fn compute_repo_dir_name(&self, spec: &RepoSpec) -> Result<String>;
}
```

## Impact Analysis

### Modified Functions

1. `RepoSpec::parse()` - Add owner extraction
2. `RepoSpec::dir_name()` - Change to progressive qualification algorithm
3. `generate_hash()` - Reduce from 12 to 6 characters
4. `add_repo()` - Use conflict-aware dir_name resolution
5. `workspace::generate_workspace_path()` - Already uses repo dir_name, inherits changes

### Modified Specs

1. **repo-url-resolution** - Update directory name generation requirements
2. **repo-add** - Update storage path requirements and conflict detection
3. **workspace-create** - Update hierarchical storage path examples

### Unchanged Behavior

1. URL normalization logic
2. Git worktree functionality
3. Repository listing
4. Workspace listing
5. Removal operations (work with whatever directory name exists)

## Edge Cases

### No Owner Extractable

**Scenario**: User adds `https://git.company.internal/repo.git`

**Behavior**:
- `owner = None`
- Simple name: `repo`
- If conflict: fallback directly to `repo-{hash}` (skip owner qualification)

### Same Repo, Different Protocols

**Current behavior**: Treated as separate repositories

**Proposed behavior**: Still treated as separate (no change)

**Rationale**: Different normalized URLs = different repos per current spec

**Example**:
- `https://github.com/torvalds/linux.git` → `linux/` or `torvalds-linux/`
- `git@github.com:torvalds/linux.git` → `linux-a1b2c3/` (conflict with first)

**Future consideration**: Protocol-aware deduplication is out of scope

### Organization Names with Hyphens

**Scenario**: `https://github.com/my-org/my-repo.git`

**Behavior**:
- `owner = "my-org"`
- Qualified name: `my-org-my-repo`

**Consideration**: Ambiguous parsing (`my-org-my-repo` could be owner=my, name=org-my-repo)

**Mitigation**: Not a problem because directory name is derived from spec, not parsed back

### Fork Detection

**Scenario**: User wants `torvalds/linux` and `greg/linux` as different remotes of same repo

**Current proposal**: Treated as separate repositories
- `torvalds-linux/`
- `greg-linux/`

**Future work**: Implement fork detection and consolidation
- Detect forks via git remote comparison or user input
- Merge into single repo directory with multiple remotes
- Out of scope for this change

## Testing Strategy

### Unit Tests

1. `test_owner_extraction_github_https()`
2. `test_owner_extraction_github_ssh()`
3. `test_owner_extraction_gitlab()`
4. `test_owner_extraction_no_owner()`
5. `test_dir_name_simple_no_conflict()`
6. `test_dir_name_owner_qualified_on_conflict()`
7. `test_dir_name_hash_qualified_on_double_conflict()`
8. `test_short_hash_length()`

### Integration Tests

1. Add repository with no conflicts → simple name used
2. Add second repository with same name → owner qualification
3. Add repository with no extractable owner → hash fallback
4. Workspace creation uses correct directory name
5. Listing shows correct paths

## Migration Considerations

### For Existing Users

This is a **breaking change** - existing repository directories and workspaces will not work with the new naming scheme.

**Migration approach**: Fresh start only
- User backs up any uncommitted work from existing workspaces
- Re-initialize forge (or clear `.forge/` directory)
- Re-add repositories with `forge add` (uses new naming)
- Recreate workspaces as needed

**Justification**:
- Project is in early development with single user
- UX improvement is worth the breaking change
- Automated migration tooling would add complexity for no current benefit
- Fresh start is simplest and ensures clean state
