use anyhow::{Result, anyhow};
use sha2::{Sha256, Digest};
use crate::Protocol;

/// Represents a repository specification that can be resolved to a clone URL
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepoSpec {
    /// The original specification string provided by the user
    pub original: String,
    /// The resolved clone URL
    pub clone_url: String,
    /// The normalized URL used for deduplication
    pub normalized_url: String,
    /// The deterministic hash derived from the normalized URL
    pub hash: String,
    /// The base name of the repository
    pub name: String,
}

impl RepoSpec {
    /// Parse and resolve a repository specification
    ///
    /// Supports three formats:
    /// - Full URL: `https://github.com/owner/repo.git` or `git@github.com:owner/repo.git`
    /// - Provider prefix: `github:owner/repo` or `gitlab:owner/repo`
    /// - Shorthand: `owner/repo` (uses default provider from config)
    pub fn parse(
        spec: &str,
        default_provider: &str,
        protocol: Option<Protocol>,
    ) -> Result<Self> {
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

        Ok(RepoSpec {
            original,
            clone_url,
            normalized_url,
            hash,
            name,
        })
    }

    /// Get the directory name for this repository (format: `<name>-<hash>`)
    pub fn dir_name(&self) -> String {
        format!("{}-{}", self.name, self.hash)
    }
}

/// Resolve provider-prefixed shorthand (e.g., github:owner/repo)
fn resolve_provider_shorthand(spec: &str, protocol: Option<Protocol>) -> Result<String> {
    let protocol = protocol.ok_or_else(|| {
        anyhow!("Protocol preference not set. Please configure it first.")
    })?;

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
    let protocol = protocol.ok_or_else(|| {
        anyhow!("Protocol preference not set. Please configure it first.")
    })?;

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
        return Err(anyhow!("Invalid repository format: '{}'. Expected 'owner/repo'", owner_repo));
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

/// Normalize a URL to a canonical form for deduplication
fn normalize_url(url: &str) -> Result<String> {
    let url = url.trim();

    // Handle SSH format: git@host:owner/repo.git
    if url.starts_with("git@") {
        let without_prefix = &url[4..]; // Remove "git@"
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
        let without_scheme = if url.starts_with("https://") {
            &url[8..]
        } else {
            &url[7..]
        };

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

/// Generate a deterministic hash from a normalized URL (truncated to 12 characters)
fn generate_hash(url: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(url.as_bytes());
    let result = hasher.finalize();

    // Convert to hex and take first 12 characters
    let hex = format!("{:x}", result);
    hex[..12].to_string()
}

/// Extract repository name from a normalized URL
fn extract_repo_name(url: &str) -> Result<String> {
    // Find the last path component
    let path = if url.starts_with("git@") {
        // SSH format: git@host:owner/repo.git
        url.split(':').nth(1).ok_or_else(|| anyhow!("Invalid SSH URL"))?
    } else {
        // HTTPS format: https://host/owner/repo.git
        url.split("://").nth(1)
            .ok_or_else(|| anyhow!("Invalid URL"))?
    };

    // Get the last path component
    let name = path.split('/').last()
        .ok_or_else(|| anyhow!("Cannot extract repository name from URL"))?;

    // Remove .git suffix if present
    let name = if name.ends_with(".git") {
        &name[..name.len() - 4]
    } else {
        name
    };

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
        ).unwrap();

        assert_eq!(spec.clone_url, "https://github.com/owner/repo.git");
        assert_eq!(spec.normalized_url, "https://github.com/owner/repo.git");
        assert_eq!(spec.name, "repo");
        assert_eq!(spec.hash.len(), 12);
    }

    #[test]
    fn test_parse_full_ssh_url() {
        let spec = RepoSpec::parse(
            "git@github.com:owner/repo.git",
            "github",
            Some(Protocol::Ssh),
        ).unwrap();

        assert_eq!(spec.clone_url, "git@github.com:owner/repo.git");
        assert_eq!(spec.normalized_url, "git@github.com:owner/repo.git");
        assert_eq!(spec.name, "repo");
    }

    #[test]
    fn test_parse_github_shorthand_https() {
        let spec = RepoSpec::parse(
            "github:owner/repo",
            "github",
            Some(Protocol::Https),
        ).unwrap();

        assert_eq!(spec.clone_url, "https://github.com/owner/repo.git");
        assert_eq!(spec.normalized_url, "https://github.com/owner/repo.git");
        assert_eq!(spec.name, "repo");
    }

    #[test]
    fn test_parse_github_shorthand_ssh() {
        let spec = RepoSpec::parse(
            "github:owner/repo",
            "github",
            Some(Protocol::Ssh),
        ).unwrap();

        assert_eq!(spec.clone_url, "git@github.com:owner/repo.git");
        assert_eq!(spec.normalized_url, "git@github.com:owner/repo.git");
        assert_eq!(spec.name, "repo");
    }

    #[test]
    fn test_parse_gitlab_shorthand_https() {
        let spec = RepoSpec::parse(
            "gitlab:owner/repo",
            "github",
            Some(Protocol::Https),
        ).unwrap();

        assert_eq!(spec.clone_url, "https://gitlab.com/owner/repo.git");
        assert_eq!(spec.normalized_url, "https://gitlab.com/owner/repo.git");
        assert_eq!(spec.name, "repo");
    }

    #[test]
    fn test_parse_gitlab_shorthand_ssh() {
        let spec = RepoSpec::parse(
            "gitlab:owner/repo",
            "github",
            Some(Protocol::Ssh),
        ).unwrap();

        assert_eq!(spec.clone_url, "git@gitlab.com:owner/repo.git");
        assert_eq!(spec.normalized_url, "git@gitlab.com:owner/repo.git");
        assert_eq!(spec.name, "repo");
    }

    #[test]
    fn test_parse_owner_repo_default_provider_https() {
        let spec = RepoSpec::parse(
            "owner/repo",
            "github",
            Some(Protocol::Https),
        ).unwrap();

        assert_eq!(spec.clone_url, "https://github.com/owner/repo.git");
        assert_eq!(spec.normalized_url, "https://github.com/owner/repo.git");
        assert_eq!(spec.name, "repo");
    }

    #[test]
    fn test_parse_owner_repo_default_provider_ssh() {
        let spec = RepoSpec::parse(
            "owner/repo",
            "github",
            Some(Protocol::Ssh),
        ).unwrap();

        assert_eq!(spec.clone_url, "git@github.com:owner/repo.git");
        assert_eq!(spec.normalized_url, "git@github.com:owner/repo.git");
        assert_eq!(spec.name, "repo");
    }

    #[test]
    fn test_parse_without_protocol_fails() {
        let result = RepoSpec::parse(
            "owner/repo",
            "github",
            None,
        );

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Protocol preference not set"));
    }

    #[test]
    fn test_normalize_adds_git_suffix() {
        let spec = RepoSpec::parse(
            "https://github.com/owner/repo",
            "github",
            Some(Protocol::Https),
        ).unwrap();

        assert_eq!(spec.normalized_url, "https://github.com/owner/repo.git");
    }

    #[test]
    fn test_normalize_lowercase_hostname() {
        let spec = RepoSpec::parse(
            "https://GitHub.com/owner/repo.git",
            "github",
            Some(Protocol::Https),
        ).unwrap();

        assert_eq!(spec.normalized_url, "https://github.com/owner/repo.git");
    }

    #[test]
    fn test_same_url_produces_same_hash() {
        let spec1 = RepoSpec::parse(
            "https://github.com/owner/repo.git",
            "github",
            Some(Protocol::Https),
        ).unwrap();

        let spec2 = RepoSpec::parse(
            "https://github.com/owner/repo.git",
            "github",
            Some(Protocol::Https),
        ).unwrap();

        assert_eq!(spec1.hash, spec2.hash);
        assert_eq!(spec1.dir_name(), spec2.dir_name());
    }

    #[test]
    fn test_different_urls_produce_different_hashes() {
        let spec1 = RepoSpec::parse(
            "https://github.com/owner/repo1.git",
            "github",
            Some(Protocol::Https),
        ).unwrap();

        let spec2 = RepoSpec::parse(
            "https://github.com/owner/repo2.git",
            "github",
            Some(Protocol::Https),
        ).unwrap();

        assert_ne!(spec1.hash, spec2.hash);
        assert_ne!(spec1.dir_name(), spec2.dir_name());
    }

    #[test]
    fn test_https_and_ssh_urls_are_distinct() {
        let spec1 = RepoSpec::parse(
            "https://github.com/owner/repo.git",
            "github",
            Some(Protocol::Https),
        ).unwrap();

        let spec2 = RepoSpec::parse(
            "git@github.com:owner/repo.git",
            "github",
            Some(Protocol::Ssh),
        ).unwrap();

        assert_ne!(spec1.normalized_url, spec2.normalized_url);
        assert_ne!(spec1.hash, spec2.hash);
    }

    #[test]
    fn test_dir_name_format() {
        let spec = RepoSpec::parse(
            "https://github.com/owner/myproject.git",
            "github",
            Some(Protocol::Https),
        ).unwrap();

        assert_eq!(spec.name, "myproject");
        assert!(spec.dir_name().starts_with("myproject-"));
        assert_eq!(spec.dir_name().len(), "myproject-".len() + 12);
    }

    #[test]
    fn test_same_name_different_urls_have_different_dirs() {
        let spec1 = RepoSpec::parse(
            "https://github.com/owner1/utils.git",
            "github",
            Some(Protocol::Https),
        ).unwrap();

        let spec2 = RepoSpec::parse(
            "https://github.com/owner2/utils.git",
            "github",
            Some(Protocol::Https),
        ).unwrap();

        assert_eq!(spec1.name, "utils");
        assert_eq!(spec2.name, "utils");
        assert_ne!(spec1.dir_name(), spec2.dir_name());
    }

    #[test]
    fn test_unsupported_provider_fails() {
        let result = RepoSpec::parse(
            "bitbucket:owner/repo",
            "github",
            Some(Protocol::Https),
        );

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Unsupported provider"));
    }

    #[test]
    fn test_invalid_owner_repo_format_fails() {
        let result = RepoSpec::parse(
            "github:invalid",
            "github",
            Some(Protocol::Https),
        );

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid repository format"));
    }
}
