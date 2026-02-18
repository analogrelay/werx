use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::TempDir;

/// Test context that provides an isolated environment for integration tests.
///
/// This struct creates a temporary HOME directory with git config set up,
/// ensuring tests don't affect the user's actual `~/werx` and that git
/// commands work in CI environments where git config may not be set.
pub struct TestContext {
    /// Temporary directory that serves as HOME for this test
    home_dir: TempDir,
    /// Path to the werx directory (inside the temp HOME or custom location)
    werx_path: PathBuf,
}

// Allow dead_code since not all tests use all methods
#[allow(dead_code)]
impl TestContext {
    /// Create a new test context with a temporary HOME directory.
    ///
    /// This sets up:
    /// - A temporary HOME directory
    /// - Git config with user.name, user.email, and commit.gpgsign=false
    /// - A werx directory path (can be customized with `with_werx_path`)
    pub fn new() -> Self {
        let home_dir = TempDir::new().expect("Failed to create temp HOME directory");

        // Set up git config in the temp home
        let gitconfig_path = home_dir.path().join(".gitconfig");
        std::fs::write(
            &gitconfig_path,
            r#"[user]
    name = Test User
    email = test@example.com
[commit]
    gpgsign = false
[init]
    defaultBranch = main
"#,
        )
        .expect("Failed to write .gitconfig");

        let werx_path = home_dir.path().join("werx");

        Self {
            home_dir,
            werx_path,
        }
    }

    /// Create a new test context with a custom werx path.
    ///
    /// The werx path will be relative to the temporary HOME directory.
    pub fn with_werx_subpath(subpath: &str) -> Self {
        let mut ctx = Self::new();
        ctx.werx_path = ctx.home_dir.path().join(subpath);
        ctx
    }

    /// Get the temporary HOME directory path.
    pub fn home(&self) -> &Path {
        self.home_dir.path()
    }

    /// Get the werx directory path.
    pub fn werx_path(&self) -> &Path {
        &self.werx_path
    }

    /// Get the werx path as a string (for passing to commands).
    pub fn werx_path_str(&self) -> &str {
        self.werx_path
            .to_str()
            .expect("werx path should be valid UTF-8")
    }

    /// Run a werx command with the test environment.
    ///
    /// This automatically sets:
    /// - HOME to the temporary directory
    /// - WERX_DIR to the werx path (unless overridden in extra_env)
    pub fn run_werx(&self, args: &[&str], extra_env: &[(&str, &str)]) -> std::process::Output {
        let binary = env!("CARGO_BIN_EXE_werx");
        let mut cmd = Command::new(binary);
        cmd.args(args);

        // Set HOME to our temp directory
        cmd.env("HOME", self.home_dir.path());

        // Set WERX_DIR unless overridden
        let has_werx_dir = extra_env.iter().any(|(k, _)| *k == "WERX_DIR");
        if !has_werx_dir {
            cmd.env("WERX_DIR", &self.werx_path);
        }

        // Add any extra environment variables
        for (key, value) in extra_env {
            cmd.env(key, value);
        }

        cmd.output().expect("Failed to execute werx command")
    }

    /// Run a git command with the test environment.
    ///
    /// This automatically sets HOME to the temporary directory so git
    /// picks up our test .gitconfig.
    pub fn run_git(&self, args: &[&str]) -> std::process::Output {
        let mut cmd = Command::new("git");
        cmd.args(args);
        cmd.env("HOME", self.home_dir.path());
        cmd.output().expect("Failed to execute git command")
    }

    /// Run a git command in a specific directory with the test environment.
    pub fn run_git_in(&self, dir: &Path, args: &[&str]) -> std::process::Output {
        let mut cmd = Command::new("git");
        cmd.args(args);
        cmd.current_dir(dir);
        cmd.env("HOME", self.home_dir.path());
        cmd.output().expect("Failed to execute git command")
    }

    /// Initialize werx in this test context.
    ///
    /// Convenience method that runs `werx init` with the werx path and HTTPS protocol.
    pub fn init_werx(&self) -> std::process::Output {
        self.run_werx(&["init", self.werx_path_str(), "--protocol", "https"], &[])
    }

    /// Initialize werx with SSH protocol.
    pub fn init_werx_ssh(&self) -> std::process::Output {
        self.run_werx(&["init", self.werx_path_str(), "--protocol", "ssh"], &[])
    }

    /// Create a repository in this test context.
    ///
    /// Convenience method that runs `werx create` with the given owner/repo.
    pub fn create_repo(&self, owner_repo: &str) -> std::process::Output {
        self.run_werx(&["create", owner_repo], &[])
    }
}

impl Default for TestContext {
    fn default() -> Self {
        Self::new()
    }
}

// Legacy functions for backwards compatibility during migration

/// Run a werx command with arguments and environment variables.
///
/// **Deprecated**: Use `TestContext::run_werx` instead for proper test isolation.
///
/// This function still works but doesn't provide HOME isolation, which means:
/// - Tests may affect the user's actual `~/werx`
/// - Git commands may fail in CI without proper git config
#[allow(dead_code)]
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
///
/// **Note**: This function sets git config per-repository. For new tests,
/// prefer using `TestContext` which sets up a global git config.
#[allow(dead_code)]
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
#[allow(dead_code)]
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
#[allow(dead_code)]
pub fn assert_failure(output: &std::process::Output) {
    if output.status.success() {
        eprintln!("Command succeeded unexpectedly");
        eprintln!("stdout: {}", String::from_utf8_lossy(&output.stdout));
        eprintln!("stderr: {}", String::from_utf8_lossy(&output.stderr));
        panic!("Command should have failed");
    }
}
