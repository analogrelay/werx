## ADDED Requirements

### Requirement: Create workspace from GitHub issue reference
The system SHALL accept `#<number>` as the branch argument to `werx wt create`. When the number resolves to a GitHub issue, the system SHALL find or create a work branch for that issue using the branch naming service, then create a worktree for it.

#### Scenario: Issue resolved, existing work branch found
- **WHEN** user runs `werx wt create #42`
- **AND** the number resolves to a GitHub issue
- **AND** a local branch named `<username>/42-*` already exists
- **THEN** a worktree is created for that branch (or reported as already existing)

#### Scenario: Issue resolved, no existing branch — user confirms topic
- **WHEN** user runs `werx wt create #42` in an interactive terminal
- **AND** the number resolves to a GitHub issue titled `"Update auth service"`
- **AND** no existing work branch for issue 42 exists
- **THEN** the system derives a topic slug via the branch naming service (AI-assisted if an agent is configured, otherwise from the issue title)
- **AND** the system prompts for a topic slug pre-filled with the derived slug
- **AND** on confirmation, creates branch `<username>/42-<topic>` off the repo's default branch
- **AND** creates a worktree for the new branch

#### Scenario: Issue resolved, non-interactive context
- **WHEN** user runs `werx wt create #42` in a non-interactive context
- **AND** the number resolves to a GitHub issue
- **AND** no existing work branch exists
- **THEN** the branch is created using the slug from the branch naming service (AI-assisted if configured, otherwise title-based) without prompting
- **AND** a worktree is created for the new branch

#### Scenario: Worktree already exists for the branch
- **WHEN** a worktree already exists for the resolved branch
- **THEN** the command reports the existing worktree path
- **AND** exits with code 0 without creating a duplicate

---

### Requirement: Create workspace from GitHub PR reference
When `#<number>` resolves to a GitHub pull request, the system SHALL create a worktree for the PR's HEAD branch.

#### Scenario: PR resolved, HEAD branch checked out
- **WHEN** user runs `werx wt create #99`
- **AND** the number resolves to a GitHub PR with HEAD branch `"ashley/99-fix-thing"`
- **THEN** the system fetches the HEAD branch from `origin` if not present locally
- **AND** creates a worktree for `ashley/99-fix-thing`

#### Scenario: PR worktree already exists
- **WHEN** user runs `werx wt create #99`
- **AND** a worktree for the PR's HEAD branch already exists
- **THEN** the existing worktree path is reported
- **AND** exits with code 0

#### Scenario: PR number check precedes issue check
- **WHEN** `#<number>` is provided and the number is both a valid PR and issue (which is always true on GitHub)
- **THEN** the PR resolution path is taken

---

### Requirement: GitHub reference resolution error handling
The system SHALL produce clear errors when a `#<number>` argument cannot be resolved.

#### Scenario: Number is neither issue nor PR
- **WHEN** user runs `werx wt create #9999`
- **AND** no PR or issue with that number exists in the repository
- **THEN** the command fails with error `"#9999 is not a known issue or PR in <owner>/<repo>"`
- **AND** exits with a non-zero code

#### Scenario: `gh` not available
- **WHEN** user runs `werx wt create #42`
- **AND** `gh` is not in `$PATH`
- **THEN** the command fails with error explaining that GitHub integration requires `gh` CLI

#### Scenario: No GitHub metadata for the repo
- **WHEN** user runs `werx wt create #42` for a repo with no `werx-repo.toml`
- **THEN** the command fails with error indicating the repo has no GitHub metadata
- **AND** suggests running `werx repo add` to re-register the repository with GitHub detection enabled
