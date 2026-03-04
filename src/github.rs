use anyhow::{Context, Result, anyhow};
use serde::Deserialize;
use std::process::Command;

use crate::cmd;

/// Check if the `gh` CLI is available in PATH.
pub fn is_gh_available() -> bool {
    Command::new("gh")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

// ── JSON response types ───────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct GhRepoView {
    #[serde(rename = "isFork")]
    pub is_fork: bool,
    pub parent: Option<GhParentRepo>,
    #[serde(rename = "defaultBranchRef")]
    pub default_branch_ref: GhBranchRef,
}

#[derive(Debug, Deserialize)]
pub struct GhParentRepo {
    /// e.g. "upstream-owner/repo"
    #[serde(rename = "nameWithOwner")]
    pub name_with_owner: String,
    #[serde(rename = "defaultBranchRef")]
    pub default_branch_ref: Option<GhBranchRef>,
}

#[derive(Debug, Deserialize)]
pub struct GhBranchRef {
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct GhIssue {
    pub title: String,
    pub body: String,
}

#[derive(Debug, Deserialize)]
pub struct GhPr {
    #[serde(rename = "headRefName")]
    pub head_ref_name: String,
}

// ── API functions ─────────────────────────────────────────────────────────────

/// Fetch repository metadata (fork status, parent, default branch).
pub fn fetch_repo_meta(owner: &str, repo: &str) -> Result<GhRepoView> {
    let repo_slug = format!("{}/{}", owner, repo);
    let output = cmd::run(
        Command::new("gh")
            .args(["repo", "view", &repo_slug, "--json", "isFork,parent,defaultBranchRef"]),
    )
    .context("Failed to run gh repo view")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("gh repo view failed for {}: {}", repo_slug, stderr));
    }

    let json = String::from_utf8_lossy(&output.stdout);
    serde_json::from_str(&json)
        .context(format!("Failed to parse gh repo view output for {}", repo_slug))
}

/// Fetch the authenticated user's GitHub login.
pub fn fetch_username() -> Result<String> {
    let output = cmd::run(Command::new("gh").args(["api", "user", "--jq", ".login"]))
        .context("Failed to run gh api user")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("gh api user failed: {}", stderr));
    }

    let username = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if username.is_empty() {
        return Err(anyhow!("gh api user returned empty username"));
    }
    Ok(username)
}

/// Fetch a GitHub issue's title and body.
pub fn fetch_issue(owner: &str, repo: &str, number: u64) -> Result<GhIssue> {
    let output = cmd::run(
        Command::new("gh").args([
            "issue",
            "view",
            &number.to_string(),
            "--repo",
            &format!("{}/{}", owner, repo),
            "--json",
            "title,body",
        ]),
    )
    .context("Failed to run gh issue view")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("gh issue view #{} failed: {}", number, stderr));
    }

    let json = String::from_utf8_lossy(&output.stdout);
    serde_json::from_str(&json).context("Failed to parse gh issue view output")
}

/// Fetch a GitHub PR's HEAD branch name.
pub fn fetch_pr(owner: &str, repo: &str, number: u64) -> Result<GhPr> {
    let output = cmd::run(
        Command::new("gh").args([
            "pr",
            "view",
            &number.to_string(),
            "--repo",
            &format!("{}/{}", owner, repo),
            "--json",
            "headRefName",
        ]),
    )
    .context("Failed to run gh pr view")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("gh pr view #{} failed: {}", number, stderr));
    }

    let json = String::from_utf8_lossy(&output.stdout);
    serde_json::from_str(&json).context("Failed to parse gh pr view output")
}

// ── URL parsing ───────────────────────────────────────────────────────────────

/// Parse a clone URL and return `(owner, repo)` if it is a GitHub URL.
/// Handles both SSH (`git@github.com:owner/repo.git`) and
/// HTTPS (`https://github.com/owner/repo.git`) formats.
pub fn parse_github_owner_repo(url: &str) -> Option<(String, String)> {
    let url = url.trim();
    let path = if let Some(p) = url.strip_prefix("git@github.com:") {
        p
    } else if let Some(p) = url.strip_prefix("https://github.com/") {
        p
    } else {
        return None;
    };
    let path = path.strip_suffix(".git").unwrap_or(path);
    let mut parts = path.splitn(2, '/');
    let owner = parts.next().filter(|s| !s.is_empty())?;
    let repo = parts.next().filter(|s| !s.is_empty())?;
    Some((owner.to_string(), repo.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_github_owner_repo_https() {
        let result = parse_github_owner_repo("https://github.com/alice/my-project.git");
        assert_eq!(result, Some(("alice".to_string(), "my-project".to_string())));
    }

    #[test]
    fn test_parse_github_owner_repo_ssh() {
        let result = parse_github_owner_repo("git@github.com:alice/my-project.git");
        assert_eq!(result, Some(("alice".to_string(), "my-project".to_string())));
    }

    #[test]
    fn test_parse_github_owner_repo_https_no_git_suffix() {
        let result = parse_github_owner_repo("https://github.com/alice/my-project");
        assert_eq!(result, Some(("alice".to_string(), "my-project".to_string())));
    }

    #[test]
    fn test_parse_github_owner_repo_non_github() {
        assert!(parse_github_owner_repo("https://gitlab.com/alice/repo.git").is_none());
        assert!(parse_github_owner_repo("git@gitlab.com:alice/repo.git").is_none());
    }

    #[test]
    fn test_parse_github_owner_repo_invalid() {
        assert!(parse_github_owner_repo("not-a-url").is_none());
        assert!(parse_github_owner_repo("https://github.com/").is_none());
    }
}
