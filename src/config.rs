use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Protocol preference for Git clone operations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Protocol {
    Ssh,
    Https,
}

impl Protocol {
    pub fn as_str(&self) -> &'static str {
        match self {
            Protocol::Ssh => "ssh",
            Protocol::Https => "https",
        }
    }
}

impl std::fmt::Display for Protocol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::str::FromStr for Protocol {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "ssh" => Ok(Protocol::Ssh),
            "https" => Ok(Protocol::Https),
            _ => Err(anyhow::anyhow!("Invalid protocol: {}", s)),
        }
    }
}

/// Provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    /// Default Git hosting provider for shorthand repository specifications
    #[serde(default = "default_provider")]
    pub default: String,

    /// Protocol preference for Git clone operations (SSH or HTTPS)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub protocol: Option<Protocol>,
}

fn default_provider() -> String {
    "github".to_string()
}

impl Default for ProviderConfig {
    fn default() -> Self {
        ProviderConfig {
            default: default_provider(),
            protocol: None,
        }
    }
}

/// Configuration for a specific agent provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentProviderConfig {
    /// The command to run for this agent
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,

    /// Additional arguments to pass to the agent
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub args: Vec<String>,
}

impl Default for AgentProviderConfig {
    fn default() -> Self {
        AgentProviderConfig {
            command: None,
            args: Vec::new(),
        }
    }
}

/// Per-repository agent preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoAgentConfig {
    /// Preferred agent for this repository
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preferred_agent: Option<String>,
}

impl Default for RepoAgentConfig {
    fn default() -> Self {
        RepoAgentConfig {
            preferred_agent: None,
        }
    }
}

/// Agent configuration section
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AgentConfig {
    /// Default agent to use when none is specified
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<String>,

    /// Per-provider configuration overrides
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub providers: HashMap<String, AgentProviderConfig>,

    /// Per-repository preferences (keyed by normalized URL like "github.com/owner/repo")
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub repos: HashMap<String, RepoAgentConfig>,
}

/// Werx configuration stored in .werx/config.toml
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Provider settings
    #[serde(default)]
    pub provider: ProviderConfig,

    /// Agent settings
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agents: Option<AgentConfig>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            provider: ProviderConfig::default(),
            agents: None,
        }
    }
}

impl Config {
    /// Load config from a file, or return default if file doesn't exist
    pub fn load(path: &Path) -> Result<Self> {
        if !path.exists() {
            return Ok(Config::default());
        }

        let contents = fs::read_to_string(path)
            .context(format!("Failed to read config file '{}'", path.display()))?;

        let config: Config = toml::from_str(&contents)
            .context(format!("Failed to parse config file '{}'", path.display()))?;

        Ok(config)
    }

    /// Save config to a file
    pub fn save(&self, path: &Path) -> Result<()> {
        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent).context(format!(
                    "Failed to create config directory '{}'",
                    parent.display()
                ))?;
            }
        }

        let contents = toml::to_string_pretty(self).context("Failed to serialize config")?;

        fs::write(path, contents)
            .context(format!("Failed to write config file '{}'", path.display()))?;

        Ok(())
    }

    /// Set the protocol preference
    pub fn set_protocol(&mut self, protocol: Protocol) {
        self.provider.protocol = Some(protocol);
    }

    /// Get the protocol preference if set
    pub fn protocol(&self) -> Option<Protocol> {
        self.provider.protocol
    }

    /// Get the default provider
    pub fn default_provider(&self) -> &str {
        &self.provider.default
    }

    /// Get the default agent type name (e.g., "opencode", "claude")
    pub fn default_agent(&self) -> Option<&str> {
        self.agents.as_ref().and_then(|a| a.default.as_deref())
    }

    /// Get the preferred agent for a specific repository
    /// The repo_key should be in the format "github.com/owner/repo"
    pub fn preferred_agent_for_repo(&self, repo_key: &str) -> Option<&str> {
        self.agents
            .as_ref()
            .and_then(|a| a.repos.get(repo_key))
            .and_then(|r| r.preferred_agent.as_deref())
    }

    /// Get custom provider configuration for an agent type
    pub fn agent_provider_config(&self, agent_type: &str) -> Option<&AgentProviderConfig> {
        self.agents
            .as_ref()
            .and_then(|a| a.providers.get(agent_type))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.default_provider(), "github");
        assert_eq!(config.protocol(), None);
    }

    #[test]
    fn test_save_and_load_config() {
        let temp = TempDir::new().unwrap();
        let config_path = temp.path().join("config.toml");

        let mut config = Config::default();
        config.set_protocol(Protocol::Ssh);
        config.save(&config_path).expect("Should save config");

        let loaded = Config::load(&config_path).expect("Should load config");
        assert_eq!(loaded.default_provider(), "github");
        assert_eq!(loaded.protocol(), Some(Protocol::Ssh));
    }

    #[test]
    fn test_load_nonexistent_config() {
        let temp = TempDir::new().unwrap();
        let config_path = temp.path().join("nonexistent");

        let config = Config::load(&config_path).expect("Should return default");
        assert_eq!(config.default_provider(), "github");
        assert_eq!(config.protocol(), None);
    }

    #[test]
    fn test_save_config_creates_parent_directory() {
        let temp = TempDir::new().unwrap();
        let config_path = temp.path().join("subdir").join("config.toml");

        let config = Config::default();
        config.save(&config_path).expect("Should save config");

        assert!(config_path.exists());
    }

    #[test]
    fn test_protocol_serialization() {
        let temp = TempDir::new().unwrap();
        let config_path = temp.path().join("config.toml");

        let mut config = Config::default();
        config.set_protocol(Protocol::Https);
        config.save(&config_path).expect("Should save");

        let contents = fs::read_to_string(&config_path).unwrap();
        assert!(contents.contains("[provider]"));
        assert!(contents.contains("protocol = \"https\""));
    }

    #[test]
    fn test_protocol_from_str() {
        assert_eq!("ssh".parse::<Protocol>().unwrap(), Protocol::Ssh);
        assert_eq!("https".parse::<Protocol>().unwrap(), Protocol::Https);
        assert_eq!("SSH".parse::<Protocol>().unwrap(), Protocol::Ssh);
        assert_eq!("HTTPS".parse::<Protocol>().unwrap(), Protocol::Https);
        assert!("invalid".parse::<Protocol>().is_err());
    }

    #[test]
    fn test_protocol_display() {
        assert_eq!(Protocol::Ssh.to_string(), "ssh");
        assert_eq!(Protocol::Https.to_string(), "https");
    }
}
