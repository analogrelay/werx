## ADDED Requirements

### Requirement: Upstream-aware branch fast-forward for fork repos
During the Plan phase, for repositories where `werx-repo.toml` indicates `is_fork = true`, the system SHALL check each local branch against the fetched `upstream` remote refs. If a branch of the same name exists in `upstream`, the plan action for that branch SHALL be `FastForwardFromUpstream` rather than the normal `FastForward` from `origin`. The Execute phase SHALL advance the local branch ref to the `upstream/<branch>` tip and then push the result to `origin`.

#### Scenario: Branch fast-forwarded from upstream
- **WHEN** sync runs on a fork repo
- **AND** local branch `main` can be fast-forwarded to `upstream/main`
- **THEN** the local `main` ref is advanced to `upstream/main`
- **AND** `main` is pushed to `origin`

#### Scenario: Branch diverged from upstream is skipped
- **WHEN** local branch `main` has diverged from `upstream/main` (cannot fast-forward)
- **THEN** the branch is recorded as `Skipped` with reason `"diverged from upstream — needs manual rebase"`
- **AND** no push to `origin` is attempted for that branch

#### Scenario: Branch with no upstream counterpart uses normal sync
- **WHEN** local branch `feature/my-change` has no counterpart in `upstream`
- **THEN** the normal fast-forward/push logic from `origin` applies

#### Scenario: Repo with no fork metadata uses normal sync
- **WHEN** a repo has no `werx-repo.toml` or `is_fork = false`
- **THEN** upstream-aware logic is skipped entirely
- **AND** sync behaves as before this change

#### Scenario: upstream remote absent despite fork metadata
- **WHEN** `werx-repo.toml` shows `is_fork = true` but no `upstream` remote exists
- **THEN** a warning is emitted: `"Fork repo missing upstream remote — run werx repo refresh"`
- **AND** the affected repo's branches fall back to normal origin-based sync
