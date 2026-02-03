use std::process::Command;

/// Run a werx command with arguments and environment variables
pub fn run_werx(args: &[&str], env: &[(&str, &str)]) -> std::process::Output {
    let mut cmd = Command::new("cargo");
    cmd.args(&["run", "--"]);
    cmd.args(args);

    for (key, value) in env {
        cmd.env(key, value);
    }

    cmd.output().expect("Failed to execute werx command")
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
