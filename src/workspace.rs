use anyhow::{anyhow, Context, Result};
use dialoguer::{theme::ColorfulTheme, Confirm, Input, Select};
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use serde::Serialize;
use skim::prelude::*;
use std::fs;
use std::io::{Cursor, IsTerminal};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Arc;

use crate::{repos, RepoInfo, RepoSpec, Werx};

/// Workspace information
#[derive(Debug, Clone, Serialize)]
pub struct Workspace {
    /// The workspace name (e.g., "feature-branch")
    pub name: String,
    /// The full path to the workspace directory
    pub path: PathBuf,
    /// The repository name (directory name in .werx/repos/)
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

/// Extended workspace status with remote tracking information
#[derive(Debug, Clone, Serialize)]
pub struct WorkspaceStatusDetails {
    /// Workspace has uncommitted changes (modified, staged, or untracked files)
    pub uncommitted_changes: bool,
    /// Workspace branch is not pushed to any remote
    pub unpushed_branch: bool,
    /// Workspace branch is merged to the default branch and pushed
    pub merged_branch: bool,
    /// The name of the current branch
    pub branch_name: Option<String>,
    /// The default/main branch for this repository
    pub default_branch: Option<String>,
    /// Detailed status information
    pub status_details: Option<StatusDetails>,
}

/// Detailed status information for a workspace
#[derive(Debug, Clone, Serialize)]
pub struct StatusDetails {
    /// List of modified files (staged or unstaged)
    pub modified_files: Vec<String>,
    /// List of untracked files
    pub untracked_files: Vec<String>,
}

/// Information about branch merge status
#[derive(Debug, Clone, Serialize)]
pub struct MergeDetails {
    /// The branch that this branch is merged into
    pub merged_into: String,
    /// The remote tracking branch, if any
    pub remote_tracking: Option<String>,
}

impl Werx {
    /// Get the workspaces directory (same as the root directory)
    pub fn workspaces_dir(&self) -> PathBuf {
        self.root.clone()
    }
}

/// List all workspaces in the Werx
pub fn list_workspaces(werx: &Werx) -> Result<Vec<Workspace>> {
    let mut workspaces = Vec::new();

    // Get all repositories
    let repos = repos::list_repos(werx)?;

    // For each repository, get its worktrees
    for repo in repos {
        if !repo.valid {
            continue;
        }

        let repo_path = werx.repos_dir().join(&repo.dir_name);
        match list_repo_worktrees(werx, &repo_path, &repo.dir_name) {
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
    werx: &Werx,
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
    parse_worktree_output(werx, &stdout, repo_name)
}

/// Parse the output of `git worktree list --porcelain`
fn parse_worktree_output(werx: &Werx, output: &str, repo_name: &str) -> Result<Vec<Workspace>> {
    let mut workspaces = Vec::new();
    let werx_root = &werx.root;

    let mut current_worktree: Option<(PathBuf, Option<String>, WorkspaceStatus)> = None;

    for line in output.lines() {
        if line.starts_with("worktree ") {
            // Save previous worktree if it exists and is under werx root
            if let Some((path, branch, status)) = current_worktree.take() {
                if path.starts_with(werx_root) {
                    // Extract workspace name from path
                    if let Some(name) = extract_workspace_name(&path, werx_root, repo_name) {
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
        if path.starts_with(werx_root) {
            if let Some(name) = extract_workspace_name(&path, werx_root, repo_name) {
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
/// Expected format: <werx_root>/<repo_name>/<workspace_name>
fn extract_workspace_name(path: &PathBuf, werx_root: &PathBuf, repo_name: &str) -> Option<String> {
    // Strip werx root
    let relative = path.strip_prefix(werx_root).ok()?;

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
pub fn find_repository(werx: &Werx, repo_spec: &str) -> Result<RepoInfo> {
    let config = werx.load_config()?;

    // Parse the repository specification to get the directory name
    let spec = RepoSpec::parse(repo_spec, config.default_provider(), config.protocol())
        .context("Failed to parse repository specification")?;

    // Get repository info to determine correct directory name
    let repos = repos::list_repos(werx)?;
    let dir_name = spec.dir_name(&repos);

    // Check if repository exists
    let repo_dir = werx.repos_dir().join(&dir_name);
    if !repo_dir.exists() {
        return Err(anyhow!(
            "Repository not found: {}\n\nRun 'werx repos list' to see available repositories.",
            spec.original
        ));
    }

    repos
        .into_iter()
        .find(|r| r.dir_name == dir_name)
        .ok_or_else(|| anyhow!("Repository not found: {}", spec.original))
}

/// Detect if the current directory is within a workspace and return its repository info
pub fn detect_current_workspace(current_dir: &Path, werx: &Werx) -> Result<Option<RepoInfo>> {
    // Check if current directory is under the werx root
    if !current_dir.starts_with(&werx.root) {
        return Ok(None);
    }

    // Get all workspaces
    let workspaces = list_workspaces(werx)?;

    // Find if current directory is within any workspace
    for workspace in workspaces {
        if current_dir.starts_with(&workspace.path) {
            // Found a workspace that contains the current directory
            // Return the repository info for this workspace
            let repos = repos::list_repos(werx)?;
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
pub fn select_repository(werx: &Werx) -> Result<RepoInfo> {
    // Check if we're in an interactive terminal
    if !std::io::stdin().is_terminal() {
        return Err(anyhow!(
            "Cannot prompt for repository selection in non-interactive mode.\n\
             Please specify a repository explicitly or run from within a workspace."
        ));
    }

    // Get all repositories
    let repos = repos::list_repos(werx)?;

    if repos.is_empty() {
        return Err(anyhow!(
            "No repositories found in Werx.\n\n\
             Run 'werx add <repo>' to add a repository first."
        ));
    }

    // Filter out invalid repositories
    let valid_repos: Vec<&RepoInfo> = repos.iter().filter(|r| r.valid).collect();

    if valid_repos.is_empty() {
        return Err(anyhow!(
            "No valid repositories found in Werx.\n\n\
             Run 'werx repos list' to see repository status."
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

/// Select a repository using fuzzy search with skim
pub fn fuzzy_select_repository(werx: &Werx) -> Result<Option<RepoInfo>> {
    // Check if we're in an interactive terminal
    if !std::io::stdin().is_terminal() {
        return Err(anyhow!(
            "Interactive terminal required for fuzzy search.\n\
             Please specify a repository explicitly."
        ));
    }

    // Get all repositories
    let repos = repos::list_repos(werx)?;

    if repos.is_empty() {
        return Err(anyhow!(
            "No repositories found in Werx.\n\n\
             Run 'werx add <repo>' to add a repository first."
        ));
    }

    // Filter out invalid repositories
    let valid_repos: Vec<RepoInfo> = repos.into_iter().filter(|r| r.valid).collect();

    if valid_repos.is_empty() {
        return Err(anyhow!(
            "No valid repositories found in Werx.\n\n\
             Run 'werx repos list' to see repository status."
        ));
    }

    // Create display strings for each repo
    let display_strings: Vec<String> = valid_repos
        .iter()
        .map(|r| {
            if let Some(branch) = &r.default_branch {
                format!("{} ({})", r.clone_url, branch)
            } else {
                r.clone_url.clone()
            }
        })
        .collect();

    // Create a string containing all items for skim to read
    let item_reader = SkimItemReader::default();
    let items_str = display_strings.join("\n");
    let item_stream = Cursor::new(items_str);
    let rx_item = item_reader.of_bufread(item_stream);

    // Configure skim options
    let options = SkimOptionsBuilder::default()
        .height(Some("50%"))
        .multi(false)
        .prompt(Some("Select repository: "))
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

        // Find the matching repository
        for repo in valid_repos {
            let display = if let Some(branch) = &repo.default_branch {
                format!("{} ({})", repo.clone_url, branch)
            } else {
                repo.clone_url.clone()
            };
            if display == selected_text.as_ref() {
                return Ok(Some(repo));
            }
        }
    }

    Ok(None)
}

/// Prompt the user for a new branch name, with option to change base branch
pub fn prompt_branch_name(werx: &Werx, repo_info: &RepoInfo) -> Result<(String, String)> {
    // Check if we're in an interactive terminal
    if !std::io::stdin().is_terminal() {
        return Err(anyhow!(
            "Cannot prompt for branch name in non-interactive mode.\n\
             Please specify a branch with --branch."
        ));
    }

    let default_base = repo_info.default_branch.as_deref().unwrap_or("main");

    let mut base_branch = default_base.to_string();

    loop {
        println!();
        println!(
            "Creating a new branch based on '{}'. Press Tab to change base branch.",
            base_branch
        );

        // Use a custom prompt that can detect Tab
        match prompt_branch_name_with_tab_handler(&base_branch)? {
            BranchPromptResult::BranchName(name) => {
                return Ok((name, base_branch));
            }
            BranchPromptResult::ChangeBase => {
                // Show branch selector
                if let Some(new_base) = select_branch(werx, repo_info)? {
                    base_branch = new_base;
                }
                // Loop back to prompt for branch name
            }
        }
    }
}

/// Result from the branch name prompt
enum BranchPromptResult {
    /// User entered a branch name
    BranchName(String),
    /// User wants to change the base branch
    ChangeBase,
}

/// Prompt for branch name, detecting Tab key to change base
fn prompt_branch_name_with_tab_handler(_base_branch: &str) -> Result<BranchPromptResult> {
    use crossterm::{
        cursor,
        event::{self, Event, KeyCode, KeyModifiers},
        execute,
        terminal::{disable_raw_mode, enable_raw_mode},
    };
    use std::io::{stdout, Write};

    let theme = ColorfulTheme::default();

    print!("{} New branch name: ", theme.prompt_prefix);
    stdout().flush()?;

    enable_raw_mode()?;

    let mut input = String::new();

    loop {
        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key_event) = event::read()? {
                match key_event.code {
                    KeyCode::Tab => {
                        disable_raw_mode()?;
                        println!();
                        return Ok(BranchPromptResult::ChangeBase);
                    }
                    KeyCode::Enter => {
                        disable_raw_mode()?;
                        println!();
                        let name = input.trim().to_string();
                        if name.is_empty() {
                            return Err(anyhow!("Branch name cannot be empty"));
                        }
                        return Ok(BranchPromptResult::BranchName(name));
                    }
                    KeyCode::Char(c) => {
                        if key_event.modifiers.contains(KeyModifiers::CONTROL) && c == 'c' {
                            disable_raw_mode()?;
                            return Err(anyhow!("Cancelled"));
                        }
                        input.push(c);
                        print!("{}", c);
                        stdout().flush()?;
                    }
                    KeyCode::Backspace => {
                        if !input.is_empty() {
                            input.pop();
                            execute!(stdout(), cursor::MoveLeft(1))?;
                            print!(" ");
                            execute!(stdout(), cursor::MoveLeft(1))?;
                            stdout().flush()?;
                        }
                    }
                    KeyCode::Esc => {
                        disable_raw_mode()?;
                        return Err(anyhow!("Cancelled"));
                    }
                    _ => {}
                }
            }
        }
    }
}

/// Select a branch from the repository using skim fuzzy finder
fn select_branch(werx: &Werx, repo_info: &RepoInfo) -> Result<Option<String>> {
    let repo_path = werx.repos_dir().join(&repo_info.dir_name);

    // Get list of branches
    let output = Command::new("git")
        .arg("-C")
        .arg(&repo_path)
        .arg("branch")
        .arg("-a")
        .arg("--format=%(refname:short)")
        .output()
        .context("Failed to list branches")?;

    if !output.status.success() {
        return Err(anyhow!("Failed to list branches"));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let branches: Vec<String> = stdout
        .lines()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        // Clean up remote branch names (origin/main -> main)
        .map(|s| {
            if let Some(stripped) = s.strip_prefix("origin/") {
                stripped.to_string()
            } else {
                s
            }
        })
        // Remove duplicates
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();

    if branches.is_empty() {
        return Err(anyhow!("No branches found in repository"));
    }

    // Create skim input
    let item_reader = SkimItemReader::default();
    let items_str = branches.join("\n");
    let item_stream = Cursor::new(items_str);
    let rx_item = item_reader.of_bufread(item_stream);

    let options = SkimOptionsBuilder::default()
        .height(Some("50%"))
        .multi(false)
        .prompt(Some("Select base branch: "))
        .build()
        .map_err(|e| anyhow!("Failed to build fuzzy finder options: {}", e))?;

    let output = Skim::run_with(&options, Some(rx_item))
        .ok_or_else(|| anyhow!("Fuzzy finder failed to run"))?;

    if output.is_abort {
        return Ok(None);
    }

    if let Some(selected) = output.selected_items.first() {
        return Ok(Some(selected.text().to_string()));
    }

    Ok(None)
}

/// Generate workspace path in hierarchical format: <repo_name>/<workspace_name>
pub fn generate_workspace_path(werx: &Werx, repo_name: &str, workspace_name: &str) -> PathBuf {
    werx.root.join(repo_name).join(workspace_name)
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
    werx: &Werx,
    repo_info: &RepoInfo,
    workspace_name: &str,
    branch: &str,
) -> Result<PathBuf> {
    // Generate the hierarchical workspace path: <werx-root>/<repo-name>/<workspace-name>
    let workspace_path = generate_workspace_path(werx, &repo_info.dir_name, workspace_name);

    // Check if workspace already exists
    if workspace_path.exists() {
        return Err(anyhow!(
            "Workspace already exists at: {}\n\n\
             Choose a different workspace name or remove the existing workspace first.",
            workspace_path.display()
        ));
    }

    // Create the repository directory if it doesn't exist
    let repo_dir_in_werx = werx.root.join(&repo_info.dir_name);
    if !repo_dir_in_werx.exists() {
        fs::create_dir_all(&repo_dir_in_werx).context(format!(
            "Failed to create directory '{}'",
            repo_dir_in_werx.display()
        ))?;
    }

    // Get the bare repository path
    let bare_repo_path = werx.repos_dir().join(&repo_info.dir_name);

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
pub fn remove_workspace(werx: &Werx, workspace_path: &str) -> Result<()> {
    // Get all workspaces
    let workspaces = list_workspaces(werx)?;

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
             Run 'werx workspace list' to see available workspaces.",
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
    let bare_repo_path = werx.repos_dir().join(&workspace.repository);

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
    let repo_dir_in_werx = werx.root.join(&workspace.repository);
    if repo_dir_in_werx.exists() && repo_dir_in_werx.is_dir() {
        // Check if directory is empty
        if let Ok(mut entries) = fs::read_dir(&repo_dir_in_werx) {
            if entries.next().is_none() {
                // Directory is empty, remove it
                let _ = fs::remove_dir(&repo_dir_in_werx);
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
            let branch_str = ws.branch.as_deref().unwrap_or("(detached)");
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
pub fn find_workspace_matches<'a>(workspaces: &'a [Workspace], query: &str) -> Vec<&'a Workspace> {
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
            matcher.fuzzy_match(&display, &query_lower).is_some() || display.contains(&query_lower)
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
                return Err(anyhow!("No workspaces match query: '{}'", q));
            }

            // Multiple matches - fall through to fuzzy search with pre-filled query
        }
    }

    // Launch interactive fuzzy search (with query pre-filled if provided)
    fuzzy_select_workspace(workspaces, query)
}

/// Check if a workspace's branch exists on any remote
pub fn check_branch_pushed(workspace_path: &Path, branch: &str) -> Result<bool> {
    // Get list of remote branches
    let output = Command::new("git")
        .arg("-C")
        .arg(workspace_path)
        .arg("branch")
        .arg("-r")
        .output()
        .context("Failed to execute git branch -r")?;

    if !output.status.success() {
        // If git command fails, assume branch is unpushed
        return Ok(false);
    }

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Parse remote branches and check if any match our branch
    for line in stdout.lines() {
        let trimmed = line.trim();
        // Remote branches are in format: origin/branch-name or remote/branch-name
        if let Some(remote_branch) = trimmed.split('/').nth(1) {
            if remote_branch == branch {
                return Ok(true);
            }
        }
    }

    Ok(false)
}

/// Get the default branch for a repository
pub fn get_default_branch(repo_path: &Path) -> Result<String> {
    // First try to get the default branch from symbolic-ref
    let output = Command::new("git")
        .arg("-C")
        .arg(repo_path)
        .arg("symbolic-ref")
        .arg("refs/remotes/origin/HEAD")
        .output();

    if let Ok(output) = output {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let branch_ref = stdout.trim();
            // Extract branch name from refs/remotes/origin/main
            if let Some(branch_name) = branch_ref.strip_prefix("refs/remotes/origin/") {
                return Ok(branch_name.to_string());
            }
        }
    }

    // Fallback: try common default branch names
    for default_name in &["main", "master", "develop"] {
        let output = Command::new("git")
            .arg("-C")
            .arg(repo_path)
            .arg("rev-parse")
            .arg("--verify")
            .arg(format!("refs/remotes/origin/{}", default_name))
            .output();

        if let Ok(output) = output {
            if output.status.success() {
                return Ok(default_name.to_string());
            }
        }
    }

    Err(anyhow!("Could not determine default branch"))
}

/// Check if a workspace's branch is merged to the default branch
pub fn check_branch_merged(
    workspace_path: &Path,
    branch: &str,
    default_branch: &str,
) -> Result<bool> {
    // Don't check if we're on the default branch itself
    if branch == default_branch {
        return Ok(false);
    }

    // Check if the branch is fully merged using git merge-base
    let output = Command::new("git")
        .arg("-C")
        .arg(workspace_path)
        .arg("merge-base")
        .arg("--is-ancestor")
        .arg(branch)
        .arg(format!("origin/{}", default_branch))
        .output()
        .context("Failed to execute git merge-base")?;

    // Exit code 0 means it's an ancestor (merged)
    // Exit code 1 means it's not an ancestor (not merged)
    // Other exit codes indicate errors
    Ok(output.status.success())
}

/// Get comprehensive status for a workspace
pub fn get_workspace_status_details(
    workspace: &Workspace,
    werx: &Werx,
) -> Result<WorkspaceStatusDetails> {
    let path = &workspace.path;

    // Check if workspace exists
    if !path.exists() {
        return Ok(WorkspaceStatusDetails {
            uncommitted_changes: false,
            unpushed_branch: false,
            merged_branch: false,
            branch_name: workspace.branch.clone(),
            default_branch: None,
            status_details: None,
        });
    }

    // Get branch name
    let branch_name = workspace.branch.clone();

    // Check uncommitted changes
    let basic_status = check_workspace_status(path)?;
    let uncommitted_changes = matches!(
        basic_status,
        WorkspaceStatus::Modified | WorkspaceStatus::Untracked
    );

    // Get detailed status if there are uncommitted changes
    let status_details = if uncommitted_changes {
        Some(get_detailed_status(path)?)
    } else {
        None
    };

    // Get repository path for default branch checking
    let repo_path = werx.repos_dir().join(&workspace.repository);

    // Get default branch
    let default_branch = get_default_branch(&repo_path).ok();

    // Check if branch is pushed (skip if no branch name)
    let unpushed_branch = if let Some(ref branch) = branch_name {
        !check_branch_pushed(path, branch).unwrap_or(true)
    } else {
        false
    };

    // Check if branch is merged (only if we have both branch names and branch is pushed)
    let merged_branch = if let (Some(branch), Some(default)) = (&branch_name, &default_branch) {
        if !unpushed_branch && branch != default {
            check_branch_merged(path, branch, default).unwrap_or(false)
        } else {
            false
        }
    } else {
        false
    };

    Ok(WorkspaceStatusDetails {
        uncommitted_changes,
        unpushed_branch,
        merged_branch,
        branch_name,
        default_branch,
        status_details,
    })
}

/// Get detailed status information for a workspace
fn get_detailed_status(path: &Path) -> Result<StatusDetails> {
    let output = Command::new("git")
        .arg("-C")
        .arg(path)
        .arg("status")
        .arg("--porcelain")
        .output()
        .context("Failed to execute git status")?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut modified_files = Vec::new();
    let mut untracked_files = Vec::new();

    for line in stdout.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        // Parse git status porcelain format
        if trimmed.starts_with("??") {
            // Untracked file
            let file = trimmed.strip_prefix("??").unwrap_or("").trim();
            untracked_files.push(file.to_string());
        } else {
            // Modified file (staged or unstaged)
            let file = if trimmed.len() > 3 {
                trimmed[3..].trim().to_string()
            } else {
                continue;
            };
            modified_files.push(file);
        }
    }

    Ok(StatusDetails {
        modified_files,
        untracked_files,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_workspace_struct() {
        let workspace = Workspace {
            name: "feature-branch".to_string(),
            path: PathBuf::from("/tmp/werx/myrepo/feature-branch"),
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
        let werx = Werx {
            root: PathBuf::from("/tmp/werx"),
        };

        assert_eq!(werx.workspaces_dir(), PathBuf::from("/tmp/werx"));
    }

    #[test]
    fn test_parse_worktree_output() {
        let werx = Werx {
            root: PathBuf::from("/home/user/werx"),
        };

        let output = r#"worktree /home/user/werx/myrepo/main
HEAD abc123def456
branch refs/heads/main

worktree /home/user/werx/myrepo/feature-branch
HEAD def789ghi012
branch refs/heads/feature-branch

"#;

        let workspaces = parse_worktree_output(&werx, output, "myrepo").unwrap();

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
    fn test_parse_worktree_output_filters_non_werx_workspaces() {
        let werx = Werx {
            root: PathBuf::from("/home/user/werx"),
        };

        let output = r#"worktree /home/user/werx/myrepo/main
HEAD abc123def456
branch refs/heads/main

worktree /home/user/other-location/myrepo
HEAD def789ghi012
branch refs/heads/other

"#;

        let workspaces = parse_worktree_output(&werx, output, "myrepo").unwrap();

        // Should only include the workspace under werx root
        assert_eq!(workspaces.len(), 1);
        assert_eq!(workspaces[0].name, "main");
    }

    #[test]
    fn test_extract_workspace_name() {
        let werx_root = PathBuf::from("/home/user/werx");

        // Standard case
        let path = PathBuf::from("/home/user/werx/myrepo/feature-branch");
        let name = extract_workspace_name(&path, &werx_root, "myrepo");
        assert_eq!(name, Some("feature-branch".to_string()));

        // Multi-level workspace name
        let path = PathBuf::from("/home/user/werx/myrepo/nested/workspace");
        let name = extract_workspace_name(&path, &werx_root, "myrepo");
        assert_eq!(name, Some("nested/workspace".to_string()));

        // Outside werx root
        let path = PathBuf::from("/home/user/other/myrepo/feature");
        let name = extract_workspace_name(&path, &werx_root, "myrepo");
        assert_eq!(name, None);

        // Wrong repo name
        let path = PathBuf::from("/home/user/werx/otherrepo/feature");
        let name = extract_workspace_name(&path, &werx_root, "myrepo");
        assert_eq!(name, None);
    }

    #[test]
    fn test_detect_current_workspace_outside_werx() {
        let werx = Werx {
            root: PathBuf::from("/home/user/werx"),
        };
        let current_dir = PathBuf::from("/home/user/other");

        let result = detect_current_workspace(&current_dir, &werx).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_detect_current_workspace_in_werx_root_but_not_in_workspace() {
        let werx = Werx {
            root: PathBuf::from("/home/user/werx"),
        };
        let current_dir = PathBuf::from("/home/user/werx");

        // This will return None because there are no workspaces
        let result = detect_current_workspace(&current_dir, &werx).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_generate_workspace_path() {
        let werx = Werx {
            root: PathBuf::from("/home/user/werx"),
        };

        let path = generate_workspace_path(&werx, "myrepo-abc123", "feature-branch");
        assert_eq!(
            path,
            PathBuf::from("/home/user/werx/myrepo-abc123/feature-branch")
        );

        // Test with multi-level workspace name
        let path = generate_workspace_path(&werx, "myrepo-abc123", "nested/workspace");
        assert_eq!(
            path,
            PathBuf::from("/home/user/werx/myrepo-abc123/nested/workspace")
        );
    }

    #[test]
    fn test_find_workspace_matches_single() {
        let workspaces = vec![
            Workspace {
                name: "main".to_string(),
                path: PathBuf::from("/tmp/werx/myrepo/main"),
                repository: "myrepo".to_string(),
                branch: Some("main".to_string()),
                status: WorkspaceStatus::Clean,
            },
            Workspace {
                name: "feature-x".to_string(),
                path: PathBuf::from("/tmp/werx/myrepo/feature-x"),
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
                path: PathBuf::from("/tmp/werx/myrepo/feature-a"),
                repository: "myrepo".to_string(),
                branch: Some("feature-a".to_string()),
                status: WorkspaceStatus::Clean,
            },
            Workspace {
                name: "feature-b".to_string(),
                path: PathBuf::from("/tmp/werx/myrepo/feature-b"),
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
        let workspaces = vec![Workspace {
            name: "main".to_string(),
            path: PathBuf::from("/tmp/werx/myrepo/main"),
            repository: "myrepo".to_string(),
            branch: Some("main".to_string()),
            status: WorkspaceStatus::Clean,
        }];

        let matches = find_workspace_matches(&workspaces, "nonexistent");
        assert_eq!(matches.len(), 0);
    }

    #[test]
    fn test_find_workspace_matches_case_insensitive() {
        let workspaces = vec![Workspace {
            name: "main".to_string(),
            path: PathBuf::from("/tmp/werx/MyRepo/main"),
            repository: "MyRepo".to_string(),
            branch: Some("main".to_string()),
            status: WorkspaceStatus::Clean,
        }];

        let matches = find_workspace_matches(&workspaces, "myrepo");
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].repository, "MyRepo");
    }
}
