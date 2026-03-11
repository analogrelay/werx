use anyhow::{Context, Result, anyhow};
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

/// GitHub configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GithubConfig {
    /// Cached GitHub username for branch naming
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    /// Branch naming pattern (currently only "username/issue-topic" is supported)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub branch_pattern: Option<String>,
}

/// Agent configuration for AI-assisted branch slug generation
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AgentConfig {
    /// Agent to use for slug generation ("claude" or "copilot")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent: Option<String>,
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

    /// GitHub settings
    #[serde(default)]
    pub github: GithubConfig,

    /// Agent settings for AI-assisted branch naming
    #[serde(default)]
    pub agent: AgentConfig,
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

// ── Dotted-path config helpers ────────────────────────────────────────────────

/// Get a value from the config file at the given dotted path.
/// Returns the TOML value as a string, or None if the key doesn't exist.
pub fn config_get_value(config_path: &Path, key: &str) -> Result<Option<String>> {
    let raw = load_raw_toml(config_path)?;
    let parts: Vec<&str> = key.split('.').collect();
    let mut current = &raw;
    for part in &parts {
        match current {
            toml::Value::Table(t) => {
                if let Some(v) = t.get(*part) {
                    current = v;
                } else {
                    return Ok(None);
                }
            }
            _ => return Ok(None),
        }
    }
    Ok(Some(value_to_display_string(current)))
}

/// Set a value in the config file at the given dotted path.
pub fn config_set_value(config_path: &Path, key: &str, value: &str) -> Result<()> {
    let mut raw = load_raw_toml(config_path)?;
    let parts: Vec<&str> = key.split('.').collect();
    if parts.is_empty() {
        return Err(anyhow!("Key cannot be empty"));
    }
    let parsed_value = parse_toml_value(value);
    set_nested(&mut raw, &parts, parsed_value)?;
    save_raw_toml(config_path, &raw)?;
    // Validate round-trip
    Config::load(config_path)?;
    Ok(())
}

/// Delete a value from the config file at the given dotted path.
pub fn config_delete_value(config_path: &Path, key: &str) -> Result<bool> {
    let mut raw = load_raw_toml(config_path)?;
    let parts: Vec<&str> = key.split('.').collect();
    if parts.is_empty() {
        return Err(anyhow!("Key cannot be empty"));
    }
    let existed = delete_nested(&mut raw, &parts);
    if existed {
        save_raw_toml(config_path, &raw)?;
    }
    Ok(existed)
}

fn load_raw_toml(config_path: &Path) -> Result<toml::Value> {
    if !config_path.exists() {
        return Ok(toml::Value::Table(toml::map::Map::new()));
    }
    let contents = fs::read_to_string(config_path)
        .context(format!("Failed to read config file '{}'", config_path.display()))?;
    let value: toml::Value = toml::from_str(&contents)
        .context(format!("Failed to parse config file '{}'", config_path.display()))?;
    Ok(value)
}

fn save_raw_toml(config_path: &Path, value: &toml::Value) -> Result<()> {
    if let Some(parent) = config_path.parent()
        && !parent.exists()
    {
        fs::create_dir_all(parent)?;
    }
    let contents = toml::to_string_pretty(value).context("Failed to serialize config")?;
    fs::write(config_path, contents)
        .context(format!("Failed to write config file '{}'", config_path.display()))?;
    Ok(())
}

fn parse_toml_value(s: &str) -> toml::Value {
    match s {
        "true" => return toml::Value::Boolean(true),
        "false" => return toml::Value::Boolean(false),
        _ => {}
    }
    if let Ok(n) = s.parse::<i64>() {
        return toml::Value::Integer(n);
    }
    // Try TOML array syntax like ["a", "b"]
    if s.starts_with('[') {
        if let Ok(v) = toml::from_str::<toml::Value>(&format!("x = {}", s)) {
            if let toml::Value::Table(mut t) = v {
                if let Some(arr) = t.remove("x") {
                    return arr;
                }
            }
        }
    }
    toml::Value::String(s.to_string())
}

fn set_nested(root: &mut toml::Value, parts: &[&str], value: toml::Value) -> Result<()> {
    if parts.len() == 1 {
        if let toml::Value::Table(t) = root {
            t.insert(parts[0].to_string(), value);
            return Ok(());
        }
        return Err(anyhow!("Cannot set key on non-table value"));
    }
    if let toml::Value::Table(t) = root {
        let entry = t
            .entry(parts[0].to_string())
            .or_insert_with(|| toml::Value::Table(toml::map::Map::new()));
        set_nested(entry, &parts[1..], value)
    } else {
        Err(anyhow!("Cannot traverse non-table value at '{}'", parts[0]))
    }
}

fn delete_nested(root: &mut toml::Value, parts: &[&str]) -> bool {
    if parts.len() == 1 {
        if let toml::Value::Table(t) = root {
            return t.remove(parts[0]).is_some();
        }
        return false;
    }
    if let toml::Value::Table(t) = root {
        if let Some(child) = t.get_mut(parts[0]) {
            return delete_nested(child, &parts[1..]);
        }
    }
    false
}

fn value_to_display_string(value: &toml::Value) -> String {
    match value {
        toml::Value::String(s) => s.clone(),
        toml::Value::Integer(n) => n.to_string(),
        toml::Value::Float(f) => f.to_string(),
        toml::Value::Boolean(b) => b.to_string(),
        toml::Value::Array(_) | toml::Value::Table(_) | toml::Value::Datetime(_) => {
            toml::to_string_pretty(value).unwrap_or_else(|_| format!("{:?}", value))
        }
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

    // ===== GithubConfig / AgentConfig serialization tests (tasks 1.1-1.3) =====

    #[test]
    fn test_github_config_defaults() {
        let config = Config::default();
        assert!(config.github.username.is_none());
        assert!(config.github.branch_pattern.is_none());
        assert!(config.agent.agent.is_none());
    }

    #[test]
    fn test_github_config_round_trip() {
        let temp = TempDir::new().unwrap();
        let path = temp.path().join("config.toml");

        let mut config = Config::default();
        config.github.username = Some("alice".to_string());
        config.github.branch_pattern = Some("username/issue-topic".to_string());
        config.agent.agent = Some("claude".to_string());
        config.save(&path).unwrap();

        let loaded = Config::load(&path).unwrap();
        assert_eq!(loaded.github.username.as_deref(), Some("alice"));
        assert_eq!(loaded.github.branch_pattern.as_deref(), Some("username/issue-topic"));
        assert_eq!(loaded.agent.agent.as_deref(), Some("claude"));
    }

    #[test]
    fn test_github_config_toml_sections() {
        let temp = TempDir::new().unwrap();
        let path = temp.path().join("config.toml");

        let mut config = Config::default();
        config.github.username = Some("bob".to_string());
        config.agent.agent = Some("copilot".to_string());
        config.save(&path).unwrap();

        let contents = fs::read_to_string(&path).unwrap();
        assert!(contents.contains("[github]"), "expected [github] section");
        assert!(contents.contains("[agent]"), "expected [agent] section");
        assert!(contents.contains("username = \"bob\""));
        assert!(contents.contains("agent = \"copilot\""));
    }

    #[test]
    fn test_config_without_github_sections_is_valid() {
        let toml = r#"
[provider]
default = "github"
"#;
        let config: Config = toml::from_str(toml).unwrap();
        assert!(config.github.username.is_none());
        assert!(config.agent.agent.is_none());
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
