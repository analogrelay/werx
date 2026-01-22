use anyhow::{Context, Result};
use dialoguer::{Select, theme::ColorfulTheme};
use std::fs;
use std::path::PathBuf;

use crate::validation::validate_forge_path;
use crate::{Forge, Protocol};

/// Initialize a Forge at the specified path
///
/// Creates the directory structure:
/// - <root>/                     (Forge root, for workspaces)
/// - <root>/.forge/              (Internal directory)
/// - <root>/.forge/repos/        (Repository storage)
/// - <root>/.forge/config        (Configuration file, also serves as marker)
///
/// If force is true, will reinitialize an existing Forge
/// If protocol is None, will prompt the user for protocol preference
pub fn initialize_forge(path: PathBuf, force: bool, protocol: Option<Protocol>) -> Result<Forge> {
    // Validate the path
    validate_forge_path(&path, force).context("Path validation failed")?;

    // Prompt for protocol preference BEFORE creating anything
    // This way if user Ctrl-C's, nothing has been created yet
    let protocol = match protocol {
        Some(p) => p,
        None => prompt_for_protocol()?,
    };

    // Now that we have all inputs, create the directory structure
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

    // Create config file with protocol preference
    let mut config = crate::Config::default();
    config.set_protocol(protocol);
    forge
        .save_config(&config)
        .context("Failed to create Forge config file")?;

    Ok(forge)
}

/// Prompt user to choose protocol preference
fn prompt_for_protocol() -> Result<Protocol> {
    println!();
    println!("Choose your preferred Git protocol for repository operations:");
    println!();

    let options = vec![
        "SSH   (git@github.com:owner/repo.git)",
        "HTTPS (https://github.com/owner/repo.git)",
    ];

    let selection = Select::with_theme(&ColorfulTheme::default())
        .items(&options)
        .default(0)
        .interact()?;

    let protocol = match selection {
        0 => Protocol::Ssh,
        1 => Protocol::Https,
        _ => unreachable!(),
    };

    println!();
    println!("✓ Protocol preference set to: {}", protocol);

    Ok(protocol)
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
    use crate::FORGE_CONFIG;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_initialize_forge_creates_structure() {
        let temp = TempDir::new().unwrap();
        let forge_path = temp.path().join("test-forge");

        let forge = initialize_forge(forge_path.clone(), false, Some(Protocol::Https))
            .expect("Should initialize forge");

        assert_eq!(forge.root, forge_path);
        assert!(forge_path.exists());
        assert!(forge.forge_dir().exists());
        assert!(forge.repos_dir().exists());
        assert!(forge.config_file().exists());
    }

    #[test]
    fn test_initialize_forge_in_existing_empty_directory() {
        let temp = TempDir::new().unwrap();

        let forge = initialize_forge(temp.path().to_path_buf(), false, Some(Protocol::Ssh))
            .expect("Should initialize forge");

        assert!(forge.forge_dir().exists());
        assert!(forge.repos_dir().exists());
        assert!(forge.config_file().exists());
    }

    #[test]
    fn test_initialize_forge_creates_parent_directories() {
        let temp = TempDir::new().unwrap();
        let forge_path = temp.path().join("parent").join("child").join("forge");

        let forge = initialize_forge(forge_path.clone(), false, Some(Protocol::Https))
            .expect("Should initialize forge");

        assert!(forge_path.exists());
        assert!(forge.forge_dir().exists());
        assert!(forge.repos_dir().exists());
        assert!(forge.config_file().exists());
    }

    #[test]
    fn test_initialize_forge_fails_without_force() {
        let temp = TempDir::new().unwrap();
        let forge_dir = temp.path().join(".forge");
        fs::create_dir(&forge_dir).unwrap();
        let config_path = forge_dir.join(FORGE_CONFIG);
        let cfg = crate::Config::default();
        cfg.save(&config_path).unwrap();

        let result = initialize_forge(temp.path().to_path_buf(), false, Some(Protocol::Https));
        assert!(result.is_err());
    }

    #[test]
    fn test_initialize_forge_succeeds_with_force() {
        let temp = TempDir::new().unwrap();
        let forge_dir = temp.path().join(".forge");
        fs::create_dir(&forge_dir).unwrap();
        let config_path = forge_dir.join(FORGE_CONFIG);
        let cfg = crate::Config::default();
        cfg.save(&config_path).unwrap();

        let result = initialize_forge(temp.path().to_path_buf(), true, Some(Protocol::Ssh));
        assert!(result.is_ok());
    }

    #[test]
    fn test_initialize_forge_preserves_existing_content_with_force() {
        let temp = TempDir::new().unwrap();
        let existing_file = temp.path().join("existing.txt");
        fs::write(&existing_file, b"important data").unwrap();

        let forge = initialize_forge(temp.path().to_path_buf(), true, Some(Protocol::Https))
            .expect("Should initialize forge");

        assert!(existing_file.exists());
        assert_eq!(fs::read(&existing_file).unwrap(), b"important data");
        assert!(forge.config_file().exists());
    }

    #[test]
    fn test_initialize_forge_saves_protocol_preference() {
        let temp = TempDir::new().unwrap();
        let forge_path = temp.path().join("test-forge");

        let forge = initialize_forge(forge_path.clone(), false, Some(Protocol::Ssh))
            .expect("Should initialize forge");

        let config = forge.load_config().expect("Should load config");
        assert_eq!(config.default_provider(), "github");
        assert_eq!(config.protocol(), Some(Protocol::Ssh));
    }
}
