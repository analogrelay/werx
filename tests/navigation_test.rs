mod common;

use common::{TestContext, assert_success};

#[test]
fn test_shell_init_bash_outputs_valid_code() {
    let ctx = TestContext::new();

    let output = ctx.run_werx(&["shell", "init", "bash"], &[]);

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("werx()"));
    assert!(stdout.contains("@werx:"));
    assert!(stdout.contains("change_directory"));
    assert!(stdout.contains("WERX_BIN"));
}

#[test]
fn test_shell_init_zsh_outputs_valid_code() {
    let ctx = TestContext::new();

    let output = ctx.run_werx(&["shell", "init", "zsh"], &[]);

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("werx()"));
    assert!(stdout.contains("@werx:"));
    assert!(stdout.contains("change_directory"));
    assert!(stdout.contains("WERX_BIN"));
}

#[test]
fn test_shell_init_unsupported_shell() {
    let ctx = TestContext::new();

    let output = ctx.run_werx(&["shell", "init", "fish"], &[]);

    assert!(!output.status.success());

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Unsupported shell"));
}

// =============================================================================
// Navigation tests with real workspaces
// =============================================================================

#[test]
fn test_go_with_exact_match() {
    let ctx = TestContext::new();
    ctx.init_werx();

    let create_output = ctx.run_werx(&["create", "navtest/myrepo"], &[]);
    assert_success(&create_output);

    // Create a temp file to receive directives (simulating the shell hook)
    let directive_file = tempfile::NamedTempFile::new().expect("create temp directive file");
    let directive_path = directive_file.path().to_str().unwrap().to_string();

    // Use 'werx go' with an exact query that matches, passing the directive file
    let output = ctx.run_werx(
        &["go", "myrepo/main"],
        &[("WERX_DIRECTIVE_FILE", &directive_path)],
    );

    assert_success(&output);

    // The change_directory directive is written to WERX_DIRECTIVE_FILE
    let directives = std::fs::read_to_string(&directive_path).expect("read directive file");
    assert!(
        directives.contains("@werx:change_directory:"),
        "Should write change_directory directive to file. Got: {}",
        directives
    );
}

// Note: Testing the interactive fuzzy search and full werx integration is challenging
// in integration tests as they require a TTY and full werx setup with git repositories.
// The core logic is tested in unit tests, and the interactive behavior should be tested manually.
