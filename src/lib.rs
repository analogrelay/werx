pub mod config;
pub mod directive;
pub mod init;
pub mod path;
pub mod repo_spec;
pub mod repos;
pub mod shell;
pub mod validation;
pub mod workspace;

use std::path::PathBuf;

pub use config::{Config, Protocol};
pub use repo_spec::RepoSpec;
pub use repos::{CreatedRepoInfo, RepoInfo};
pub use workspace::{Workspace, WorkspaceStatus};

/// Internal directory that contains all Forge metadata and repositories
pub const FORGE_DIR: &str = ".forge";

/// Subdirectory within .forge/ for storing repository clones
pub const REPOS_SUBDIR: &str = "repos";

/// Configuration file within .forge/ that stores settings and acts as Forge marker
pub const FORGE_CONFIG: &str = "config.toml";

/// Result type for Forge operations
pub type Result<T> = anyhow::Result<T>;

/// Represents a Forge location
#[derive(Debug, Clone)]
pub struct Forge {
    pub root: PathBuf,
}

impl Forge {
    /// Check if a directory is a Forge
    pub fn exists_at(path: &std::path::Path) -> bool {
        let forge_dir = path.join(FORGE_DIR);
        let config = forge_dir.join(FORGE_CONFIG);
        forge_dir.exists() && config.exists()
    }

    /// Get the .forge directory
    pub fn forge_dir(&self) -> PathBuf {
        self.root.join(FORGE_DIR)
    }

    /// Get the repos directory (inside .forge/)
    pub fn repos_dir(&self) -> PathBuf {
        self.forge_dir().join(REPOS_SUBDIR)
    }

    /// Get the config file path (inside .forge/)
    pub fn config_file(&self) -> PathBuf {
        self.forge_dir().join(FORGE_CONFIG)
    }

    /// Load the Forge configuration
    pub fn load_config(&self) -> Result<Config> {
        Config::load(&self.config_file())
    }

    /// Save the Forge configuration
    pub fn save_config(&self, config: &Config) -> Result<()> {
        config.save(&self.config_file())
    }
}
