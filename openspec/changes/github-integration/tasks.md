## 1. Config: GitHub and Agent tables

- [x] 1.1 Add `[github]` table to `Config` struct: `username: Option<String>`, `branch_pattern: Option<String>`
- [x] 1.2 Add `[agent]` table to `Config` struct: `agent: Option<String>` (values: `"claude"`, `"copilot"`)
- [x] 1.3 Write serialization round-trip tests for both new config tables

## 2. Per-repo metadata: werx-repo.toml

- [x] 2.1 Define `RepoGithubMeta` struct with fields: `owner`, `repo`, `is_fork`, `upstream_owner?`, `upstream_repo?`, `default_branch`, `upstream_default_branch?`
- [x] 2.2 Implement `RepoGithubMeta::load(repo_dir)` — returns `Ok(None)` if file absent, error only on malformed TOML
- [x] 2.3 Implement `RepoGithubMeta::save(repo_dir)`
- [x] 2.4 Write unit tests for load/save round-trip and missing-file case

## 3. GitHub API helpers (via `gh` CLI)

- [x] 3.1 Add `src/github.rs` module with `is_gh_available() -> bool` (probe `gh` in PATH)
- [x] 3.2 Implement `fetch_repo_meta(owner, repo) -> Result<GhRepoView>` — runs `gh repo view <owner>/<repo> --json isFork,parent,defaultBranchRef` and deserializes JSON
- [x] 3.3 Implement `fetch_username() -> Result<String>` — runs `gh api user --jq '.login'`
- [x] 3.4 Implement `fetch_issue(owner, repo, number) -> Result<GhIssue>` — runs `gh issue view <N> --json title,body`
- [x] 3.5 Implement `fetch_pr(owner, repo, number) -> Result<GhPr>` — runs `gh pr view <N> --json headRefName`

## 4. Fork tracking at repo add

- [x] 4.1 After successful bare clone in `repos::add_repo`, call `detect_and_save_fork_meta(werx, &repo_dir, &spec)` — skips silently if `gh` unavailable or remote is non-GitHub
- [x] 4.2 Implement `detect_and_save_fork_meta`: calls `fetch_repo_meta`, writes `werx-repo.toml`, prints warning on API failure but does not abort
- [x] 4.3 After writing metadata, if `is_fork = true`, call `ensure_upstream_remote(&repo_dir, &upstream_clone_url)` — uses same protocol as origin
- [x] 4.4 Implement `ensure_upstream_remote`: adds remote if absent, updates URL if wrong, no-ops if correct
- [ ] 4.5 Integration test: mock `gh` output, verify `werx-repo.toml` written and upstream remote set

## 5. Branch naming service

- [x] 5.1 Add `src/branch_naming.rs` module
- [x] 5.2 Implement `slugify(text: &str) -> String` — lowercase, non-alphanum → hyphen, collapse, trim
- [x] 5.3 Implement `resolve_username(werx: &Werx, config: &mut Config) -> Result<String>` — checks cache, fetches via `gh api user`, saves to config, or prompts if `gh` unavailable
- [x] 5.4 Implement `make_branch_name(username, issue_num: Option<u64>, topic: &str) -> String` following the `username/[N-]topic` pattern
- [x] 5.5 Write unit tests for `slugify` (cases: uppercase, spaces, special chars, consecutive hyphens, leading/trailing)
- [x] 5.6 Write unit tests for `make_branch_name` (with and without issue number)

## 6. Coding agent slug generation

- [x] 6.1 Implement `build_slug_prompt(title: &str, body: &str) -> String` — returns the full prompt template from design.md
- [x] 6.2 Implement `invoke_agent(agent: &str, prompt: &str) -> Result<String>` — writes prompt to tempfile, dispatches to correct CLI command per agent, captures stdout
- [x] 6.3 Implement `extract_branch_slug(output: &str) -> Option<String>` — regex scan for `<branch-slug>([^<]+)</branch-slug>`, normalizes capture with `slugify`
- [x] 6.4 Implement `generate_slug(werx: &Werx, config: &Config, title: &str, body: &str) -> String` — orchestrates: check agent config → invoke → extract → fallback to `slugify(title)` with warning
- [x] 6.5 Write unit tests for `extract_branch_slug` (clean tag, tag in prose, no tag, empty string)
- [x] 6.6 Write unit tests for `build_slug_prompt` verifying key sections are present

## 7. `werx wt create #N` — issue/PR resolution

- [x] 7.1 Parse `#<number>` argument format in `workspace::create` command arg handling
- [x] 7.2 On `#N` argument: require `werx-repo.toml` to exist for the target repo, error with helpful message if absent
- [x] 7.3 Attempt `fetch_pr` first; on success, extract `headRefName` and proceed to worktree creation for that branch
- [x] 7.4 On PR fetch failure, attempt `fetch_issue`; on success proceed to issue flow
- [x] 7.5 Issue flow (interactive): call `generate_slug` for default, prompt user with pre-filled slug, resolve final branch name via `make_branch_name`, create branch off default branch if absent, create worktree
- [x] 7.6 Issue flow (non-interactive): call `generate_slug`, use result directly, create branch and worktree without prompting
- [x] 7.7 Detect existing worktree for resolved branch — report path and exit 0 without duplicating
- [x] 7.8 Error when number resolves to neither PR nor issue
- [x] 7.9 Error with clear message when `gh` not in PATH

## 8. Upstream-aware sync

- [x] 8.1 In the Plan phase, load `RepoGithubMeta` for each repo being synced
- [x] 8.2 For fork repos: after fetching remotes, check each local branch against `upstream/<branch>` refs
- [x] 8.3 If `upstream/<branch>` exists and local tip is an ancestor: plan action is `FastForwardFromUpstream`
- [x] 8.4 If `upstream/<branch>` exists but local has diverged: plan action is `Skipped("diverged from upstream — needs manual rebase")`; suppress normal origin push for this branch
- [x] 8.5 Execute `FastForwardFromUpstream`: advance local branch ref to `upstream/<branch>` tip, then push to `origin`
- [x] 8.6 If fork metadata present but `upstream` remote missing: emit warning, fall back to normal origin sync for that repo
- [x] 8.7 Ensure branches with no upstream counterpart continue through normal sync logic unaffected

## 9. CHANGELOG and cleanup

- [x] 9.1 Update `CHANGELOG.md` with entries for: fork tracking, upstream sync, `wt create #N`, branch naming, agent integration
