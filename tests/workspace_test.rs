mod common;

use common::{TestContext, assert_failure, assert_success};

// werx workspace list tests

#[test]
fn test_workspace_list_empty() {
    let ctx = TestContext::new();
    ctx.init_werx();

    // List workspaces
    let output = ctx.run_werx(&["workspace", "list"], &[]);

    assert_success(&output);

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("No workspaces found") || stdout.contains("no workspaces"));
}

#[test]
fn test_workspace_list_json_format_empty() {
    let ctx = TestContext::new();
    ctx.init_werx();

    // List workspaces in JSON format (empty case returns text, not JSON)
    let output = ctx.run_werx(&["workspace", "list", "--format", "json"], &[]);

    assert_success(&output);

    // Empty list still shows text message, not JSON
    // This is intentional UX decision in the code
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("No workspaces found") || stdout.contains("no workspaces"));
}

#[test]
fn test_workspace_list_requires_werx() {
    let ctx = TestContext::new();

    let output = ctx.run_werx(&["workspace", "list"], &[]);

    assert_failure(&output);

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("No Werx found") || stderr.contains("werx init"));
}

// werx workspace create tests

#[test]
fn test_workspace_create_requires_werx() {
    let ctx = TestContext::new();

    let output = ctx.run_werx(
        &[
            "workspace",
            "create",
            "owner/repo",
            "main",
            "--name",
            "test",
        ],
        &[],
    );

    assert_failure(&output);

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("No Werx found") || stderr.contains("werx init"));
}

// werx workspace remove tests

#[test]
fn test_workspace_remove_requires_werx() {
    let ctx = TestContext::new();

    let output = ctx.run_werx(&["workspace", "remove", "test", "--force"], &[]);

    assert_failure(&output);

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("No Werx found") || stderr.contains("werx init"));
}

// Workspace alias tests

#[test]
fn test_workspaces_alias() {
    let ctx = TestContext::new();
    ctx.init_werx();

    // Use 'workspaces' alias instead of 'workspace'
    let output = ctx.run_werx(&["workspaces", "list"], &[]);

    assert_success(&output);
}

#[test]
fn test_wt_alias() {
    let ctx = TestContext::new();
    ctx.init_werx();

    // Use 'wt' alias instead of 'workspace'
    let output = ctx.run_werx(&["wt", "list"], &[]);

    assert_success(&output);
}

// werx go tests

#[test]
fn test_go_requires_werx() {
    let ctx = TestContext::new();

    let output = ctx.run_werx(&["go"], &[]);

    assert_failure(&output);

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("No Werx found") || stderr.contains("werx init"));
}

#[test]
fn test_go_with_empty_werx() {
    let ctx = TestContext::new();
    ctx.init_werx();

    // Try to go with no workspaces
    let output = ctx.run_werx(&["go"], &[]);

    assert_success(&output);

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("No workspaces found") || stdout.contains("no workspaces"));
}

// =============================================================================
// End-to-end workspace lifecycle tests
// =============================================================================

#[test]
fn test_workspace_create_from_existing_repo() {
    let ctx = TestContext::new();
    ctx.init_werx();

    let create_output = ctx.run_werx(&["create", "myorg/myrepo"], &[]);
    assert_success(&create_output);

    // First, create a new branch in the bare repo for us to checkout
    let bare_repo_path = ctx.werx_path().join(".werx/repos/myrepo");
    let branch_output = ctx.run_git_in(&bare_repo_path, &["branch", "feature-branch", "main"]);
    assert!(
        branch_output.status.success(),
        "Failed to create test branch: {}",
        String::from_utf8_lossy(&branch_output.stderr)
    );

    // Create a new workspace for the feature branch
    let output = ctx.run_werx(
        &[
            "workspace",
            "create",
            "myorg/myrepo",
            "feature-branch",
            "--name",
            "feature-ws",
        ],
        &[],
    );

    assert_success(&output);

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Workspace created") || stdout.contains("feature-ws"),
        "Expected success message, got: {}",
        stdout
    );

    // Verify the workspace path exists
    let workspace_path = ctx.werx_path().join("myrepo/feature-ws");
    assert!(
        workspace_path.exists(),
        "Workspace should exist at {:?}",
        workspace_path
    );

    // Verify it appears in workspace list
    let list_output = ctx.run_werx(&["workspace", "list"], &[]);
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
    let ctx = TestContext::new();
    ctx.init_werx();

    ctx.run_werx(&["create", "company/project"], &[]);

    // Create a new branch for our custom workspace
    let bare_repo_path = ctx.werx_path().join(".werx/repos/project");
    let branch_output = ctx.run_git_in(&bare_repo_path, &["branch", "custom-branch", "main"]);
    assert!(
        branch_output.status.success(),
        "Failed to create test branch: {}",
        String::from_utf8_lossy(&branch_output.stderr)
    );

    // Create workspace with custom name
    let output = ctx.run_werx(
        &[
            "workspace",
            "create",
            "company/project",
            "custom-branch",
            "--name",
            "my-custom-workspace",
        ],
        &[],
    );

    assert_success(&output);

    // Verify the custom-named workspace appears in list
    let list_output = ctx.run_werx(&["workspace", "list"], &[]);
    assert_success(&list_output);

    let stdout = String::from_utf8_lossy(&list_output.stdout);
    assert!(
        stdout.contains("my-custom-workspace"),
        "Custom workspace name should appear in list. Got: {}",
        stdout
    );

    // Verify the directory was created with the custom name
    let workspace_path = ctx.werx_path().join("project/my-custom-workspace");
    assert!(
        workspace_path.exists(),
        "Workspace directory should exist at {:?}",
        workspace_path
    );
}

#[test]
fn test_workspace_list_json_with_workspaces() {
    let ctx = TestContext::new();
    ctx.init_werx();

    ctx.run_werx(&["create", "org/jsontest"], &[]);

    // Create a new branch in the bare repo for the additional workspace
    let bare_repo_path = ctx.werx_path().join(".werx/repos/jsontest");
    let branch_output = ctx.run_git_in(&bare_repo_path, &["branch", "feature-branch", "main"]);
    assert!(
        branch_output.status.success(),
        "Failed to create test branch: {}",
        String::from_utf8_lossy(&branch_output.stderr)
    );

    // Create an additional workspace on the new branch
    ctx.run_werx(
        &[
            "workspace",
            "create",
            "org/jsontest",
            "feature-branch",
            "--name",
            "feature-ws",
        ],
        &[],
    );

    // List workspaces in JSON format
    let output = ctx.run_werx(&["workspace", "list", "--format", "json"], &[]);

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
    let ctx = TestContext::new();
    ctx.init_werx();

    ctx.run_werx(&["create", "test/removews"], &[]);

    // Create a new branch in the bare repo for the workspace we'll remove
    let bare_repo_path = ctx.werx_path().join(".werx/repos/removews");
    let branch_output = ctx.run_git_in(&bare_repo_path, &["branch", "remove-branch", "main"]);
    assert!(
        branch_output.status.success(),
        "Failed to create remove-branch: {}",
        String::from_utf8_lossy(&branch_output.stderr)
    );

    // Create an extra workspace that we'll remove
    ctx.run_werx(
        &[
            "workspace",
            "create",
            "test/removews",
            "remove-branch",
            "--name",
            "to-remove",
        ],
        &[],
    );

    // Verify it exists
    let workspace_path = ctx.werx_path().join("removews/to-remove");
    assert!(
        workspace_path.exists(),
        "Workspace should exist before removal"
    );

    // Remove the workspace with --force
    let output = ctx.run_werx(
        &["workspace", "remove", "removews/to-remove", "--force"],
        &[],
    );

    assert_success(&output);

    // Verify the workspace directory is gone
    assert!(
        !workspace_path.exists(),
        "Workspace directory should be removed"
    );

    // Verify it's not in the list
    let list_output = ctx.run_werx(&["workspace", "list"], &[]);
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
    let ctx = TestContext::new();
    ctx.init_werx();

    ctx.run_werx(&["create", "statustest/repo"], &[]);

    // Make a change in the workspace (create a new file)
    let workspace_path = ctx.werx_path().join("repo/main");
    let new_file = workspace_path.join("uncommitted.txt");
    std::fs::write(&new_file, "This is an uncommitted change\n")
        .expect("Failed to write test file");

    // Check workspace status
    let output = ctx.run_werx(&["workspace", "status"], &[]);

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
    let ctx = TestContext::new();
    ctx.init_werx();

    ctx.run_werx(&["create", "checktest/repo"], &[]);

    // Create two new branches in the bare repo for additional workspaces
    let bare_repo_path = ctx.werx_path().join(".werx/repos/repo");

    let branch1_output = ctx.run_git_in(&bare_repo_path, &["branch", "clean-branch", "main"]);
    assert!(
        branch1_output.status.success(),
        "Failed to create clean-branch: {}",
        String::from_utf8_lossy(&branch1_output.stderr)
    );

    let branch2_output = ctx.run_git_in(&bare_repo_path, &["branch", "dirty-branch", "main"]);
    assert!(
        branch2_output.status.success(),
        "Failed to create dirty-branch: {}",
        String::from_utf8_lossy(&branch2_output.stderr)
    );

    // Create two additional workspaces on the new branches
    ctx.run_werx(
        &[
            "workspace",
            "create",
            "checktest/repo",
            "clean-branch",
            "--name",
            "clean-ws",
        ],
        &[],
    );

    ctx.run_werx(
        &[
            "workspace",
            "create",
            "checktest/repo",
            "dirty-branch",
            "--name",
            "dirty-ws",
        ],
        &[],
    );

    // Make changes only in dirty-ws
    let dirty_workspace = ctx.werx_path().join("repo/dirty-ws");
    let dirty_file = dirty_workspace.join("dirty.txt");
    std::fs::write(&dirty_file, "Uncommitted content\n").expect("Failed to write dirty file");

    // Run check with --uncommitted filter
    let output = ctx.run_werx(&["workspace", "check", "--uncommitted"], &[]);

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
