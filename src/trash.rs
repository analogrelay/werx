use anyhow::{Context, Result, anyhow};
use std::path::Path;
use std::process::Command;

/// Move a branch to the trash namespace: `werx/trash/<original>/<date>`.
///
/// `date` must be in `YYYYMMDD` format and is supplied by the caller so that
/// tests can pass a fixed value without mocking system time.
///
/// Returns the final trash branch name (including any collision suffix).
pub fn branch_trash(repo_path: &Path, branch: &str, date: &str) -> Result<String> {
    // Verify the source branch exists by resolving its SHA.
    let sha = resolve_ref(repo_path, branch)?;

    // Compute the base trash name.
    let base_trash_name = format!("werx/trash/{}/{}", branch, date);

    // Find a unique trash name, appending -2, -3, … on collision.
    let trash_name = unique_trash_name(repo_path, &base_trash_name)?;

    // Create the trash ref pointing at the original SHA.
    create_ref(repo_path, &trash_name, &sha)?;

    // Delete the original branch ref.
    delete_branch(repo_path, branch)?;

    Ok(trash_name)
}

// ── helpers ──────────────────────────────────────────────────────────────────

fn resolve_ref(repo_path: &Path, branch: &str) -> Result<String> {
    let output = Command::new("git")
        .args(["-C", &repo_path.to_string_lossy()])
        .args(["rev-parse", &format!("refs/heads/{}", branch)])
        .output()
        .context("Failed to run git rev-parse")?;

    if !output.status.success() {
        return Err(anyhow!(
            "Branch '{}' not found in repository '{}'",
            branch,
            repo_path.display()
        ));
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn ref_exists(repo_path: &Path, ref_name: &str) -> Result<bool> {
    let output = Command::new("git")
        .args(["-C", &repo_path.to_string_lossy()])
        .args(["rev-parse", "--verify", &format!("refs/heads/{}", ref_name)])
        .output()
        .context("Failed to run git rev-parse --verify")?;

    Ok(output.status.success())
}

fn unique_trash_name(repo_path: &Path, base: &str) -> Result<String> {
    if !ref_exists(repo_path, base)? {
        return Ok(base.to_string());
    }
    let mut n = 2u32;
    loop {
        let candidate = format!("{}-{}", base, n);
        if !ref_exists(repo_path, &candidate)? {
            return Ok(candidate);
        }
        n += 1;
    }
}

fn create_ref(repo_path: &Path, ref_name: &str, sha: &str) -> Result<()> {
    let output = Command::new("git")
        .args(["-C", &repo_path.to_string_lossy()])
        .args(["update-ref", &format!("refs/heads/{}", ref_name), sha])
        .output()
        .context("Failed to run git update-ref")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("Failed to create trash ref '{}': {}", ref_name, stderr));
    }
    Ok(())
}

fn delete_branch(repo_path: &Path, branch: &str) -> Result<()> {
    // Use update-ref -d for reliable deletion in bare repos regardless of merge status
    let output = Command::new("git")
        .args(["-C", &repo_path.to_string_lossy()])
        .args(["update-ref", "-d", &format!("refs/heads/{}", branch)])
        .output()
        .context("Failed to run git update-ref -d")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("Failed to delete branch '{}': {}", branch, stderr));
    }
    Ok(())
}

// ── tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Command;
    use tempfile::TempDir;

    /// Create a bare git repo with a commit on `branch`.
    fn setup_bare_repo_with_branch(branch: &str) -> (TempDir, std::path::PathBuf) {
        let dir = TempDir::new().unwrap();
        let path = dir.path().to_path_buf();

        // Init bare repo
        Command::new("git")
            .args(["init", "--bare", &path.to_string_lossy()])
            .output()
            .unwrap();

        // Create commit via commit-tree
        let tree = Command::new("git")
            .args(["-C", &path.to_string_lossy()])
            .args(["hash-object", "-t", "tree", "/dev/null"])
            .output()
            .unwrap();
        let tree_sha = String::from_utf8_lossy(&tree.stdout).trim().to_string();

        let commit = Command::new("git")
            .args(["-C", &path.to_string_lossy()])
            .args(["commit-tree", &tree_sha, "-m", "init"])
            .output()
            .unwrap();
        let commit_sha = String::from_utf8_lossy(&commit.stdout).trim().to_string();

        // Create the branch
        Command::new("git")
            .args(["-C", &path.to_string_lossy()])
            .args(["update-ref", &format!("refs/heads/{}", branch), &commit_sha])
            .output()
            .unwrap();

        (dir, path)
    }

    #[test]
    fn test_simple_branch_name() {
        let (_dir, path) = setup_bare_repo_with_branch("my-feature");
        let result = branch_trash(&path, "my-feature", "20260302");
        assert!(result.is_ok(), "{:?}", result);
        let trash_name = result.unwrap();
        assert_eq!(trash_name, "werx/trash/my-feature/20260302");

        // Original should be gone
        assert!(!ref_exists(&path, "my-feature").unwrap());
        // Trash ref should exist
        assert!(ref_exists(&path, "werx/trash/my-feature/20260302").unwrap());
    }

    #[test]
    fn test_branch_name_with_slashes() {
        let (_dir, path) = setup_bare_repo_with_branch("feature/my-feature");
        let result = branch_trash(&path, "feature/my-feature", "20260302");
        assert!(result.is_ok(), "{:?}", result);
        let trash_name = result.unwrap();
        assert_eq!(trash_name, "werx/trash/feature/my-feature/20260302");

        assert!(!ref_exists(&path, "feature/my-feature").unwrap());
        assert!(ref_exists(&path, "werx/trash/feature/my-feature/20260302").unwrap());
    }

    #[test]
    fn test_single_collision() {
        let (_dir, path) = setup_bare_repo_with_branch("my-feature");

        // Pre-create the collision
        let sha = resolve_ref(&path, "my-feature").unwrap();
        create_ref(&path, "werx/trash/my-feature/20260302", &sha).unwrap();

        let result = branch_trash(&path, "my-feature", "20260302");
        assert!(result.is_ok(), "{:?}", result);
        let trash_name = result.unwrap();
        assert_eq!(trash_name, "werx/trash/my-feature/20260302-2");
    }

    #[test]
    fn test_multiple_collisions() {
        let (_dir, path) = setup_bare_repo_with_branch("my-feature");

        let sha = resolve_ref(&path, "my-feature").unwrap();
        create_ref(&path, "werx/trash/my-feature/20260302", &sha).unwrap();
        create_ref(&path, "werx/trash/my-feature/20260302-2", &sha).unwrap();

        let result = branch_trash(&path, "my-feature", "20260302");
        assert!(result.is_ok(), "{:?}", result);
        let trash_name = result.unwrap();
        assert_eq!(trash_name, "werx/trash/my-feature/20260302-3");
    }

    #[test]
    fn test_missing_source_branch() {
        let (_dir, path) = setup_bare_repo_with_branch("other-branch");
        let result = branch_trash(&path, "nonexistent", "20260302");
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("nonexistent"), "expected branch name in error: {}", err);
    }
}
