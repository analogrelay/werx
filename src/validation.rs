use anyhow::{Context, Result, anyhow};
use std::fs;
use std::path::PathBuf;

use crate::Forge;

/// Validate that a path is suitable for a Forge
pub fn validate_forge_path(path: &PathBuf, force: bool) -> Result<()> {
    // Check if path exists
    if path.exists() {
        // Check if it's a regular file
        if path.is_file() {
            return Err(anyhow!(
                "Path '{}' is a regular file, not a directory",
                path.display()
            ));
        }

        // Check if it's already a Forge
        if Forge::exists_at(path) && !force {
            return Err(anyhow!(
                "A Forge already exists at '{}'. Use --force to re-initialize.",
                path.display()
            ));
        }

        // Check if directory is not empty (and not a Forge being forced)
        if !force && is_non_empty_directory(path)? {
            return Err(anyhow!(
                "Directory '{}' is not empty. Use --force to initialize anyway.",
                path.display()
            ));
        }
    } else {
        // Path doesn't exist - check if we can create it
        // Validate parent directory can be created
        if let Some(parent) = path.parent()
            && !parent.exists()
        {
            // Parent doesn't exist - we'll need to create it
            // Just check that the path is valid
            validate_path_is_creatable(parent)?;
        }
    }

    Ok(())
}

/// Check if a directory is non-empty
fn is_non_empty_directory(path: &PathBuf) -> Result<bool> {
    if !path.is_dir() {
        return Ok(false);
    }

    let entries =
        fs::read_dir(path).context(format!("Failed to read directory '{}'", path.display()))?;

    Ok(entries.count() > 0)
}

/// Validate that a path can be created
fn validate_path_is_creatable(path: &std::path::Path) -> Result<()> {
    // Check if any parent exists
    let mut current = path.to_path_buf();
    loop {
        if current.exists() {
            // Found an existing parent - check if it's writable
            if current.is_file() {
                return Err(anyhow!(
                    "Cannot create directory: '{}' is a file",
                    current.display()
                ));
            }

            // Check write permission by attempting to create and remove a temp file
            // This is a bit crude but works across platforms
            let test_file = current.join(".forge_permission_test");
            fs::write(&test_file, b"test").context(format!(
                "Permission denied: cannot write to '{}'",
                current.display()
            ))?;
            let _ = fs::remove_file(&test_file);

            return Ok(());
        }

        // Move up to parent
        match current.parent() {
            Some(parent) => current = parent.to_path_buf(),
            None => {
                // Reached root without finding existing directory
                return Err(anyhow!(
                    "Cannot create directory: no parent directory exists for '{}'",
                    path.display()
                ));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_validate_new_directory() {
        let temp = TempDir::new().unwrap();
        let forge_path = temp.path().join("new-forge");

        let result = validate_forge_path(&forge_path, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_empty_existing_directory() {
        let temp = TempDir::new().unwrap();

        let result = validate_forge_path(&temp.path().to_path_buf(), false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_reject_regular_file() {
        let temp = TempDir::new().unwrap();
        let file_path = temp.path().join("file.txt");
        fs::write(&file_path, b"test").unwrap();

        let result = validate_forge_path(&file_path, false);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("regular file"));
    }

    #[test]
    fn test_reject_existing_forge_without_force() {
        let temp = TempDir::new().unwrap();
        let forge_dir = temp.path().join(".forge");
        fs::create_dir(&forge_dir).unwrap();
        let config = forge_dir.join("config.toml");
        let cfg = crate::Config::default();
        cfg.save(&config).unwrap();

        let result = validate_forge_path(&temp.path().to_path_buf(), false);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("already exists"));
    }

    #[test]
    fn test_allow_existing_forge_with_force() {
        let temp = TempDir::new().unwrap();
        let forge_dir = temp.path().join(".forge");
        fs::create_dir(&forge_dir).unwrap();
        let config = forge_dir.join("config.toml");
        let cfg = crate::Config::default();
        cfg.save(&config).unwrap();

        let result = validate_forge_path(&temp.path().to_path_buf(), true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_reject_non_empty_directory_without_force() {
        let temp = TempDir::new().unwrap();
        let file_path = temp.path().join("existing-file.txt");
        fs::write(&file_path, b"test").unwrap();

        let result = validate_forge_path(&temp.path().to_path_buf(), false);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not empty"));
    }

    #[test]
    fn test_allow_non_empty_directory_with_force() {
        let temp = TempDir::new().unwrap();
        let file_path = temp.path().join("existing-file.txt");
        fs::write(&file_path, b"test").unwrap();

        let result = validate_forge_path(&temp.path().to_path_buf(), true);
        assert!(result.is_ok());
    }
}
