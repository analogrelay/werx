use std::path::Path;
use std::process::Command;

/// Run a werx command with arguments and environment variables
pub fn run_werx(args: &[&str], env: &[(&str, &str)]) -> std::process::Output {
    // Use the pre-built binary from cargo test, not `cargo run`
    let binary = env!("CARGO_BIN_EXE_werx");
    let mut cmd = Command::new(binary);
    cmd.args(args);

    for (key, value) in env {
        cmd.env(key, value);
    }

    cmd.output().expect("Failed to execute werx command")
}

/// Create a synthetic bare git repository for testing (no network required)
///
/// This creates a bare repository with an initial commit on the specified branch,
/// suitable for use with `werx add` without network access.
pub fn create_test_bare_repo(path: &Path, default_branch: &str) {
    // Create the bare repository
    let output = Command::new("git")
        .args(["init", "--bare", path.to_str().unwrap()])
        .output()
        .expect("Failed to create bare repository");

    if !output.status.success() {
        panic!(
            "Failed to init bare repo: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    // Set the default branch (HEAD)
    let output = Command::new("git")
        .args([
            "-C",
            path.to_str().unwrap(),
            "symbolic-ref",
            "HEAD",
            &format!("refs/heads/{}", default_branch),
        ])
        .output()
        .expect("Failed to set HEAD");

    if !output.status.success() {
        panic!(
            "Failed to set HEAD: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    // Create a temporary non-bare clone to make an initial commit
    let temp_clone = path.parent().unwrap().join("temp-clone");
    let output = Command::new("git")
        .args([
            "clone",
            path.to_str().unwrap(),
            temp_clone.to_str().unwrap(),
        ])
        .output()
        .expect("Failed to clone bare repo");

    if !output.status.success() {
        panic!(
            "Failed to clone: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    // Configure git user for the commit
    Command::new("git")
        .args([
            "-C",
            temp_clone.to_str().unwrap(),
            "config",
            "user.email",
            "test@test.com",
        ])
        .output()
        .expect("Failed to set git email");

    Command::new("git")
        .args([
            "-C",
            temp_clone.to_str().unwrap(),
            "config",
            "user.name",
            "Test User",
        ])
        .output()
        .expect("Failed to set git name");

    // Disable GPG signing for test commits
    Command::new("git")
        .args([
            "-C",
            temp_clone.to_str().unwrap(),
            "config",
            "commit.gpgsign",
            "false",
        ])
        .output()
        .expect("Failed to disable GPG signing");

    // Create an initial commit
    let readme_path = temp_clone.join("README.md");
    std::fs::write(&readme_path, "# Test Repository\n").expect("Failed to write README");

    Command::new("git")
        .args(["-C", temp_clone.to_str().unwrap(), "add", "."])
        .output()
        .expect("Failed to git add");

    let output = Command::new("git")
        .args([
            "-C",
            temp_clone.to_str().unwrap(),
            "commit",
            "-m",
            "Initial commit",
        ])
        .output()
        .expect("Failed to commit");

    if !output.status.success() {
        panic!(
            "Failed to commit: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    // Push to the bare repo
    let output = Command::new("git")
        .args([
            "-C",
            temp_clone.to_str().unwrap(),
            "push",
            "origin",
            default_branch,
        ])
        .output()
        .expect("Failed to push");

    if !output.status.success() {
        panic!(
            "Failed to push: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    // Clean up temp clone
    std::fs::remove_dir_all(&temp_clone).expect("Failed to remove temp clone");
}

/// Check if tmux is available on this system
pub fn tmux_available() -> bool {
    Command::new("tmux")
        .args(["-V"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Check if the werx-agents tmux session exists
pub fn werx_agents_session_exists() -> bool {
    Command::new("tmux")
        .args(["has-session", "-t", "werx-agents"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Kill the werx-agents tmux session if it exists (for test cleanup)
pub fn cleanup_werx_agents_session() {
    if werx_agents_session_exists() {
        let _ = Command::new("tmux")
            .args(["kill-session", "-t", "werx-agents"])
            .output();
    }
}

/// Assert that a command succeeded
pub fn assert_success(output: &std::process::Output) {
    if !output.status.success() {
        eprintln!("Command failed with status: {}", output.status);
        eprintln!("stdout: {}", String::from_utf8_lossy(&output.stdout));
        eprintln!("stderr: {}", String::from_utf8_lossy(&output.stderr));
        panic!("Command should have succeeded");
    }
}

/// Assert that a command failed
pub fn assert_failure(output: &std::process::Output) {
    if output.status.success() {
        eprintln!("Command succeeded unexpectedly");
        eprintln!("stdout: {}", String::from_utf8_lossy(&output.stdout));
        eprintln!("stderr: {}", String::from_utf8_lossy(&output.stderr));
        panic!("Command should have failed");
    }
}
