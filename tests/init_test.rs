mod common;

use common::{TestContext, assert_failure, assert_success};

#[test]
fn test_init_with_explicit_path() {
    let ctx = TestContext::new();

    let output = ctx.run_werx(&["init", ctx.werx_path_str(), "--protocol", "https"], &[]);

    assert_success(&output);

    // Verify directory structure was created
    assert!(ctx.werx_path().exists());
    assert!(ctx.werx_path().join(".werx").exists());
    assert!(ctx.werx_path().join(".werx/repos").exists());
    assert!(ctx.werx_path().join(".werx/config.toml").exists());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Werx initialized successfully"));
}

#[test]
fn test_init_with_werx_dir_env() {
    let ctx = TestContext::with_werx_subpath("env-werx");

    let output = ctx.run_werx(&["init", "--protocol", "ssh"], &[]);

    assert_success(&output);

    // Verify directory structure was created at WERX_DIR location
    assert!(ctx.werx_path().exists());
    assert!(ctx.werx_path().join(".werx").exists());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Werx initialized successfully"));
}

#[test]
fn test_init_command_arg_overrides_env() {
    let ctx = TestContext::new();
    let env_path = ctx.home().join("env-path");
    let arg_path = ctx.home().join("arg-path");

    let output = ctx.run_werx(
        &["init", arg_path.to_str().unwrap(), "--protocol", "https"],
        &[("WERX_DIR", env_path.to_str().unwrap())],
    );

    assert_success(&output);

    // Verify werx was created at arg_path, not env_path
    assert!(arg_path.exists());
    assert!(arg_path.join(".werx").exists());
    assert!(!env_path.exists());
}

#[test]
fn test_init_prevents_reinit_without_force() {
    let ctx = TestContext::new();

    // First init should succeed
    let output = ctx.run_werx(&["init", ctx.werx_path_str(), "--protocol", "https"], &[]);
    assert_success(&output);

    // Second init without --force should fail
    let output = ctx.run_werx(&["init", ctx.werx_path_str()], &[]);
    assert_failure(&output);

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("already exists") || stderr.contains("Werx already"));
}

#[test]
fn test_init_allows_reinit_with_force() {
    let ctx = TestContext::new();

    // First init
    let output = ctx.run_werx(&["init", ctx.werx_path_str(), "--protocol", "https"], &[]);
    assert_success(&output);

    // Second init with --force should succeed
    let output = ctx.run_werx(
        &["init", ctx.werx_path_str(), "--force", "--protocol", "ssh"],
        &[],
    );
    assert_success(&output);

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Werx initialized successfully"));
}

#[test]
fn test_init_creates_parent_directories() {
    let ctx = TestContext::new();
    let nested_path = ctx.home().join("parent/child/werx");

    let output = ctx.run_werx(
        &["init", nested_path.to_str().unwrap(), "--protocol", "https"],
        &[],
    );

    assert_success(&output);

    // Verify nested directories were created
    assert!(nested_path.exists());
    assert!(nested_path.join(".werx").exists());
}

#[test]
fn test_init_protocol_preference_saved() {
    let ctx = TestContext::new();

    // Init with SSH protocol
    let output = ctx.run_werx(&["init", ctx.werx_path_str(), "--protocol", "ssh"], &[]);
    assert_success(&output);

    // Verify config file contains protocol preference
    let config_path = ctx.werx_path().join(".werx/config.toml");
    assert!(config_path.exists());

    let config_content = std::fs::read_to_string(config_path).unwrap();
    assert!(config_content.contains("ssh") || config_content.contains("SSH"));
}
