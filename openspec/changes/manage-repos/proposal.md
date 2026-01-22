# Proposal: Manage Repos

## Change ID
`manage-repos`

## Summary
Add repository management commands to the Forge CLI that enable users to add, list, and remove repositories. Repositories are stored as bare Git clones in the Forge's internal directory structure, using deterministic naming based on clone URLs. The commands support flexible repository specification through full URLs or shorthand notation.

## Motivation
The Forge system needs a way to manage repositories that can be discovered and used for creating workspaces. Currently, `forge init` creates the directory structure including `.forge/repos/`, but there's no way to populate it with repositories or manage them.

Users need to:
- Add repositories to the Forge from remote sources (GitHub, GitLab, etc.)
- List all repositories managed by the Forge
- Remove repositories when they're no longer needed
- Specify repositories flexibly using either full clone URLs or convenient shorthand notation

## Scope
This change introduces three new commands:
- `forge repos add <repo>` - Add a repository to the Forge
- `forge repos list` - List all repositories in the Forge
- `forge repos remove <repo>` - Remove a repository from the Forge
- `forge add <repo>` - Alias for `forge repos add <repo>` for convenience

### In Scope
- Repository storage using bare Git clones in `.forge/repos/`
- Deterministic directory naming based on clone URL to ensure uniqueness
- Support for full clone URLs (e.g., `https://github.com/owner/repo.git`)
- Support for shorthand notation with provider prefix (e.g., `github:owner/repo`)
- Support for shorthand notation without provider (e.g., `owner/repo` using default provider)
- Configuration file for default provider settings in `.forge/`
- Prevention of duplicate repository additions
- Listing repositories with useful metadata
- Removing repositories from the Forge

### Out of Scope
- Repository syncing/updating (can be added later)
- Authentication management (relies on system Git configuration)
- Repository validation beyond Git operations
- Workspace creation (separate future capability)
- Multi-provider configuration (only one default provider)

## Proposed Changes

### New Capabilities
1. **repo-add** - Add repositories to the Forge
2. **repo-list** - List repositories in the Forge
3. **repo-remove** - Remove repositories from the Forge
4. **repo-url-resolution** - Resolve repository specifications to clone URLs

### Modified Capabilities
1. **forge-init** - Add protocol preference prompt during initialization

### Dependencies
- Requires: `forge-init` (repositories are stored in `.forge/repos/` which is created during init)
- Sequential: Must implement URL resolution before add/remove commands can work
- Parallel: Add, list, and remove commands can be implemented in parallel once URL resolution is complete

## Implementation Notes

### Repository Storage Structure
Repositories are stored as bare clones in `<forge-root>/.forge/repos/<name>-<hash>/` where:
- `<name>` is the base name of the repository (e.g., for `https://github.com/owner/repo.git`, the name is `repo`)
- `<hash>` is a deterministic hash derived from the normalized clone URL (truncated to reasonable length, e.g., 12 characters)

This naming scheme provides:
- Human-readable directory names that indicate what repository they contain
- Deterministic naming: Same URL always produces same directory name
- Uniqueness: The hash suffix prevents collisions when multiple repos have the same base name
- No problematic filesystem characters

### Configuration File
A configuration file at `.forge/config` stores:
- **Default provider**: The Git hosting provider to use for shorthand notation (defaults to "github")
- **Protocol preference**: Whether to use SSH or HTTPS for clone URLs (no default - prompts user if not set)
- Potentially other settings in future

The configuration file serves dual purposes:
- Stores Forge configuration settings
- Acts as the Forge marker file (replaces the separate `.forge/marker` file)

The presence of `.forge/config` indicates a valid Forge. This simplifies Forge detection and avoids maintaining two separate files for marker and configuration purposes.

**Protocol Preference Handling**:
- During `forge init`, the user is prompted to choose SSH or HTTPS as their preferred protocol
- If the config doesn't contain a protocol preference (e.g., for forges created before this feature), the user is prompted during the first `forge add` operation
- The chosen preference is saved to config for future operations
- This ensures all shorthand URLs (`github:owner/repo` or `owner/repo`) resolve consistently to the user's preferred protocol

### URL Resolution Logic
1. If input contains `://`, treat as full clone URL (use as-is)
2. If input contains `:` but not `://`, treat as `provider:owner/repo`
3. Otherwise, treat as `owner/repo` and use default provider from config (defaults to "github")
4. For shorthand formats (steps 2-3):
   - Read protocol preference from config (SSH or HTTPS)
   - If protocol preference is not set, prompt user to choose and save to config
   - Resolve to appropriate clone URL based on protocol:
     - HTTPS: `github:owner/repo` → `https://github.com/owner/repo.git`
     - SSH: `github:owner/repo` → `git@github.com:owner/repo.git`

### Deterministic Directory Naming
Directory names use the format `<name>-<hash>` where:
- `<name>` is extracted from the repository URL (the final path component without `.git` extension)
- `<hash>` is a SHA256 hash of the normalized clone URL, truncated to 12 characters

Example: `https://github.com/owner/myproject.git` → `myproject-a1b2c3d4e5f6`

This ensures:
- Human readability: Users can identify repositories by scanning directory names
- Deterministic: Same URL always produces same directory name
- Unique: Hash suffix prevents collisions between repos with same base name
- Filesystem safe: No problematic characters in directory names

## Known Limitations

### Multiple Clone URLs for Same Repository
A single Git repository may have multiple valid clone URLs (HTTPS vs SSH, different hostnames, etc.). The current design treats each distinct normalized URL as a separate repository.

**Examples of URLs that won't be deduplicated:**
- `https://github.com/owner/repo.git` vs `git@github.com:owner/repo.git`
- `https://github.com/owner/repo.git` vs `https://www.github.com/owner/repo.git`

**Rationale:**
- Some providers (like GitHub) have standardized URL formats that could enable HTTPS/SSH equivalence detection
- However, this is not universal across all Git hosting providers
- Implementing provider-specific logic adds significant complexity
- Users can work around this by consistently using one URL format

**Future Consideration:**
A future enhancement could add provider-specific URL canonicalization to detect common equivalences (e.g., GitHub HTTPS ↔ SSH), but this is explicitly out of scope for the initial implementation.

## Open Questions
None - the design is straightforward based on the requirements.

## Alternatives Considered

### Alternative: Use full URL as directory name
**Rejected** - URLs contain characters that are problematic for filesystems (slashes, colons) and can be very long.

### Alternative: Use sequential numbering for repositories
**Rejected** - Makes it impossible to detect duplicates without reading all repositories, and order-dependent names are fragile.

### Alternative: Use owner/repo structure in directory hierarchy
**Rejected** - Adds complexity, makes provider switching harder, and doesn't handle collisions between providers well.

### Alternative: Store config in separate file from marker
**Decided: Use config file as marker** - The `.forge/config` file serves as both the configuration store and the Forge marker. This is simpler than maintaining two files, and since the marker file isn't user-facing, there's no benefit to keeping it separate. Forge detection checks for the presence of `.forge/config` instead of `.forge/marker`.

## Related Changes
- Future: Repository syncing/pulling updates
- Future: Workspace creation from repositories
- Future: Repository status/health checking
