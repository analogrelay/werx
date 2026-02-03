//! Agent management module for spawning and managing AI coding agents.
//!
//! This module provides functionality to:
//! - Detect available coding agents (OpenCode, Claude Code, GitHub Copilot CLI)
//! - Spawn agents in isolated worktrees
//! - Manage agent sessions via tmux
//! - List, attach to, and kill running agents

mod manage;
mod names;
mod providers;
mod spawn;
mod tmux;

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub use manage::{attach_to_agent, find_agent, get_agent_status, kill_agent, list_agents};
pub use names::generate_agent_name;
pub use providers::{detect_providers, get_default_provider, AgentProvider};
pub use spawn::spawn_agent;
pub use tmux::{
    get_agent_status_from_tmux, tmux_attach, tmux_create_session, tmux_create_window,
    tmux_is_available, tmux_kill_window, tmux_list_windows, tmux_select_window,
    tmux_session_exists, TmuxSession,
};

/// The name of the tmux session used for all forge agents
pub const FORGE_AGENTS_SESSION: &str = "forge-agents";

/// Supported agent types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AgentType {
    /// OpenCode - the preferred default agent
    OpenCode,
    /// Claude Code
    Claude,
    /// GitHub Copilot CLI
    Copilot,
}

impl AgentType {
    /// Get the display name for this agent type
    pub fn display_name(&self) -> &'static str {
        match self {
            AgentType::OpenCode => "OpenCode",
            AgentType::Claude => "Claude Code",
            AgentType::Copilot => "GitHub Copilot",
        }
    }

    /// Get the identifier used in configuration
    pub fn id(&self) -> &'static str {
        match self {
            AgentType::OpenCode => "opencode",
            AgentType::Claude => "claude",
            AgentType::Copilot => "copilot",
        }
    }
}

impl std::fmt::Display for AgentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

impl std::str::FromStr for AgentType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "opencode" => Ok(AgentType::OpenCode),
            "claude" => Ok(AgentType::Claude),
            "copilot" => Ok(AgentType::Copilot),
            _ => Err(anyhow!(
                "Unknown agent type: {}. Valid types: opencode, claude, copilot",
                s
            )),
        }
    }
}

/// Status of a running agent
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum AgentStatus {
    /// Agent is running normally
    Running,
    /// Agent process has exited successfully
    Exited,
    /// Agent process has exited with an error
    Failed,
    /// Agent status is unknown
    Unknown,
}

impl std::fmt::Display for AgentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AgentStatus::Running => write!(f, "running"),
            AgentStatus::Exited => write!(f, "exited"),
            AgentStatus::Failed => write!(f, "failed"),
            AgentStatus::Unknown => write!(f, "unknown"),
        }
    }
}

/// Represents a spawned agent instance
#[derive(Debug, Clone, Serialize)]
pub struct Agent {
    /// Human-readable name (e.g., "arcane_aegis")
    pub name: String,
    /// Type of agent (OpenCode, Claude, Copilot)
    pub agent_type: AgentType,
    /// Repository this agent is working on
    pub repository: String,
    /// Path to the agent's worktree
    pub worktree_path: PathBuf,
    /// Current status of the agent
    pub status: AgentStatus,
    /// The branch the worktree is based on
    pub branch: Option<String>,
}

impl Agent {
    /// Get a display string for listing agents
    pub fn display(&self) -> String {
        format!(
            "{} ({}) - {}",
            self.name,
            self.agent_type.display_name(),
            self.repository
        )
    }
}

/// Information about a spawned agent for display
#[derive(Debug, Clone, Serialize)]
pub struct SpawnResult {
    /// The agent that was spawned
    pub agent: Agent,
    /// Instructions for attaching to the agent
    pub attach_command: String,
    /// If set, indicates an agent branch was created (with reason)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_branch: Option<String>,
}

impl SpawnResult {
    /// Create a new spawn result
    pub fn new(agent: Agent) -> Self {
        let attach_command = format!("forge agent attach {}", agent.name);
        Self {
            agent,
            attach_command,
            created_branch: None,
        }
    }
}

/// Options for spawning an agent
#[derive(Debug, Clone, Default)]
pub struct SpawnOptions {
    /// Specific agent type to use (overrides preferences)
    pub agent_type: Option<AgentType>,
    /// Branch to checkout (defaults to repository's default branch)
    pub branch: Option<String>,
    /// Initial prompt to send to the agent
    pub prompt: Option<String>,
}
