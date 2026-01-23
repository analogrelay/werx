use std::process::Command;

#[test]
fn test_shell_init_bash_outputs_valid_code() {
    let output = Command::new("cargo")
        .args(&["run", "--", "shell", "init", "bash"])
        .output()
        .expect("Failed to execute forge shell init bash");

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("forge()"));
    assert!(stdout.contains("@forge:"));
    assert!(stdout.contains("change_directory"));
    assert!(stdout.contains("FORGE_BIN"));
}

#[test]
fn test_shell_init_zsh_outputs_valid_code() {
    let output = Command::new("cargo")
        .args(&["run", "--", "shell", "init", "zsh"])
        .output()
        .expect("Failed to execute forge shell init zsh");

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("forge()"));
    assert!(stdout.contains("@forge:"));
    assert!(stdout.contains("change_directory"));
    assert!(stdout.contains("FORGE_BIN"));
}

#[test]
fn test_shell_init_unsupported_shell() {
    let output = Command::new("cargo")
        .args(&["run", "--", "shell", "init", "fish"])
        .output()
        .expect("Failed to execute forge shell init fish");

    assert!(!output.status.success());

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Unsupported shell"));
}

// Note: Testing the interactive fuzzy search and full forge integration is challenging
// in integration tests as they require a TTY and full forge setup with git repositories.
// The core logic is tested in unit tests, and the interactive behavior should be tested manually.
