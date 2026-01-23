mod common;

use common::{assert_failure, assert_success, run_forge};
use tempfile::TempDir;

// forge workspace list tests

#[test]
fn test_workspace_list_empty() {
    let temp_dir = TempDir::new().unwrap();
    let forge_path = temp_dir.path().join("ws-empty-forge");

    // Initialize forge
    run_forge(
        &["init", forge_path.to_str().unwrap(), "--protocol", "https"],
        &[]
    );

    // List workspaces
    let output = run_forge(
        &["workspace", "list"],
        &[("FORGE_DIR", forge_path.to_str().unwrap())]
    );

    assert_success(&output);

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("No workspaces found") || stdout.contains("no workspaces"));
}

#[test]
fn test_workspace_list_json_format_empty() {
    let temp_dir = TempDir::new().unwrap();
    let forge_path = temp_dir.path().join("ws-json-forge");

    // Initialize forge
    run_forge(
        &["init", forge_path.to_str().unwrap(), "--protocol", "https"],
        &[]
    );

    // List workspaces in JSON format (empty case returns text, not JSON)
    let output = run_forge(
        &["workspace", "list", "--format", "json"],
        &[("FORGE_DIR", forge_path.to_str().unwrap())]
    );

    assert_success(&output);

    // Empty list still shows text message, not JSON
    // This is intentional UX decision in the code
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("No workspaces found") || stdout.contains("no workspaces"));
}

#[test]
fn test_workspace_list_requires_forge() {
    let temp_dir = TempDir::new().unwrap();
    let non_forge_path = temp_dir.path().join("not-a-forge");

    let output = run_forge(
        &["workspace", "list"],
        &[("FORGE_DIR", non_forge_path.to_str().unwrap())]
    );

    assert_failure(&output);

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("No Forge found") || stderr.contains("forge init"));
}

// forge workspace create tests

#[test]
fn test_workspace_create_requires_forge() {
    let temp_dir = TempDir::new().unwrap();
    let non_forge_path = temp_dir.path().join("not-a-forge");

    let output = run_forge(
        &["workspace", "create", "owner/repo", "main", "--name", "test"],
        &[("FORGE_DIR", non_forge_path.to_str().unwrap())]
    );

    assert_failure(&output);

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("No Forge found") || stderr.contains("forge init"));
}

// forge workspace remove tests

#[test]
fn test_workspace_remove_requires_forge() {
    let temp_dir = TempDir::new().unwrap();
    let non_forge_path = temp_dir.path().join("not-a-forge");

    let output = run_forge(
        &["workspace", "remove", "test", "--force"],
        &[("FORGE_DIR", non_forge_path.to_str().unwrap())]
    );

    assert_failure(&output);

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("No Forge found") || stderr.contains("forge init"));
}

// Workspace alias tests

#[test]
fn test_workspaces_alias() {
    let temp_dir = TempDir::new().unwrap();
    let forge_path = temp_dir.path().join("alias-forge");

    // Initialize forge
    run_forge(
        &["init", forge_path.to_str().unwrap(), "--protocol", "https"],
        &[]
    );

    // Use 'workspaces' alias instead of 'workspace'
    let output = run_forge(
        &["workspaces", "list"],
        &[("FORGE_DIR", forge_path.to_str().unwrap())]
    );

    assert_success(&output);
}

#[test]
fn test_wt_alias() {
    let temp_dir = TempDir::new().unwrap();
    let forge_path = temp_dir.path().join("wt-alias-forge");

    // Initialize forge
    run_forge(
        &["init", forge_path.to_str().unwrap(), "--protocol", "https"],
        &[]
    );

    // Use 'wt' alias instead of 'workspace'
    let output = run_forge(
        &["wt", "list"],
        &[("FORGE_DIR", forge_path.to_str().unwrap())]
    );

    assert_success(&output);
}

// forge go tests

#[test]
fn test_go_requires_forge() {
    let temp_dir = TempDir::new().unwrap();
    let non_forge_path = temp_dir.path().join("not-a-forge");

    let output = run_forge(
        &["go"],
        &[("FORGE_DIR", non_forge_path.to_str().unwrap())]
    );

    assert_failure(&output);

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("No Forge found") || stderr.contains("forge init"));
}

#[test]
fn test_go_with_empty_forge() {
    let temp_dir = TempDir::new().unwrap();
    let forge_path = temp_dir.path().join("empty-go-forge");

    // Initialize forge
    run_forge(
        &["init", forge_path.to_str().unwrap(), "--protocol", "https"],
        &[]
    );

    // Try to go with no workspaces
    let output = run_forge(
        &["go"],
        &[("FORGE_DIR", forge_path.to_str().unwrap())]
    );

    assert_success(&output);

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("No workspaces found") || stdout.contains("no workspaces"));
}
