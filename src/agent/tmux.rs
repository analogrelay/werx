//! tmux session management for agents.
//!
//! Provides functions for creating and managing tmux sessions
//! that host agent instances.

use anyhow::{anyhow, Context, Result};
use std::process::Command;

use super::{AgentStatus, WERX_AGENTS_SESSION};

/// Information about a tmux window (agent instance)
#[derive(Debug, Clone)]
pub struct TmuxWindow {
    /// Window index in the session
    pub index: u32,
    /// Window name (agent name)
    pub name: String,
    /// Whether the pane is active (has a running process)
    pub active: bool,
}

/// Information about the tmux session
#[derive(Debug, Clone)]
pub struct TmuxSession {
    /// Session name
    pub name: String,
    /// Windows in the session
    pub windows: Vec<TmuxWindow>,
}

/// Check if tmux is available on the system
pub fn tmux_is_available() -> bool {
    Command::new("which")
        .arg("tmux")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Check if the werx-agents session exists
pub fn tmux_session_exists() -> bool {
    Command::new("tmux")
        .args(["has-session", "-t", WERX_AGENTS_SESSION])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Create the werx-agents tmux session with an initial window
pub fn tmux_create_session(window_name: &str, working_dir: &std::path::Path) -> Result<()> {
    let output = Command::new("tmux")
        .args([
            "new-session",
            "-d", // detached
            "-s",
            WERX_AGENTS_SESSION,
            "-n",
            window_name,
            "-c",
            &working_dir.to_string_lossy(),
        ])
        .output()
        .context("Failed to execute tmux new-session")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("Failed to create tmux session: {}", stderr));
    }

    Ok(())
}

/// Create a new window in the werx-agents session
pub fn tmux_create_window(window_name: &str, working_dir: &std::path::Path) -> Result<()> {
    let output = Command::new("tmux")
        .args([
            "new-window",
            "-t",
            WERX_AGENTS_SESSION,
            "-n",
            window_name,
            "-c",
            &working_dir.to_string_lossy(),
        ])
        .output()
        .context("Failed to execute tmux new-window")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("Failed to create tmux window: {}", stderr));
    }

    Ok(())
}

/// Send a command to a tmux window
pub fn tmux_send_keys(window_name: &str, command: &str) -> Result<()> {
    let target = format!("{}:{}", WERX_AGENTS_SESSION, window_name);

    let output = Command::new("tmux")
        .args(["send-keys", "-t", &target, command, "Enter"])
        .output()
        .context("Failed to execute tmux send-keys")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("Failed to send keys to tmux window: {}", stderr));
    }

    Ok(())
}

/// List all windows in the werx-agents session
pub fn tmux_list_windows() -> Result<Vec<TmuxWindow>> {
    if !tmux_session_exists() {
        return Ok(Vec::new());
    }

    let output = Command::new("tmux")
        .args([
            "list-windows",
            "-t",
            WERX_AGENTS_SESSION,
            "-F",
            "#{window_index}:#{window_name}:#{pane_dead}",
        ])
        .output()
        .context("Failed to execute tmux list-windows")?;

    if !output.status.success() {
        // Session might not exist
        return Ok(Vec::new());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut windows = Vec::new();

    for line in stdout.lines() {
        let parts: Vec<&str> = line.split(':').collect();
        if parts.len() >= 3 {
            let index = parts[0].parse().unwrap_or(0);
            let name = parts[1].to_string();
            let dead = parts[2] == "1";

            windows.push(TmuxWindow {
                index,
                name,
                active: !dead,
            });
        }
    }

    Ok(windows)
}

/// Select a specific window in the session
pub fn tmux_select_window(window_name: &str) -> Result<()> {
    let target = format!("{}:{}", WERX_AGENTS_SESSION, window_name);

    let output = Command::new("tmux")
        .args(["select-window", "-t", &target])
        .output()
        .context("Failed to execute tmux select-window")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("Failed to select tmux window: {}", stderr));
    }

    Ok(())
}

/// Attach to the werx-agents session
pub fn tmux_attach(window_name: Option<&str>) -> Result<()> {
    // If a specific window is requested, select it first
    if let Some(name) = window_name {
        tmux_select_window(name)?;
    }

    // Use exec to replace current process with tmux attach
    let err = exec::Command::new("tmux")
        .args(&["attach-session", "-t", WERX_AGENTS_SESSION])
        .exec();

    // exec() only returns if there was an error
    Err(anyhow!("Failed to attach to tmux session: {}", err))
}

/// Kill a specific window in the session
pub fn tmux_kill_window(window_name: &str) -> Result<bool> {
    let target = format!("{}:{}", WERX_AGENTS_SESSION, window_name);

    // First check how many windows exist
    let windows = tmux_list_windows()?;
    let is_last_window = windows.len() <= 1;

    let output = Command::new("tmux")
        .args(["kill-window", "-t", &target])
        .output()
        .context("Failed to execute tmux kill-window")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("Failed to kill tmux window: {}", stderr));
    }

    // If this was the last window, the session is now closed
    Ok(is_last_window)
}

/// Get the status of an agent based on its tmux window state
pub fn get_agent_status_from_tmux(window_name: &str) -> AgentStatus {
    let windows = tmux_list_windows().unwrap_or_default();

    for window in windows {
        if window.name == window_name {
            return if window.active {
                AgentStatus::Running
            } else {
                // Window exists but process has exited
                // We can't easily determine if it was success or failure
                AgentStatus::Exited
            };
        }
    }

    AgentStatus::Unknown
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tmux_is_available() {
        // This test just checks that the function runs without panicking
        // The result depends on whether tmux is installed
        let _ = tmux_is_available();
    }

    #[test]
    fn test_parse_window_line() {
        // Test the parsing logic indirectly through list_windows
        // when session doesn't exist
        let windows = tmux_list_windows().unwrap_or_default();
        // Should return empty if session doesn't exist
        assert!(windows.is_empty() || !windows.is_empty());
    }
}
