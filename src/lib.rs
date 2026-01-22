pub mod init;
pub mod path;
pub mod validation;

use std::path::PathBuf;

/// Internal directory that contains all Forge metadata and repositories
pub const FORGE_DIR: &str = ".forge";

/// Subdirectory within .forge/ for storing repository clones
pub const REPOS_SUBDIR: &str = "repos";

/// Marker file within .forge/ that indicates a directory is a Forge
pub const FORGE_MARKER: &str = "marker";

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
        let marker = forge_dir.join(FORGE_MARKER);
        forge_dir.exists() && marker.exists()
    }

    /// Get the .forge directory
    pub fn forge_dir(&self) -> PathBuf {
        self.root.join(FORGE_DIR)
    }

    /// Get the repos directory (inside .forge/)
    pub fn repos_dir(&self) -> PathBuf {
        self.forge_dir().join(REPOS_SUBDIR)
    }

    /// Get the marker file path (inside .forge/)
    pub fn marker_file(&self) -> PathBuf {
        self.forge_dir().join(FORGE_MARKER)
    }
}
