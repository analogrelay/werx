use crate::Protocol;
use anyhow::{Result, anyhow};
use sha2::{Digest, Sha256};

/// Result of checking for directory name conflicts
#[derive(Debug, PartialEq, Eq)]
pub enum ConflictResult {
    /// No conflict - directory doesn't exist or is the same repository
    NoConflict,
    /// Same repository (same normalized URL)
    Duplicate,
    /// Different repository with same directory name
    Different,
}

/// Represents a repository specification that can be resolved to a clone URL
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepoSpec {
    /// The original specification string provided by the user
    pub original: String,
    /// The resolved clone URL
    pub clone_url: String,
    /// The normalized URL used for deduplication
    pub normalized_url: String,
    /// The deterministic hash derived from the normalized URL (6 characters)
    pub hash: String,
    /// The base name of the repository
    pub name: String,
    /// The owner extracted from the clone URL (if available)
    pub owner: Option<String>,
}

impl RepoSpec {
    /// Parse and resolve a repository specification
    ///
    /// Supports three formats:
    /// - Full URL: `https://github.com/owner/repo.git` or `git@github.com:owner/repo.git`
    /// - Provider prefix: `github:owner/repo` or `gitlab:owner/repo`
    /// - Shorthand: `owner/repo` (uses default provider from config)
    pub fn parse(spec: &str, default_provider: &str, protocol: Option<Protocol>) -> Result<Self> {
        let original = spec.to_string();

        // Determine if this is a full URL or shorthand
        let clone_url = if spec.contains("://") {
            // Full URL - use as-is
            spec.to_string()
        } else if spec.contains(':') && !spec.starts_with("git@") {
            // Provider-prefixed shorthand (e.g., github:owner/repo)
            resolve_provider_shorthand(spec, protocol)?
        } else if spec.starts_with("git@") {
            // SSH URL format - use as-is
            spec.to_string()
        } else {
            // Owner/repo shorthand - use default provider
            resolve_shorthand_with_provider(spec, default_provider, protocol)?
        };

        // Normalize the URL for deduplication
        let normalized_url = normalize_url(&clone_url)?;

        // Generate deterministic hash from normalized URL
        let hash = generate_hash(&normalized_url);

        // Extract repository name from URL
        let name = extract_repo_name(&normalized_url)?;

        // Extract owner from clone URL
        let owner = extract_owner(&clone_url);

        Ok(RepoSpec {
            original,
            clone_url,
            normalized_url,
            hash,
            name,
            owner,
        })
    }

    /// Get the directory name for this repository with conflict-aware progressive qualification
    ///
    /// Tries progressively more qualified names:
    /// 1. Simple: `<name>`
    /// 2. Owner-qualified: `<owner>-<name>` (if owner available)
    /// 3. Hash-qualified: `<owner>-<name>-<hash>` or `<name>-<hash>` (if no owner)
    ///
    /// Returns the first name that doesn't conflict with a different repository.
    pub fn dir_name(&self, existing_repos: &[crate::RepoInfo]) -> String {
        // Try simple name first
        let simple_name = self.name.clone();
        match check_dir_conflict(&simple_name, self, existing_repos) {
            ConflictResult::NoConflict => return simple_name,
            ConflictResult::Duplicate => return simple_name, // Same repo, use simple name
            ConflictResult::Different => {
                // Conflict with different repo, try owner-qualified
            }
        }

        // Try owner-qualified name
        if let Some(owner) = &self.owner {
            let qualified_name = format!("{}-{}", owner, self.name);
            match check_dir_conflict(&qualified_name, self, existing_repos) {
                ConflictResult::NoConflict => return qualified_name,
                ConflictResult::Duplicate => return qualified_name, // Same repo
                ConflictResult::Different => {
                    // Conflict with different repo, fall through to hash-qualified
                }
            }

            // Hash-qualified with owner
            return format!("{}-{}-{}", owner, self.name, self.hash);
        }

        // No owner available, use hash-qualified name directly
        format!("{}-{}", self.name, self.hash)
    }
}

/// Check if a directory name conflicts with existing repositories
///
/// Returns:
/// - `NoConflict`: Directory doesn't exist
/// - `Duplicate`: Directory exists and contains the same repository (same normalized URL)
/// - `Different`: Directory exists and contains a different repository
fn check_dir_conflict(
    dir_name: &str,
    spec: &RepoSpec,
    existing_repos: &[crate::RepoInfo],
) -> ConflictResult {
    // Find if any existing repo has this directory name
    if let Some(existing) = existing_repos.iter().find(|r| r.dir_name == dir_name) {
        // Directory exists - check if it's the same repository
        if existing.normalized_url == spec.normalized_url {
            ConflictResult::Duplicate
        } else {
            ConflictResult::Different
        }
    } else {
        ConflictResult::NoConflict
    }
}

/// Resolve provider-prefixed shorthand (e.g., github:owner/repo)
fn resolve_provider_shorthand(spec: &str, protocol: Option<Protocol>) -> Result<String> {
    let protocol = protocol
        .ok_or_else(|| anyhow!("Protocol preference not set. Please configure it first."))?;

    let parts: Vec<&str> = spec.splitn(2, ':').collect();
    if parts.len() != 2 {
        return Err(anyhow!("Invalid provider-prefixed format: {}", spec));
    }

    let provider = parts[0];
    let owner_repo = parts[1];

    resolve_url_for_provider(provider, owner_repo, protocol)
}

/// Resolve owner/repo shorthand using default provider
fn resolve_shorthand_with_provider(
    spec: &str,
    provider: &str,
    protocol: Option<Protocol>,
) -> Result<String> {
    let protocol = protocol
        .ok_or_else(|| anyhow!("Protocol preference not set. Please configure it first."))?;

    resolve_url_for_provider(provider, spec, protocol)
}

/// Resolve URL for a given provider and owner/repo
fn resolve_url_for_provider(
    provider: &str,
    owner_repo: &str,
    protocol: Protocol,
) -> Result<String> {
    // Ensure owner_repo has the right format
    if !owner_repo.contains('/') {
        return Err(anyhow!(
            "Invalid repository format: '{}'. Expected 'owner/repo'",
            owner_repo
        ));
    }

    let url = match (provider.to_lowercase().as_str(), protocol) {
        ("github", Protocol::Https) => format!("https://github.com/{}.git", owner_repo),
        ("github", Protocol::Ssh) => format!("git@github.com:{}.git", owner_repo),
        ("gitlab", Protocol::Https) => format!("https://gitlab.com/{}.git", owner_repo),
        ("gitlab", Protocol::Ssh) => format!("git@gitlab.com:{}.git", owner_repo),
        (provider, _) => {
            return Err(anyhow!("Unsupported provider: {}", provider));
        }
    };

    Ok(url)
}

/// Extract owner from a clone URL
///
/// Supports GitHub/GitLab URLs in both HTTPS and SSH formats:
/// - HTTPS: `https://github.com/owner/repo.git` → `owner`
/// - SSH: `git@github.com:owner/repo.git` → `owner`
///
/// Returns `None` for non-standard URL formats or if owner cannot be extracted.
fn extract_owner(url: &str) -> Option<String> {
    let url = url.trim();

    // Handle SSH format: git@host:owner/repo.git
    if let Some(without_prefix) = url.strip_prefix("git@") {
        let parts: Vec<&str> = without_prefix.splitn(2, ':').collect();

        if parts.len() == 2 {
            let path = parts[1];
            // Extract owner from path (first component)
            if let Some(owner) = path.split('/').next()
                && !owner.is_empty()
            {
                return Some(owner.to_lowercase());
            }
        }
        return None;
    }

    // Handle HTTPS format: https://host/owner/repo.git
    if url.starts_with("https://") || url.starts_with("http://") {
        let without_scheme = url
            .strip_prefix("https://")
            .or_else(|| url.strip_prefix("http://"))
            .unwrap();

        let parts: Vec<&str> = without_scheme.splitn(2, '/').collect();
        if parts.len() == 2 {
            let path = parts[1];
            // Extract owner from path (first component)
            if let Some(owner) = path.split('/').next()
                && !owner.is_empty()
            {
                return Some(owner.to_lowercase());
            }
        }
        return None;
    }

    None
}

/// Normalize a URL to a canonical form for deduplication
pub fn normalize_url(url: &str) -> Result<String> {
    let url = url.trim();

    // Handle SSH format: git@host:owner/repo.git
    if let Some(without_prefix) = url.strip_prefix("git@") {
        let parts: Vec<&str> = without_prefix.splitn(2, ':').collect();

        if parts.len() != 2 {
            return Err(anyhow!("Invalid SSH URL format: {}", url));
        }

        let host = parts[0].to_lowercase();
        let path = parts[1];
        let path = ensure_git_suffix(path);

        return Ok(format!("git@{}:{}", host, path));
    }

    // Handle HTTPS format: https://host/owner/repo.git
    if url.starts_with("https://") || url.starts_with("http://") {
        let without_scheme = url
            .strip_prefix("https://")
            .or_else(|| url.strip_prefix("http://"))
            .unwrap();

        let parts: Vec<&str> = without_scheme.splitn(2, '/').collect();
        if parts.len() != 2 {
            return Err(anyhow!("Invalid HTTPS URL format: {}", url));
        }

        let host = parts[0].to_lowercase();
        let path = parts[1];
        let path = ensure_git_suffix(path);

        // Always normalize to https
        return Ok(format!("https://{}/{}", host, path));
    }

    Err(anyhow!("Unsupported URL format: {}", url))
}

/// Ensure path ends with .git suffix
fn ensure_git_suffix(path: &str) -> String {
    if path.ends_with(".git") {
        path.to_string()
    } else {
        format!("{}.git", path)
    }
}

/// Generate a deterministic hash from a normalized URL (truncated to 6 characters)
fn generate_hash(url: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(url.as_bytes());
    let result = hasher.finalize();

    // Convert to hex and take first 6 characters
    let hex = format!("{:x}", result);
    hex[..6].to_string()
}

/// Extract repository name from a normalized URL
fn extract_repo_name(url: &str) -> Result<String> {
    // Find the last path component
    let path = if url.starts_with("git@") {
        // SSH format: git@host:owner/repo.git
        url.split(':')
            .nth(1)
            .ok_or_else(|| anyhow!("Invalid SSH URL"))?
    } else {
        // HTTPS format: https://host/owner/repo.git
        url.split("://")
            .nth(1)
            .ok_or_else(|| anyhow!("Invalid URL"))?
    };

    // Get the last path component
    let name = path
        .split('/')
        .next_back()
        .ok_or_else(|| anyhow!("Cannot extract repository name from URL"))?;

    // Remove .git suffix if present
    let name = name.strip_suffix(".git").unwrap_or(name);

    if name.is_empty() {
        return Err(anyhow!("Empty repository name"));
    }

    Ok(name.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_full_https_url() {
        let spec = RepoSpec::parse(
            "https://github.com/owner/repo.git",
            "github",
            Some(Protocol::Https),
        )
        .unwrap();

        assert_eq!(spec.clone_url, "https://github.com/owner/repo.git");
        assert_eq!(spec.normalized_url, "https://github.com/owner/repo.git");
        assert_eq!(spec.name, "repo");
        assert_eq!(spec.owner, Some("owner".to_string()));
        assert_eq!(spec.hash.len(), 6);
    }

    #[test]
    fn test_parse_full_ssh_url() {
        let spec = RepoSpec::parse(
            "git@github.com:owner/repo.git",
            "github",
            Some(Protocol::Ssh),
        )
        .unwrap();

        assert_eq!(spec.clone_url, "git@github.com:owner/repo.git");
        assert_eq!(spec.normalized_url, "git@github.com:owner/repo.git");
        assert_eq!(spec.name, "repo");
    }

    #[test]
    fn test_parse_github_shorthand_https() {
        let spec = RepoSpec::parse("github:owner/repo", "github", Some(Protocol::Https)).unwrap();

        assert_eq!(spec.clone_url, "https://github.com/owner/repo.git");
        assert_eq!(spec.normalized_url, "https://github.com/owner/repo.git");
        assert_eq!(spec.name, "repo");
    }

    #[test]
    fn test_parse_github_shorthand_ssh() {
        let spec = RepoSpec::parse("github:owner/repo", "github", Some(Protocol::Ssh)).unwrap();

        assert_eq!(spec.clone_url, "git@github.com:owner/repo.git");
        assert_eq!(spec.normalized_url, "git@github.com:owner/repo.git");
        assert_eq!(spec.name, "repo");
    }

    #[test]
    fn test_parse_gitlab_shorthand_https() {
        let spec = RepoSpec::parse("gitlab:owner/repo", "github", Some(Protocol::Https)).unwrap();

        assert_eq!(spec.clone_url, "https://gitlab.com/owner/repo.git");
        assert_eq!(spec.normalized_url, "https://gitlab.com/owner/repo.git");
        assert_eq!(spec.name, "repo");
    }

    #[test]
    fn test_parse_gitlab_shorthand_ssh() {
        let spec = RepoSpec::parse("gitlab:owner/repo", "github", Some(Protocol::Ssh)).unwrap();

        assert_eq!(spec.clone_url, "git@gitlab.com:owner/repo.git");
        assert_eq!(spec.normalized_url, "git@gitlab.com:owner/repo.git");
        assert_eq!(spec.name, "repo");
    }

    #[test]
    fn test_parse_owner_repo_default_provider_https() {
        let spec = RepoSpec::parse("owner/repo", "github", Some(Protocol::Https)).unwrap();

        assert_eq!(spec.clone_url, "https://github.com/owner/repo.git");
        assert_eq!(spec.normalized_url, "https://github.com/owner/repo.git");
        assert_eq!(spec.name, "repo");
    }

    #[test]
    fn test_parse_owner_repo_default_provider_ssh() {
        let spec = RepoSpec::parse("owner/repo", "github", Some(Protocol::Ssh)).unwrap();

        assert_eq!(spec.clone_url, "git@github.com:owner/repo.git");
        assert_eq!(spec.normalized_url, "git@github.com:owner/repo.git");
        assert_eq!(spec.name, "repo");
    }

    #[test]
    fn test_parse_without_protocol_fails() {
        let result = RepoSpec::parse("owner/repo", "github", None);

        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Protocol preference not set")
        );
    }

    #[test]
    fn test_normalize_adds_git_suffix() {
        let spec = RepoSpec::parse(
            "https://github.com/owner/repo",
            "github",
            Some(Protocol::Https),
        )
        .unwrap();

        assert_eq!(spec.normalized_url, "https://github.com/owner/repo.git");
    }

    #[test]
    fn test_normalize_lowercase_hostname() {
        let spec = RepoSpec::parse(
            "https://GitHub.com/owner/repo.git",
            "github",
            Some(Protocol::Https),
        )
        .unwrap();

        assert_eq!(spec.normalized_url, "https://github.com/owner/repo.git");
    }

    #[test]
    fn test_same_url_produces_same_hash() {
        let spec1 = RepoSpec::parse(
            "https://github.com/owner/repo.git",
            "github",
            Some(Protocol::Https),
        )
        .unwrap();

        let spec2 = RepoSpec::parse(
            "https://github.com/owner/repo.git",
            "github",
            Some(Protocol::Https),
        )
        .unwrap();

        assert_eq!(spec1.hash, spec2.hash);
        assert_eq!(spec1.dir_name(&[]), spec2.dir_name(&[]));
    }

    #[test]
    fn test_different_urls_produce_different_hashes() {
        let spec1 = RepoSpec::parse(
            "https://github.com/owner/repo1.git",
            "github",
            Some(Protocol::Https),
        )
        .unwrap();

        let spec2 = RepoSpec::parse(
            "https://github.com/owner/repo2.git",
            "github",
            Some(Protocol::Https),
        )
        .unwrap();

        assert_ne!(spec1.hash, spec2.hash);
        assert_ne!(spec1.dir_name(&[]), spec2.dir_name(&[]));
    }

    #[test]
    fn test_https_and_ssh_urls_are_distinct() {
        let spec1 = RepoSpec::parse(
            "https://github.com/owner/repo.git",
            "github",
            Some(Protocol::Https),
        )
        .unwrap();

        let spec2 = RepoSpec::parse(
            "git@github.com:owner/repo.git",
            "github",
            Some(Protocol::Ssh),
        )
        .unwrap();

        assert_ne!(spec1.normalized_url, spec2.normalized_url);
        assert_ne!(spec1.hash, spec2.hash);
    }

    #[test]
    fn test_dir_name_format() {
        let spec = RepoSpec::parse(
            "https://github.com/owner/myproject.git",
            "github",
            Some(Protocol::Https),
        )
        .unwrap();

        assert_eq!(spec.name, "myproject");
        // With no conflicts, should use simple name
        assert_eq!(spec.dir_name(&[]), "myproject");
    }

    #[test]
    fn test_same_name_different_urls_have_different_dirs() {
        use crate::RepoInfo;

        let spec1 = RepoSpec::parse(
            "https://github.com/owner1/utils.git",
            "github",
            Some(Protocol::Https),
        )
        .unwrap();

        let spec2 = RepoSpec::parse(
            "https://github.com/owner2/utils.git",
            "github",
            Some(Protocol::Https),
        )
        .unwrap();

        assert_eq!(spec1.name, "utils");
        assert_eq!(spec2.name, "utils");

        // First repo gets simple name
        assert_eq!(spec1.dir_name(&[]), "utils");

        // Second repo should get owner-qualified name when first exists
        let existing_repos = vec![RepoInfo {
            dir_name: "utils".to_string(),
            clone_url: spec1.clone_url.clone(),
            normalized_url: spec1.normalized_url.clone(),
            default_branch: Some("main".to_string()),
            valid: true,
            error: None,
        }];
        assert_eq!(spec2.dir_name(&existing_repos), "owner2-utils");
    }

    #[test]
    fn test_unsupported_provider_fails() {
        let result = RepoSpec::parse("bitbucket:owner/repo", "github", Some(Protocol::Https));

        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Unsupported provider")
        );
    }

    #[test]
    fn test_invalid_owner_repo_format_fails() {
        let result = RepoSpec::parse("github:invalid", "github", Some(Protocol::Https));

        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Invalid repository format")
        );
    }

    #[test]
    fn test_extract_owner_github_https() {
        let spec = RepoSpec::parse(
            "https://github.com/torvalds/linux.git",
            "github",
            Some(Protocol::Https),
        )
        .unwrap();

        assert_eq!(spec.owner, Some("torvalds".to_string()));
        assert_eq!(spec.name, "linux");
    }

    #[test]
    fn test_extract_owner_github_ssh() {
        let spec = RepoSpec::parse(
            "git@github.com:torvalds/linux.git",
            "github",
            Some(Protocol::Ssh),
        )
        .unwrap();

        assert_eq!(spec.owner, Some("torvalds".to_string()));
        assert_eq!(spec.name, "linux");
    }

    #[test]
    fn test_extract_owner_gitlab_https() {
        let spec = RepoSpec::parse(
            "https://gitlab.com/gitlab-org/gitlab.git",
            "gitlab",
            Some(Protocol::Https),
        )
        .unwrap();

        assert_eq!(spec.owner, Some("gitlab-org".to_string()));
        assert_eq!(spec.name, "gitlab");
    }

    #[test]
    fn test_extract_owner_gitlab_ssh() {
        let spec = RepoSpec::parse(
            "git@gitlab.com:gitlab-org/gitlab.git",
            "gitlab",
            Some(Protocol::Ssh),
        )
        .unwrap();

        assert_eq!(spec.owner, Some("gitlab-org".to_string()));
        assert_eq!(spec.name, "gitlab");
    }

    #[test]
    fn test_extract_owner_non_standard_url() {
        // Test with a URL that might not have an extractable owner
        let spec = RepoSpec::parse(
            "https://git.company.internal/repo.git",
            "github",
            Some(Protocol::Https),
        )
        .unwrap();

        // Since this URL has a path component, it should extract "repo" as owner
        // But the structure is different - this tests the edge case
        assert_eq!(spec.name, "repo");
    }

    #[test]
    fn test_owner_normalized_to_lowercase() {
        let spec = RepoSpec::parse(
            "https://github.com/MyOrg/MyRepo.git",
            "github",
            Some(Protocol::Https),
        )
        .unwrap();

        assert_eq!(spec.owner, Some("myorg".to_string()));
        assert_eq!(spec.name, "MyRepo"); // Note: name is NOT normalized
    }

    #[test]
    fn test_hash_length() {
        let spec = RepoSpec::parse(
            "https://github.com/owner/repo.git",
            "github",
            Some(Protocol::Https),
        )
        .unwrap();

        assert_eq!(spec.hash.len(), 6);
    }

    #[test]
    fn test_dir_name_simple_no_conflict() {
        let spec = RepoSpec::parse(
            "https://github.com/torvalds/linux.git",
            "github",
            Some(Protocol::Https),
        )
        .unwrap();

        let existing_repos = vec![];
        assert_eq!(spec.dir_name(&existing_repos), "linux");
    }

    #[test]
    fn test_dir_name_owner_qualified_on_conflict() {
        use crate::RepoInfo;

        let spec = RepoSpec::parse(
            "https://github.com/torvalds/linux.git",
            "github",
            Some(Protocol::Https),
        )
        .unwrap();

        // Simulate an existing repo with name "linux" but different URL
        let existing_repos = vec![RepoInfo {
            dir_name: "linux".to_string(),
            clone_url: "https://github.com/greg/linux.git".to_string(),
            normalized_url: "https://github.com/greg/linux.git".to_string(),
            default_branch: Some("main".to_string()),
            valid: true,
            error: None,
        }];

        assert_eq!(spec.dir_name(&existing_repos), "torvalds-linux");
    }

    #[test]
    fn test_dir_name_hash_qualified_on_double_conflict() {
        use crate::RepoInfo;

        let spec = RepoSpec::parse(
            "https://github.com/user3/linux.git",
            "github",
            Some(Protocol::Https),
        )
        .unwrap();

        // Simulate existing repos with both simple and owner-qualified names
        let existing_repos = vec![
            RepoInfo {
                dir_name: "linux".to_string(),
                clone_url: "https://github.com/torvalds/linux.git".to_string(),
                normalized_url: "https://github.com/torvalds/linux.git".to_string(),
                default_branch: Some("main".to_string()),
                valid: true,
                error: None,
            },
            RepoInfo {
                dir_name: "user3-linux".to_string(),
                clone_url: "https://github.com/user2/linux.git".to_string(),
                normalized_url: "https://github.com/user2/linux.git".to_string(),
                default_branch: Some("main".to_string()),
                valid: true,
                error: None,
            },
        ];

        // Should use hash-qualified name
        let dir_name = spec.dir_name(&existing_repos);
        assert!(dir_name.starts_with("user3-linux-"));
        assert_eq!(dir_name.len(), "user3-linux-".len() + 6);
    }

    #[test]
    fn test_dir_name_detects_duplicate_simple() {
        use crate::RepoInfo;

        let spec = RepoSpec::parse(
            "https://github.com/torvalds/linux.git",
            "github",
            Some(Protocol::Https),
        )
        .unwrap();

        // Same repo already exists with simple name
        let existing_repos = vec![RepoInfo {
            dir_name: "linux".to_string(),
            clone_url: "https://github.com/torvalds/linux.git".to_string(),
            normalized_url: "https://github.com/torvalds/linux.git".to_string(),
            default_branch: Some("main".to_string()),
            valid: true,
            error: None,
        }];

        // Should still return simple name (it's the same repo)
        assert_eq!(spec.dir_name(&existing_repos), "linux");
    }

    #[test]
    fn test_dir_name_detects_duplicate_qualified() {
        use crate::RepoInfo;

        let spec = RepoSpec::parse(
            "https://github.com/torvalds/linux.git",
            "github",
            Some(Protocol::Https),
        )
        .unwrap();

        // Same repo already exists with qualified name
        let existing_repos = vec![RepoInfo {
            dir_name: "torvalds-linux".to_string(),
            clone_url: "https://github.com/torvalds/linux.git".to_string(),
            normalized_url: "https://github.com/torvalds/linux.git".to_string(),
            default_branch: Some("main".to_string()),
            valid: true,
            error: None,
        }];

        // Should return simple name since it doesn't conflict
        assert_eq!(spec.dir_name(&existing_repos), "linux");
    }

    #[test]
    fn test_dir_name_no_owner_uses_hash_on_conflict() {
        use crate::RepoInfo;

        // Create a spec with a URL that doesn't have an extractable owner
        let spec = RepoSpec {
            original: "custom".to_string(),
            clone_url: "https://git.internal/repo.git".to_string(),
            normalized_url: "https://git.internal/repo.git".to_string(),
            hash: "abc123".to_string(),
            name: "repo".to_string(),
            owner: None,
        };

        // Simulate an existing repo with same name
        let existing_repos = vec![RepoInfo {
            dir_name: "repo".to_string(),
            clone_url: "https://github.com/someone/repo.git".to_string(),
            normalized_url: "https://github.com/someone/repo.git".to_string(),
            default_branch: Some("main".to_string()),
            valid: true,
            error: None,
        }];

        // Should use hash-qualified name (no owner to qualify with)
        assert_eq!(spec.dir_name(&existing_repos), "repo-abc123");
    }
}
