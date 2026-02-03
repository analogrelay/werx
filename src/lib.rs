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

pub use agent::{
    attach_to_agent, detect_providers, find_agent, get_agent_status, get_default_provider,
    kill_agent, list_agents, spawn_agent, Agent, AgentProvider, AgentStatus, AgentType,
    SpawnOptions, SpawnResult,
};
pub use config::{Config, Protocol};
pub use directive::emit_change_directory;
pub use init::initialize_werx;
pub use path::resolve_werx_path;
pub use repo_spec::RepoSpec;
pub use repos::{add_repo, create_repo, list_repos, remove_repo, CreatedRepoInfo, RepoInfo};
pub use shell::cmd_shell_init;
pub use validation::validate_werx_path;
pub use workspace::{
    check_workspace_status, confirm_workspace_removal, create_worktree, detect_current_workspace,
    find_repository, fuzzy_select_repository, generate_workspace_path,
    get_workspace_status_details, list_workspaces, prompt_branch_name, prompt_workspace_name,
    remove_workspace, select_repository, select_workspace_with_query, Workspace, WorkspaceStatus,
    WorkspaceStatusDetails,
};

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
