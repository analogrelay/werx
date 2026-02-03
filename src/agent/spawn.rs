//! Agent spawning functionality.
//!
//! Handles creating worktrees and spawning agents in tmux sessions.

use anyhow::{anyhow, Context, Result};
use std::path::PathBuf;

use crate::workspace::{create_worktree, generate_workspace_path};
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
    let branch = options
        .branch
        .clone()
        .or_else(|| repo_info.default_branch.clone())
        .ok_or_else(|| anyhow!("Could not determine branch. Please specify with --branch."))?;

    // Get existing agent names to avoid collisions
    let existing_names = get_existing_agent_names()?;

    // Generate a unique agent name
    let agent_name = generate_agent_name(&existing_names);

    // Create a worktree for this agent
    let worktree_path = create_agent_worktree(forge, repo_info, &agent_name, &branch)?;

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
        branch: Some(branch),
    };

    Ok(SpawnResult::new(agent))
}

/// Get names of existing agents from tmux windows
fn get_existing_agent_names() -> Result<Vec<String>> {
    let windows = tmux_list_windows()?;
    Ok(windows.into_iter().map(|w| w.name).collect())
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

    // Create the worktree
    create_worktree(forge, repo_info, agent_name, branch)
        .context("Failed to create worktree for agent")
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
                cmd_parts.push("--prompt".to_string());
                cmd_parts.push(shell_escape(prompt_text));
            }
            AgentType::Copilot => {
                // Copilot may handle prompts differently
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
