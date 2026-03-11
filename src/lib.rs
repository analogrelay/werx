pub mod branch_naming;
pub mod cmd;
pub mod config;
pub mod directive;
pub mod github;
pub mod init;
pub mod path;
pub mod repo_meta;
pub mod repo_spec;
pub mod reporter;
pub mod repos;
pub mod shell;
pub mod sync;
pub mod trash;
pub mod validation;
pub mod workspace;

use std::path::PathBuf;

pub use config::{AgentConfig, Config, GithubConfig, Protocol, SyncConfig, config_get_value, config_set_value, config_delete_value};
pub use reporter::Reporter;

/// Application-wide context threaded through commands.
pub struct AppContext {
    pub verbose: bool,
    pub reporter: reporter::Reporter,
}

impl AppContext {
    pub fn new(verbose: bool) -> Self {
        Self {
            verbose,
            reporter: reporter::Reporter::new(verbose),
        }
    }
}
pub use repo_meta::RepoGithubMeta;
pub use sync::run_sync;
pub use directive::emit_change_directory;
pub use init::initialize_werx;
pub use path::resolve_werx_path;
pub use repo_spec::RepoSpec;
pub use repos::{CreatedRepoInfo, RepoInfo, add_repo, create_repo, list_repos, remove_repo};
pub use shell::cmd_shell_init;
pub use validation::validate_werx_path;
pub use workspace::{
    Workspace, WorkspaceStatus, WorkspaceStatusDetails, check_workspace_status,
    confirm_workspace_removal, create_worktree, detect_current_workspace, find_repository,
    fuzzy_select_repository, generate_workspace_path, get_workspace_status_details,
    list_workspaces, prompt_branch_name, prompt_workspace_name, remove_workspace,
    select_repository, select_workspace_with_query,
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
