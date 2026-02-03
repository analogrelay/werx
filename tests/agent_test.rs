mod common;

use common::{
    assert_failure, assert_success, cleanup_werx_agents_session, run_werx, tmux_available,
};
use tempfile::TempDir;

// =============================================================================
// Agent command tests
// =============================================================================

#[test]
fn test_agent_providers() {
    // This command doesn't require a werx or tmux, it just lists available providers
    let output = run_werx(&["agent", "providers"], &[]);

    assert_success(&output);

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should show available providers section
    assert!(
        stdout.contains("Available Agent Providers") || stdout.contains("Provider"),
        "Should show providers. Got: {}",
        stdout
    );

    // Should list at least one known agent type
    assert!(
        stdout.contains("OpenCode") || stdout.contains("Claude") || stdout.contains("Copilot"),
        "Should list known agent types. Got: {}",
        stdout
    );
}

#[test]
fn test_agent_list_empty() {
    let temp_dir = TempDir::new().unwrap();
    let werx_path = temp_dir.path().join("agent-empty-werx");

    // Initialize werx
    run_werx(
        &["init", werx_path.to_str().unwrap(), "--protocol", "https"],
        &[],
    );

    // Make sure we don't have any agents running from previous tests
    cleanup_werx_agents_session();

    // List agents when none are running
    let output = run_werx(
        &["agent", "list"],
        &[("WERX_DIR", werx_path.to_str().unwrap())],
    );

    assert_success(&output);

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("No agents") || stdout.contains("no agents"),
        "Should show no agents message. Got: {}",
        stdout
    );
}

#[test]
fn test_agent_list_requires_werx() {
    let temp_dir = TempDir::new().unwrap();
    let non_werx_path = temp_dir.path().join("not-a-werx");

    let output = run_werx(
        &["agent", "list"],
        &[("WERX_DIR", non_werx_path.to_str().unwrap())],
    );

    assert_failure(&output);

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("No Werx found") || stderr.contains("werx init"));
}

#[test]
fn test_agent_spawn_requires_werx() {
    let temp_dir = TempDir::new().unwrap();
    let non_werx_path = temp_dir.path().join("not-a-werx");

    let output = run_werx(
        &["agent", "spawn", "owner/repo", "-b", "main"],
        &[("WERX_DIR", non_werx_path.to_str().unwrap())],
    );

    assert_failure(&output);

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("No Werx found") || stderr.contains("werx init"));
}

#[test]
fn test_agent_kill_requires_werx() {
    let temp_dir = TempDir::new().unwrap();
    let non_werx_path = temp_dir.path().join("not-a-werx");

    let output = run_werx(
        &["agent", "kill", "some-agent"],
        &[("WERX_DIR", non_werx_path.to_str().unwrap())],
    );

    assert_failure(&output);

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("No Werx found") || stderr.contains("werx init"));
}

// =============================================================================
// Agent lifecycle test (requires tmux)
// =============================================================================

#[test]
fn test_agent_spawn_requires_tmux() {
    // This test verifies the error message when tmux is not available
    // We can't easily simulate "no tmux" but we can test the error path
    // by checking the spawn behavior

    if !tmux_available() {
        // If tmux is not available, verify that spawn fails with appropriate error
        let temp_dir = TempDir::new().unwrap();
        let werx_path = temp_dir.path().join("agent-no-tmux-werx");

        run_werx(
            &["init", werx_path.to_str().unwrap(), "--protocol", "https"],
            &[],
        );

        run_werx(
            &["create", "test/agentrepo"],
            &[("WERX_DIR", werx_path.to_str().unwrap())],
        );

        let output = run_werx(
            &["agent", "spawn", "test/agentrepo", "-b", "main"],
            &[("WERX_DIR", werx_path.to_str().unwrap())],
        );

        assert_failure(&output);

        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            stderr.contains("tmux") || stderr.contains("not found"),
            "Should mention tmux requirement. Got: {}",
            stderr
        );
    }
    // If tmux IS available, the full lifecycle test below will cover the spawn case
}

/// Full agent lifecycle test - only runs if tmux is available
/// This test spawns an agent, verifies it's in the list, and kills it
#[test]
fn test_agent_spawn_and_kill_workflow() {
    if !tmux_available() {
        eprintln!("Skipping test_agent_spawn_and_kill_workflow: tmux not available");
        return;
    }

    // Clean up any existing agent session from previous test runs
    cleanup_werx_agents_session();

    let temp_dir = TempDir::new().unwrap();
    let werx_path = temp_dir.path().join("agent-lifecycle-werx");

    // Initialize werx and create a repository
    run_werx(
        &["init", werx_path.to_str().unwrap(), "--protocol", "https"],
        &[],
    );

    run_werx(
        &["create", "lifecycle/testrepo"],
        &[("WERX_DIR", werx_path.to_str().unwrap())],
    );

    // Spawn an agent (using opencode by default, or whatever is available)
    // We need to specify a branch to avoid interactive prompts
    let spawn_output = run_werx(
        &["agent", "spawn", "lifecycle/testrepo", "-b", "main"],
        &[("WERX_DIR", werx_path.to_str().unwrap())],
    );

    // The spawn might fail if no agent provider is available, which is fine
    // We want to test the workflow if it succeeds
    if !spawn_output.status.success() {
        let stderr = String::from_utf8_lossy(&spawn_output.stderr);
        // If it failed because no provider is available, that's acceptable
        if stderr.contains("No available agent")
            || stderr.contains("not found")
            || stderr.contains("not available")
        {
            eprintln!(
                "Skipping agent lifecycle test: no agent provider available. Error: {}",
                stderr
            );
            return;
        }
        // Otherwise, it's an unexpected failure
        panic!(
            "Agent spawn failed unexpectedly: {}",
            String::from_utf8_lossy(&spawn_output.stderr)
        );
    }

    let stdout = String::from_utf8_lossy(&spawn_output.stdout);
    assert!(
        stdout.contains("Agent spawned") || stdout.contains("spawned successfully"),
        "Should show success message. Got: {}",
        stdout
    );

    // Extract agent name from output (format: "Name: <agent_name>")
    let agent_name = stdout
        .lines()
        .find(|line| line.contains("Name:"))
        .and_then(|line| line.split(':').nth(1))
        .map(|s| s.trim())
        .expect("Should find agent name in output");

    // Verify agent appears in list
    let list_output = run_werx(
        &["agent", "list"],
        &[("WERX_DIR", werx_path.to_str().unwrap())],
    );
    assert_success(&list_output);

    let list_stdout = String::from_utf8_lossy(&list_output.stdout);
    assert!(
        list_stdout.contains(agent_name),
        "Agent '{}' should appear in list. Got: {}",
        agent_name,
        list_stdout
    );

    // Kill the agent
    let kill_output = run_werx(
        &["agent", "kill", agent_name],
        &[("WERX_DIR", werx_path.to_str().unwrap())],
    );
    assert_success(&kill_output);

    let kill_stdout = String::from_utf8_lossy(&kill_output.stdout);
    assert!(
        kill_stdout.contains("terminated") || kill_stdout.contains("killed"),
        "Should show kill confirmation. Got: {}",
        kill_stdout
    );

    // Verify agent is no longer in list
    let list_output2 = run_werx(
        &["agent", "list"],
        &[("WERX_DIR", werx_path.to_str().unwrap())],
    );

    let list_stdout2 = String::from_utf8_lossy(&list_output2.stdout);
    assert!(
        !list_stdout2.contains(agent_name) || list_stdout2.contains("No agents"),
        "Killed agent should not appear in list. Got: {}",
        list_stdout2
    );

    // Final cleanup
    cleanup_werx_agents_session();
}

#[test]
fn test_agent_status_requires_werx() {
    let temp_dir = TempDir::new().unwrap();
    let non_werx_path = temp_dir.path().join("not-a-werx");

    let output = run_werx(
        &["agent", "status"],
        &[("WERX_DIR", non_werx_path.to_str().unwrap())],
    );

    assert_failure(&output);

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("No Werx found") || stderr.contains("werx init"));
}
