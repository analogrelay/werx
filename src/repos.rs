use anyhow::{Context, Result, anyhow};
use console::style;
use dialoguer::{Confirm, Select, theme::ColorfulTheme};
use serde::Serialize;
use std::fs;
use std::path::Path;
use std::process::Command;

use crate::{AppContext, Protocol, RepoGithubMeta, RepoSpec, Werx, cmd, github};
use crate::reporter::OperationHandle;

/// Repository information for listing
#[derive(Debug, Clone, Serialize)]
pub struct RepoInfo {
    /// The directory name in .werx/repos/
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

/// Add a repository to the Werx
pub fn add_repo(werx: &Werx, repo_spec: &str, ctx: &AppContext) -> Result<RepoSpec> {
    // Load config
    let mut config = werx.load_config()?;

    // Parse the repository specification
    // If protocol is not set, we'll prompt for it
    let spec = match RepoSpec::parse(repo_spec, config.default_provider(), config.protocol()) {
        Ok(spec) => spec,
        Err(e) if e.to_string().contains("Protocol preference not set") => {
            // Protocol not set - prompt user
            let protocol = prompt_for_protocol()?;
            config.set_protocol(protocol);
            werx.save_config(&config)?;

            // Try parsing again with the new protocol
            RepoSpec::parse(repo_spec, config.default_provider(), config.protocol())?
        }
        Err(e) => return Err(e),
    };

    // Get existing repositories for conflict detection
    let existing_repos = list_repos(werx)?;

    // Determine directory name with conflict resolution
    let dir_name = spec.dir_name(&existing_repos);

    // Check if this is actually a duplicate (same normalized URL)
    if let Some(existing) = existing_repos.iter().find(|r| r.dir_name == dir_name)
        && existing.normalized_url == spec.normalized_url
    {
        return Err(anyhow!(
            "Repository already exists: {}\n  Location: .werx/repos/{}",
            spec.original,
            dir_name
        ));
    }

    // Clone the repository
    let repo_dir = werx.repos_dir().join(&dir_name);
    tracing::info!("Cloning repository: {}", spec.clone_url);
    let handle = ctx.reporter.start_operation(&format!("Cloning {}", spec.clone_url));
    match clone_bare_repo(&spec.clone_url, &repo_dir, &handle) {
        Ok(()) => handle.finish_ok(&format!("Cloned {}", spec.clone_url)),
        Err(e) => {
            handle.finish_err(&format!("Failed to clone {}", spec.clone_url));
            return Err(e);
        }
    }

    // Detect and persist GitHub fork metadata (best-effort; never aborts the add)
    detect_and_save_fork_meta(&spec, &repo_dir, config.protocol());

    tracing::info!("Repository added successfully: {} → .werx/repos/{}", spec.clone_url, dir_name);
    ctx.reporter.println(&format!(
        "\n{}\n\n  Specification: {}\n  Clone URL:     {}\n  Location:      .werx/repos/{}\n",
        style("Repository added successfully!").bold().green(),
        style(&spec.original).cyan(),
        spec.clone_url,
        style(&dir_name).cyan(),
    ));

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

/// List all repositories in the Werx
pub fn list_repos(werx: &Werx) -> Result<Vec<RepoInfo>> {
    let repos_dir = werx.repos_dir();

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
                tracing::warn!("Failed to read directory entry: {}", e);
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
    let normalized_url =
        crate::repo_spec::normalize_url(&clone_url).unwrap_or_else(|_| clone_url.clone());

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
    let output = cmd::run(Command::new("git")
        .arg("-C")
        .arg(repo_path)
        .arg("config")
        .arg("--get")
        .arg("remote.origin.url"))
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
    let output = cmd::run(Command::new("git")
        .arg("-C")
        .arg(repo_path)
        .arg("symbolic-ref")
        .arg("HEAD"))
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

/// Remove a repository from the Werx
pub fn remove_repo(werx: &Werx, repo_spec: &str, force: bool) -> Result<()> {
    // Load config
    let config = werx.load_config()?;

    // Parse the repository specification to find the directory
    let spec = RepoSpec::parse(repo_spec, config.default_provider(), config.protocol())
        .context("Failed to parse repository specification")?;

    // Get existing repositories to determine correct directory name
    let existing_repos = list_repos(werx)?;
    let dir_name = spec.dir_name(&existing_repos);

    // Check if repository exists
    let repo_dir = werx.repos_dir().join(&dir_name);
    if !repo_dir.exists() {
        return Err(anyhow!(
            "Repository not found: {}\n\nRun 'werx repos list' to see available repositories.",
            spec.original
        ));
    }

    // Confirm removal unless --force
    if !force {
        println!();
        println!("About to remove repository:");
        println!("  Specification: {}", spec.original);
        println!("  Clone URL:     {}", spec.clone_url);
        println!("  Location:      .werx/repos/{}", dir_name);
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

// ── Fork detection ────────────────────────────────────────────────────────────

/// After a successful bare clone, detect whether the repo is a GitHub fork and persist metadata.
/// Silently skips if `gh` is unavailable or the remote is not a GitHub URL.
/// Prints a warning on API failure but never aborts the `add_repo` operation.
fn detect_and_save_fork_meta(spec: &RepoSpec, repo_dir: &Path, protocol: Option<Protocol>) {
    let (owner, repo_name) = match github::parse_github_owner_repo(&spec.clone_url) {
        Some(pair) => pair,
        None => return, // Not a GitHub URL
    };

    if !github::is_gh_available() {
        tracing::debug!(
            "gh CLI not available, skipping fork detection for {}/{}",
            owner,
            repo_name
        );
        return;
    }

    let gh_meta = match github::fetch_repo_meta(&owner, &repo_name) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("Warning: failed to fetch GitHub metadata for {}/{}: {}", owner, repo_name, e);
            return;
        }
    };

    let default_branch = gh_meta.default_branch_ref.name.clone();

    let (is_fork, upstream_owner, upstream_repo, upstream_default_branch, upstream_url) =
        if gh_meta.is_fork {
            if let Some(parent) = gh_meta.parent {
                let up_owner = parent.owner.login.clone();
                let up_repo = parent.name.clone();
                if up_owner.is_empty() || up_repo.is_empty() {
                    eprintln!("Warning: could not parse upstream repo from '{}'", parent.name_with_owner());
                    return;
                }
                let up_branch = parent.default_branch_ref.as_ref().map(|b| b.name.clone());
                let url = generate_upstream_url(&spec.clone_url, &up_owner, &up_repo, protocol);
                (true, Some(up_owner), Some(up_repo), up_branch, Some(url))
            } else {
                (true, None, None, None, None)
            }
        } else {
            (false, None, None, None, None)
        };

    let meta = RepoGithubMeta {
        owner,
        repo: repo_name,
        is_fork,
        upstream_owner,
        upstream_repo,
        default_branch,
        upstream_default_branch,
    };

    if let Err(e) = meta.save(repo_dir) {
        eprintln!("Warning: failed to save fork metadata: {}", e);
        return;
    }

    if let Some(url) = upstream_url {
        if let Err(e) = ensure_upstream_remote(repo_dir, &url) {
            eprintln!("Warning: failed to configure upstream remote: {}", e);
        }
    }
}

/// Add the `upstream` remote if absent, or update its URL if it points elsewhere.
pub fn ensure_upstream_remote(repo_dir: &Path, upstream_url: &str) -> Result<()> {
    let check = cmd::run(
        Command::new("git")
            .args(["-C", &repo_dir.to_string_lossy()])
            .args(["remote", "get-url", "upstream"]),
    )
    .context("Failed to query upstream remote")?;

    if check.status.success() {
        let existing = String::from_utf8_lossy(&check.stdout).trim().to_string();
        if existing == upstream_url {
            tracing::debug!("upstream remote already correct ({})", upstream_url);
            return Ok(());
        }
        tracing::debug!("updating upstream remote URL to {}", upstream_url);
        let out = cmd::run(
            Command::new("git")
                .args(["-C", &repo_dir.to_string_lossy()])
                .args(["remote", "set-url", "upstream", upstream_url]),
        )
        .context("Failed to update upstream remote URL")?;
        if !out.status.success() {
            let stderr = String::from_utf8_lossy(&out.stderr);
            return Err(anyhow!("git remote set-url upstream failed: {}", stderr));
        }
    } else {
        tracing::debug!("adding upstream remote: {}", upstream_url);
        let out = cmd::run(
            Command::new("git")
                .args(["-C", &repo_dir.to_string_lossy()])
                .args(["remote", "add", "upstream", upstream_url]),
        )
        .context("Failed to add upstream remote")?;
        if !out.status.success() {
            let stderr = String::from_utf8_lossy(&out.stderr);
            return Err(anyhow!("git remote add upstream failed: {}", stderr));
        }
    }
    Ok(())
}

/// Generate the upstream clone URL using the same protocol as the origin.
fn generate_upstream_url(
    origin_url: &str,
    up_owner: &str,
    up_repo: &str,
    protocol: Option<Protocol>,
) -> String {
    // Prefer explicit protocol, otherwise infer from origin URL format
    let use_ssh = match protocol {
        Some(Protocol::Ssh) => true,
        Some(Protocol::Https) => false,
        None => origin_url.starts_with("git@"),
    };
    if use_ssh {
        format!("git@github.com:{}/{}.git", up_owner, up_repo)
    } else {
        format!("https://github.com/{}/{}.git", up_owner, up_repo)
    }
}

/// Clone a repository as a bare clone
fn clone_bare_repo(url: &str, dest: &Path, handle: &OperationHandle) -> Result<()> {
    let output = cmd::run_with_reporter(
        Command::new("git")
            .arg("clone")
            .arg("--bare")
            .arg(url)
            .arg(dest),
        handle,
    )
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

/// Information about a newly created repository
#[derive(Debug, Clone)]
pub struct CreatedRepoInfo {
    /// The directory name in .werx/repos/
    pub dir_name: String,
    /// The owner component
    pub owner: String,
    /// The repository name component
    pub name: String,
    /// The path to the bare repository
    pub bare_repo_path: std::path::PathBuf,
}

/// Parse and validate a repository specification in owner/repo format
pub fn parse_repo_spec(spec: &str) -> Result<(String, String)> {
    let parts: Vec<&str> = spec.split('/').collect();

    if parts.len() != 2 {
        return Err(anyhow!(
            "Invalid repository specification: '{}'\n\n\
             Expected format: owner/repo\n\
             Examples: mycompany/awesome-project, alice/utils",
            spec
        ));
    }

    let owner = parts[0].trim();
    let name = parts[1].trim();

    // Validate owner format (alphanumeric and hyphens)
    if owner.is_empty()
        || !owner
            .chars()
            .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
    {
        return Err(anyhow!(
            "Invalid owner format: '{}'\n\n\
             Owner must contain only alphanumeric characters, hyphens, and underscores.",
            owner
        ));
    }

    // Validate repo name format
    if name.is_empty()
        || !name
            .chars()
            .all(|c| c.is_alphanumeric() || c == '-' || c == '_' || c == '.')
    {
        return Err(anyhow!(
            "Invalid repository name format: '{}'\n\n\
             Repository name must contain only alphanumeric characters, hyphens, underscores, and dots.",
            name
        ));
    }

    Ok((owner.to_string(), name.to_string()))
}

/// Create a new repository from scratch
pub fn create_repo(werx: &Werx, repo_spec: &str) -> Result<CreatedRepoInfo> {
    // Parse and validate the owner/repo format
    let (owner, name) = parse_repo_spec(repo_spec)?;

    // Load config for protocol (needed for generating clone URL)
    let config = werx.load_config()?;

    // Get existing repositories for conflict detection
    let existing_repos = list_repos(werx)?;

    // Generate the expected clone URL for duplicate detection
    let expected_url =
        generate_origin_url(&owner, &name, config.protocol(), config.default_provider());
    let expected_normalized = crate::repo_spec::normalize_url(&expected_url)
        .unwrap_or_else(|_| expected_url.to_lowercase());

    // Check for duplicate by checking if any existing repo has a matching normalized URL
    for repo in &existing_repos {
        if repo.normalized_url == expected_normalized {
            return Err(anyhow!(
                "Repository already exists: {}/{}\n  Location: .werx/repos/{}",
                owner,
                name,
                repo.dir_name
            ));
        }
    }

    // Compute directory name using progressive qualification
    let dir_name = compute_create_dir_name(&name, &owner, &existing_repos);

    // Create the bare repository
    let repo_dir = werx.repos_dir().join(&dir_name);

    tracing::info!("Creating repository: {}/{}", owner, name);

    // Initialize bare repository
    init_bare_repo(&repo_dir)?;

    // Configure remote origin URL for future publishing
    let clone_url =
        generate_origin_url(&owner, &name, config.protocol(), config.default_provider());
    if let Err(e) = configure_origin(&repo_dir, &clone_url) {
        // Clean up on failure
        let _ = fs::remove_dir_all(&repo_dir);
        return Err(e);
    }

    // Initialize main branch with empty commit
    if let Err(e) = init_main_branch(&repo_dir) {
        // Clean up on failure
        let _ = fs::remove_dir_all(&repo_dir);
        return Err(e);
    }

    Ok(CreatedRepoInfo {
        dir_name,
        owner,
        name,
        bare_repo_path: repo_dir,
    })
}

/// Compute directory name for a new repository using progressive qualification
fn compute_create_dir_name(name: &str, owner: &str, existing_repos: &[RepoInfo]) -> String {
    // Try simple name first
    let simple_name = name.to_string();
    if !existing_repos.iter().any(|r| r.dir_name == simple_name) {
        return simple_name;
    }

    // Try owner-qualified name
    let qualified_name = format!("{}-{}", owner.to_lowercase(), name);
    if !existing_repos.iter().any(|r| r.dir_name == qualified_name) {
        return qualified_name;
    }

    // Use hash-qualified name
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(format!("{}/{}", owner, name).as_bytes());
    let result = hasher.finalize();
    let hash = format!("{:x}", result);
    let hash_suffix = &hash[..6];

    format!("{}-{}-{}", owner.to_lowercase(), name, hash_suffix)
}

/// Initialize a new bare git repository
fn init_bare_repo(dest: &Path) -> Result<()> {
    // Create the directory
    fs::create_dir_all(dest).context(format!(
        "Failed to create repository directory '{}'",
        dest.display()
    ))?;

    let output = cmd::run(Command::new("git")
        .arg("init")
        .arg("--bare")
        .arg(dest))
        .context("Failed to execute git init --bare command")?;

    if !output.status.success() {
        // Clean up partial init if it exists
        if dest.exists() {
            let _ = fs::remove_dir_all(dest);
        }

        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("Git init failed:\n{}", stderr));
    }

    Ok(())
}

/// Generate origin URL for future publishing
fn generate_origin_url(
    owner: &str,
    name: &str,
    protocol: Option<crate::Protocol>,
    default_provider: &str,
) -> String {
    let provider = default_provider.to_lowercase();

    match (provider.as_str(), protocol) {
        ("github", Some(crate::Protocol::Ssh)) => format!("git@github.com:{}/{}.git", owner, name),
        ("github", _) => format!("https://github.com/{}/{}.git", owner, name),
        ("gitlab", Some(crate::Protocol::Ssh)) => format!("git@gitlab.com:{}/{}.git", owner, name),
        ("gitlab", _) => format!("https://gitlab.com/{}/{}.git", owner, name),
        // Default to GitHub HTTPS
        _ => format!("https://github.com/{}/{}.git", owner, name),
    }
}

/// Configure remote origin URL
fn configure_origin(repo_path: &Path, url: &str) -> Result<()> {
    let output = cmd::run(Command::new("git")
        .arg("-C")
        .arg(repo_path)
        .arg("remote")
        .arg("add")
        .arg("origin")
        .arg(url))
        .context("Failed to execute git remote add command")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("Failed to configure remote origin:\n{}", stderr));
    }

    Ok(())
}

/// Initialize main branch with an empty commit
fn init_main_branch(repo_path: &Path) -> Result<()> {
    // Use git hash-object and update-ref to create an empty commit
    // First, create an empty tree
    let output = cmd::run(Command::new("git")
        .arg("-C")
        .arg(repo_path)
        .arg("hash-object")
        .arg("-t")
        .arg("tree")
        .arg("/dev/null"))
        .context("Failed to create empty tree")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("Failed to create empty tree:\n{}", stderr));
    }

    let tree_hash = String::from_utf8_lossy(&output.stdout).trim().to_string();

    // Create a commit with the empty tree
    let output = cmd::run(Command::new("git")
        .arg("-C")
        .arg(repo_path)
        .arg("commit-tree")
        .arg(&tree_hash)
        .arg("-m")
        .arg("Initial commit"))
        .context("Failed to create initial commit")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("Failed to create initial commit:\n{}", stderr));
    }

    let commit_hash = String::from_utf8_lossy(&output.stdout).trim().to_string();

    // Update refs/heads/main to point to the new commit
    let output = cmd::run(Command::new("git")
        .arg("-C")
        .arg(repo_path)
        .arg("update-ref")
        .arg("refs/heads/main")
        .arg(&commit_hash))
        .context("Failed to update main ref")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("Failed to update main branch:\n{}", stderr));
    }

    // Set HEAD to point to main
    let output = cmd::run(Command::new("git")
        .arg("-C")
        .arg(repo_path)
        .arg("symbolic-ref")
        .arg("HEAD")
        .arg("refs/heads/main"))
        .context("Failed to set HEAD")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("Failed to set HEAD:\n{}", stderr));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    // ===== parse_repo_spec tests =====

    #[test]
    fn test_parse_repo_spec_valid_simple() {
        let result = parse_repo_spec("owner/repo");
        assert!(result.is_ok());
        let (owner, name) = result.unwrap();
        assert_eq!(owner, "owner");
        assert_eq!(name, "repo");
    }

    #[test]
    fn test_parse_repo_spec_valid_with_hyphens() {
        let result = parse_repo_spec("my-company/awesome-project");
        assert!(result.is_ok());
        let (owner, name) = result.unwrap();
        assert_eq!(owner, "my-company");
        assert_eq!(name, "awesome-project");
    }

    #[test]
    fn test_parse_repo_spec_valid_with_underscores() {
        let result = parse_repo_spec("my_company/awesome_project");
        assert!(result.is_ok());
        let (owner, name) = result.unwrap();
        assert_eq!(owner, "my_company");
        assert_eq!(name, "awesome_project");
    }

    #[test]
    fn test_parse_repo_spec_valid_with_dots_in_repo() {
        let result = parse_repo_spec("owner/repo.name");
        assert!(result.is_ok());
        let (owner, name) = result.unwrap();
        assert_eq!(owner, "owner");
        assert_eq!(name, "repo.name");
    }

    #[test]
    fn test_parse_repo_spec_valid_with_numbers() {
        let result = parse_repo_spec("user123/project456");
        assert!(result.is_ok());
        let (owner, name) = result.unwrap();
        assert_eq!(owner, "user123");
        assert_eq!(name, "project456");
    }

    #[test]
    fn test_parse_repo_spec_invalid_no_slash() {
        let result = parse_repo_spec("ownerrepo");
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("Invalid repository specification"));
        assert!(err.contains("Expected format: owner/repo"));
    }

    #[test]
    fn test_parse_repo_spec_invalid_too_many_slashes() {
        let result = parse_repo_spec("owner/repo/extra");
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("Invalid repository specification"));
    }

    #[test]
    fn test_parse_repo_spec_invalid_empty_owner() {
        let result = parse_repo_spec("/repo");
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("Invalid owner format"));
    }

    #[test]
    fn test_parse_repo_spec_invalid_empty_name() {
        let result = parse_repo_spec("owner/");
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("Invalid repository name format"));
    }

    #[test]
    fn test_parse_repo_spec_invalid_owner_special_chars() {
        let result = parse_repo_spec("owner@bad/repo");
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("Invalid owner format"));
    }

    #[test]
    fn test_parse_repo_spec_invalid_name_special_chars() {
        let result = parse_repo_spec("owner/repo@bad");
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("Invalid repository name format"));
    }

    #[test]
    fn test_parse_repo_spec_trims_whitespace() {
        let result = parse_repo_spec(" owner / repo ");
        assert!(result.is_ok());
        let (owner, name) = result.unwrap();
        assert_eq!(owner, "owner");
        assert_eq!(name, "repo");
    }

    // ===== compute_create_dir_name tests =====

    #[test]
    fn test_compute_create_dir_name_simple_no_conflict() {
        let existing: Vec<RepoInfo> = vec![];
        let result = compute_create_dir_name("myrepo", "owner", &existing);
        assert_eq!(result, "myrepo");
    }

    #[test]
    fn test_compute_create_dir_name_owner_qualified_on_conflict() {
        let existing = vec![RepoInfo {
            dir_name: "myrepo".to_string(),
            clone_url: "https://github.com/other/myrepo.git".to_string(),
            normalized_url: "github.com/other/myrepo".to_string(),
            default_branch: Some("main".to_string()),
            valid: true,
            error: None,
        }];
        let result = compute_create_dir_name("myrepo", "newowner", &existing);
        assert_eq!(result, "newowner-myrepo");
    }

    #[test]
    fn test_compute_create_dir_name_hash_qualified_on_double_conflict() {
        let existing = vec![
            RepoInfo {
                dir_name: "myrepo".to_string(),
                clone_url: "https://github.com/first/myrepo.git".to_string(),
                normalized_url: "github.com/first/myrepo".to_string(),
                default_branch: Some("main".to_string()),
                valid: true,
                error: None,
            },
            RepoInfo {
                dir_name: "thirdowner-myrepo".to_string(),
                clone_url: "https://github.com/thirdowner/myrepo.git".to_string(),
                normalized_url: "github.com/thirdowner/myrepo".to_string(),
                default_branch: Some("main".to_string()),
                valid: true,
                error: None,
            },
        ];
        let result = compute_create_dir_name("myrepo", "thirdowner", &existing);
        // Should get hash-qualified name since both simple and owner-qualified are taken
        assert!(result.starts_with("thirdowner-myrepo-"));
        assert!(result.len() > "thirdowner-myrepo-".len());
        // Verify it's a 6-char hex suffix
        let suffix = &result["thirdowner-myrepo-".len()..];
        assert_eq!(suffix.len(), 6);
        assert!(suffix.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_compute_create_dir_name_lowercase_owner() {
        let existing = vec![RepoInfo {
            dir_name: "myrepo".to_string(),
            clone_url: "https://github.com/other/myrepo.git".to_string(),
            normalized_url: "github.com/other/myrepo".to_string(),
            default_branch: Some("main".to_string()),
            valid: true,
            error: None,
        }];
        let result = compute_create_dir_name("myrepo", "MyOwner", &existing);
        assert_eq!(result, "myowner-myrepo");
    }

    #[test]
    fn test_compute_create_dir_name_hash_is_deterministic() {
        let existing = vec![
            RepoInfo {
                dir_name: "myrepo".to_string(),
                clone_url: "https://github.com/first/myrepo.git".to_string(),
                normalized_url: "github.com/first/myrepo".to_string(),
                default_branch: Some("main".to_string()),
                valid: true,
                error: None,
            },
            RepoInfo {
                dir_name: "owner-myrepo".to_string(),
                clone_url: "https://github.com/owner/myrepo.git".to_string(),
                normalized_url: "github.com/owner/myrepo".to_string(),
                default_branch: Some("main".to_string()),
                valid: true,
                error: None,
            },
        ];
        let result1 = compute_create_dir_name("myrepo", "owner", &existing);
        let result2 = compute_create_dir_name("myrepo", "owner", &existing);
        assert_eq!(result1, result2);
    }

    // ===== generate_origin_url tests =====

    #[test]
    fn test_generate_origin_url_github_ssh() {
        let url = generate_origin_url("myowner", "myrepo", Some(crate::Protocol::Ssh), "github");
        assert_eq!(url, "git@github.com:myowner/myrepo.git");
    }

    #[test]
    fn test_generate_origin_url_github_https() {
        let url = generate_origin_url("myowner", "myrepo", Some(crate::Protocol::Https), "github");
        assert_eq!(url, "https://github.com/myowner/myrepo.git");
    }

    #[test]
    fn test_generate_origin_url_github_no_protocol() {
        let url = generate_origin_url("myowner", "myrepo", None, "github");
        assert_eq!(url, "https://github.com/myowner/myrepo.git");
    }

    #[test]
    fn test_generate_origin_url_gitlab_ssh() {
        let url = generate_origin_url("myowner", "myrepo", Some(crate::Protocol::Ssh), "gitlab");
        assert_eq!(url, "git@gitlab.com:myowner/myrepo.git");
    }

    #[test]
    fn test_generate_origin_url_gitlab_https() {
        let url = generate_origin_url("myowner", "myrepo", Some(crate::Protocol::Https), "gitlab");
        assert_eq!(url, "https://gitlab.com/myowner/myrepo.git");
    }

    #[test]
    fn test_generate_origin_url_unknown_provider_defaults_to_github() {
        let url = generate_origin_url("myowner", "myrepo", None, "unknown");
        assert_eq!(url, "https://github.com/myowner/myrepo.git");
    }
}
