use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;

use crate::Forge;
use crate::validation::validate_forge_path;

/// Initialize a Forge at the specified path
///
/// Creates the directory structure:
/// - <root>/                     (Forge root, for workspaces)
/// - <root>/.forge/              (Internal directory)
/// - <root>/.forge/repos/        (Repository storage)
/// - <root>/.forge/marker        (Marker file)
///
/// If force is true, will reinitialize an existing Forge
pub fn initialize_forge(path: PathBuf, force: bool) -> Result<Forge> {
    // Validate the path
    validate_forge_path(&path, force).context("Path validation failed")?;

    // Create the root directory if it doesn't exist
    if !path.exists() {
        fs::create_dir_all(&path)
            .context(format!("Failed to create directory '{}'", path.display()))?;
    }

    // Create forge structure
    let forge = Forge { root: path };

    // Create .forge directory
    create_directory(&forge.forge_dir(), ".forge")?;

    // Create repos subdirectory inside .forge
    create_directory(&forge.repos_dir(), "repos")?;

    // Create marker file inside .forge
    fs::write(forge.marker_file(), b"").context(format!(
        "Failed to create Forge marker file '{}'",
        forge.marker_file().display()
    ))?;

    Ok(forge)
}

/// Helper to create a directory with context
fn create_directory(path: &PathBuf, name: &str) -> Result<()> {
    if !path.exists() {
        fs::create_dir_all(path).context(format!(
            "Failed to create {} directory '{}'",
            name,
            path.display()
        ))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::FORGE_MARKER;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_initialize_forge_creates_structure() {
        let temp = TempDir::new().unwrap();
        let forge_path = temp.path().join("test-forge");

        let forge = initialize_forge(forge_path.clone(), false).expect("Should initialize forge");

        assert_eq!(forge.root, forge_path);
        assert!(forge_path.exists());
        assert!(forge.forge_dir().exists());
        assert!(forge.repos_dir().exists());
        assert!(forge.marker_file().exists());
    }

    #[test]
    fn test_initialize_forge_in_existing_empty_directory() {
        let temp = TempDir::new().unwrap();

        let forge =
            initialize_forge(temp.path().to_path_buf(), false).expect("Should initialize forge");

        assert!(forge.forge_dir().exists());
        assert!(forge.repos_dir().exists());
        assert!(forge.marker_file().exists());
    }

    #[test]
    fn test_initialize_forge_creates_parent_directories() {
        let temp = TempDir::new().unwrap();
        let forge_path = temp.path().join("parent").join("child").join("forge");

        let forge = initialize_forge(forge_path.clone(), false).expect("Should initialize forge");

        assert!(forge_path.exists());
        assert!(forge.forge_dir().exists());
        assert!(forge.repos_dir().exists());
        assert!(forge.marker_file().exists());
    }

    #[test]
    fn test_initialize_forge_fails_without_force() {
        let temp = TempDir::new().unwrap();
        let forge_dir = temp.path().join(".forge");
        fs::create_dir(&forge_dir).unwrap();
        let marker = forge_dir.join(FORGE_MARKER);
        fs::write(&marker, b"").unwrap();

        let result = initialize_forge(temp.path().to_path_buf(), false);
        assert!(result.is_err());
    }

    #[test]
    fn test_initialize_forge_succeeds_with_force() {
        let temp = TempDir::new().unwrap();
        let forge_dir = temp.path().join(".forge");
        fs::create_dir(&forge_dir).unwrap();
        let marker = forge_dir.join(FORGE_MARKER);
        fs::write(&marker, b"").unwrap();

        let result = initialize_forge(temp.path().to_path_buf(), true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_initialize_forge_preserves_existing_content_with_force() {
        let temp = TempDir::new().unwrap();
        let existing_file = temp.path().join("existing.txt");
        fs::write(&existing_file, b"important data").unwrap();

        let forge =
            initialize_forge(temp.path().to_path_buf(), true).expect("Should initialize forge");

        assert!(existing_file.exists());
        assert_eq!(fs::read(&existing_file).unwrap(), b"important data");
        assert!(forge.marker_file().exists());
    }
}
