mod common;

use common::{assert_failure, assert_success, run_forge};
use tempfile::TempDir;

#[test]
fn test_init_with_explicit_path() {
    let temp_dir = TempDir::new().unwrap();
    let forge_path = temp_dir.path().join("test-forge");

    let output = run_forge(
        &["init", forge_path.to_str().unwrap(), "--protocol", "https"],
        &[]
    );

    assert_success(&output);

    // Verify directory structure was created
    assert!(forge_path.exists());
    assert!(forge_path.join(".forge").exists());
    assert!(forge_path.join(".forge/repos").exists());
    assert!(forge_path.join(".forge/config.toml").exists());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Forge initialized successfully"));
}

#[test]
fn test_init_with_forge_dir_env() {
    let temp_dir = TempDir::new().unwrap();
    let forge_path = temp_dir.path().join("env-forge");

    let output = run_forge(
        &["init", "--protocol", "ssh"],
        &[("FORGE_DIR", forge_path.to_str().unwrap())]
    );

    assert_success(&output);

    // Verify directory structure was created at FORGE_DIR location
    assert!(forge_path.exists());
    assert!(forge_path.join(".forge").exists());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Forge initialized successfully"));
}

#[test]
fn test_init_command_arg_overrides_env() {
    let temp_dir = TempDir::new().unwrap();
    let env_path = temp_dir.path().join("env-path");
    let arg_path = temp_dir.path().join("arg-path");

    let output = run_forge(
        &["init", arg_path.to_str().unwrap(), "--protocol", "https"],
        &[("FORGE_DIR", env_path.to_str().unwrap())]
    );

    assert_success(&output);

    // Verify forge was created at arg_path, not env_path
    assert!(arg_path.exists());
    assert!(arg_path.join(".forge").exists());
    assert!(!env_path.exists());
}

#[test]
fn test_init_prevents_reinit_without_force() {
    let temp_dir = TempDir::new().unwrap();
    let forge_path = temp_dir.path().join("existing-forge");

    // First init should succeed
    let output = run_forge(
        &["init", forge_path.to_str().unwrap(), "--protocol", "https"],
        &[]
    );
    assert_success(&output);

    // Second init without --force should fail
    let output = run_forge(
        &["init", forge_path.to_str().unwrap()],
        &[]
    );
    assert_failure(&output);

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("already exists") || stderr.contains("Forge already"));
}

#[test]
fn test_init_allows_reinit_with_force() {
    let temp_dir = TempDir::new().unwrap();
    let forge_path = temp_dir.path().join("force-forge");

    // First init
    let output = run_forge(
        &["init", forge_path.to_str().unwrap(), "--protocol", "https"],
        &[]
    );
    assert_success(&output);

    // Second init with --force should succeed
    let output = run_forge(
        &["init", forge_path.to_str().unwrap(), "--force", "--protocol", "ssh"],
        &[]
    );
    assert_success(&output);

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Forge initialized successfully"));
}

#[test]
fn test_init_creates_parent_directories() {
    let temp_dir = TempDir::new().unwrap();
    let forge_path = temp_dir.path().join("parent/child/forge");

    let output = run_forge(
        &["init", forge_path.to_str().unwrap(), "--protocol", "https"],
        &[]
    );

    assert_success(&output);

    // Verify nested directories were created
    assert!(forge_path.exists());
    assert!(forge_path.join(".forge").exists());
}

#[test]
fn test_init_protocol_preference_saved() {
    let temp_dir = TempDir::new().unwrap();
    let forge_path = temp_dir.path().join("protocol-forge");

    // Init with SSH protocol
    let output = run_forge(
        &["init", forge_path.to_str().unwrap(), "--protocol", "ssh"],
        &[]
    );
    assert_success(&output);

    // Verify config file contains protocol preference
    let config_path = forge_path.join(".forge/config.toml");
    assert!(config_path.exists());

    let config_content = std::fs::read_to_string(config_path).unwrap();
    assert!(config_content.contains("ssh") || config_content.contains("SSH"));
}
