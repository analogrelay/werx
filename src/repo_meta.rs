use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

const REPO_META_FILE: &str = "werx-repo.toml";

/// GitHub metadata for a managed repository, stored in werx-repo.toml beside the bare clone.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoGithubMeta {
    pub owner: String,
    pub repo: String,
    pub is_fork: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub upstream_owner: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub upstream_repo: Option<String>,
    pub default_branch: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub upstream_default_branch: Option<String>,
}

/// Top-level TOML wrapper — the file uses a `[github]` table.
#[derive(Serialize, Deserialize)]
struct RepoMetaFile {
    github: RepoGithubMeta,
}

impl RepoGithubMeta {
    /// Load from `<repo_dir>/werx-repo.toml`.
    /// Returns `Ok(None)` if the file is absent; errors only on malformed TOML.
    pub fn load(repo_dir: &Path) -> Result<Option<Self>> {
        let path = repo_dir.join(REPO_META_FILE);
        if !path.exists() {
            return Ok(None);
        }
        let contents = fs::read_to_string(&path)
            .context(format!("Failed to read '{}'", path.display()))?;
        let file: RepoMetaFile = toml::from_str(&contents)
            .context(format!("Failed to parse '{}'", path.display()))?;
        Ok(Some(file.github))
    }

    /// Save to `<repo_dir>/werx-repo.toml`.
    pub fn save(&self, repo_dir: &Path) -> Result<()> {
        let path = repo_dir.join(REPO_META_FILE);
        let file = RepoMetaFile {
            github: self.clone(),
        };
        let contents = toml::to_string_pretty(&file).context("Failed to serialize repo meta")?;
        fs::write(&path, contents)
            .context(format!("Failed to write '{}'", path.display()))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn sample_meta() -> RepoGithubMeta {
        RepoGithubMeta {
            owner: "alice".to_string(),
            repo: "my-project".to_string(),
            is_fork: true,
            upstream_owner: Some("original-owner".to_string()),
            upstream_repo: Some("my-project".to_string()),
            default_branch: "main".to_string(),
            upstream_default_branch: Some("main".to_string()),
        }
    }

    #[test]
    fn test_round_trip_fork() {
        let dir = TempDir::new().unwrap();
        let meta = sample_meta();
        meta.save(dir.path()).unwrap();

        let loaded = RepoGithubMeta::load(dir.path()).unwrap().unwrap();
        assert_eq!(loaded.owner, "alice");
        assert_eq!(loaded.repo, "my-project");
        assert!(loaded.is_fork);
        assert_eq!(loaded.upstream_owner.as_deref(), Some("original-owner"));
        assert_eq!(loaded.upstream_repo.as_deref(), Some("my-project"));
        assert_eq!(loaded.default_branch, "main");
        assert_eq!(loaded.upstream_default_branch.as_deref(), Some("main"));
    }

    #[test]
    fn test_round_trip_non_fork() {
        let dir = TempDir::new().unwrap();
        let meta = RepoGithubMeta {
            owner: "bob".to_string(),
            repo: "solo-project".to_string(),
            is_fork: false,
            upstream_owner: None,
            upstream_repo: None,
            default_branch: "main".to_string(),
            upstream_default_branch: None,
        };
        meta.save(dir.path()).unwrap();

        let loaded = RepoGithubMeta::load(dir.path()).unwrap().unwrap();
        assert!(!loaded.is_fork);
        assert!(loaded.upstream_owner.is_none());
    }

    #[test]
    fn test_missing_file_returns_none() {
        let dir = TempDir::new().unwrap();
        let result = RepoGithubMeta::load(dir.path()).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_malformed_toml_returns_error() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("werx-repo.toml"), b"not valid toml ][[[").unwrap();
        let result = RepoGithubMeta::load(dir.path());
        assert!(result.is_err());
    }

    #[test]
    fn test_toml_structure_uses_github_table() {
        let dir = TempDir::new().unwrap();
        let meta = sample_meta();
        meta.save(dir.path()).unwrap();

        let contents = fs::read_to_string(dir.path().join("werx-repo.toml")).unwrap();
        assert!(contents.contains("[github]"), "expected [github] table header");
        assert!(contents.contains("owner = \"alice\""));
        assert!(contents.contains("is_fork = true"));
    }
}
