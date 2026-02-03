use std::process::Command;

mod common;

use common::{assert_success, run_werx};
use tempfile::TempDir;

#[test]
fn test_shell_init_bash_outputs_valid_code() {
    let output = Command::new("cargo")
        .args(&["run", "--", "shell", "init", "bash"])
        .output()
        .expect("Failed to execute werx shell init bash");

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("werx()"));
    assert!(stdout.contains("@werx:"));
    assert!(stdout.contains("change_directory"));
    assert!(stdout.contains("WERX_BIN"));
}

#[test]
fn test_shell_init_zsh_outputs_valid_code() {
    let output = Command::new("cargo")
        .args(&["run", "--", "shell", "init", "zsh"])
        .output()
        .expect("Failed to execute werx shell init zsh");

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("werx()"));
    assert!(stdout.contains("@werx:"));
    assert!(stdout.contains("change_directory"));
    assert!(stdout.contains("WERX_BIN"));
}

#[test]
fn test_shell_init_unsupported_shell() {
    let output = Command::new("cargo")
        .args(&["run", "--", "shell", "init", "fish"])
        .output()
        .expect("Failed to execute werx shell init fish");

    assert!(!output.status.success());

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Unsupported shell"));
}

// =============================================================================
// Navigation tests with real workspaces
// =============================================================================

#[test]
fn test_go_with_exact_match() {
    let temp_dir = TempDir::new().unwrap();
    let werx_path = temp_dir.path().join("go-exact-werx");

    // Initialize werx and create a repository (creates main workspace)
    run_werx(
        &["init", werx_path.to_str().unwrap(), "--protocol", "https"],
        &[],
    );

    let create_output = run_werx(
        &["create", "navtest/myrepo"],
        &[("WERX_DIR", werx_path.to_str().unwrap())],
    );
    assert_success(&create_output);

    // Use 'werx go' with an exact query that matches
    // When running non-interactively (no TTY), if there's an exact match it should emit the directive
    let output = run_werx(
        &["go", "myrepo/main"],
        &[("WERX_DIR", werx_path.to_str().unwrap())],
    );

    assert_success(&output);

    // The change_directory directive is emitted to stderr (by design)
    // so that the shell wrapper can intercept it while letting stdout pass through
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Should emit a change_directory directive with the path
    // The directive format is: @werx:change_directory:/path/to/workspace
    assert!(
        stderr.contains("@werx:change_directory:"),
        "Should emit change_directory directive to stderr. Got: {}",
        stderr
    );
}

// Note: Testing the interactive fuzzy search and full werx integration is challenging
// in integration tests as they require a TTY and full werx setup with git repositories.
// The core logic is tested in unit tests, and the interactive behavior should be tested manually.
