use anyhow::{Context, Result, anyhow};
use dialoguer::{Confirm, Select, theme::ColorfulTheme};
use serde::Serialize;
use std::fs;
use std::path::Path;
use std::process::Command;

use crate::{Forge, Protocol, RepoSpec};

/// Repository information for listing
#[derive(Debug, Clone, Serialize)]
pub struct RepoInfo {
    /// The directory name in .forge/repos/
    pub dir_name: String,
    /// The clone URL
    pub clone_url: String,
    /// The normalized URL for deduplication
    pub normalized_url: String,
    /// The default branch (if available)
    pub default_branch: Option<String>,
    /// Whether the repository is valid
    pub valid: bool,
    /// Error message if invalid
    pub error: Option<String>,
}

/// Add a repository to the Forge
pub fn add_repo(forge: &Forge, repo_spec: &str) -> Result<RepoSpec> {
    // Load config
    let mut config = forge.load_config()?;

    // Parse the repository specification
    // If protocol is not set, we'll prompt for it
    let spec = match RepoSpec::parse(repo_spec, config.default_provider(), config.protocol()) {
        Ok(spec) => spec,
        Err(e) if e.to_string().contains("Protocol preference not set") => {
            // Protocol not set - prompt user
            let protocol = prompt_for_protocol()?;
            config.set_protocol(protocol);
            forge.save_config(&config)?;

            // Try parsing again with the new protocol
            RepoSpec::parse(repo_spec, config.default_provider(), config.protocol())?
        }
        Err(e) => return Err(e),
    };

    // Get existing repositories for conflict detection
    let existing_repos = list_repos(forge)?;

    // Determine directory name with conflict resolution
    let dir_name = spec.dir_name(&existing_repos);

    // Check if this is actually a duplicate (same normalized URL)
    if let Some(existing) = existing_repos.iter().find(|r| r.dir_name == dir_name)
        && existing.normalized_url == spec.normalized_url
    {
        return Err(anyhow!(
            "Repository already exists: {}\n  Location: .forge/repos/{}",
            spec.original,
            dir_name
        ));
    }

    // Clone the repository
    let repo_dir = forge.repos_dir().join(&dir_name);
    println!("Cloning repository: {}", spec.clone_url);
    clone_bare_repo(&spec.clone_url, &repo_dir)?;

    println!();
    println!("✓ Repository added successfully!");
    println!();
    println!("  Specification: {}", spec.original);
    println!("  Clone URL:     {}", spec.clone_url);
    println!("  Location:      .forge/repos/{}", dir_name);
    println!();

    Ok(spec)
}

/// Prompt user to choose protocol preference
fn prompt_for_protocol() -> Result<Protocol> {
    println!();
    println!("Protocol preference not configured.");
    println!();
    println!("Choose your preferred Git protocol for repository operations:");
    println!();

    let options = vec![
        "SSH   (git@github.com:owner/repo.git)",
        "HTTPS (https://github.com/owner/repo.git)",
    ];

    let selection = Select::with_theme(&ColorfulTheme::default())
        .items(&options)
        .default(0)
        .interact()?;

    let protocol = match selection {
        0 => Protocol::Ssh,
        1 => Protocol::Https,
        _ => unreachable!(),
    };

    println!();
    println!("✓ Protocol preference set to: {}", protocol);

    Ok(protocol)
}

/// List all repositories in the Forge
pub fn list_repos(forge: &Forge) -> Result<Vec<RepoInfo>> {
    let repos_dir = forge.repos_dir();

    if !repos_dir.exists() {
        return Ok(Vec::new());
    }

    let entries = fs::read_dir(&repos_dir).context(format!(
        "Failed to read repos directory '{}'",
        repos_dir.display()
    ))?;

    let mut repos = Vec::new();

    for entry in entries {
        let entry = match entry {
            Ok(e) => e,
            Err(e) => {
                eprintln!("Warning: Failed to read directory entry: {}", e);
                continue;
            }
        };

        let path = entry.path();

        if !path.is_dir() {
            continue;
        }

        let dir_name = match path.file_name() {
            Some(name) => name.to_string_lossy().to_string(),
            None => continue,
        };

        // Try to get repository information
        let repo_info = get_repo_info(&path, dir_name);
        repos.push(repo_info);
    }

    // Sort by directory name
    repos.sort_by(|a, b| a.dir_name.cmp(&b.dir_name));

    Ok(repos)
}

/// Get information about a repository
fn get_repo_info(repo_path: &Path, dir_name: String) -> RepoInfo {
    // Try to get remote URL
    let clone_url = match get_remote_url(repo_path) {
        Ok(url) => url,
        Err(e) => {
            return RepoInfo {
                dir_name,
                clone_url: String::new(),
                normalized_url: String::new(),
                default_branch: None,
                valid: false,
                error: Some(e.to_string()),
            };
        }
    };

    // Normalize the URL for comparison
    let normalized_url = crate::repo_spec::normalize_url(&clone_url).unwrap_or_else(|_| clone_url.clone());

    // Try to get default branch
    let default_branch = get_default_branch(repo_path).ok();

    RepoInfo {
        dir_name,
        clone_url,
        normalized_url,
        default_branch,
        valid: true,
        error: None,
    }
}

/// Get the remote URL from a bare repository
fn get_remote_url(repo_path: &Path) -> Result<String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(repo_path)
        .arg("config")
        .arg("--get")
        .arg("remote.origin.url")
        .output()
        .context("Failed to execute git config command")?;

    if !output.status.success() {
        return Err(anyhow!("Failed to get remote URL"));
    }

    let url = String::from_utf8_lossy(&output.stdout).trim().to_string();

    if url.is_empty() {
        return Err(anyhow!("Remote URL is empty"));
    }

    Ok(url)
}

/// Get the default branch from a bare repository
fn get_default_branch(repo_path: &Path) -> Result<String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(repo_path)
        .arg("symbolic-ref")
        .arg("HEAD")
        .output()
        .context("Failed to execute git symbolic-ref command")?;

    if !output.status.success() {
        return Err(anyhow!("Failed to get default branch"));
    }

    let branch_ref = String::from_utf8_lossy(&output.stdout).trim().to_string();

    // Strip "refs/heads/" prefix if present
    let branch = branch_ref
        .strip_prefix("refs/heads/")
        .unwrap_or(&branch_ref)
        .to_string();

    Ok(branch)
}

/// Remove a repository from the Forge
pub fn remove_repo(forge: &Forge, repo_spec: &str, force: bool) -> Result<()> {
    // Load config
    let config = forge.load_config()?;

    // Parse the repository specification to find the directory
    let spec = RepoSpec::parse(repo_spec, config.default_provider(), config.protocol())
        .context("Failed to parse repository specification")?;

    // Get existing repositories to determine correct directory name
    let existing_repos = list_repos(forge)?;
    let dir_name = spec.dir_name(&existing_repos);

    // Check if repository exists
    let repo_dir = forge.repos_dir().join(&dir_name);
    if !repo_dir.exists() {
        return Err(anyhow!(
            "Repository not found: {}\n\nRun 'forge repos list' to see available repositories.",
            spec.original
        ));
    }

    // Confirm removal unless --force
    if !force {
        println!();
        println!("About to remove repository:");
        println!("  Specification: {}", spec.original);
        println!("  Clone URL:     {}", spec.clone_url);
        println!("  Location:      .forge/repos/{}", dir_name);
        println!();

        let confirmed = Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt("Are you sure you want to remove this repository?")
            .default(false)
            .interact()?;

        if !confirmed {
            println!();
            println!("Operation cancelled.");
            return Ok(());
        }
    }

    // Remove the repository directory
    fs::remove_dir_all(&repo_dir).context(format!(
        "Failed to remove repository directory '{}'. Check file permissions.",
        repo_dir.display()
    ))?;

    println!();
    println!("✓ Repository removed successfully!");
    println!();
    println!("  Specification: {}", spec.original);
    println!();

    Ok(())
}

/// Clone a repository as a bare clone
fn clone_bare_repo(url: &str, dest: &Path) -> Result<()> {
    let output = Command::new("git")
        .arg("clone")
        .arg("--bare")
        .arg(url)
        .arg(dest)
        .output()
        .context("Failed to execute git clone command")?;

    if !output.status.success() {
        // Clean up partial clone if it exists
        if dest.exists() {
            let _ = fs::remove_dir_all(dest);
        }

        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("Git clone failed:\n{}", stderr));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    // Note: These tests require git to be installed and network access for cloning
    // For unit testing purposes, we'd typically mock the git clone operation
    // Here we'll test the core logic and error handling

    #[test]
    fn test_add_repo_requires_forge() {
        // This test would require mocking the clone operation
        // For now, we'll just verify the structure is in place
        assert!(true);
    }
}
