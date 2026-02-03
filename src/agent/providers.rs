//! Agent provider detection and configuration.
//!
//! Detects available coding agents by checking for executables in PATH.

use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::process::Command;

use super::AgentType;

/// Information about an available agent provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentProvider {
    /// The type of agent
    pub agent_type: AgentType,
    /// The command to execute
    pub command: String,
    /// Additional arguments for the command
    #[serde(default)]
    pub args: Vec<String>,
    /// Whether this provider is available on the system
    pub available: bool,
    /// Path to the executable (if found)
    pub path: Option<String>,
}

impl AgentProvider {
    /// Create a new provider with specified command
    fn new(agent_type: AgentType, command: String, args: Vec<String>) -> Self {
        Self {
            agent_type,
            command,
            args,
            available: false,
            path: None,
        }
    }

    /// Create a provider with default settings for the given type
    fn default_for(agent_type: AgentType) -> Self {
        let (command, args) = match agent_type {
            AgentType::OpenCode => ("opencode".to_string(), vec![]),
            AgentType::Claude => ("claude".to_string(), vec![]),
            AgentType::Copilot => ("copilot".to_string(), vec![]),
        };
        Self::new(agent_type, command, args)
    }

    /// Get the full command with arguments as a string
    pub fn full_command(&self) -> String {
        if self.args.is_empty() {
            self.command.clone()
        } else {
            format!("{} {}", self.command, self.args.join(" "))
        }
    }

    /// Build the command to execute this agent in a given directory
    pub fn build_command(&self, working_dir: &std::path::Path, prompt: Option<&str>) -> Command {
        let mut cmd = Command::new(&self.command);
        cmd.current_dir(working_dir);

        // Add base args
        for arg in &self.args {
            cmd.arg(arg);
        }

        // Handle prompt based on agent type
        if let Some(prompt_text) = prompt {
            match self.agent_type {
                AgentType::OpenCode => {
                    // OpenCode accepts prompt via --prompt flag
                    cmd.arg("--prompt").arg(prompt_text);
                }
                AgentType::Claude => {
                    // Claude Code accepts prompt via --prompt flag
                    cmd.arg("--prompt").arg(prompt_text);
                }
                AgentType::Copilot => {
                    // GitHub Copilot CLI - prompt goes to the subcommand
                    // For now, just add it as an argument
                    cmd.arg(prompt_text);
                }
            }
        }

        cmd
    }
}

/// Detect all available agent providers on the system
pub fn detect_providers() -> Vec<AgentProvider> {
    let mut providers = Vec::new();

    // Check OpenCode
    let mut opencode = AgentProvider::default_for(AgentType::OpenCode);
    if let Some(path) = find_executable("opencode") {
        opencode.available = true;
        opencode.path = Some(path);
    }
    providers.push(opencode);

    // Check Claude Code
    let mut claude = AgentProvider::default_for(AgentType::Claude);
    if let Some(path) = find_executable("claude") {
        claude.available = true;
        claude.path = Some(path);
    }
    providers.push(claude);

    // Check GitHub Copilot CLI - prefer standalone `copilot` over `gh copilot`
    let copilot = if let Some(path) = find_executable("copilot") {
        // Standalone copilot CLI found
        let mut provider = AgentProvider::default_for(AgentType::Copilot);
        provider.available = true;
        provider.path = Some(path);
        provider
    } else if let Some(path) = find_executable("gh") {
        // Check if gh copilot extension is installed
        let mut provider = AgentProvider::new(
            AgentType::Copilot,
            "gh".to_string(),
            vec!["copilot".to_string()],
        );
        if check_gh_copilot_extension() {
            provider.available = true;
            provider.path = Some(path);
        }
        provider
    } else {
        // Neither found, return default (unavailable)
        AgentProvider::default_for(AgentType::Copilot)
    };
    providers.push(copilot);

    providers
}

/// Get the default provider based on availability and preferences
pub fn get_default_provider(
    providers: &[AgentProvider],
    preferred: Option<AgentType>,
) -> Result<&AgentProvider> {
    // If a specific type is preferred, try that first
    if let Some(pref_type) = preferred {
        if let Some(provider) = providers
            .iter()
            .find(|p| p.agent_type == pref_type && p.available)
        {
            return Ok(provider);
        }
        // If preferred type isn't available, return an error
        return Err(anyhow!(
            "Requested agent '{}' is not available. Run 'werx agent providers' to see available agents.",
            pref_type.id()
        ));
    }

    // Default preference order: OpenCode > Claude > Copilot
    let preference_order = [AgentType::OpenCode, AgentType::Claude, AgentType::Copilot];

    for agent_type in preference_order {
        if let Some(provider) = providers
            .iter()
            .find(|p| p.agent_type == agent_type && p.available)
        {
            return Ok(provider);
        }
    }

    Err(anyhow!(
        "No coding agents are available.\n\n\
         Install one of the following:\n  \
         - OpenCode: https://opencode.ai\n  \
         - Claude Code: https://claude.ai/code\n  \
         - GitHub Copilot CLI: gh extension install github/gh-copilot"
    ))
}

/// Find an executable in PATH and return its full path
fn find_executable(name: &str) -> Option<String> {
    let output = Command::new("which").arg(name).output().ok()?;

    if output.status.success() {
        let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !path.is_empty() {
            return Some(path);
        }
    }

    None
}

/// Check if the GitHub Copilot extension is installed
fn check_gh_copilot_extension() -> bool {
    let output = Command::new("gh").args(["extension", "list"]).output();

    match output {
        Ok(output) if output.status.success() => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            stdout.contains("copilot") || stdout.contains("gh-copilot")
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_provider_default_for() {
        let provider = AgentProvider::default_for(AgentType::OpenCode);
        assert_eq!(provider.command, "opencode");
        assert!(provider.args.is_empty());
        assert!(!provider.available);
    }

    #[test]
    fn test_agent_provider_copilot_default() {
        // Default copilot uses standalone CLI
        let provider = AgentProvider::default_for(AgentType::Copilot);
        assert_eq!(provider.command, "copilot");
        assert!(provider.args.is_empty());
    }

    #[test]
    fn test_agent_provider_gh_copilot() {
        // gh copilot variant
        let provider = AgentProvider::new(
            AgentType::Copilot,
            "gh".to_string(),
            vec!["copilot".to_string()],
        );
        assert_eq!(provider.command, "gh");
        assert_eq!(provider.args, vec!["copilot"]);
        assert_eq!(provider.full_command(), "gh copilot");
    }

    #[test]
    fn test_full_command() {
        let provider = AgentProvider::new(
            AgentType::Copilot,
            "gh".to_string(),
            vec!["copilot".to_string()],
        );
        assert_eq!(provider.full_command(), "gh copilot");

        let provider = AgentProvider::default_for(AgentType::OpenCode);
        assert_eq!(provider.full_command(), "opencode");

        let provider = AgentProvider::default_for(AgentType::Copilot);
        assert_eq!(provider.full_command(), "copilot");
    }

    #[test]
    fn test_detect_providers_returns_all_types() {
        let providers = detect_providers();
        assert_eq!(providers.len(), 3);

        assert!(
            providers
                .iter()
                .any(|p| p.agent_type == AgentType::OpenCode)
        );
        assert!(providers.iter().any(|p| p.agent_type == AgentType::Claude));
        assert!(providers.iter().any(|p| p.agent_type == AgentType::Copilot));
    }
}
