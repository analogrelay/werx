mod common;

use common::{TestContext, assert_failure, assert_success};

// werx repos list tests

#[test]
fn test_repos_list_empty() {
    let ctx = TestContext::new();
    ctx.init_werx();

    // List repos
    let output = ctx.run_werx(&["repos", "list"], &[]);

    assert_success(&output);

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("No repositories found") || stdout.contains("no repositories"));
}

#[test]
fn test_repos_list_json_format_empty() {
    let ctx = TestContext::new();
    ctx.init_werx();

    // List repos in JSON format (empty case returns text, not JSON)
    let output = ctx.run_werx(&["repos", "list", "--format", "json"], &[]);

    assert_success(&output);

    // Empty list still shows text message, not JSON
    // This is intentional UX decision in the code
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("No repositories found") || stdout.contains("no repositories"));
}

#[test]
fn test_repos_list_requires_werx() {
    let ctx = TestContext::new();
    // Don't init - just try to list from non-existent werx

    let output = ctx.run_werx(&["repos", "list"], &[]);

    assert_failure(&output);

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("No Werx found") || stderr.contains("werx init"));
}

// werx repos add tests

#[test]
fn test_add_repo_requires_werx() {
    let ctx = TestContext::new();

    let output = ctx.run_werx(&["add", "owner/repo"], &[]);

    assert_failure(&output);

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("No Werx found") || stderr.contains("werx init"));
}

#[test]
fn test_repos_add_alias() {
    let ctx = TestContext::new();

    // Test 'repos add' produces same error as 'add'
    let output = ctx.run_werx(&["repos", "add", "owner/repo"], &[]);

    assert_failure(&output);

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("No Werx found") || stderr.contains("werx init"));
}

// werx repos remove tests

#[test]
fn test_remove_repo_requires_werx() {
    let ctx = TestContext::new();

    let output = ctx.run_werx(&["repos", "remove", "owner/repo", "--force"], &[]);

    assert_failure(&output);

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("No Werx found") || stderr.contains("werx init"));
}

// werx create tests

#[test]
fn test_create_repo_requires_werx() {
    let ctx = TestContext::new();

    let output = ctx.run_werx(&["create", "owner/repo"], &[]);

    assert_failure(&output);

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("No Werx found") || stderr.contains("werx init"));
}

#[test]
fn test_repos_create_alias() {
    let ctx = TestContext::new();

    // Test 'repos create' produces same error as 'create'
    let output = ctx.run_werx(&["repos", "create", "owner/repo"], &[]);

    assert_failure(&output);

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("No Werx found") || stderr.contains("werx init"));
}

#[test]
fn test_create_repo_invalid_format_no_slash() {
    let ctx = TestContext::new();
    ctx.init_werx();

    let output = ctx.run_werx(&["create", "invalidformat"], &[]);

    assert_failure(&output);

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Invalid repository specification")
            || stderr.contains("Expected format: owner/repo")
    );
}

#[test]
fn test_create_repo_invalid_format_special_chars() {
    let ctx = TestContext::new();
    ctx.init_werx();

    let output = ctx.run_werx(&["create", "owner@bad/repo"], &[]);

    assert_failure(&output);

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Invalid owner format"));
}

#[test]
fn test_create_repo_success() {
    let ctx = TestContext::new();
    ctx.init_werx();

    let output = ctx.run_werx(&["create", "mycompany/awesome-project"], &[]);

    assert_success(&output);

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Repository created successfully")
            || stdout.contains("mycompany/awesome-project")
    );

    // Verify the repository appears in list
    let list_output = ctx.run_werx(&["repos", "list"], &[]);

    assert_success(&list_output);

    let list_stdout = String::from_utf8_lossy(&list_output.stdout);
    assert!(list_stdout.contains("awesome-project"));
}

#[test]
fn test_create_repo_duplicate_detection() {
    let ctx = TestContext::new();
    ctx.init_werx();

    // Create first repository
    let output1 = ctx.run_werx(&["create", "owner/myrepo"], &[]);
    assert_success(&output1);

    // Try to create the same repository again
    let output2 = ctx.run_werx(&["create", "owner/myrepo"], &[]);

    assert_failure(&output2);

    let stderr = String::from_utf8_lossy(&output2.stderr);
    assert!(stderr.contains("already exists"));
}

#[test]
fn test_create_repo_creates_worktree() {
    let ctx = TestContext::new();
    ctx.init_werx();

    let output = ctx.run_werx(&["create", "testowner/testrepo"], &[]);

    assert_success(&output);

    // Verify worktree was created by checking the workspace directory exists
    let worktree_path = ctx.werx_path().join("testrepo").join("main");
    assert!(
        worktree_path.exists(),
        "Worktree directory should exist at {:?}",
        worktree_path
    );

    // Verify it's a valid git worktree
    let git_file = worktree_path.join(".git");
    assert!(git_file.exists(), ".git file should exist in worktree");
}

#[test]
fn test_create_repo_progressive_naming() {
    let ctx = TestContext::new();
    ctx.init_werx();

    // Create first repository with name "utils"
    let output1 = ctx.run_werx(&["create", "alice/utils"], &[]);
    assert_success(&output1);

    // Create second repository with same name but different owner
    let output2 = ctx.run_werx(&["create", "bob/utils"], &[]);
    assert_success(&output2);

    // Verify both exist with proper naming
    let list_output = ctx.run_werx(&["repos", "list"], &[]);
    assert_success(&list_output);

    let list_stdout = String::from_utf8_lossy(&list_output.stdout);
    // First one should be simple "utils"
    // Second one should be "bob-utils"
    assert!(list_stdout.contains("utils"));
    assert!(list_stdout.contains("bob-utils"));
}

// =============================================================================
// End-to-end repository lifecycle tests with synthetic repos
// =============================================================================

// Note: test_add_local_repo_and_list was removed because `werx add` only supports
// HTTP/HTTPS and SSH URLs, not local file:// URLs. Testing actual cloning would
// require network access. The `werx create` command is tested instead to validate
// the end-to-end repository workflow without network access.

#[test]
fn test_repos_list_json_format_with_repos() {
    let ctx = TestContext::new();
    ctx.init_werx();

    let output = ctx.run_werx(&["create", "testowner/jsontest"], &[]);
    assert_success(&output);

    // List repos in JSON format
    let list_output = ctx.run_werx(&["repos", "list", "--format", "json"], &[]);

    assert_success(&list_output);

    let stdout = String::from_utf8_lossy(&list_output.stdout);

    // Validate JSON structure
    let json: serde_json::Value =
        serde_json::from_str(&stdout).expect("Output should be valid JSON");

    assert!(json.is_array(), "JSON output should be an array");
    let repos = json.as_array().unwrap();
    assert_eq!(repos.len(), 1, "Should have exactly one repository");

    let repo = &repos[0];
    assert!(repo.get("dir_name").is_some(), "Should have dir_name field");
    assert!(
        repo.get("clone_url").is_some(),
        "Should have clone_url field"
    );
    assert!(repo.get("valid").is_some(), "Should have valid field");
    assert_eq!(repo.get("dir_name").unwrap().as_str().unwrap(), "jsontest");
}

#[test]
fn test_repos_remove_existing_repo() {
    let ctx = TestContext::new();
    ctx.init_werx();

    let create_output = ctx.run_werx(&["create", "owner/removeme"], &[]);
    assert_success(&create_output);

    // Verify it exists
    let list_output = ctx.run_werx(&["repos", "list"], &[]);
    let stdout = String::from_utf8_lossy(&list_output.stdout);
    assert!(
        stdout.contains("removeme"),
        "Repository should exist before removal"
    );

    // Remove the repository with --force (skip confirmation)
    let remove_output = ctx.run_werx(&["repos", "remove", "owner/removeme", "--force"], &[]);

    assert_success(&remove_output);

    // Verify it's gone
    let list_output2 = ctx.run_werx(&["repos", "list"], &[]);

    let stdout2 = String::from_utf8_lossy(&list_output2.stdout);
    assert!(
        !stdout2.contains("removeme") || stdout2.contains("No repositories"),
        "Repository should be removed. Got: {}",
        stdout2
    );
}

#[test]
fn test_create_then_list_workflow() {
    let ctx = TestContext::new();
    ctx.init_werx();

    // Create a repository
    let output = ctx.run_werx(&["create", "acme/workflow-test"], &[]);

    assert_success(&output);

    // Verify the bare repo was created in .werx/repos/
    let bare_repo_path = ctx.werx_path().join(".werx/repos/workflow-test");
    assert!(
        bare_repo_path.exists(),
        "Bare repository should exist at {:?}",
        bare_repo_path
    );

    // Verify it's a bare repository (has HEAD file directly)
    let head_path = bare_repo_path.join("HEAD");
    assert!(head_path.exists(), "HEAD file should exist in bare repo");

    // Verify repos list shows it
    let list_output = ctx.run_werx(&["repos", "list"], &[]);
    assert_success(&list_output);

    let stdout = String::from_utf8_lossy(&list_output.stdout);
    assert!(stdout.contains("workflow-test"));

    // Verify workspace was also created
    let workspace_path = ctx.werx_path().join("workflow-test/main");
    assert!(
        workspace_path.exists(),
        "Workspace should exist at {:?}",
        workspace_path
    );
}
