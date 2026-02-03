pub mod agent;
pub mod config;
pub mod directive;
pub mod init;
pub mod namedata;
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

/// Internal directory that contains all Werx metadata and repositories
pub const WERX_DIR: &str = ".werx";

/// Subdirectory within .werx/ for storing repository clones
pub const REPOS_SUBDIR: &str = "repos";

/// Configuration file within .werx/ that stores settings and acts as Werx marker
pub const WERX_CONFIG: &str = "config.toml";

/// Result type for Werx operations
pub type Result<T> = anyhow::Result<T>;

/// Represents a Werx location
#[derive(Debug, Clone)]
pub struct Werx {
    pub root: PathBuf,
}

impl Werx {
    /// Check if a directory is a Werx
    pub fn exists_at(path: &std::path::Path) -> bool {
        let werx_dir = path.join(WERX_DIR);
        let config = werx_dir.join(WERX_CONFIG);
        werx_dir.exists() && config.exists()
    }

    /// Get the .werx directory
    pub fn werx_dir(&self) -> PathBuf {
        self.root.join(WERX_DIR)
    }

    /// Get the repos directory (inside .werx/)
    pub fn repos_dir(&self) -> PathBuf {
        self.werx_dir().join(REPOS_SUBDIR)
    }

    /// Get the config file path (inside .werx/)
    pub fn config_file(&self) -> PathBuf {
        self.werx_dir().join(WERX_CONFIG)
    }

    /// Load the Werx configuration
    pub fn load_config(&self) -> Result<Config> {
        Config::load(&self.config_file())
    }

    /// Save the Werx configuration
    pub fn save_config(&self, config: &Config) -> Result<()> {
        config.save(&self.config_file())
    }
}
