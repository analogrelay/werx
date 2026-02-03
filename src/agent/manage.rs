//! Agent management operations: list, status, attach, kill.

use anyhow::{Result, anyhow};
use std::path::PathBuf;

use crate::Werx;
use crate::workspace::{list_workspaces, remove_workspace};

use super::tmux::{
    tmux_attach, tmux_is_available, tmux_kill_window, tmux_list_windows, tmux_session_exists,
};
use super::{Agent, AgentStatus, AgentType};

/// List all running agents
pub fn list_agents(werx: &Werx) -> Result<Vec<Agent>> {
    if !tmux_is_available() {
        return Ok(Vec::new());
    }

    if !tmux_session_exists() {
        return Ok(Vec::new());
    }

    let windows = tmux_list_windows()?;
    let workspaces = list_workspaces(werx)?;

    let mut agents = Vec::new();

    for window in windows {
        // Find the matching workspace for this agent
        let workspace = workspaces.iter().find(|ws| ws.name == window.name);

        let (repository, worktree_path, branch) = if let Some(ws) = workspace {
            (ws.repository.clone(), ws.path.clone(), ws.branch.clone())
        } else {
            // Agent exists but workspace not found - might have been cleaned up
            ("unknown".to_string(), PathBuf::from("unknown"), None)
        };

        // Determine agent type from the workspace name or command
        // For now, default to Unknown since we can't easily determine the type
        let agent_type = AgentType::OpenCode; // Default assumption

        let status = if window.active {
            AgentStatus::Running
        } else {
            AgentStatus::Exited
        };

        agents.push(Agent {
            name: window.name,
            agent_type,
            repository,
            worktree_path,
            status,
            branch,
        });
    }

    Ok(agents)
}

/// Get detailed status for a specific agent
pub fn get_agent_status(werx: &Werx, agent_name: &str) -> Result<Agent> {
    let agents = list_agents(werx)?;

    agents
        .into_iter()
        .find(|a| a.name == agent_name)
        .ok_or_else(|| {
            anyhow!(
                "Agent '{}' not found.\n\nRun 'werx agent list' to see running agents.",
                agent_name
            )
        })
}

/// Attach to the agent tmux session
pub fn attach_to_agent(agent_name: Option<&str>) -> Result<()> {
    if !tmux_is_available() {
        return Err(anyhow!(
            "tmux is required but was not found.\n\n\
             Install tmux to use agent management."
        ));
    }

    if !tmux_session_exists() {
        return Err(anyhow!(
            "No agents are currently running.\n\n\
             Run 'werx agent spawn' to start an agent."
        ));
    }

    // If a specific agent is requested, verify it exists
    if let Some(name) = agent_name {
        let windows = tmux_list_windows()?;
        if !windows.iter().any(|w| w.name == name) {
            let available: Vec<String> = windows.iter().map(|w| w.name.clone()).collect();
            return Err(anyhow!(
                "Agent '{}' not found.\n\nAvailable agents: {}",
                name,
                available.join(", ")
            ));
        }
    }

    // This will exec into tmux, replacing the current process
    tmux_attach(agent_name)
}

/// Kill an agent and optionally clean up its worktree
pub fn kill_agent(werx: &Werx, agent_name: &str, cleanup: bool) -> Result<bool> {
    if !tmux_session_exists() {
        return Err(anyhow!(
            "No agents are currently running.\n\n\
             Run 'werx agent list' to see agent status."
        ));
    }

    // Verify the agent exists
    let windows = tmux_list_windows()?;
    if !windows.iter().any(|w| w.name == agent_name) {
        let available: Vec<String> = windows.iter().map(|w| w.name.clone()).collect();
        return Err(anyhow!(
            "Agent '{}' not found.\n\nAvailable agents: {}",
            agent_name,
            available.join(", ")
        ));
    }

    // Find the workspace path before killing
    let workspaces = list_workspaces(werx)?;
    let workspace = workspaces.iter().find(|ws| ws.name == agent_name);

    // Kill the tmux window
    let session_closed = tmux_kill_window(agent_name)?;

    // Clean up worktree if requested
    if cleanup && let Some(ws) = workspace {
        let workspace_path = format!("{}/{}", ws.repository, ws.name);
        remove_workspace(werx, &workspace_path)?;
    }

    Ok(session_closed)
}

/// Find an agent by name (partial match supported)
pub fn find_agent(werx: &Werx, query: &str) -> Result<Agent> {
    let agents = list_agents(werx)?;

    // Exact match first
    if let Some(agent) = agents.iter().find(|a| a.name == query) {
        return Ok(agent.clone());
    }

    // Partial match
    let matches: Vec<&Agent> = agents.iter().filter(|a| a.name.contains(query)).collect();

    match matches.len() {
        0 => Err(anyhow!(
            "No agent matching '{}' found.\n\nRun 'werx agent list' to see running agents.",
            query
        )),
        1 => Ok(matches[0].clone()),
        _ => {
            let names: Vec<&str> = matches.iter().map(|a| a.name.as_str()).collect();
            Err(anyhow!(
                "Multiple agents match '{}': {}\n\nPlease be more specific.",
                query,
                names.join(", ")
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_status_display() {
        assert_eq!(AgentStatus::Running.to_string(), "running");
        assert_eq!(AgentStatus::Exited.to_string(), "exited");
    }
}
