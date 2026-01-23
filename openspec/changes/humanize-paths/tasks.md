# Implementation Tasks: Humanize Repository and Workspace Paths

## Task 1: Add owner extraction to RepoSpec

**Description**: Extend `RepoSpec` struct and implement owner extraction from clone URLs.

**Files to modify**:
- `src/repo_spec.rs`

**Changes**:
1. Add `owner: Option<String>` field to `RepoSpec` struct
2. Implement `extract_owner(url: &str) -> Option<String>` function
   - Parse HTTPS URLs: `https://provider.com/owner/repo.git` → `owner`
   - Parse SSH URLs: `git@provider.com:owner/repo.git` → `owner`
   - Handle standard providers (GitHub, GitLab)
   - Return `None` for non-standard URL formats
3. Call `extract_owner()` in `RepoSpec::parse()` and populate `owner` field
4. Normalize owner to lowercase

**Tests**:
- `test_extract_owner_github_https()`
- `test_extract_owner_github_ssh()`
- `test_extract_owner_gitlab_https()`
- `test_extract_owner_gitlab_ssh()`
- `test_extract_owner_non_standard_url()`
- `test_owner_normalized_to_lowercase()`

**Validation**: Run `cargo test repo_spec` and verify all tests pass.

---

## Task 2: Shorten hash length from 12 to 6 characters

**Description**: Reduce hash length for directory qualification to 6 characters.

**Files to modify**:
- `src/repo_spec.rs`

**Changes**:
1. Modify `generate_hash()` to truncate SHA-256 to 6 characters instead of 12
2. Update `hash` field documentation to reflect 6-character length

**Tests**:
- `test_hash_length()` - verify hash is exactly 6 characters
- Update existing tests that assert hash length

**Validation**: Run `cargo test repo_spec` and verify hash-related tests pass.

---

## Task 3: Implement progressive qualification in dir_name()

**Description**: Replace deterministic `dir_name()` with conflict-aware progressive qualification.

**Files to modify**:
- `src/repo_spec.rs`
- `src/repos.rs`

**Changes**:
1. Change `RepoSpec::dir_name()` signature from `pub fn dir_name(&self) -> String` to `pub fn dir_name(&self, existing_repos: &[RepoInfo]) -> String`
2. Implement progressive qualification algorithm:
   - Try simple name (`{name}`)
   - Check if exists and has different URL → try owner-qualified (`{owner}-{name}`)
   - Check if exists and has different URL → use hash-qualified (`{owner}-{name}-{hash}` or `{name}-{hash}` if no owner)
   - If URL matches at any level → this indicates duplicate (caller should handle)
3. Add helper function `check_dir_conflict(dir_name: &str, spec: &RepoSpec, existing_repos: &[RepoInfo]) -> ConflictResult` where `ConflictResult` is `NoConflict | Duplicate | Different`

**Tests**:
- `test_dir_name_simple_no_conflict()`
- `test_dir_name_owner_qualified_on_conflict()`
- `test_dir_name_hash_qualified_on_double_conflict()`
- `test_dir_name_detects_duplicate_simple()`
- `test_dir_name_detects_duplicate_qualified()`
- `test_dir_name_no_owner_uses_hash_on_conflict()`

**Validation**: Run `cargo test repo_spec` and verify directory naming logic.

---

## Task 4: Update add_repo to use conflict-aware directory naming

**Description**: Modify repository addition to use new progressive qualification and detect duplicates properly.

**Files to modify**:
- `src/repos.rs`

**Changes**:
1. Update `add_repo()` to:
   - Call `list_repos()` to get existing repositories
   - Pass existing repos to `spec.dir_name()`
   - Check for duplicate before attempting clone (if `dir_name()` signals duplicate)
   - Use the returned directory name for `clone_bare_repo()`
2. Improve error messages:
   - "Repository already exists" → include which directory name matched
   - "Using owner-qualified name due to conflict" → informational message when qualification happens
3. Update success message to show human-readable directory name

**Tests**:
- Integration test: Add repository with no conflicts → simple name
- Integration test: Add second repo with same name → owner qualification
- Integration test: Attempt to add duplicate → error

**Validation**: Run `cargo test repos` and verify repository addition logic.

---

## Task 5: Update workspace path generation

**Description**: Verify workspace paths use new repository directory names (should inherit from `dir_name()` changes).

**Files to modify**:
- `src/workspace.rs` (review only, likely no changes needed)

**Changes**:
1. Review `generate_workspace_path()` - verify it uses `repo_info.dir_name` correctly
2. Review `create_worktree()` - verify it uses generated paths correctly
3. Update any hardcoded examples in comments or error messages

**Tests**:
- `test_generate_workspace_path()` - update to test both simple and qualified names
- Integration test: Create workspace for repo with simple name
- Integration test: Create workspace for repo with qualified name

**Validation**: Run `cargo test workspace` and verify workspace paths are correct.

---

## Task 6: Update all affected call sites

**Description**: Find and update all locations that call `dir_name()` to pass existing repositories.

**Files to modify**:
- `src/repos.rs`
- `src/workspace.rs`
- Any other files calling `RepoSpec::dir_name()`

**Changes**:
1. Search for all `.dir_name()` calls: `rg "\.dir_name\(\)" --type rust`
2. Update each call site to:
   - Load existing repos via `list_repos()` if not already available
   - Pass existing repos to `dir_name()`
3. Handle any compile errors from signature change

**Validation**: Run `cargo build` and fix all compilation errors, then `cargo test`.

---

## Task 7: Update repository listing and removal

**Description**: Ensure `list_repos()` and `remove_repo()` work with new directory names.

**Files to modify**:
- `src/repos.rs`

**Changes**:
1. Review `list_repos()` - should work with any directory name (no changes expected)
2. Review `remove_repo()` - uses `spec.dir_name()`, update to pass existing repos
3. Review `get_repo_info()` - should work with any directory name (no changes expected)

**Tests**:
- Test listing repositories with simple, owner-qualified, and hash-qualified names
- Test removing repositories with various naming schemes

**Validation**: Run `cargo test repos::tests::test_list` and `cargo test repos::tests::test_remove`.

---

## Task 8: Update integration tests

**Description**: Add or update end-to-end integration tests covering the full workflow.

**Files to modify**:
- `tests/integration_test.rs` (or create if doesn't exist)

**Tests**:
1. **Test: Simple workflow**
   - Init forge
   - Add repository → verify simple name used
   - Create workspace → verify path is `<forge>/<name>/<workspace>/`
   - List workspaces → verify paths shown correctly

2. **Test: Conflict resolution workflow**
   - Init forge
   - Add first repo `owner1/utils` → verify simple name `utils/`
   - Add second repo `owner2/utils` → verify qualified name `owner2-utils/`
   - Create workspaces for both → verify correct paths
   - List workspaces → verify both repos distinguishable

3. **Test: No owner fallback**
   - Add repository with non-standard URL (no owner extractable)
   - Add second repo with same name → verify hash qualification

**Validation**: Run `cargo test --test integration_test` and verify all tests pass.

---

## Task 9: Update spec deltas with implementation notes

**Description**: Review and finalize spec delta documents after implementation.

**Files to modify**:
- `openspec/changes/humanize-paths/specs/*/spec.md`

**Changes**:
1. Add any edge cases discovered during implementation
2. Update scenarios if implementation revealed gaps
3. Ensure all requirements are testable and tested

**Validation**: Read through each spec delta and verify implementation matches.

---

## Dependencies

```
Task 1 (owner extraction)
  ↓
Task 2 (shorten hash) ───→ Task 3 (progressive qualification)
                              ↓
                           Task 4 (update add_repo)
                              ↓
Task 5 (workspace paths) ←────┤
  ↓                            ↓
Task 6 (update call sites) ←──┘
  ↓
Task 7 (listing/removal)
  ↓
Task 8 (integration tests)
  ↓
Task 9 (finalize specs)
```

## Parallelizable Work

- Tasks 1, 2 can be done concurrently
- Task 8 (integration tests) can start once Tasks 1-7 are complete

## Estimated Complexity

- **Task 1**: Medium (owner extraction logic)
- **Task 2**: Low (simple truncation change)
- **Task 3**: High (complex conflict detection algorithm)
- **Task 4**: Medium (integrate with existing add_repo flow)
- **Task 5**: Low (likely no changes needed)
- **Task 6**: Low (mechanical updates)
- **Task 7**: Low (review and verify)
- **Task 8**: Medium (comprehensive integration tests)
- **Task 9**: Low (review and polish)

## Acceptance Criteria

All tasks complete when:
1. `cargo build` succeeds with no warnings
2. `cargo test` passes all unit and integration tests
3. `cargo clippy` reports no issues
4. Manual testing confirms:
   - Adding first repo uses simple name
   - Adding second repo with same name uses owner qualification
   - Workspaces created in correct directories with readable paths
   - Listing shows human-readable paths
5. All spec deltas validated with `openspec validate humanize-paths --strict`
