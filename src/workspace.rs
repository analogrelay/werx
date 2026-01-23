use anyhow::{Context, Result, anyhow};
use dialoguer::{Confirm, Input, Select, theme::ColorfulTheme};
use fuzzy_matcher::FuzzyMatcher;
use fuzzy_matcher::skim::SkimMatcherV2;
use serde::Serialize;
use skim::prelude::*;
use std::fs;
use std::io::{Cursor, IsTerminal};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Arc;

use crate::{Forge, RepoInfo, RepoSpec, repos};

/// Workspace information
#[derive(Debug, Clone, Serialize)]
pub struct Workspace {
    /// The workspace name (e.g., "feature-branch")
    pub name: String,
    /// The full path to the workspace directory
    pub path: PathBuf,
    /// The repository name (directory name in .forge/repos/)
    pub repository: String,
    /// The branch this workspace is tracking
    pub branch: Option<String>,
    /// The status of the workspace
    pub status: WorkspaceStatus,
}

/// Status of a workspace
#[derive(Debug, Clone, Serialize, PartialEq)]
pub enum WorkspaceStatus {
    /// Workspace is clean with no uncommitted changes
    Clean,
    /// Workspace has modified files
    Modified,
    /// Workspace has untracked files
    Untracked,
    /// Workspace is locked (git operation in progress)
    Locked,
    /// Workspace is prunable (directory doesn't exist but git knows about it)
    Prunable,
}

impl Forge {
    /// Get the workspaces directory (same as the root directory)
    pub fn workspaces_dir(&self) -> PathBuf {
        self.root.clone()
    }
}

/// List all workspaces in the Forge
pub fn list_workspaces(forge: &Forge) -> Result<Vec<Workspace>> {
    let mut workspaces = Vec::new();

    // Get all repositories
    let repos = repos::list_repos(forge)?;

    // For each repository, get its worktrees
    for repo in repos {
        if !repo.valid {
            continue;
        }

        let repo_path = forge.repos_dir().join(&repo.dir_name);
        match list_repo_worktrees(forge, &repo_path, &repo.dir_name) {
            Ok(mut repo_workspaces) => {
                workspaces.append(&mut repo_workspaces);
            }
            Err(e) => {
                eprintln!(
                    "Warning: Failed to list worktrees for {}: {}",
                    repo.dir_name, e
                );
                continue;
            }
        }
    }

    // Sort by repository, then by name
    workspaces.sort_by(|a, b| a.repository.cmp(&b.repository).then(a.name.cmp(&b.name)));

    Ok(workspaces)
}

/// List worktrees for a specific repository
fn list_repo_worktrees(
    forge: &Forge,
    repo_path: &PathBuf,
    repo_name: &str,
) -> Result<Vec<Workspace>> {
    let output = Command::new("git")
        .arg("-C")
        .arg(repo_path)
        .arg("worktree")
        .arg("list")
        .arg("--porcelain")
        .output()
        .context("Failed to execute git worktree list")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!("Git worktree list failed: {}", stderr));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    parse_worktree_output(forge, &stdout, repo_name)
}

/// Parse the output of `git worktree list --porcelain`
fn parse_worktree_output(forge: &Forge, output: &str, repo_name: &str) -> Result<Vec<Workspace>> {
    let mut workspaces = Vec::new();
    let forge_root = &forge.root;

    let mut current_worktree: Option<(PathBuf, Option<String>, WorkspaceStatus)> = None;

    for line in output.lines() {
        if line.starts_with("worktree ") {
            // Save previous worktree if it exists and is under forge root
            if let Some((path, branch, status)) = current_worktree.take() {
                if path.starts_with(forge_root) {
                    // Extract workspace name from path
                    if let Some(name) = extract_workspace_name(&path, forge_root, repo_name) {
                        workspaces.push(Workspace {
                            name,
                            path,
                            repository: repo_name.to_string(),
                            branch,
                            status,
                        });
                    }
                }
            }

            // Parse new worktree path
            let path_str = line.strip_prefix("worktree ").unwrap_or("");
            let path = PathBuf::from(path_str);
            current_worktree = Some((path, None, WorkspaceStatus::Clean));
        } else if line.starts_with("branch ") {
            if let Some((_, ref mut branch, _)) = current_worktree {
                let branch_ref = line.strip_prefix("branch ").unwrap_or("");
                // Strip "refs/heads/" prefix if present
                let branch_name = branch_ref
                    .strip_prefix("refs/heads/")
                    .unwrap_or(branch_ref)
                    .to_string();
                *branch = Some(branch_name);
            }
        } else if line.starts_with("locked") {
            if let Some((_, _, ref mut status)) = current_worktree {
                *status = WorkspaceStatus::Locked;
            }
        } else if line.starts_with("prunable") {
            if let Some((_, _, ref mut status)) = current_worktree {
                *status = WorkspaceStatus::Prunable;
            }
        }
    }

    // Don't forget the last worktree
    if let Some((path, branch, status)) = current_worktree {
        if path.starts_with(forge_root) {
            if let Some(name) = extract_workspace_name(&path, forge_root, repo_name) {
                workspaces.push(Workspace {
                    name,
                    path,
                    repository: repo_name.to_string(),
                    branch,
                    status,
                });
            }
        }
    }

    Ok(workspaces)
}

/// Extract workspace name from the full path
/// Expected format: <forge_root>/<repo_name>/<workspace_name>
fn extract_workspace_name(path: &PathBuf, forge_root: &PathBuf, repo_name: &str) -> Option<String> {
    // Strip forge root
    let relative = path.strip_prefix(forge_root).ok()?;

    // Strip repository name
    let relative = relative.strip_prefix(repo_name).ok()?;

    // The remaining path should be the workspace name (could be multi-level)
    let workspace_name = relative
        .to_string_lossy()
        .trim_start_matches('/')
        .to_string();

    if workspace_name.is_empty() {
        None
    } else {
        Some(workspace_name)
    }
}

/// Find a repository by specification string
pub fn find_repository(forge: &Forge, repo_spec: &str) -> Result<RepoInfo> {
    let config = forge.load_config()?;

    // Parse the repository specification to get the directory name
    let spec = RepoSpec::parse(repo_spec, config.default_provider(), config.protocol())
        .context("Failed to parse repository specification")?;

    // Get repository info to determine correct directory name
    let repos = repos::list_repos(forge)?;
    let dir_name = spec.dir_name(&repos);

    // Check if repository exists
    let repo_dir = forge.repos_dir().join(&dir_name);
    if !repo_dir.exists() {
        return Err(anyhow!(
            "Repository not found: {}\n\nRun 'forge repos list' to see available repositories.",
            spec.original
        ));
    }

    repos
        .into_iter()
        .find(|r| r.dir_name == dir_name)
        .ok_or_else(|| anyhow!("Repository not found: {}", spec.original))
}

/// Detect if the current directory is within a workspace and return its repository info
pub fn detect_current_workspace(current_dir: &Path, forge: &Forge) -> Result<Option<RepoInfo>> {
    // Check if current directory is under the forge root
    if !current_dir.starts_with(&forge.root) {
        return Ok(None);
    }

    // Get all workspaces
    let workspaces = list_workspaces(forge)?;

    // Find if current directory is within any workspace
    for workspace in workspaces {
        if current_dir.starts_with(&workspace.path) {
            // Found a workspace that contains the current directory
            // Return the repository info for this workspace
            let repos = repos::list_repos(forge)?;
            if let Some(repo) = repos
                .into_iter()
                .find(|r| r.dir_name == workspace.repository)
            {
                return Ok(Some(repo));
            }
        }
    }

    Ok(None)
}

/// Interactively select a repository from the available repositories
pub fn select_repository(forge: &Forge) -> Result<RepoInfo> {
    // Check if we're in an interactive terminal
    if !std::io::stdin().is_terminal() {
        return Err(anyhow!(
            "Cannot prompt for repository selection in non-interactive mode.\n\
             Please specify a repository explicitly or run from within a workspace."
        ));
    }

    // Get all repositories
    let repos = repos::list_repos(forge)?;

    if repos.is_empty() {
        return Err(anyhow!(
            "No repositories found in Forge.\n\n\
             Run 'forge add <repo>' to add a repository first."
        ));
    }

    // Filter out invalid repositories
    let valid_repos: Vec<&RepoInfo> = repos.iter().filter(|r| r.valid).collect();

    if valid_repos.is_empty() {
        return Err(anyhow!(
            "No valid repositories found in Forge.\n\n\
             Run 'forge repos list' to see repository status."
        ));
    }

    println!();
    println!("Select a repository:");
    println!();

    // Create display items for the selector
    let items: Vec<String> = valid_repos
        .iter()
        .map(|r| {
            if let Some(branch) = &r.default_branch {
                format!("{} ({})", r.clone_url, branch)
            } else {
                r.clone_url.clone()
            }
        })
        .collect();

    let selection = Select::with_theme(&ColorfulTheme::default())
        .items(&items)
        .default(0)
        .interact()?;

    Ok(valid_repos[selection].clone())
}

/// Generate workspace path in hierarchical format: <repo_name>/<workspace_name>
pub fn generate_workspace_path(forge: &Forge, repo_name: &str, workspace_name: &str) -> PathBuf {
    forge.root.join(repo_name).join(workspace_name)
}

/// Prompt the user for a workspace name, with a suggested default
pub fn prompt_workspace_name(default: &str) -> Result<String> {
    // Check if we're in an interactive terminal
    if !std::io::stdin().is_terminal() {
        // Non-interactive mode: use the default
        return Ok(default.to_string());
    }

    println!();
    let name: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Workspace name")
        .default(default.to_string())
        .interact_text()?;

    if name.trim().is_empty() {
        return Err(anyhow!("Workspace name cannot be empty"));
    }

    Ok(name)
}

/// Create a git worktree for a repository with hierarchical structure
pub fn create_worktree(
    forge: &Forge,
    repo_info: &RepoInfo,
    workspace_name: &str,
    branch: &str,
) -> Result<PathBuf> {
    // Generate the hierarchical workspace path: <forge-root>/<repo-name>/<workspace-name>
    let workspace_path = generate_workspace_path(forge, &repo_info.dir_name, workspace_name);

    // Check if workspace already exists
    if workspace_path.exists() {
        return Err(anyhow!(
            "Workspace already exists at: {}\n\n\
             Choose a different workspace name or remove the existing workspace first.",
            workspace_path.display()
        ));
    }

    // Create the repository directory if it doesn't exist
    let repo_dir_in_forge = forge.root.join(&repo_info.dir_name);
    if !repo_dir_in_forge.exists() {
        fs::create_dir_all(&repo_dir_in_forge).context(format!(
            "Failed to create directory '{}'",
            repo_dir_in_forge.display()
        ))?;
    }

    // Get the bare repository path
    let bare_repo_path = forge.repos_dir().join(&repo_info.dir_name);

    // Execute git worktree add
    let output = Command::new("git")
        .arg("-C")
        .arg(&bare_repo_path)
        .arg("worktree")
        .arg("add")
        .arg(&workspace_path)
        .arg(branch)
        .output()
        .context("Failed to execute git worktree add")?;

    if !output.status.success() {
        // Clean up any partially created directories
        if workspace_path.exists() {
            let _ = fs::remove_dir_all(&workspace_path);
        }

        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("Git worktree add failed:\n{}", stderr));
    }

    Ok(workspace_path)
}

/// Check the status of a workspace for uncommitted changes
pub fn check_workspace_status(path: &Path) -> Result<WorkspaceStatus> {
    if !path.exists() {
        return Ok(WorkspaceStatus::Prunable);
    }

    let output = Command::new("git")
        .arg("-C")
        .arg(path)
        .arg("status")
        .arg("--porcelain")
        .output()
        .context("Failed to execute git status")?;

    if !output.status.success() {
        return Err(anyhow!("Failed to get workspace status"));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);

    if stdout.is_empty() {
        Ok(WorkspaceStatus::Clean)
    } else {
        // Check for modified vs untracked files
        let has_modified = stdout.lines().any(|line| {
            let trimmed = line.trim();
            !trimmed.is_empty() && !trimmed.starts_with("??")
        });

        if has_modified {
            Ok(WorkspaceStatus::Modified)
        } else {
            Ok(WorkspaceStatus::Untracked)
        }
    }
}

/// Confirm workspace removal with the user
pub fn confirm_workspace_removal(
    name: &str,
    path: &Path,
    status: &WorkspaceStatus,
    force: bool,
) -> Result<bool> {
    // Skip confirmation if force flag is set
    if force {
        return Ok(true);
    }

    // Check if we're in an interactive terminal
    if !std::io::stdin().is_terminal() {
        return Err(anyhow!(
            "Cannot prompt for confirmation in non-interactive mode.\n\
             Use --force to skip confirmation."
        ));
    }

    println!();
    println!("About to remove workspace:");
    println!("  Name:   {}", name);
    println!("  Path:   {}", path.display());

    match status {
        WorkspaceStatus::Clean => {}
        WorkspaceStatus::Modified => {
            println!();
            println!("⚠️  WARNING: Workspace has uncommitted changes!");
        }
        WorkspaceStatus::Untracked => {
            println!();
            println!("⚠️  WARNING: Workspace has untracked files!");
        }
        WorkspaceStatus::Locked => {
            println!();
            println!("⚠️  WARNING: Workspace is locked (git operation in progress)!");
        }
        WorkspaceStatus::Prunable => {
            println!();
            println!("Note: Workspace directory is missing (will prune git metadata)");
        }
    }

    println!();

    let confirmed = Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt("Are you sure you want to remove this workspace?")
        .default(false)
        .interact()?;

    Ok(confirmed)
}

/// Remove a workspace and clean up hierarchical directories
pub fn remove_workspace(forge: &Forge, workspace_path: &str) -> Result<()> {
    // Get all workspaces
    let workspaces = list_workspaces(forge)?;

    // Find the workspace by path (support both "repo/workspace" and just "workspace")
    let matching_workspaces: Vec<&Workspace> = workspaces
        .iter()
        .filter(|w| {
            // Match full path "repo/workspace"
            let full_name = format!("{}/{}", w.repository, w.name);
            full_name == workspace_path || w.name == workspace_path
        })
        .collect();

    if matching_workspaces.is_empty() {
        return Err(anyhow!(
            "Workspace not found: {}\n\n\
             Run 'forge workspace list' to see available workspaces.",
            workspace_path
        ));
    }

    if matching_workspaces.len() > 1 {
        // Ambiguous workspace name
        println!();
        println!(
            "Multiple workspaces match '{}'. Please specify the full path:",
            workspace_path
        );
        println!();
        for ws in &matching_workspaces {
            println!("  {}/{}", ws.repository, ws.name);
        }
        println!();
        return Err(anyhow!("Ambiguous workspace name"));
    }

    let workspace = matching_workspaces[0];

    // Get the bare repository path
    let bare_repo_path = forge.repos_dir().join(&workspace.repository);

    // Try to remove the worktree using git
    let output = Command::new("git")
        .arg("-C")
        .arg(&bare_repo_path)
        .arg("worktree")
        .arg("remove")
        .arg(&workspace.path)
        .output()
        .context("Failed to execute git worktree remove")?;

    if !output.status.success() {
        // If git worktree remove fails, try with --force (for orphaned directories)
        let output = Command::new("git")
            .arg("-C")
            .arg(&bare_repo_path)
            .arg("worktree")
            .arg("remove")
            .arg("--force")
            .arg(&workspace.path)
            .output()
            .context("Failed to execute git worktree remove --force")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!("Git worktree remove failed:\n{}", stderr));
        }
    }

    // Remove the workspace directory if it still exists
    if workspace.path.exists() {
        fs::remove_dir_all(&workspace.path).context(format!(
            "Failed to remove directory '{}'",
            workspace.path.display()
        ))?;
    }

    // Clean up empty repository directory
    let repo_dir_in_forge = forge.root.join(&workspace.repository);
    if repo_dir_in_forge.exists() && repo_dir_in_forge.is_dir() {
        // Check if directory is empty
        if let Ok(mut entries) = fs::read_dir(&repo_dir_in_forge) {
            if entries.next().is_none() {
                // Directory is empty, remove it
                let _ = fs::remove_dir(&repo_dir_in_forge);
            }
        }
    }

    // Prune worktree metadata to clean up any orphaned entries
    let _ = Command::new("git")
        .arg("-C")
        .arg(&bare_repo_path)
        .arg("worktree")
        .arg("prune")
        .output();

    Ok(())
}

/// Wrapper for displaying workspace items in fuzzy finder
struct WorkspaceItem {
    #[allow(dead_code)]
    workspace: Workspace,
    display: String,
    preview: String,
}

impl SkimItem for WorkspaceItem {
    fn text(&self) -> Cow<'_, str> {
        Cow::Borrowed(&self.display)
    }

    fn preview(&self, _context: PreviewContext) -> ItemPreview {
        ItemPreview::Text(self.preview.clone())
    }
}

/// Select a workspace using fuzzy search
pub fn fuzzy_select_workspace(
    workspaces: Vec<Workspace>,
    initial_query: Option<String>,
) -> Result<Option<Workspace>> {
    if workspaces.is_empty() {
        return Ok(None);
    }

    // Check if we're in an interactive terminal
    if !std::io::stdin().is_terminal() {
        return Err(anyhow!(
            "Interactive terminal required for fuzzy search.\n\
             Use a direct query match or run in an interactive terminal."
        ));
    }

    // Build skim items
    let items: Vec<Arc<dyn SkimItem>> = workspaces
        .iter()
        .map(|ws| {
            let display = format!("{}/{}", ws.repository, ws.name);
            let branch_str = ws
                .branch
                .as_deref()
                .unwrap_or("(detached)");
            let preview = format!("branch: {} | {}", branch_str, ws.path.display());

            Arc::new(WorkspaceItem {
                workspace: ws.clone(),
                display,
                preview,
            }) as Arc<dyn SkimItem>
        })
        .collect();

    // Create a string containing all items for skim to read
    let item_reader = SkimItemReader::default();
    let items_str = items
        .iter()
        .map(|item| item.text().to_string())
        .collect::<Vec<_>>()
        .join("\n");
    let item_stream = Cursor::new(items_str);
    let rx_item = item_reader.of_bufread(item_stream);

    // Configure skim options
    let options = SkimOptionsBuilder::default()
        .height(Some("50%"))
        .multi(false)
        .query(initial_query.as_deref())
        .prompt(Some("Select workspace: "))
        .build()
        .map_err(|e| anyhow!("Failed to build fuzzy finder options: {}", e))?;

    // Run skim
    let output = Skim::run_with(&options, Some(rx_item))
        .ok_or_else(|| anyhow!("Fuzzy finder failed to run"))?;

    // Handle user cancellation
    if output.is_abort {
        return Ok(None);
    }

    // Get the selected item
    if let Some(selected) = output.selected_items.first() {
        let selected_text = selected.text();

        // Find the matching workspace
        for ws in workspaces {
            let display = format!("{}/{}", ws.repository, ws.name);
            if display == selected_text.as_ref() {
                return Ok(Some(ws));
            }
        }
    }

    Ok(None)
}

/// Find workspaces that match a given query using fuzzy matching
pub fn find_workspace_matches<'a>(
    workspaces: &'a [Workspace],
    query: &str,
) -> Vec<&'a Workspace> {
    if query.is_empty() {
        return workspaces.iter().collect();
    }

    let matcher = SkimMatcherV2::default();
    let query_lower = query.to_lowercase();

    workspaces
        .iter()
        .filter(|ws| {
            let display = format!("{}/{}", ws.repository, ws.name).to_lowercase();
            // Use fuzzy matching, but also support substring matching
            matcher.fuzzy_match(&display, &query_lower).is_some()
                || display.contains(&query_lower)
        })
        .collect()
}

/// Select a workspace with an optional query, using direct navigation if exactly one match
pub fn select_workspace_with_query(
    workspaces: Vec<Workspace>,
    query: Option<String>,
) -> Result<Option<Workspace>> {
    if workspaces.is_empty() {
        return Ok(None);
    }

    // If a query is provided, try to find matches
    if let Some(ref q) = query {
        if !q.is_empty() {
            let matches = find_workspace_matches(&workspaces, q);

            // If exactly one match, return it directly (no interactive selection)
            if matches.len() == 1 {
                return Ok(Some(matches[0].clone()));
            }

            // If no matches, return an error
            if matches.is_empty() {
                return Err(anyhow!(
                    "No workspaces match query: '{}'",
                    q
                ));
            }

            // Multiple matches - fall through to fuzzy search with pre-filled query
        }
    }

    // Launch interactive fuzzy search (with query pre-filled if provided)
    fuzzy_select_workspace(workspaces, query)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_workspace_struct() {
        let workspace = Workspace {
            name: "feature-branch".to_string(),
            path: PathBuf::from("/tmp/forge/myrepo/feature-branch"),
            repository: "myrepo".to_string(),
            branch: Some("feature-branch".to_string()),
            status: WorkspaceStatus::Clean,
        };

        assert_eq!(workspace.name, "feature-branch");
        assert_eq!(workspace.repository, "myrepo");
        assert_eq!(workspace.status, WorkspaceStatus::Clean);
    }

    #[test]
    fn test_workspaces_dir() {
        let forge = Forge {
            root: PathBuf::from("/tmp/forge"),
        };

        assert_eq!(forge.workspaces_dir(), PathBuf::from("/tmp/forge"));
    }

    #[test]
    fn test_parse_worktree_output() {
        let forge = Forge {
            root: PathBuf::from("/home/user/forge"),
        };

        let output = r#"worktree /home/user/forge/myrepo/main
HEAD abc123def456
branch refs/heads/main

worktree /home/user/forge/myrepo/feature-branch
HEAD def789ghi012
branch refs/heads/feature-branch

"#;

        let workspaces = parse_worktree_output(&forge, output, "myrepo").unwrap();

        assert_eq!(workspaces.len(), 2);

        assert_eq!(workspaces[0].name, "main");
        assert_eq!(workspaces[0].repository, "myrepo");
        assert_eq!(workspaces[0].branch, Some("main".to_string()));
        assert_eq!(workspaces[0].status, WorkspaceStatus::Clean);

        assert_eq!(workspaces[1].name, "feature-branch");
        assert_eq!(workspaces[1].repository, "myrepo");
        assert_eq!(workspaces[1].branch, Some("feature-branch".to_string()));
        assert_eq!(workspaces[1].status, WorkspaceStatus::Clean);
    }

    #[test]
    fn test_parse_worktree_output_filters_non_forge_workspaces() {
        let forge = Forge {
            root: PathBuf::from("/home/user/forge"),
        };

        let output = r#"worktree /home/user/forge/myrepo/main
HEAD abc123def456
branch refs/heads/main

worktree /home/user/other-location/myrepo
HEAD def789ghi012
branch refs/heads/other

"#;

        let workspaces = parse_worktree_output(&forge, output, "myrepo").unwrap();

        // Should only include the workspace under forge root
        assert_eq!(workspaces.len(), 1);
        assert_eq!(workspaces[0].name, "main");
    }

    #[test]
    fn test_extract_workspace_name() {
        let forge_root = PathBuf::from("/home/user/forge");

        // Standard case
        let path = PathBuf::from("/home/user/forge/myrepo/feature-branch");
        let name = extract_workspace_name(&path, &forge_root, "myrepo");
        assert_eq!(name, Some("feature-branch".to_string()));

        // Multi-level workspace name
        let path = PathBuf::from("/home/user/forge/myrepo/nested/workspace");
        let name = extract_workspace_name(&path, &forge_root, "myrepo");
        assert_eq!(name, Some("nested/workspace".to_string()));

        // Outside forge root
        let path = PathBuf::from("/home/user/other/myrepo/feature");
        let name = extract_workspace_name(&path, &forge_root, "myrepo");
        assert_eq!(name, None);

        // Wrong repo name
        let path = PathBuf::from("/home/user/forge/otherrepo/feature");
        let name = extract_workspace_name(&path, &forge_root, "myrepo");
        assert_eq!(name, None);
    }

    #[test]
    fn test_detect_current_workspace_outside_forge() {
        let forge = Forge {
            root: PathBuf::from("/home/user/forge"),
        };
        let current_dir = PathBuf::from("/home/user/other");

        let result = detect_current_workspace(&current_dir, &forge).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_detect_current_workspace_in_forge_root_but_not_in_workspace() {
        let forge = Forge {
            root: PathBuf::from("/home/user/forge"),
        };
        let current_dir = PathBuf::from("/home/user/forge");

        // This will return None because there are no workspaces
        let result = detect_current_workspace(&current_dir, &forge).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_generate_workspace_path() {
        let forge = Forge {
            root: PathBuf::from("/home/user/forge"),
        };

        let path = generate_workspace_path(&forge, "myrepo-abc123", "feature-branch");
        assert_eq!(
            path,
            PathBuf::from("/home/user/forge/myrepo-abc123/feature-branch")
        );

        // Test with multi-level workspace name
        let path = generate_workspace_path(&forge, "myrepo-abc123", "nested/workspace");
        assert_eq!(
            path,
            PathBuf::from("/home/user/forge/myrepo-abc123/nested/workspace")
        );
    }

    #[test]
    fn test_find_workspace_matches_single() {
        let workspaces = vec![
            Workspace {
                name: "main".to_string(),
                path: PathBuf::from("/tmp/forge/myrepo/main"),
                repository: "myrepo".to_string(),
                branch: Some("main".to_string()),
                status: WorkspaceStatus::Clean,
            },
            Workspace {
                name: "feature-x".to_string(),
                path: PathBuf::from("/tmp/forge/myrepo/feature-x"),
                repository: "myrepo".to_string(),
                branch: Some("feature-x".to_string()),
                status: WorkspaceStatus::Clean,
            },
        ];

        let matches = find_workspace_matches(&workspaces, "main");
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].name, "main");
    }

    #[test]
    fn test_find_workspace_matches_multiple() {
        let workspaces = vec![
            Workspace {
                name: "feature-a".to_string(),
                path: PathBuf::from("/tmp/forge/myrepo/feature-a"),
                repository: "myrepo".to_string(),
                branch: Some("feature-a".to_string()),
                status: WorkspaceStatus::Clean,
            },
            Workspace {
                name: "feature-b".to_string(),
                path: PathBuf::from("/tmp/forge/myrepo/feature-b"),
                repository: "myrepo".to_string(),
                branch: Some("feature-b".to_string()),
                status: WorkspaceStatus::Clean,
            },
        ];

        let matches = find_workspace_matches(&workspaces, "feature");
        assert_eq!(matches.len(), 2);
    }

    #[test]
    fn test_find_workspace_matches_none() {
        let workspaces = vec![
            Workspace {
                name: "main".to_string(),
                path: PathBuf::from("/tmp/forge/myrepo/main"),
                repository: "myrepo".to_string(),
                branch: Some("main".to_string()),
                status: WorkspaceStatus::Clean,
            },
        ];

        let matches = find_workspace_matches(&workspaces, "nonexistent");
        assert_eq!(matches.len(), 0);
    }

    #[test]
    fn test_find_workspace_matches_case_insensitive() {
        let workspaces = vec![
            Workspace {
                name: "main".to_string(),
                path: PathBuf::from("/tmp/forge/MyRepo/main"),
                repository: "MyRepo".to_string(),
                branch: Some("main".to_string()),
                status: WorkspaceStatus::Clean,
            },
        ];

        let matches = find_workspace_matches(&workspaces, "myrepo");
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].repository, "MyRepo");
    }
}
