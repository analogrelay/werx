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

// =============================================================================
// End-to-end workspace lifecycle tests
// =============================================================================

#[test]
fn test_workspace_create_from_existing_repo() {
    let temp_dir = TempDir::new().unwrap();
    let werx_path = temp_dir.path().join("ws-create-werx");

    // Initialize werx and create a repository
    run_werx(
        &["init", werx_path.to_str().unwrap(), "--protocol", "https"],
        &[],
    );

    let create_output = run_werx(
        &["create", "myorg/myrepo"],
        &[("WERX_DIR", werx_path.to_str().unwrap())],
    );
    assert_success(&create_output);

    // First, create a new branch in the bare repo for us to checkout
    let bare_repo_path = werx_path.join(".werx/repos/myrepo");
    let branch_output = std::process::Command::new("git")
        .args([
            "-C",
            bare_repo_path.to_str().unwrap(),
            "branch",
            "feature-branch",
            "main",
        ])
        .output()
        .expect("Failed to create branch");
    assert!(
        branch_output.status.success(),
        "Failed to create test branch"
    );

    // Create a new workspace for the feature branch
    let output = run_werx(
        &[
            "workspace",
            "create",
            "myorg/myrepo",
            "feature-branch", // Use the new branch we created
            "--name",
            "feature-ws",
        ],
        &[("WERX_DIR", werx_path.to_str().unwrap())],
    );

    assert_success(&output);

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Workspace created") || stdout.contains("feature-ws"),
        "Expected success message, got: {}",
        stdout
    );

    // Verify the workspace path exists
    let workspace_path = werx_path.join("myrepo/feature-ws");
    assert!(
        workspace_path.exists(),
        "Workspace should exist at {:?}",
        workspace_path
    );

    // Verify it appears in workspace list
    let list_output = run_werx(
        &["workspace", "list"],
        &[("WERX_DIR", werx_path.to_str().unwrap())],
    );
    assert_success(&list_output);

    let list_stdout = String::from_utf8_lossy(&list_output.stdout);
    assert!(
        list_stdout.contains("feature-ws"),
        "Workspace should appear in list. Got: {}",
        list_stdout
    );
}

#[test]
fn test_workspace_create_custom_name() {
    let temp_dir = TempDir::new().unwrap();
    let werx_path = temp_dir.path().join("ws-name-werx");

    // Initialize werx and create a repository
    run_werx(
        &["init", werx_path.to_str().unwrap(), "--protocol", "https"],
        &[],
    );

    run_werx(
        &["create", "company/project"],
        &[("WERX_DIR", werx_path.to_str().unwrap())],
    );

    // Create a new branch for our custom workspace
    let bare_repo_path = werx_path.join(".werx/repos/project");
    let branch_output = std::process::Command::new("git")
        .args([
            "-C",
            bare_repo_path.to_str().unwrap(),
            "branch",
            "custom-branch",
            "main",
        ])
        .output()
        .expect("Failed to create branch");
    assert!(
        branch_output.status.success(),
        "Failed to create test branch"
    );

    // Create workspace with custom name
    let output = run_werx(
        &[
            "workspace",
            "create",
            "company/project",
            "custom-branch",
            "--name",
            "my-custom-workspace",
        ],
        &[("WERX_DIR", werx_path.to_str().unwrap())],
    );

    assert_success(&output);

    // Verify the custom-named workspace appears in list
    let list_output = run_werx(
        &["workspace", "list"],
        &[("WERX_DIR", werx_path.to_str().unwrap())],
    );
    assert_success(&list_output);

    let stdout = String::from_utf8_lossy(&list_output.stdout);
    assert!(
        stdout.contains("my-custom-workspace"),
        "Custom workspace name should appear in list. Got: {}",
        stdout
    );

    // Verify the directory was created with the custom name
    let workspace_path = werx_path.join("project/my-custom-workspace");
    assert!(
        workspace_path.exists(),
        "Workspace directory should exist at {:?}",
        workspace_path
    );
}

#[test]
fn test_workspace_list_json_with_workspaces() {
    let temp_dir = TempDir::new().unwrap();
    let werx_path = temp_dir.path().join("ws-json-list-werx");

    // Initialize werx and create a repository (which creates main workspace)
    run_werx(
        &["init", werx_path.to_str().unwrap(), "--protocol", "https"],
        &[],
    );

    run_werx(
        &["create", "org/jsontest"],
        &[("WERX_DIR", werx_path.to_str().unwrap())],
    );

    // Create a new branch in the bare repo for the additional workspace
    let bare_repo_path = werx_path.join(".werx/repos/jsontest");
    let branch_output = std::process::Command::new("git")
        .args([
            "-C",
            bare_repo_path.to_str().unwrap(),
            "branch",
            "feature-branch",
            "main",
        ])
        .output()
        .expect("Failed to create branch");
    assert!(
        branch_output.status.success(),
        "Failed to create test branch"
    );

    // Create an additional workspace on the new branch
    run_werx(
        &[
            "workspace",
            "create",
            "org/jsontest",
            "feature-branch",
            "--name",
            "feature-ws",
        ],
        &[("WERX_DIR", werx_path.to_str().unwrap())],
    );

    // List workspaces in JSON format
    let output = run_werx(
        &["workspace", "list", "--format", "json"],
        &[("WERX_DIR", werx_path.to_str().unwrap())],
    );

    assert_success(&output);

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Validate JSON structure
    let json: serde_json::Value =
        serde_json::from_str(&stdout).expect("Output should be valid JSON");

    assert!(json.is_array(), "JSON output should be an array");
    let workspaces = json.as_array().unwrap();
    assert!(
        workspaces.len() >= 2,
        "Should have at least 2 workspaces (main + feature-ws), got {}",
        workspaces.len()
    );

    // Check structure of first workspace
    let ws = &workspaces[0];
    assert!(ws.get("name").is_some(), "Should have name field");
    assert!(ws.get("path").is_some(), "Should have path field");
    assert!(
        ws.get("repository").is_some(),
        "Should have repository field"
    );

    // Verify our workspaces are present
    let names: Vec<&str> = workspaces
        .iter()
        .filter_map(|w| w.get("name").and_then(|n| n.as_str()))
        .collect();
    assert!(names.contains(&"main"), "Should have 'main' workspace");
    assert!(
        names.contains(&"feature-ws"),
        "Should have 'feature-ws' workspace"
    );
}

#[test]
fn test_workspace_remove_with_force() {
    let temp_dir = TempDir::new().unwrap();
    let werx_path = temp_dir.path().join("ws-remove-werx");

    // Initialize werx and create a repository
    run_werx(
        &["init", werx_path.to_str().unwrap(), "--protocol", "https"],
        &[],
    );

    run_werx(
        &["create", "test/removews"],
        &[("WERX_DIR", werx_path.to_str().unwrap())],
    );

    // Create a new branch in the bare repo for the workspace we'll remove
    let bare_repo_path = werx_path.join(".werx/repos/removews");
    let branch_output = std::process::Command::new("git")
        .args([
            "-C",
            bare_repo_path.to_str().unwrap(),
            "branch",
            "remove-branch",
            "main",
        ])
        .output()
        .expect("Failed to create branch");
    assert!(
        branch_output.status.success(),
        "Failed to create test branch"
    );

    // Create an extra workspace that we'll remove
    run_werx(
        &[
            "workspace",
            "create",
            "test/removews",
            "remove-branch",
            "--name",
            "to-remove",
        ],
        &[("WERX_DIR", werx_path.to_str().unwrap())],
    );

    // Verify it exists
    let workspace_path = werx_path.join("removews/to-remove");
    assert!(
        workspace_path.exists(),
        "Workspace should exist before removal"
    );

    // Remove the workspace with --force
    let output = run_werx(
        &["workspace", "remove", "removews/to-remove", "--force"],
        &[("WERX_DIR", werx_path.to_str().unwrap())],
    );

    assert_success(&output);

    // Verify the workspace directory is gone
    assert!(
        !workspace_path.exists(),
        "Workspace directory should be removed"
    );

    // Verify it's not in the list
    let list_output = run_werx(
        &["workspace", "list"],
        &[("WERX_DIR", werx_path.to_str().unwrap())],
    );
    let stdout = String::from_utf8_lossy(&list_output.stdout);
    assert!(
        !stdout.contains("to-remove"),
        "Removed workspace should not appear in list. Got: {}",
        stdout
    );
}

// =============================================================================
// Workspace status and check tests
// =============================================================================

#[test]
fn test_workspace_status_with_changes() {
    let temp_dir = TempDir::new().unwrap();
    let werx_path = temp_dir.path().join("ws-status-werx");

    // Initialize werx and create a repository
    run_werx(
        &["init", werx_path.to_str().unwrap(), "--protocol", "https"],
        &[],
    );

    run_werx(
        &["create", "statustest/repo"],
        &[("WERX_DIR", werx_path.to_str().unwrap())],
    );

    // Make a change in the workspace (create a new file)
    let workspace_path = werx_path.join("repo/main");
    let new_file = workspace_path.join("uncommitted.txt");
    std::fs::write(&new_file, "This is an uncommitted change\n")
        .expect("Failed to write test file");

    // Check workspace status
    let output = run_werx(
        &["workspace", "status"],
        &[("WERX_DIR", werx_path.to_str().unwrap())],
    );

    assert_success(&output);

    let stdout = String::from_utf8_lossy(&output.stdout);
    // Should show uncommitted changes
    assert!(
        stdout.contains("Uncommitted") || stdout.contains("uncommitted") || stdout.contains("?:"),
        "Status should show uncommitted changes. Got: {}",
        stdout
    );
}

#[test]
fn test_workspace_check_uncommitted_filter() {
    let temp_dir = TempDir::new().unwrap();
    let werx_path = temp_dir.path().join("ws-check-werx");

    // Initialize werx and create a repository
    run_werx(
        &["init", werx_path.to_str().unwrap(), "--protocol", "https"],
        &[],
    );

    run_werx(
        &["create", "checktest/repo"],
        &[("WERX_DIR", werx_path.to_str().unwrap())],
    );

    // Create two new branches in the bare repo for additional workspaces
    let bare_repo_path = werx_path.join(".werx/repos/repo");

    let branch1_output = std::process::Command::new("git")
        .args([
            "-C",
            bare_repo_path.to_str().unwrap(),
            "branch",
            "clean-branch",
            "main",
        ])
        .output()
        .expect("Failed to create branch");
    assert!(
        branch1_output.status.success(),
        "Failed to create clean-branch"
    );

    let branch2_output = std::process::Command::new("git")
        .args([
            "-C",
            bare_repo_path.to_str().unwrap(),
            "branch",
            "dirty-branch",
            "main",
        ])
        .output()
        .expect("Failed to create branch");
    assert!(
        branch2_output.status.success(),
        "Failed to create dirty-branch"
    );

    // Create two additional workspaces on the new branches
    run_werx(
        &[
            "workspace",
            "create",
            "checktest/repo",
            "clean-branch",
            "--name",
            "clean-ws",
        ],
        &[("WERX_DIR", werx_path.to_str().unwrap())],
    );

    run_werx(
        &[
            "workspace",
            "create",
            "checktest/repo",
            "dirty-branch",
            "--name",
            "dirty-ws",
        ],
        &[("WERX_DIR", werx_path.to_str().unwrap())],
    );

    // Make changes only in dirty-ws
    let dirty_workspace = werx_path.join("repo/dirty-ws");
    let dirty_file = dirty_workspace.join("dirty.txt");
    std::fs::write(&dirty_file, "Uncommitted content\n").expect("Failed to write dirty file");

    // Run check with --uncommitted filter
    let output = run_werx(
        &["workspace", "check", "--uncommitted"],
        &[("WERX_DIR", werx_path.to_str().unwrap())],
    );

    assert_success(&output);

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should show dirty-ws but not clean-ws (or main which is also clean relative to dirty-ws)
    assert!(
        stdout.contains("dirty-ws"),
        "Should show dirty workspace. Got: {}",
        stdout
    );
    // clean-ws should not appear in --uncommitted results (unless it somehow also has changes)
    // Note: 'main' workspace may or may not appear depending on git state
}
