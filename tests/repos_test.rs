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
        &[]
    );

    // List repos
    let output = run_forge(
        &["repos", "list"],
        &[("FORGE_DIR", forge_path.to_str().unwrap())]
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
        &[]
    );

    // List repos in JSON format (empty case returns text, not JSON)
    let output = run_forge(
        &["repos", "list", "--format", "json"],
        &[("FORGE_DIR", forge_path.to_str().unwrap())]
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
        &[("FORGE_DIR", non_forge_path.to_str().unwrap())]
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
        &[("FORGE_DIR", non_forge_path.to_str().unwrap())]
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
        &[("FORGE_DIR", non_forge_path.to_str().unwrap())]
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
        &[("FORGE_DIR", non_forge_path.to_str().unwrap())]
    );

    assert_failure(&output);

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("No Forge found") || stderr.contains("forge init"));
}
