use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
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

/// Sync configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SyncConfig {
    /// Remotes to fetch from during sync (defaults to ["origin", "upstream"] if absent)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remotes: Option<Vec<String>>,
}

static DEFAULT_SYNC_REMOTES: std::sync::LazyLock<Vec<String>> = std::sync::LazyLock::new(|| {
    vec!["origin".to_string(), "upstream".to_string()]
});

/// Werx configuration stored in .werx/config.toml
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Config {
    /// Provider settings
    #[serde(default)]
    pub provider: ProviderConfig,

    /// Sync settings
    #[serde(default)]
    pub sync: SyncConfig,
}

impl Config {
    /// Load config from a file, or return default if file doesn't exist
    pub fn load(path: &Path) -> Result<Self> {
        if !path.exists() {
            tracing::debug!("Config file not found at '{}', using defaults", path.display());
            return Ok(Config::default());
        }

        tracing::debug!("Loading config from '{}'", path.display());
        let contents = fs::read_to_string(path)
            .context(format!("Failed to read config file '{}'", path.display()))?;

        let config: Config = toml::from_str(&contents)
            .context(format!("Failed to parse config file '{}'", path.display()))?;

        tracing::debug!("Config loaded: provider={}, protocol={:?}", config.default_provider(), config.protocol());
        Ok(config)
    }

    /// Save config to a file
    pub fn save(&self, path: &Path) -> Result<()> {
        // Ensure parent directory exists
        if let Some(parent) = path.parent()
            && !parent.exists()
        {
            fs::create_dir_all(parent).context(format!(
                "Failed to create config directory '{}'",
                parent.display()
            ))?;
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

    /// Get the list of remotes to fetch from during sync.
    /// Returns the configured list or the default ["origin", "upstream"].
    pub fn sync_remotes(&self) -> &[String] {
        self.sync
            .remotes
            .as_deref()
            .unwrap_or(&DEFAULT_SYNC_REMOTES)
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

    // ===== SyncConfig / sync_remotes tests (task 3.4) =====

    #[test]
    fn test_sync_remotes_default_when_absent() {
        let config = Config::default();
        let remotes = config.sync_remotes();
        assert_eq!(remotes, &["origin", "upstream"]);
    }

    #[test]
    fn test_sync_remotes_from_config() {
        let toml = r#"
[sync]
remotes = ["origin", "myupstream"]
"#;
        let config: Config = toml::from_str(toml).unwrap();
        assert_eq!(config.sync_remotes(), &["origin", "myupstream"]);
    }

    #[test]
    fn test_sync_remotes_serialize_deserialize() {
        let temp = TempDir::new().unwrap();
        let path = temp.path().join("config.toml");

        let mut config = Config::default();
        config.sync.remotes = Some(vec!["origin".to_string(), "fork".to_string()]);
        config.save(&path).unwrap();

        let loaded = Config::load(&path).unwrap();
        assert_eq!(loaded.sync_remotes(), &["origin", "fork"]);

        let contents = fs::read_to_string(&path).unwrap();
        assert!(contents.contains("[sync]"));
        assert!(contents.contains("remotes"));
    }

    #[test]
    fn test_sync_remotes_absent_key_yields_defaults() {
        let toml = r#"
[provider]
default = "github"
"#;
        let config: Config = toml::from_str(toml).unwrap();
        assert_eq!(config.sync_remotes(), &["origin", "upstream"]);
    }
}
