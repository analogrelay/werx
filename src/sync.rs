use anyhow::{Context, Result, anyhow};
use console::style;
use dialoguer::Confirm;
use dialoguer::theme::ColorfulTheme;
use rayon::prelude::*;
use std::fmt::Write as FmtWrite;
use std::io::IsTerminal;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Mutex;

use crate::{AppContext, Werx, cmd, repo_meta::RepoGithubMeta, repos};
use crate::reporter::OperationHandle;
use crate::trash::branch_trash;

// ── Plan data model (task 4.x) ────────────────────────────────────────────────

/// A planned action for a single branch.
#[derive(Debug, Clone)]
pub enum BranchAction {
    FastForward {
        branch: String,
        from_sha: String,
        to_sha: String,
    },
    /// Fast-forward local branch from `upstream/<branch>`, then push to origin.
    FastForwardFromUpstream {
        branch: String,
        to_sha: String,
    },
    Rebase {
        branch: String,
        onto_sha: String,
    },
    Push {
        branch: String,
        remote: String,
    },
    Trash {
        branch: String,
        reason: String,
    },
    Skip {
        branch: String,
        reason: String,
    },
}

/// Planned actions for a single repository.
#[derive(Debug, Clone)]
pub struct RepoPlan {
    pub repo: String,
    pub actions: Vec<BranchAction>,
}

/// The complete sync plan across all repositories.
#[derive(Debug, Clone)]
pub struct SyncPlan {
    pub repos: Vec<RepoPlan>,
}

impl SyncPlan {
    /// Returns true if the plan contains any mutating actions.
    pub fn has_mutations(&self) -> bool {
        self.repos.iter().any(|r| {
            r.actions.iter().any(|a| !matches!(a, BranchAction::Skip { .. }))
        })
    }

    /// Returns all skipped actions with their repo name.
    pub fn skipped_actions(&self) -> Vec<(&str, &BranchAction)> {
        let mut result = Vec::new();
        for repo in &self.repos {
            for action in &repo.actions {
                if matches!(action, BranchAction::Skip { .. }) {
                    result.push((repo.repo.as_str(), action));
                }
            }
        }
        result
    }
}

/// Render a `SyncPlan` as a human-readable multi-line string.
pub fn format_plan(plan: &SyncPlan) -> String {
    let mut out = String::new();
    if plan.repos.is_empty() {
        let _ = writeln!(out, "Nothing to sync.");
        return out;
    }
    for repo in &plan.repos {
        let _ = writeln!(out, "  {}:", style(&repo.repo).cyan().bold());
        if repo.actions.is_empty() {
            let _ = writeln!(out, "    (up to date)");
            continue;
        }
        for action in &repo.actions {
            match action {
                BranchAction::FastForward { branch, from_sha, to_sha } => {
                    let _ = writeln!(
                        out,
                        "    {} {} {}..{}",
                        style("^").green(),
                        branch,
                        &from_sha[..8.min(from_sha.len())],
                        &to_sha[..8.min(to_sha.len())]
                    );
                }
                BranchAction::FastForwardFromUpstream { branch, to_sha } => {
                    let _ = writeln!(
                        out,
                        "    {} {} (from upstream) ..{}",
                        style("^").green(),
                        branch,
                        &to_sha[..8.min(to_sha.len())]
                    );
                }
                BranchAction::Rebase { branch, onto_sha } => {
                    let _ = writeln!(
                        out,
                        "    {} {} rebase onto {}",
                        style("~").yellow(),
                        branch,
                        &onto_sha[..8.min(onto_sha.len())]
                    );
                }
                BranchAction::Push { branch, remote } => {
                    let _ = writeln!(
                        out,
                        "    {} {} -> {}",
                        style("->").blue(),
                        branch,
                        remote
                    );
                }
                BranchAction::Trash { branch, reason } => {
                    let _ = writeln!(
                        out,
                        "    {} {} ({})",
                        style("trash").yellow(),
                        branch,
                        reason
                    );
                }
                BranchAction::Skip { branch, reason } => {
                    let _ = writeln!(
                        out,
                        "    {} {} ({})",
                        style("~").dim(),
                        branch,
                        reason
                    );
                }
            }
        }
    }
    out
}

// ── Worktree info (task 5.2) ──────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct WorktreeInfo {
    /// The branch name checked out in this worktree (None if detached HEAD).
    pub branch: Option<String>,
    /// The filesystem path of the worktree.
    pub path: PathBuf,
    /// Whether the worktree has uncommitted changes.
    pub dirty: bool,
}

/// List all worktrees for a bare repository.
pub fn list_worktrees(repo_path: &Path) -> Result<Vec<WorktreeInfo>> {
    let output = cmd::run(Command::new("git")
        .args(["-C", &repo_path.to_string_lossy()])
        .args(["worktree", "list", "--porcelain"]))
        .context("Failed to run git worktree list")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("git worktree list failed: {}", stderr));
    }

    let text = String::from_utf8_lossy(&output.stdout);
    let mut infos: Vec<WorktreeInfo> = Vec::new();
    let mut current_path: Option<PathBuf> = None;
    let mut current_branch: Option<String> = None;

    for line in text.lines() {
        if line.starts_with("worktree ") {
            // Flush the previous entry
            if let Some(path) = current_path.take() {
                let dirty = check_worktree_dirty(&path);
                infos.push(WorktreeInfo {
                    branch: current_branch.take(),
                    path,
                    dirty,
                });
            }
            current_path = Some(PathBuf::from(line.trim_start_matches("worktree ")));
            current_branch = None;
        } else if let Some(branch_ref) = line.strip_prefix("branch ") {
            // branch refs/heads/main → "main"
            current_branch = Some(
                branch_ref
                    .strip_prefix("refs/heads/")
                    .unwrap_or(branch_ref)
                    .to_string(),
            );
        }
    }
    // Flush final entry
    if let Some(path) = current_path {
        let dirty = check_worktree_dirty(&path);
        infos.push(WorktreeInfo {
            branch: current_branch,
            path,
            dirty,
        });
    }

    Ok(infos)
}

fn check_worktree_dirty(worktree_path: &Path) -> bool {
    if !worktree_path.exists() {
        return false;
    }
    let output = Command::new("git")
        .args(["-C", &worktree_path.to_string_lossy()])
        .args(["status", "--porcelain"])
        .output();
    match output {
        Ok(o) if o.status.success() => !o.stdout.is_empty(),
        _ => false,
    }
}

// ── Plan phase — Fetch (task 5.1) ─────────────────────────────────────────────

/// Fetch from all configured remotes for a repository.
/// Silently skips remotes that don't exist; propagates other errors.
pub fn fetch_repo(repo_path: &Path, remotes: &[String], handle: &OperationHandle) -> Result<()> {
    for remote in remotes {
        tracing::debug!("fetching remote '{}'", remote);
        let output = cmd::run_with_reporter(
            Command::new("git")
                .args(["-C", &repo_path.to_string_lossy()])
                .args(["fetch", "--tags", remote]),
            handle,
        )
        .with_context(|| format!("Failed to run git fetch for remote '{}'", remote))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            // Silently skip "no such remote" type errors
            if is_no_such_remote_error(&stderr) {
                continue;
            }
            return Err(anyhow!("git fetch {} failed: {}", remote, stderr));
        }
    }
    Ok(())
}

fn is_no_such_remote_error(stderr: &str) -> bool {
    stderr.contains("No such remote")
        || stderr.contains("no such remote")
        || stderr.contains("fatal: '") && stderr.contains("' does not appear to be a git repository")
        || stderr.contains("error: No such remote")
}

// ── Plan phase — Branch analysis (tasks 6.x) ──────────────────────────────────

#[derive(Debug, Clone)]
pub struct BranchInfo {
    /// Local branch name (e.g., "main")
    pub name: String,
    /// Local SHA
    pub local_sha: String,
    /// Upstream remote tracking ref (e.g., "refs/remotes/origin/main")
    pub upstream_ref: Option<String>,
    /// Upstream SHA (None if upstream ref doesn't resolve)
    pub upstream_sha: Option<String>,
    /// The remote name extracted from the upstream ref (e.g., "origin")
    pub upstream_remote: Option<String>,
}

/// List all local branches with their upstream tracking information.
pub fn list_branches_with_upstreams(repo_path: &Path) -> Result<Vec<BranchInfo>> {
    // Use for-each-ref to get branch name, sha, and upstream ref in one shot
    let output = cmd::run(Command::new("git")
        .args(["-C", &repo_path.to_string_lossy()])
        .args([
            "for-each-ref",
            "--format=%(refname:short) %(objectname) %(upstream) %(upstream:short)",
            "refs/heads/",
        ]))
        .context("Failed to run git for-each-ref")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("git for-each-ref failed: {}", stderr));
    }

    let text = String::from_utf8_lossy(&output.stdout);
    let mut branches = Vec::new();

    for line in text.lines() {
        let parts: Vec<&str> = line.splitn(4, ' ').collect();
        if parts.len() < 2 {
            continue;
        }
        let name = parts[0].to_string();
        let local_sha = parts[1].to_string();
        let upstream_ref = if parts.len() >= 3 && !parts[2].is_empty() {
            Some(parts[2].to_string())
        } else {
            None
        };

        // Extract remote name from upstream_short (e.g., "origin/main" → "origin")
        let upstream_remote = upstream_ref.as_ref().and_then(|uref| {
            // upstream ref is like refs/remotes/origin/main
            uref.strip_prefix("refs/remotes/")
                .and_then(|s| s.splitn(2, '/').next())
                .map(|s| s.to_string())
        });

        // Resolve upstream SHA
        let upstream_sha = if let Some(ref uref) = upstream_ref {
            resolve_sha(repo_path, uref).ok()
        } else {
            None
        };

        branches.push(BranchInfo {
            name,
            local_sha,
            upstream_ref,
            upstream_sha,
            upstream_remote,
        });
    }

    Ok(branches)
}

fn resolve_sha(repo_path: &Path, ref_name: &str) -> Result<String> {
    resolve_sha_pub(repo_path, ref_name)
}

/// Public re-export of SHA resolution for use by workspace helpers.
pub fn resolve_sha_pub(repo_path: &Path, ref_name: &str) -> Result<String> {
    let output = cmd::run(Command::new("git")
        .args(["-C", &repo_path.to_string_lossy()])
        .args(["rev-parse", ref_name]))
        .context("Failed to run git rev-parse")?;

    if !output.status.success() {
        return Err(anyhow!("ref '{}' not found", ref_name));
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

/// Check if `ancestor` is an ancestor of `descendant`.
pub fn is_ancestor(repo_path: &Path, ancestor: &str, descendant: &str) -> bool {
    Command::new("git")
        .args(["-C", &repo_path.to_string_lossy()])
        .args(["merge-base", "--is-ancestor", ancestor, descendant])
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

// ── Build the repo plan (task 6.7) ─────────────────────────────────────────────

/// Build the complete plan for a single repository.
pub fn build_repo_plan(
    repo_path: &Path,
    repo_name: &str,
    remotes: &[String],
    handle: &OperationHandle,
) -> Result<RepoPlan> {
    // 1. Fetch
    fetch_repo(repo_path, remotes, handle)?;

    // 2. Load optional GitHub fork metadata (task 8.1)
    let fork_meta = RepoGithubMeta::load(repo_path)
        .unwrap_or_else(|e| {
            tracing::warn!("Failed to load fork metadata for '{}': {}", repo_name, e);
            None
        });

    // 3. If fork, check whether `upstream` remote is present
    let upstream_remote_present = fork_meta.as_ref().map_or(false, |_| {
        check_remote_exists(repo_path, "upstream")
    });

    if fork_meta.is_some() && !upstream_remote_present {
        tracing::warn!(
            "Fork metadata present for '{}' but `upstream` remote is missing; \
             falling back to normal origin sync",
            repo_name
        );
        // task 8.6: warn and fall back
    }

    // 4. List worktrees
    let worktrees = list_worktrees(repo_path)?;

    // 5. List branches with upstreams
    let branches = list_branches_with_upstreams(repo_path)?;

    let mut actions: Vec<BranchAction> = Vec::new();

    for branch in &branches {
        // Active-worktree guard
        let active_wt = worktrees.iter().find(|wt| wt.branch.as_deref() == Some(&branch.name));

        // For fork repos with a functioning upstream remote, check `upstream/<branch>` (task 8.2)
        if fork_meta.is_some() && upstream_remote_present {
            let upstream_ref = format!("refs/remotes/upstream/{}", branch.name);
            if let Ok(upstream_sha) = resolve_sha(repo_path, &upstream_ref) {
                // upstream/<branch> exists (task 8.2)
                if is_ancestor(repo_path, &branch.local_sha, &upstream_sha) {
                    if branch.local_sha == upstream_sha {
                        // Already in sync with upstream — fall through to normal push logic
                    } else {
                        // Local is behind upstream — fast-forward from upstream (task 8.3)
                        actions.push(BranchAction::FastForwardFromUpstream {
                            branch: branch.name.clone(),
                            to_sha: upstream_sha,
                        });
                        continue;
                    }
                } else if !is_ancestor(repo_path, &upstream_sha, &branch.local_sha) {
                    // Diverged — skip this branch entirely (task 8.4)
                    actions.push(BranchAction::Skip {
                        branch: branch.name.clone(),
                        reason: "diverged from upstream — needs manual rebase".to_string(),
                    });
                    continue;
                }
                // else: upstream is ancestor of local (local is ahead) — fall through to normal logic
            }
            // No upstream/<branch> ref → fall through to normal sync logic (task 8.7)
        }

        // Normal origin-based sync logic
        match &branch.upstream_sha {
            None if branch.upstream_ref.is_some() => {
                // upstream ref existed in config but no longer resolves → stale
                if let Some(wt) = active_wt {
                    let reason = if wt.dirty {
                        "active worktree (dirty)".to_string()
                    } else {
                        "active worktree".to_string()
                    };
                    actions.push(BranchAction::Skip {
                        branch: branch.name.clone(),
                        reason,
                    });
                } else {
                    actions.push(BranchAction::Trash {
                        branch: branch.name.clone(),
                        reason: "upstream branch deleted".to_string(),
                    });
                }
            }
            Some(upstream_sha) => {
                if branch.local_sha == *upstream_sha {
                    // Already in sync, nothing to do
                } else if is_ancestor(repo_path, &branch.local_sha, upstream_sha) {
                    if let Some(wt) = active_wt {
                        if wt.dirty {
                            actions.push(BranchAction::Skip {
                                branch: branch.name.clone(),
                                reason: "active worktree (dirty)".to_string(),
                            });
                            continue;
                        }
                    }
                    actions.push(BranchAction::FastForward {
                        branch: branch.name.clone(),
                        from_sha: branch.local_sha.clone(),
                        to_sha: upstream_sha.clone(),
                    });
                } else if is_ancestor(repo_path, upstream_sha, &branch.local_sha) {
                    if let Some(remote) = &branch.upstream_remote {
                        actions.push(BranchAction::Push {
                            branch: branch.name.clone(),
                            remote: remote.clone(),
                        });
                    }
                } else {
                    // Diverged
                    if let Some(wt) = active_wt {
                        let reason = if wt.dirty {
                            "active worktree (dirty)".to_string()
                        } else {
                            "active worktree".to_string()
                        };
                        actions.push(BranchAction::Skip {
                            branch: branch.name.clone(),
                            reason,
                        });
                    } else {
                        actions.push(BranchAction::Rebase {
                            branch: branch.name.clone(),
                            onto_sha: upstream_sha.clone(),
                        });
                    }
                }
            }
            None => {
                // No upstream configured — nothing to do
            }
        }
    }

    Ok(RepoPlan {
        repo: repo_name.to_string(),
        actions,
    })
}

/// Returns true if the given remote name is configured in the repository.
fn check_remote_exists(repo_path: &Path, remote: &str) -> bool {
    Command::new("git")
        .args(["-C", &repo_path.to_string_lossy()])
        .args(["remote", "get-url", remote])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

// ── Execute phase — Fast Forward (task 7.x) ────────────────────────────────────

/// Apply a fast-forward via git update-ref.
pub fn apply_fast_forward(repo_path: &Path, branch: &str, to_sha: &str) -> Result<()> {
    tracing::debug!("fast-forwarding branch '{}' to {}", branch, &to_sha[..8.min(to_sha.len())]);
    let output = cmd::run(Command::new("git")
        .args(["-C", &repo_path.to_string_lossy()])
        .args(["update-ref", &format!("refs/heads/{}", branch), to_sha]))
        .context("Failed to run git update-ref")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!(
            "Fast-forward of '{}' failed: {}",
            branch,
            stderr
        ));
    }
    Ok(())
}

// ── Execute phase — Rebase via temporary worktree (tasks 8.x) ─────────────────

/// Outcome of a rebase attempt.
#[derive(Debug)]
pub enum RebaseOutcome {
    Success(String), // new sha after rebase
    Conflict,
}

/// RAII wrapper for a temporary git worktree.
pub struct TempWorktree {
    repo_path: PathBuf,
    wt_path: tempfile::TempDir,
}

impl TempWorktree {
    /// Create a temporary detached worktree at `branch_sha`.
    pub fn create(repo_path: &Path, branch_sha: &str) -> Result<Self> {
        let wt_dir = tempfile::TempDir::new().context("Failed to create temp directory")?;

        let output = cmd::run(Command::new("git")
            .args(["-C", &repo_path.to_string_lossy()])
            .args(["worktree", "add", "--detach", &wt_dir.path().to_string_lossy(), branch_sha]))
            .context("Failed to run git worktree add")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!("Failed to create temp worktree: {}", stderr));
        }

        Ok(TempWorktree {
            repo_path: repo_path.to_path_buf(),
            wt_path: wt_dir,
        })
    }

    pub fn path(&self) -> &Path {
        self.wt_path.path()
    }
}

impl Drop for TempWorktree {
    fn drop(&mut self) {
        let _ = Command::new("git")
            .args(["-C", &self.repo_path.to_string_lossy()])
            .args(["worktree", "remove", "--force", &self.wt_path.path().to_string_lossy()])
            .output();
    }
}

/// Perform a rebase in a temporary worktree.
pub fn apply_rebase(repo_path: &Path, branch: &str, onto_sha: &str) -> Result<RebaseOutcome> {
    // Get the branch's current SHA to use as the worktree base
    let branch_sha = resolve_sha(repo_path, &format!("refs/heads/{}", branch))
        .with_context(|| format!("Branch '{}' not found", branch))?;

    let wt = TempWorktree::create(repo_path, &branch_sha)?;

    // Attempt the rebase
    tracing::debug!("rebasing branch '{}' onto {}", branch, &onto_sha[..8.min(onto_sha.len())]);
    let rebase_output = cmd::run(Command::new("git")
        .args(["-C", &wt.path().to_string_lossy()])
        .args(["rebase", onto_sha]))
        .context("Failed to run git rebase")?;

    if rebase_output.status.success() {
        // Read the new HEAD
        let new_sha_output = cmd::run(Command::new("git")
            .args(["-C", &wt.path().to_string_lossy()])
            .args(["rev-parse", "HEAD"]))
            .context("Failed to read new HEAD after rebase")?;

        let new_sha = String::from_utf8_lossy(&new_sha_output.stdout)
            .trim()
            .to_string();

        // Advance the bare-repo branch ref
        apply_fast_forward(repo_path, branch, &new_sha)?;

        Ok(RebaseOutcome::Success(new_sha))
    } else {
        // Abort the rebase
        let _ = Command::new("git")
            .args(["-C", &wt.path().to_string_lossy()])
            .args(["rebase", "--abort"])
            .output();

        Ok(RebaseOutcome::Conflict)
    }
    // TempWorktree is dropped here, cleaning up the worktree
}

// ── Execute phase — Push (tasks 9.x) ──────────────────────────────────────────

/// Outcome of a push attempt.
#[derive(Debug)]
pub enum PushOutcome {
    Pushed,
    UpToDate,
    ForcePushRequired,
    NoUpstream,
}

/// Push a branch to its tracking remote.
pub fn push_branch(repo_path: &Path, branch: &str, remote: &str) -> Result<PushOutcome> {
    tracing::debug!("pushing branch '{}' to remote '{}'", branch, remote);
    let output = cmd::run(Command::new("git")
        .args(["-C", &repo_path.to_string_lossy()])
        .args(["push", remote, branch]))
        .context("Failed to run git push")?;

    if output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("Everything up-to-date") || stderr.contains("up to date") {
            return Ok(PushOutcome::UpToDate);
        }
        return Ok(PushOutcome::Pushed);
    }

    let stderr = String::from_utf8_lossy(&output.stderr);
    if stderr.contains("rejected")
        && (stderr.contains("non-fast-forward") || stderr.contains("stale info"))
    {
        return Ok(PushOutcome::ForcePushRequired);
    }
    if stderr.contains("has no upstream") || stderr.contains("no upstream configured") {
        return Ok(PushOutcome::NoUpstream);
    }

    Err(anyhow!("git push failed: {}", stderr))
}

// ── Execute phase — Trash Stale Branches (task 10.x) ──────────────────────────

/// Trash a stale branch (whose upstream has been deleted).
pub fn trash_stale_branch(repo_path: &Path, branch: &str, date: &str) -> Result<String> {
    branch_trash(repo_path, branch, date)
}

// ── Plan → Confirm → Execute orchestration (tasks 11.x) ──────────────────────

/// Represents the result of executing a single action.
#[derive(Debug)]
enum ActionOutcome {
    Done(String),
    Skipped(String),
}

/// Top-level sync entry point.
pub fn run_sync(
    werx: &Werx,
    repospec: Option<&str>,
    dry_run: bool,
    no_confirm: bool,
    ctx: &AppContext,
) -> Result<()> {
    let config = werx.load_config()?;
    let remotes: Vec<String> = config.sync_remotes().to_vec();

    // Resolve the repos to sync
    let all_repos = repos::list_repos(werx)?;
    let repos_to_sync: Vec<_> = if let Some(spec) = repospec {
        let repo = all_repos
            .iter()
            .find(|r| r.dir_name == spec || r.clone_url.contains(spec))
            .ok_or_else(|| anyhow!("Repository not found: {}", spec))?;
        vec![repo.clone()]
    } else {
        all_repos.into_iter().filter(|r| r.valid).collect()
    };

    if repos_to_sync.is_empty() {
        ctx.reporter.println("No repositories to sync.");
        return Ok(());
    }

    tracing::info!("Planning sync for {} repositories", repos_to_sync.len());
    ctx.reporter.println(&format!(
        "Planning sync for {} repositories...",
        repos_to_sync.len()
    ));

    // ── Plan phase (parallelized) ─────────────────────────────────────────────
    let plan_errors: Mutex<Vec<(String, anyhow::Error)>> = Mutex::new(Vec::new());
    let plans: Mutex<Vec<RepoPlan>> = Mutex::new(Vec::new());

    // Create a spinner handle per repo before going parallel
    let fetch_handles: Vec<OperationHandle> = repos_to_sync
        .iter()
        .map(|r| ctx.reporter.start_operation(&format!("Fetching {}", r.dir_name)))
        .collect();

    repos_to_sync
        .par_iter()
        .zip(fetch_handles.par_iter())
        .for_each(|(repo, handle)| {
            let repo_path = werx.repos_dir().join(&repo.dir_name);
            match build_repo_plan(&repo_path, &repo.dir_name, &remotes, handle) {
                Ok(plan) => {
                    handle.finish_ok(&repo.dir_name);
                    plans.lock().unwrap().push(plan);
                }
                Err(e) => {
                    handle.finish_err(&format!("{}: {}", repo.dir_name, e));
                    plan_errors.lock().unwrap().push((repo.dir_name.clone(), e));
                }
            }
        });

    let mut plans = plans.into_inner().unwrap();
    // Sort for stable output
    plans.sort_by(|a, b| a.repo.cmp(&b.repo));
    let plan_errors = plan_errors.into_inner().unwrap();

    // Report planning errors
    for (repo, err) in &plan_errors {
        tracing::warn!("Failed to plan '{}': {}", repo, err);
    }

    let sync_plan = SyncPlan { repos: plans };

    // ── Present plan ─────────────────────────────────────────────────────────
    ctx.reporter.println("");
    ctx.reporter.println(&format!("{}:", style("Sync Plan").bold()));
    ctx.reporter.println("");
    ctx.reporter.println(&format_plan(&sync_plan));

    // ── Dry-run early exit ────────────────────────────────────────────────────
    if dry_run {
        ctx.reporter
            .println("(dry-run mode — no changes applied)");
        return Ok(());
    }

    if !sync_plan.has_mutations() {
        ctx.reporter
            .println("Nothing to do — all repositories are up to date.");
        return Ok(());
    }

    // ── Confirmation prompt ───────────────────────────────────────────────────
    let is_tty = std::io::stdout().is_terminal();
    let should_confirm = !no_confirm && is_tty;

    if should_confirm {
        let confirmed = Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt("Proceed with sync?")
            .default(false)
            .interact()?;

        if !confirmed {
            ctx.reporter.println("Sync cancelled.");
            return Ok(());
        }
    }

    // ── Execute phase (parallelized) ──────────────────────────────────────────
    let today = chrono_today();

    let exec_errors: Mutex<Vec<(String, anyhow::Error)>> = Mutex::new(Vec::new());
    let all_outcomes: Mutex<Vec<(String, String, ActionOutcome)>> = Mutex::new(Vec::new());

    // Create a spinner handle per repo before going parallel
    let exec_handles: Vec<OperationHandle> = sync_plan
        .repos
        .iter()
        .map(|r| ctx.reporter.start_operation(&format!("Syncing {}", r.repo)))
        .collect();

    sync_plan
        .repos
        .par_iter()
        .zip(exec_handles.par_iter())
        .for_each(|(repo_plan, handle)| {
            let repo_path = werx.repos_dir().join(&repo_plan.repo);

            let mut repo_outcomes: Vec<(String, String, ActionOutcome)> = Vec::new();

            for action in &repo_plan.actions {
                let outcome = execute_action(&repo_path, action, &today);
                let (branch_name, result) = match action {
                    BranchAction::FastForward { branch, .. } => (branch.clone(), outcome),
                    BranchAction::FastForwardFromUpstream { branch, .. } => {
                        (branch.clone(), outcome)
                    }
                    BranchAction::Rebase { branch, .. } => (branch.clone(), outcome),
                    BranchAction::Push { branch, .. } => (branch.clone(), outcome),
                    BranchAction::Trash { branch, .. } => (branch.clone(), outcome),
                    BranchAction::Skip { branch, reason } => {
                        (branch.clone(), Ok(ActionOutcome::Skipped(reason.clone())))
                    }
                };

                match result {
                    Ok(out) => repo_outcomes.push((repo_plan.repo.clone(), branch_name, out)),
                    Err(e) => {
                        repo_outcomes.push((
                            repo_plan.repo.clone(),
                            branch_name.clone(),
                            ActionOutcome::Skipped(format!("error: {}", e)),
                        ));
                    }
                }
            }

            handle.finish_ok(&repo_plan.repo);
            all_outcomes.lock().unwrap().extend(repo_outcomes);
        });

    let outcomes = all_outcomes.into_inner().unwrap();
    let exec_errors = exec_errors.into_inner().unwrap();

    // ── Final summary ─────────────────────────────────────────────────────────
    print_summary(&outcomes, &exec_errors, &ctx.reporter);

    Ok(())
}

fn execute_action(repo_path: &Path, action: &BranchAction, date: &str) -> Result<ActionOutcome> {
    match action {
        BranchAction::FastForward { branch, to_sha, .. } => {
            apply_fast_forward(repo_path, branch, to_sha)?;
            Ok(ActionOutcome::Done(format!("fast-forwarded {}", branch)))
        }
        BranchAction::FastForwardFromUpstream { branch, to_sha } => {
            // Advance local branch to upstream tip (task 8.5)
            apply_fast_forward(repo_path, branch, to_sha)?;
            // Push to origin
            match push_branch(repo_path, branch, "origin")? {
                PushOutcome::Pushed | PushOutcome::UpToDate => {}
                PushOutcome::ForcePushRequired => {
                    tracing::warn!("force-push required to push '{}' to origin after upstream ff", branch);
                }
                PushOutcome::NoUpstream => {
                    tracing::debug!("'{}' has no upstream tracking branch on origin, skip push", branch);
                }
            }
            Ok(ActionOutcome::Done(format!("fast-forwarded {} from upstream", branch)))
        }
        BranchAction::Rebase { branch, onto_sha } => {
            match apply_rebase(repo_path, branch, onto_sha)? {
                RebaseOutcome::Success(_) => Ok(ActionOutcome::Done(format!("rebased {}", branch))),
                RebaseOutcome::Conflict => Ok(ActionOutcome::Skipped(format!(
                    "{}: rebase conflict — needs manual rebase",
                    branch
                ))),
            }
        }
        BranchAction::Push { branch, remote } => {
            match push_branch(repo_path, branch, remote)? {
                PushOutcome::Pushed => Ok(ActionOutcome::Done(format!("pushed {}", branch))),
                PushOutcome::UpToDate => Ok(ActionOutcome::Done(format!("{} already up to date", branch))),
                PushOutcome::ForcePushRequired => Ok(ActionOutcome::Skipped(format!(
                    "{}: force-push required",
                    branch
                ))),
                PushOutcome::NoUpstream => Ok(ActionOutcome::Skipped(format!(
                    "{}: no upstream configured",
                    branch
                ))),
            }
        }
        BranchAction::Trash { branch, .. } => {
            let trash_name = trash_stale_branch(repo_path, branch, date)?;
            Ok(ActionOutcome::Done(format!("trashed {} → {}", branch, trash_name)))
        }
        BranchAction::Skip { branch, reason } => {
            Ok(ActionOutcome::Skipped(format!("{}: {}", branch, reason)))
        }
    }
}

fn print_summary(
    outcomes: &[(String, String, ActionOutcome)],
    errors: &[(String, anyhow::Error)],
    reporter: &crate::reporter::Reporter,
) {
    reporter.println("");
    reporter.println(&format!("{}", style("Sync complete.").bold()));
    reporter.println("");

    let done: Vec<_> = outcomes
        .iter()
        .filter(|(_, _, o)| matches!(o, ActionOutcome::Done(_)))
        .collect();

    let skipped: Vec<_> = outcomes
        .iter()
        .filter(|(_, _, o)| matches!(o, ActionOutcome::Skipped(_)))
        .collect();

    if !done.is_empty() {
        reporter.println("Actions taken:");
        for (repo, _, outcome) in &done {
            if let ActionOutcome::Done(msg) = outcome {
                reporter.println(&format!(
                    "  {} [{}] {}",
                    style("✓").green(),
                    style(repo).cyan(),
                    msg
                ));
            }
        }
        reporter.println("");
    }

    if !skipped.is_empty() || !errors.is_empty() {
        reporter.println(&format!("{}:", style("Needs attention").yellow().bold()));
        for (repo, _, outcome) in &skipped {
            if let ActionOutcome::Skipped(msg) = outcome {
                reporter.println(&format!(
                    "  {} [{}] {}",
                    style("!").yellow(),
                    style(repo).cyan(),
                    msg
                ));
            }
        }
        for (repo, err) in errors {
            reporter.println(&format!(
                "  {} [{}] error: {}",
                style("✗").red(),
                style(repo).cyan(),
                err
            ));
        }
        reporter.println("");
    } else {
        reporter.println(&format!(
            "  {} No items need attention.",
            style("✓").green()
        ));
        reporter.println("");
    }
}

fn chrono_today() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let days_since_epoch = secs / 86400;
    // Compute year/month/day from days since epoch (1970-01-01)
    let mut year = 1970u32;
    let mut remaining = days_since_epoch as u32;
    loop {
        let days_in_year = if is_leap_year(year) { 366 } else { 365 };
        if remaining < days_in_year {
            break;
        }
        remaining -= days_in_year;
        year += 1;
    }
    let month_days: [u32; 12] = [
        31,
        if is_leap_year(year) { 29 } else { 28 },
        31, 30, 31, 30, 31, 31, 30, 31, 30, 31,
    ];
    let mut month = 1u32;
    for &d in &month_days {
        if remaining < d {
            break;
        }
        remaining -= d;
        month += 1;
    }
    let day = remaining + 1;
    format!("{:04}{:02}{:02}", year, month, day)
}

fn is_leap_year(y: u32) -> bool {
    (y % 4 == 0 && y % 100 != 0) || (y % 400 == 0)
}

// ── unit tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Command;
    use tempfile::TempDir;

    // ── test helpers ─────────────────────────────────────────────────────────

    fn make_commit(repo_path: &std::path::Path) -> String {
        let tree = Command::new("git")
            .args(["-C", &repo_path.to_string_lossy()])
            .args(["hash-object", "-t", "tree", "/dev/null"])
            .output()
            .unwrap();
        let tree_sha = String::from_utf8_lossy(&tree.stdout).trim().to_string();

        let parent_args: Vec<String> = {
            let head_out = Command::new("git")
                .args(["-C", &repo_path.to_string_lossy()])
                .args(["rev-parse", "--verify", "HEAD"])
                .output()
                .unwrap();
            if head_out.status.success() {
                let parent = String::from_utf8_lossy(&head_out.stdout).trim().to_string();
                vec!["-p".to_string(), parent]
            } else {
                vec![]
            }
        };

        let mut args = vec!["commit-tree".to_string(), tree_sha.clone(), "-m".to_string(), "commit".to_string()];
        args.extend(parent_args);

        let commit = Command::new("git")
            .args(["-C", &repo_path.to_string_lossy()])
            .args(args.iter().map(|s| s.as_str()))
            .output()
            .unwrap();
        String::from_utf8_lossy(&commit.stdout).trim().to_string()
    }

    fn init_bare_with_branch(branch: &str) -> (TempDir, std::path::PathBuf, String) {
        let dir = TempDir::new().unwrap();
        let path = dir.path().to_path_buf();

        Command::new("git")
            .args(["init", "--bare", &path.to_string_lossy()])
            .output()
            .unwrap();

        let commit_sha = make_commit(&path);

        Command::new("git")
            .args(["-C", &path.to_string_lossy()])
            .args(["update-ref", &format!("refs/heads/{}", branch), &commit_sha])
            .output()
            .unwrap();

        Command::new("git")
            .args(["-C", &path.to_string_lossy()])
            .args(["symbolic-ref", "HEAD", &format!("refs/heads/{}", branch)])
            .output()
            .unwrap();

        (dir, path, commit_sha)
    }

    // ── task 7.2: apply_fast_forward unit test ────────────────────────────────

    #[test]
    fn test_apply_fast_forward_advances_ref() {
        let (_dir, path, original_sha) = init_bare_with_branch("main");

        // Create a second commit to fast-forward to
        let new_sha = make_commit(&path);
        // Don't update the ref yet — simulate upstream being ahead
        // Reset main back to original
        Command::new("git")
            .args(["-C", &path.to_string_lossy()])
            .args(["update-ref", "refs/heads/main", &original_sha])
            .output()
            .unwrap();

        // Apply fast-forward
        apply_fast_forward(&path, "main", &new_sha).unwrap();

        // Verify
        let resolved = resolve_sha(&path, "refs/heads/main").unwrap();
        assert_eq!(resolved, new_sha);
        // Original commit still exists
        let original_exists = Command::new("git")
            .args(["-C", &path.to_string_lossy()])
            .args(["cat-file", "-e", &original_sha])
            .status()
            .unwrap()
            .success();
        assert!(original_exists);
    }

    // ── task 8.6: apply_rebase unit tests ────────────────────────────────────

    // Note: rebase tests require non-bare repos since git rebase needs a working tree
    // We use a temporary worktree (which is what TempWorktree does) in these tests.

    #[test]
    fn test_temp_worktree_is_cleaned_up() {
        let (_dir, path, sha) = init_bare_with_branch("main");
        let wt_path_str;
        {
            let wt = TempWorktree::create(&path, &sha).unwrap();
            wt_path_str = wt.path().to_path_buf();
            assert!(wt_path_str.exists());
        }
        // After drop, worktree should be gone from git's perspective
        // The TempDir itself is cleaned up by tempfile, so just verify git no longer lists it
        let wt_list = Command::new("git")
            .args(["-C", &path.to_string_lossy()])
            .args(["worktree", "list", "--porcelain"])
            .output()
            .unwrap();
        let output_str = String::from_utf8_lossy(&wt_list.stdout);
        assert!(!output_str.contains(&wt_path_str.to_string_lossy().as_ref()), 
            "worktree should be removed after drop: {}", output_str);
    }

    // ── task 9.3: push_branch tests ───────────────────────────────────────────

    /// Set up two bare repos: "upstream" (origin) and "local" (our bare clone)
    fn setup_push_test() -> (TempDir, std::path::PathBuf, std::path::PathBuf) {
        let dir = TempDir::new().unwrap();
        let upstream_path = dir.path().join("upstream.git");
        let local_path = dir.path().join("local.git");

        // Create upstream bare repo
        Command::new("git")
            .args(["init", "--bare", &upstream_path.to_string_lossy()])
            .output()
            .unwrap();

        // Create a commit on upstream main
        let commit = make_commit(&upstream_path);
        Command::new("git")
            .args(["-C", &upstream_path.to_string_lossy()])
            .args(["update-ref", "refs/heads/main", &commit])
            .output()
            .unwrap();

        // Clone bare
        Command::new("git")
            .args(["clone", "--bare", &upstream_path.to_string_lossy(), &local_path.to_string_lossy()])
            .output()
            .unwrap();

        (dir, upstream_path, local_path)
    }

    #[test]
    fn test_push_up_to_date() {
        let (_dir, _upstream, local) = setup_push_test();
        let result = push_branch(&local, "main", "origin");
        assert!(result.is_ok(), "{:?}", result);
        // Either UpToDate or Pushed is fine (already in sync)
        matches!(result.unwrap(), PushOutcome::UpToDate | PushOutcome::Pushed);
    }

    #[test]
    fn test_push_local_ahead() {
        let (_dir, _upstream, local) = setup_push_test();

        // Add a commit to local that's not in upstream
        let new_sha = make_commit(&local);
        Command::new("git")
            .args(["-C", &local.to_string_lossy()])
            .args(["update-ref", "refs/heads/main", &new_sha])
            .output()
            .unwrap();

        let result = push_branch(&local, "main", "origin");
        assert!(result.is_ok(), "{:?}", result);
        assert!(matches!(result.unwrap(), PushOutcome::Pushed));
    }

    #[test]
    fn test_push_force_required() {
        let (_dir, _upstream, local) = setup_push_test();

        // Make upstream diverge: add a commit to upstream that local doesn't have,
        // and also make local have a different commit
        let upstream_commit = make_commit(&local); // commit on local
        // Now make local have a DIFFERENT commit (reset + new commit)
        // First record the original sha
        let orig = resolve_sha(&local, "refs/heads/main").unwrap();
        // Add diverging commit to "upstream" via remote manipulation
        // Simpler: just make local branch diverge from origin/main
        // by resetting it to a different sha
        let _ = orig;
        let _ = upstream_commit;

        // Actually the simplest: create a new commit tree that's a sibling (not ancestor)
        // Reset origin/main to a different commit
        let alt_commit = {
            // Make a commit with different content using write-tree
            // Actually just make a second commit from the original without going through main
            let tree = Command::new("git")
                .args(["-C", &local.to_string_lossy()])
                .args(["hash-object", "-t", "tree", "/dev/null"])
                .output()
                .unwrap();
            let tree_sha = String::from_utf8_lossy(&tree.stdout).trim().to_string();
            let commit = Command::new("git")
                .args(["-C", &local.to_string_lossy()])
                .args(["commit-tree", &tree_sha, "-m", "diverging-commit"])
                .output()
                .unwrap();
            String::from_utf8_lossy(&commit.stdout).trim().to_string()
        };

        // Set local/main to the alt_commit (diverged from origin/main)
        Command::new("git")
            .args(["-C", &local.to_string_lossy()])
            .args(["update-ref", "refs/heads/main", &alt_commit])
            .output()
            .unwrap();

        let result = push_branch(&local, "main", "origin");
        assert!(result.is_ok(), "{:?}", result);
        assert!(matches!(result.unwrap(), PushOutcome::ForcePushRequired));
    }

    // ── format_plan tests ─────────────────────────────────────────────────────

    #[test]
    fn test_format_plan_empty() {
        let plan = SyncPlan { repos: vec![] };
        let output = format_plan(&plan);
        assert!(output.contains("Nothing to sync"));
    }

    #[test]
    fn test_format_plan_with_actions() {
        let plan = SyncPlan {
            repos: vec![RepoPlan {
                repo: "my-repo".to_string(),
                actions: vec![
                    BranchAction::FastForward {
                        branch: "main".to_string(),
                        from_sha: "aaaa1111".to_string(),
                        to_sha: "bbbb2222".to_string(),
                    },
                    BranchAction::Skip {
                        branch: "feature".to_string(),
                        reason: "active worktree".to_string(),
                    },
                ],
            }],
        };
        let output = format_plan(&plan);
        assert!(output.contains("my-repo"));
        assert!(output.contains("main"));
        assert!(output.contains("aaaa1111"));
        assert!(output.contains("bbbb2222"));
        assert!(output.contains("feature"));
        assert!(output.contains("active worktree"));
    }

    #[test]
    fn test_has_mutations_empty_plan() {
        let plan = SyncPlan { repos: vec![] };
        assert!(!plan.has_mutations());
    }

    #[test]
    fn test_has_mutations_only_skips() {
        let plan = SyncPlan {
            repos: vec![RepoPlan {
                repo: "r".to_string(),
                actions: vec![BranchAction::Skip {
                    branch: "b".to_string(),
                    reason: "r".to_string(),
                }],
            }],
        };
        assert!(!plan.has_mutations());
    }

    #[test]
    fn test_has_mutations_with_ff() {
        let plan = SyncPlan {
            repos: vec![RepoPlan {
                repo: "r".to_string(),
                actions: vec![BranchAction::FastForward {
                    branch: "main".to_string(),
                    from_sha: "a".to_string(),
                    to_sha: "b".to_string(),
                }],
            }],
        };
        assert!(plan.has_mutations());
    }
}
