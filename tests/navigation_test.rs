use std::process::Command;

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

// Note: Testing the interactive fuzzy search and full werx integration is challenging
// in integration tests as they require a TTY and full werx setup with git repositories.
// The core logic is tested in unit tests, and the interactive behavior should be tested manually.
