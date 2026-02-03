mod common;

use common::{assert_failure, assert_success, run_werx};
use tempfile::TempDir;

// werx workspace list tests

#[test]
fn test_workspace_list_empty() {
    let temp_dir = TempDir::new().unwrap();
    let werx_path = temp_dir.path().join("ws-empty-werx");

    // Initialize werx
    run_werx(
        &["init", werx_path.to_str().unwrap(), "--protocol", "https"],
        &[],
    );

    // List workspaces
    let output = run_werx(
        &["workspace", "list"],
        &[("WERX_DIR", werx_path.to_str().unwrap())],
    );

    assert_success(&output);

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("No workspaces found") || stdout.contains("no workspaces"));
}

#[test]
fn test_workspace_list_json_format_empty() {
    let temp_dir = TempDir::new().unwrap();
    let werx_path = temp_dir.path().join("ws-json-werx");

    // Initialize werx
    run_werx(
        &["init", werx_path.to_str().unwrap(), "--protocol", "https"],
        &[],
    );

    // List workspaces in JSON format (empty case returns text, not JSON)
    let output = run_werx(
        &["workspace", "list", "--format", "json"],
        &[("WERX_DIR", werx_path.to_str().unwrap())],
    );

    assert_success(&output);

    // Empty list still shows text message, not JSON
    // This is intentional UX decision in the code
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("No workspaces found") || stdout.contains("no workspaces"));
}

#[test]
fn test_workspace_list_requires_werx() {
    let temp_dir = TempDir::new().unwrap();
    let non_werx_path = temp_dir.path().join("not-a-werx");

    let output = run_werx(
        &["workspace", "list"],
        &[("WERX_DIR", non_werx_path.to_str().unwrap())],
    );

    assert_failure(&output);

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("No Werx found") || stderr.contains("werx init"));
}

// werx workspace create tests

#[test]
fn test_workspace_create_requires_werx() {
    let temp_dir = TempDir::new().unwrap();
    let non_werx_path = temp_dir.path().join("not-a-werx");

    let output = run_werx(
        &[
            "workspace",
            "create",
            "owner/repo",
            "main",
            "--name",
            "test",
        ],
        &[("WERX_DIR", non_werx_path.to_str().unwrap())],
    );

    assert_failure(&output);

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("No Werx found") || stderr.contains("werx init"));
}

// werx workspace remove tests

#[test]
fn test_workspace_remove_requires_werx() {
    let temp_dir = TempDir::new().unwrap();
    let non_werx_path = temp_dir.path().join("not-a-werx");

    let output = run_werx(
        &["workspace", "remove", "test", "--force"],
        &[("WERX_DIR", non_werx_path.to_str().unwrap())],
    );

    assert_failure(&output);

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("No Werx found") || stderr.contains("werx init"));
}

// Workspace alias tests

#[test]
fn test_workspaces_alias() {
    let temp_dir = TempDir::new().unwrap();
    let werx_path = temp_dir.path().join("alias-werx");

    // Initialize werx
    run_werx(
        &["init", werx_path.to_str().unwrap(), "--protocol", "https"],
        &[],
    );

    // Use 'workspaces' alias instead of 'workspace'
    let output = run_werx(
        &["workspaces", "list"],
        &[("WERX_DIR", werx_path.to_str().unwrap())],
    );

    assert_success(&output);
}

#[test]
fn test_wt_alias() {
    let temp_dir = TempDir::new().unwrap();
    let werx_path = temp_dir.path().join("wt-alias-werx");

    // Initialize werx
    run_werx(
        &["init", werx_path.to_str().unwrap(), "--protocol", "https"],
        &[],
    );

    // Use 'wt' alias instead of 'workspace'
    let output = run_werx(
        &["wt", "list"],
        &[("WERX_DIR", werx_path.to_str().unwrap())],
    );

    assert_success(&output);
}

// werx go tests

#[test]
fn test_go_requires_werx() {
    let temp_dir = TempDir::new().unwrap();
    let non_werx_path = temp_dir.path().join("not-a-werx");

    let output = run_werx(&["go"], &[("WERX_DIR", non_werx_path.to_str().unwrap())]);

    assert_failure(&output);

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("No Werx found") || stderr.contains("werx init"));
}

#[test]
fn test_go_with_empty_werx() {
    let temp_dir = TempDir::new().unwrap();
    let werx_path = temp_dir.path().join("empty-go-werx");

    // Initialize werx
    run_werx(
        &["init", werx_path.to_str().unwrap(), "--protocol", "https"],
        &[],
    );

    // Try to go with no workspaces
    let output = run_werx(&["go"], &[("WERX_DIR", werx_path.to_str().unwrap())]);

    assert_success(&output);

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("No workspaces found") || stdout.contains("no workspaces"));
}
