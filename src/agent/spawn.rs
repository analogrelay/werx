//! Agent spawning functionality.
//!
//! Handles creating worktrees and spawning agents in tmux sessions.

use anyhow::{anyhow, Context, Result};
use std::path::PathBuf;
use std::process::Command;

use crate::workspace::generate_workspace_path;
use crate::{Forge, RepoInfo};

use super::names::generate_agent_name;
use super::providers::{detect_providers, get_default_provider, AgentProvider};
use super::tmux::{
    tmux_create_session, tmux_create_window, tmux_is_available, tmux_list_windows, tmux_send_keys,
    tmux_session_exists,
};
use super::{Agent, AgentStatus, AgentType, SpawnOptions, SpawnResult};

/// Spawn a new agent for a repository
pub fn spawn_agent(
    forge: &Forge,
    repo_info: &RepoInfo,
    options: SpawnOptions,
) -> Result<SpawnResult> {
    // Check if tmux is available
    if !tmux_is_available() {
        return Err(anyhow!(
            "tmux is required for agent management but was not found.\n\n\
             Install tmux:\n  \
             - macOS: brew install tmux\n  \
             - Ubuntu/Debian: apt install tmux\n  \
             - Other: https://github.com/tmux/tmux"
        ));
    }

    // Detect available providers
    let providers = detect_providers();

    // Get the provider to use
    let provider = get_default_provider(&providers, options.agent_type)?;

    // Determine the branch to use
    let requested_branch = options
        .branch
        .clone()
        .or_else(|| repo_info.default_branch.clone())
        .ok_or_else(|| anyhow!("Could not determine branch. Please specify with --branch."))?;

    // Get existing agent names to avoid collisions
    let existing_names = get_existing_agent_names()?;

    // Generate a unique agent name
    let agent_name = generate_agent_name(&existing_names);

    // Determine the base branch for creating new branches
    let base_branch = options
        .base_branch
        .as_deref()
        .or(repo_info.default_branch.as_deref())
        .unwrap_or("main");

    // Check if the branch already has a worktree and resolve if needed
    let (actual_branch, branch_info) = resolve_branch_for_agent(
        forge,
        repo_info,
        &requested_branch,
        base_branch,
        &agent_name,
    )?;

    // Create a worktree for this agent
    let worktree_path = create_agent_worktree(forge, repo_info, &agent_name, &actual_branch)?;

    // Create or use existing tmux session and window
    create_agent_window(&agent_name, &worktree_path)?;

    // Start the agent in the tmux window
    start_agent_in_window(
        &agent_name,
        provider,
        &worktree_path,
        options.prompt.as_deref(),
    )?;

    // Build the agent info
    let agent = Agent {
        name: agent_name,
        agent_type: provider.agent_type,
        repository: repo_info.dir_name.clone(),
        worktree_path,
        status: AgentStatus::Running,
        branch: Some(actual_branch),
    };

    let mut result = SpawnResult::new(agent);
    result.created_branch = branch_info;

    Ok(result)
}

/// Get names of existing agents from tmux windows
fn get_existing_agent_names() -> Result<Vec<String>> {
    let windows = tmux_list_windows()?;
    Ok(windows.into_iter().map(|w| w.name).collect())
}

/// Check if a branch already has a worktree in this repository
fn branch_has_worktree(repo_path: &std::path::Path, branch: &str) -> Result<bool> {
    let output = Command::new("git")
        .arg("-C")
        .arg(repo_path)
        .arg("worktree")
        .arg("list")
        .arg("--porcelain")
        .output()
        .context("Failed to execute git worktree list")?;

    if !output.status.success() {
        return Ok(false);
    }

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Parse porcelain output looking for branch refs
    for line in stdout.lines() {
        if line.starts_with("branch ") {
            let branch_ref = line.strip_prefix("branch ").unwrap_or("");
            let worktree_branch = branch_ref.strip_prefix("refs/heads/").unwrap_or(branch_ref);
            if worktree_branch == branch {
                return Ok(true);
            }
        }
    }

    Ok(false)
}

/// Create a new branch for the agent based on the requested branch
fn create_agent_branch(
    repo_path: &std::path::Path,
    agent_name: &str,
    base_branch: &str,
) -> Result<String> {
    let agent_branch = format!("agent/{}", agent_name);

    let output = Command::new("git")
        .arg("-C")
        .arg(repo_path)
        .arg("branch")
        .arg(&agent_branch)
        .arg(base_branch)
        .output()
        .context("Failed to create agent branch")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!(
            "Failed to create branch '{}': {}",
            agent_branch,
            stderr
        ));
    }

    Ok(agent_branch)
}

/// Check if a branch exists in the repository
fn branch_exists(repo_path: &std::path::Path, branch: &str) -> Result<bool> {
    let output = Command::new("git")
        .arg("-C")
        .arg(repo_path)
        .arg("rev-parse")
        .arg("--verify")
        .arg(format!("refs/heads/{}", branch))
        .output()
        .context("Failed to check branch existence")?;

    Ok(output.status.success())
}

/// Create a new branch from a base branch
fn create_branch(repo_path: &std::path::Path, branch: &str, base_branch: &str) -> Result<()> {
    let output = Command::new("git")
        .arg("-C")
        .arg(repo_path)
        .arg("branch")
        .arg(branch)
        .arg(base_branch)
        .output()
        .context("Failed to create branch")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!(
            "Failed to create branch '{}' from '{}': {}",
            branch,
            base_branch,
            stderr
        ));
    }

    Ok(())
}

/// Resolve the branch to use for the agent, creating new branches if needed
///
/// Returns (actual_branch, info_message)
fn resolve_branch_for_agent(
    forge: &Forge,
    repo_info: &RepoInfo,
    requested_branch: &str,
    base_branch: &str,
    agent_name: &str,
) -> Result<(String, Option<String>)> {
    let repo_path = forge.repos_dir().join(&repo_info.dir_name);

    // Check if the requested branch exists
    let branch_already_exists = branch_exists(&repo_path, requested_branch)?;

    if !branch_already_exists {
        // Branch doesn't exist - create it from base_branch
        eprintln!(
            "Creating new branch '{}' from '{}'.",
            requested_branch, base_branch
        );
        create_branch(&repo_path, requested_branch, base_branch)?;
        let info = format!(
            "Created new branch '{}' from '{}'",
            requested_branch, base_branch
        );
        return Ok((requested_branch.to_string(), Some(info)));
    }

    // Branch exists - check if it already has a worktree
    if branch_has_worktree(&repo_path, requested_branch)? {
        // Branch already has a worktree - create an agent-specific branch
        eprintln!(
            "Note: Branch '{}' already has a worktree. Creating agent branch 'agent/{}'.",
            requested_branch, agent_name
        );
        let agent_branch = create_agent_branch(&repo_path, agent_name, requested_branch)?;
        let info = format!(
            "Created agent branch 'agent/{}' because '{}' already has a worktree",
            agent_name, requested_branch
        );
        return Ok((agent_branch, Some(info)));
    }

    // Branch exists and is available
    Ok((requested_branch.to_string(), None))
}

/// Create a worktree for the agent
fn create_agent_worktree(
    forge: &Forge,
    repo_info: &RepoInfo,
    agent_name: &str,
    branch: &str,
) -> Result<PathBuf> {
    // Agent worktrees go under <repo>/<agent_name>
    let workspace_path = generate_workspace_path(forge, &repo_info.dir_name, agent_name);

    // Check if path already exists (shouldn't happen with unique names)
    if workspace_path.exists() {
        return Err(anyhow!(
            "Worktree path already exists: {}\nThis should not happen with generated names.",
            workspace_path.display()
        ));
    }

    // Get the bare repository path
    let bare_repo_path = forge.repos_dir().join(&repo_info.dir_name);

    // Create the repository directory if it doesn't exist
    let repo_dir_in_forge = forge.root.join(&repo_info.dir_name);
    if !repo_dir_in_forge.exists() {
        std::fs::create_dir_all(&repo_dir_in_forge).context(format!(
            "Failed to create directory '{}'",
            repo_dir_in_forge.display()
        ))?;
    }

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
            let _ = std::fs::remove_dir_all(&workspace_path);
        }

        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("Git worktree add failed:\n{}", stderr));
    }

    Ok(workspace_path)
}

/// Create a tmux window for the agent
fn create_agent_window(agent_name: &str, working_dir: &PathBuf) -> Result<()> {
    if tmux_session_exists() {
        // Add a new window to existing session
        tmux_create_window(agent_name, working_dir)?;
    } else {
        // Create the session with this window
        tmux_create_session(agent_name, working_dir)?;
    }
    Ok(())
}

/// Start the agent process in its tmux window
fn start_agent_in_window(
    agent_name: &str,
    provider: &AgentProvider,
    _working_dir: &PathBuf,
    prompt: Option<&str>,
) -> Result<()> {
    // Build the command string to send to tmux
    let mut cmd_parts = vec![provider.command.clone()];
    cmd_parts.extend(provider.args.clone());

    // Add prompt if provided (based on agent type)
    if let Some(prompt_text) = prompt {
        match provider.agent_type {
            AgentType::OpenCode => {
                cmd_parts.push("--prompt".to_string());
                cmd_parts.push(shell_escape(prompt_text));
            }
            AgentType::Claude => {
                cmd_parts.push(shell_escape(prompt_text));
            }
            AgentType::Copilot => {
                cmd_parts.push("--prompt".to_string());
                cmd_parts.push(shell_escape(prompt_text));
            }
        }
    }

    let command = cmd_parts.join(" ");

    // Send the command to the tmux window
    tmux_send_keys(agent_name, &command)?;

    Ok(())
}

/// Escape a string for shell use
fn shell_escape(s: &str) -> String {
    // Simple escaping - wrap in single quotes and escape any single quotes
    format!("'{}'", s.replace('\'', "'\\''"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shell_escape_simple() {
        assert_eq!(shell_escape("hello"), "'hello'");
    }

    #[test]
    fn test_shell_escape_with_quotes() {
        assert_eq!(shell_escape("it's"), "'it'\\''s'");
    }

    #[test]
    fn test_shell_escape_with_spaces() {
        assert_eq!(shell_escape("hello world"), "'hello world'");
    }
}
