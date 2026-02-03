mod common;

use common::{assert_failure, assert_success, run_forge};
use tempfile::TempDir;

// forge repos list tests

#[test]
fn test_repos_list_empty() {
    let temp_dir = TempDir::new().unwrap();
    let forge_path = temp_dir.path().join("empty-forge");

    // Initialize forge
    run_forge(
        &["init", forge_path.to_str().unwrap(), "--protocol", "https"],
        &[],
    );

    // List repos
    let output = run_forge(
        &["repos", "list"],
        &[("FORGE_DIR", forge_path.to_str().unwrap())],
    );

    assert_success(&output);

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("No repositories found") || stdout.contains("no repositories"));
}

#[test]
fn test_repos_list_json_format_empty() {
    let temp_dir = TempDir::new().unwrap();
    let forge_path = temp_dir.path().join("json-forge");

    // Initialize forge
    run_forge(
        &["init", forge_path.to_str().unwrap(), "--protocol", "https"],
        &[],
    );

    // List repos in JSON format (empty case returns text, not JSON)
    let output = run_forge(
        &["repos", "list", "--format", "json"],
        &[("FORGE_DIR", forge_path.to_str().unwrap())],
    );

    assert_success(&output);

    // Empty list still shows text message, not JSON
    // This is intentional UX decision in the code
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("No repositories found") || stdout.contains("no repositories"));
}

#[test]
fn test_repos_list_requires_forge() {
    let temp_dir = TempDir::new().unwrap();
    let non_forge_path = temp_dir.path().join("not-a-forge");

    let output = run_forge(
        &["repos", "list"],
        &[("FORGE_DIR", non_forge_path.to_str().unwrap())],
    );

    assert_failure(&output);

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("No Forge found") || stderr.contains("forge init"));
}

// forge repos add tests

#[test]
fn test_add_repo_requires_forge() {
    let temp_dir = TempDir::new().unwrap();
    let non_forge_path = temp_dir.path().join("not-a-forge");

    let output = run_forge(
        &["add", "owner/repo"],
        &[("FORGE_DIR", non_forge_path.to_str().unwrap())],
    );

    assert_failure(&output);

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("No Forge found") || stderr.contains("forge init"));
}

#[test]
fn test_repos_add_alias() {
    let temp_dir = TempDir::new().unwrap();
    let non_forge_path = temp_dir.path().join("not-a-forge");

    // Test 'repos add' produces same error as 'add'
    let output = run_forge(
        &["repos", "add", "owner/repo"],
        &[("FORGE_DIR", non_forge_path.to_str().unwrap())],
    );

    assert_failure(&output);

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("No Forge found") || stderr.contains("forge init"));
}

// forge repos remove tests

#[test]
fn test_remove_repo_requires_forge() {
    let temp_dir = TempDir::new().unwrap();
    let non_forge_path = temp_dir.path().join("not-a-forge");

    let output = run_forge(
        &["repos", "remove", "owner/repo", "--force"],
        &[("FORGE_DIR", non_forge_path.to_str().unwrap())],
    );

    assert_failure(&output);

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("No Forge found") || stderr.contains("forge init"));
}

// forge create tests

#[test]
fn test_create_repo_requires_forge() {
    let temp_dir = TempDir::new().unwrap();
    let non_forge_path = temp_dir.path().join("not-a-forge");

    let output = run_forge(
        &["create", "owner/repo"],
        &[("FORGE_DIR", non_forge_path.to_str().unwrap())],
    );

    assert_failure(&output);

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("No Forge found") || stderr.contains("forge init"));
}

#[test]
fn test_repos_create_alias() {
    let temp_dir = TempDir::new().unwrap();
    let non_forge_path = temp_dir.path().join("not-a-forge");

    // Test 'repos create' produces same error as 'create'
    let output = run_forge(
        &["repos", "create", "owner/repo"],
        &[("FORGE_DIR", non_forge_path.to_str().unwrap())],
    );

    assert_failure(&output);

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("No Forge found") || stderr.contains("forge init"));
}

#[test]
fn test_create_repo_invalid_format_no_slash() {
    let temp_dir = TempDir::new().unwrap();
    let forge_path = temp_dir.path().join("test-forge");

    // Initialize forge
    run_forge(
        &["init", forge_path.to_str().unwrap(), "--protocol", "https"],
        &[],
    );

    let output = run_forge(
        &["create", "invalidformat"],
        &[("FORGE_DIR", forge_path.to_str().unwrap())],
    );

    assert_failure(&output);

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Invalid repository specification")
            || stderr.contains("Expected format: owner/repo")
    );
}

#[test]
fn test_create_repo_invalid_format_special_chars() {
    let temp_dir = TempDir::new().unwrap();
    let forge_path = temp_dir.path().join("test-forge");

    // Initialize forge
    run_forge(
        &["init", forge_path.to_str().unwrap(), "--protocol", "https"],
        &[],
    );

    let output = run_forge(
        &["create", "owner@bad/repo"],
        &[("FORGE_DIR", forge_path.to_str().unwrap())],
    );

    assert_failure(&output);

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Invalid owner format"));
}

#[test]
fn test_create_repo_success() {
    let temp_dir = TempDir::new().unwrap();
    let forge_path = temp_dir.path().join("test-forge");

    // Initialize forge
    run_forge(
        &["init", forge_path.to_str().unwrap(), "--protocol", "https"],
        &[],
    );

    let output = run_forge(
        &["create", "mycompany/awesome-project"],
        &[("FORGE_DIR", forge_path.to_str().unwrap())],
    );

    assert_success(&output);

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Repository created successfully")
            || stdout.contains("mycompany/awesome-project")
    );

    // Verify the repository appears in list
    let list_output = run_forge(
        &["repos", "list"],
        &[("FORGE_DIR", forge_path.to_str().unwrap())],
    );

    assert_success(&list_output);

    let list_stdout = String::from_utf8_lossy(&list_output.stdout);
    assert!(list_stdout.contains("awesome-project"));
}

#[test]
fn test_create_repo_duplicate_detection() {
    let temp_dir = TempDir::new().unwrap();
    let forge_path = temp_dir.path().join("test-forge");

    // Initialize forge
    run_forge(
        &["init", forge_path.to_str().unwrap(), "--protocol", "https"],
        &[],
    );

    // Create first repository
    let output1 = run_forge(
        &["create", "owner/myrepo"],
        &[("FORGE_DIR", forge_path.to_str().unwrap())],
    );
    assert_success(&output1);

    // Try to create the same repository again
    let output2 = run_forge(
        &["create", "owner/myrepo"],
        &[("FORGE_DIR", forge_path.to_str().unwrap())],
    );

    assert_failure(&output2);

    let stderr = String::from_utf8_lossy(&output2.stderr);
    assert!(stderr.contains("already exists"));
}

#[test]
fn test_create_repo_creates_worktree() {
    let temp_dir = TempDir::new().unwrap();
    let forge_path = temp_dir.path().join("test-forge");

    // Initialize forge
    run_forge(
        &["init", forge_path.to_str().unwrap(), "--protocol", "https"],
        &[],
    );

    let output = run_forge(
        &["create", "testowner/testrepo"],
        &[("FORGE_DIR", forge_path.to_str().unwrap())],
    );

    assert_success(&output);

    // Verify worktree was created by checking the workspace directory exists
    let worktree_path = forge_path.join("testrepo").join("main");
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
    let temp_dir = TempDir::new().unwrap();
    let forge_path = temp_dir.path().join("test-forge");

    // Initialize forge
    run_forge(
        &["init", forge_path.to_str().unwrap(), "--protocol", "https"],
        &[],
    );

    // Create first repository with name "utils"
    let output1 = run_forge(
        &["create", "alice/utils"],
        &[("FORGE_DIR", forge_path.to_str().unwrap())],
    );
    assert_success(&output1);

    // Create second repository with same name but different owner
    let output2 = run_forge(
        &["create", "bob/utils"],
        &[("FORGE_DIR", forge_path.to_str().unwrap())],
    );
    assert_success(&output2);

    // Verify both exist with proper naming
    let list_output = run_forge(
        &["repos", "list"],
        &[("FORGE_DIR", forge_path.to_str().unwrap())],
    );
    assert_success(&list_output);

    let list_stdout = String::from_utf8_lossy(&list_output.stdout);
    // First one should be simple "utils"
    // Second one should be "bob-utils"
    assert!(list_stdout.contains("utils"));
    assert!(list_stdout.contains("bob-utils"));
}
