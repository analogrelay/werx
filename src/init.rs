use anyhow::{Context, Result};
use dialoguer::{theme::ColorfulTheme, Select};
use std::fs;
use std::path::PathBuf;

use crate::validation::validate_werx_path;
use crate::{Protocol, Werx};

/// Initialize a Werx at the specified path
///
/// Creates the directory structure:
/// - <root>/                     (Werx root, for workspaces)
/// - <root>/.werx/               (Internal directory)
/// - <root>/.werx/repos/         (Repository storage)
/// - <root>/.werx/config         (Configuration file, also serves as marker)
///
/// If force is true, will reinitialize an existing Werx
/// If protocol is None, will prompt the user for protocol preference
pub fn initialize_werx(path: PathBuf, force: bool, protocol: Option<Protocol>) -> Result<Werx> {
    // Validate the path
    validate_werx_path(&path, force).context("Path validation failed")?;

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

    // Create werx structure
    let werx = Werx { root: path };

    // Create .werx directory
    create_directory(&werx.werx_dir(), ".werx")?;

    // Create repos subdirectory inside .werx
    create_directory(&werx.repos_dir(), "repos")?;

    // Create config file with protocol preference
    let mut config = crate::Config::default();
    config.set_protocol(protocol);
    werx.save_config(&config)
        .context("Failed to create Werx config file")?;

    Ok(werx)
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
    use crate::WERX_CONFIG;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_initialize_werx_creates_structure() {
        let temp = TempDir::new().unwrap();
        let werx_path = temp.path().join("test-werx");

        let werx = initialize_werx(werx_path.clone(), false, Some(Protocol::Https))
            .expect("Should initialize werx");

        assert_eq!(werx.root, werx_path);
        assert!(werx_path.exists());
        assert!(werx.werx_dir().exists());
        assert!(werx.repos_dir().exists());
        assert!(werx.config_file().exists());
    }

    #[test]
    fn test_initialize_werx_in_existing_empty_directory() {
        let temp = TempDir::new().unwrap();

        let werx = initialize_werx(temp.path().to_path_buf(), false, Some(Protocol::Ssh))
            .expect("Should initialize werx");

        assert!(werx.werx_dir().exists());
        assert!(werx.repos_dir().exists());
        assert!(werx.config_file().exists());
    }

    #[test]
    fn test_initialize_werx_creates_parent_directories() {
        let temp = TempDir::new().unwrap();
        let werx_path = temp.path().join("parent").join("child").join("werx");

        let werx = initialize_werx(werx_path.clone(), false, Some(Protocol::Https))
            .expect("Should initialize werx");

        assert!(werx_path.exists());
        assert!(werx.werx_dir().exists());
        assert!(werx.repos_dir().exists());
        assert!(werx.config_file().exists());
    }

    #[test]
    fn test_initialize_werx_fails_without_force() {
        let temp = TempDir::new().unwrap();
        let werx_dir = temp.path().join(".werx");
        fs::create_dir(&werx_dir).unwrap();
        let config_path = werx_dir.join(WERX_CONFIG);
        let cfg = crate::Config::default();
        cfg.save(&config_path).unwrap();

        let result = initialize_werx(temp.path().to_path_buf(), false, Some(Protocol::Https));
        assert!(result.is_err());
    }

    #[test]
    fn test_initialize_werx_succeeds_with_force() {
        let temp = TempDir::new().unwrap();
        let werx_dir = temp.path().join(".werx");
        fs::create_dir(&werx_dir).unwrap();
        let config_path = werx_dir.join(WERX_CONFIG);
        let cfg = crate::Config::default();
        cfg.save(&config_path).unwrap();

        let result = initialize_werx(temp.path().to_path_buf(), true, Some(Protocol::Ssh));
        assert!(result.is_ok());
    }

    #[test]
    fn test_initialize_werx_preserves_existing_content_with_force() {
        let temp = TempDir::new().unwrap();
        let existing_file = temp.path().join("existing.txt");
        fs::write(&existing_file, b"important data").unwrap();

        let werx = initialize_werx(temp.path().to_path_buf(), true, Some(Protocol::Https))
            .expect("Should initialize werx");

        assert!(existing_file.exists());
        assert_eq!(fs::read(&existing_file).unwrap(), b"important data");
        assert!(werx.config_file().exists());
    }

    #[test]
    fn test_initialize_werx_saves_protocol_preference() {
        let temp = TempDir::new().unwrap();
        let werx_path = temp.path().join("test-werx");

        let werx = initialize_werx(werx_path.clone(), false, Some(Protocol::Ssh))
            .expect("Should initialize werx");

        let config = werx.load_config().expect("Should load config");
        assert_eq!(config.default_provider(), "github");
        assert_eq!(config.protocol(), Some(Protocol::Ssh));
    }
}
