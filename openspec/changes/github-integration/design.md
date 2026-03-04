## Context

Werx manages bare-repo clones in `.werx/repos/<name>/` and worktrees layered over them. It already shells out to `git` for all VCS operations and uses `dialoguer`/`ratatui` for interactive UI. There is no existing GitHub API integration. The user runs werx primarily against GitHub repos and frequently works in personal forks of upstream projects.

## Goals / Non-Goals

**Goals:**
- Detect GitHub forks at `repo add` time and persist fork metadata per-repo
- Manage the `upstream` remote in fork repos automatically
- Extend sync to fast-forward origin-branches from upstream when a fork is present
- Allow `werx wt create #<N>` to resolve GitHub issue/PR numbers to branches
- Provide a configurable branch naming strategy (default: `<username>/[<issue#>-]<topic>`)
- Cache GitHub username in werx config to avoid repeated API calls

**Non-Goals:**
- Supporting non-GitHub remotes (GitLab, Bitbucket) in this change
- Complex branch naming template engines — just the `<username>/[<issue#>-]<topic>` pattern with a config field for future extensibility
- Push-to-upstream or PR creation flows
- Multi-account GitHub support

## Decisions

### Decision 1: Use `gh` CLI as the GitHub API client

**Chosen**: shell out to `gh` CLI for all GitHub API calls.

**Rationale**: `gh` already handles OAuth token storage, device flow, SSH key management, and enterprise GitHub hosts. Adding `octocrab` would require duplicating auth setup and shipping credentials UX that `gh` already provides perfectly. The user is a developer who certainly has `gh` installed.

**Alternatives considered**:
- `octocrab` crate: proper typed Rust API but requires auth configuration in werx itself
- Raw `reqwest` + PAT: even more auth complexity, no benefit over `gh`

**Trade-off**: werx now depends on `gh` being in `$PATH` for GitHub features. Non-GitHub repos or users without `gh` installed degrade gracefully — GitHub features are skipped rather than erroring.

### Decision 2: Store fork metadata in a per-repo `werx-repo.toml` file

**Chosen**: `.werx/repos/<name>/werx-repo.toml` stores repo-level metadata (fork status, upstream URL, default branch, github owner/repo).

**Rationale**: git config inside the bare repo is the alternative, but it mixes werx metadata with git internals. A dedicated TOML sidecar is explicit, easily inspectable, and consistent with the existing `werx.toml` pattern.

**Schema**:
```toml
[github]
owner = "ashley"
repo = "my-project"
is_fork = true
upstream_owner = "original-owner"
upstream_repo = "my-project"
default_branch = "main"
upstream_default_branch = "main"
```

This file is optional — repos added before this change, or non-GitHub repos, simply have no `werx-repo.toml`. All code touching it treats absence as "no GitHub metadata".

### Decision 3: Fork detection happens at `repo add` time

**Chosen**: when a repo is added, werx calls `gh repo view <owner>/<repo> --json isFork,parent` to detect fork status and populate `werx-repo.toml`.

**Rationale**: detection at add-time avoids repeated API calls during sync and worktree creation. The metadata can be refreshed explicitly in the future if needed, but for now we treat it as stable.

**Alternatives considered**:
- Detect at sync time: adds latency to every sync and requires network access for a read-only operation
- Detect lazily on first `wt create #N`: confusing UX, no upstream remote management until then

### Decision 4: GitHub username detection via `gh api user`

**Chosen**: on first use of any GitHub feature, run `gh api user --jq '.login'` and cache the result in `werx.toml` under `[github] username = "..."`.

**Rationale**: the username is needed for branch naming. Detecting once and caching avoids an API call on every branch creation. The cache can be manually overridden in `werx.toml` for edge cases.

### Decision 5: Branch naming as a simple pattern config, not a template engine

**Chosen**: `[github] branch_pattern = "username/issue-topic"` (enum-style, not a format string) for now.

**Rationale**: the user wants the `<username>/[<issue#>-]<topic>` pattern. Future patterns may differ in structure, but we don't yet know what they look like — premature generalization would produce wrong abstractions. The config field exists so the value can change when a second pattern is needed, at which point a real template system can be introduced.

For this change, the only supported pattern is the default; the config field is stored but alternatives are not implemented yet. A warning can be emitted if an unrecognized pattern is configured.

### Decision 6: Upstream sync — fast-forward origin branches from upstream before push

**Chosen**: during the Plan phase for a fork repo, for each local branch whose name exists in upstream's remote refs, add a `FetchFromUpstream` pre-step before the normal fast-forward/push logic.

Concretely: after fetching `upstream`, if `upstream/<branch>` exists, the plan action for that branch becomes `FastForwardFromUpstream` → push to `origin`. The branch ref is updated to the upstream tip, then pushed.

**Rationale**: developers working in forks want their fork's branches kept in sync with upstream. This is the most common "fork sync" workflow. We already have the `upstream` remote fetched as part of the existing remote-fetching requirement.

**Conflict handling**: if a local branch has diverged from upstream (cannot fast-forward from upstream), the branch is recorded as `Skipped` with reason "diverged from upstream — needs manual rebase" and the normal origin-push is also skipped for that branch.

### Decision 7: Coding agent invocation via temp-file prompt

**Chosen**: werx writes the full prompt to a temporary file and pipes it to the agent CLI. The agent is expected to include a `<branch-slug>…</branch-slug>` tag in its response; werx extracts the first match with a simple regex and discards any surrounding commentary.

**Invocation per agent**:

| Agent | Command |
|-------|---------|
| `claude` | `cat <prompt-file> \| claude --print` |
| `copilot` | `gh copilot suggest -t shell "$(<prompt-file)"` — actually: `copilot -p $(<prompt-file>)` |

Both variants are run as a subprocess; stdout is captured; stderr is discarded. Exit code non-zero → fall back to title slug.

**Prompt template** (written to the temp file):

```
You are helping generate a short git branch name slug.

Issue title: {title}
Issue body:
{body}

Produce a slug of at most 4 words that describes the work in this issue.
Rules:
- lowercase, hyphen-separated words only
- no issue number prefix
- no username prefix
- 4 words maximum

Respond with ONLY the tag and slug, nothing else:
<branch-slug>the-slug-here</branch-slug>

Do not include any explanation, preamble, or other text.
```

**Response parsing**: scan stdout for the first `<branch-slug>([^<]+)</branch-slug>` match. If found, apply normal slug normalization (lowercase, collapse hyphens, trim) to the captured group. If no match, treat as failure and fall back to the title slug.

**Rationale**: the XML-style tag makes the slug trivially extractable even when the model adds preamble, caveats, or markdown. Writing the prompt to a temp file avoids shell-quoting hazards with issue bodies containing backticks, quotes, or newlines.

### Decision 8: `werx wt create #N` — resolution strategy

**Chosen**: parse `#<number>` as a GitHub reference. Call `gh pr view <N>` first; if that succeeds, treat as PR. If it fails (not a PR), call `gh issue view <N>`; if that succeeds, treat as issue. If both fail, error.

- **PR**: fetch the PR's HEAD branch name from the JSON response (`headRefName`). Create a worktree for that branch (checking it out from origin if it exists, or creating a tracking branch otherwise).
- **Issue**: call the branch naming service to produce a branch name of the form `<username>/<issue#>-<topic>` where `<topic>` is prompted from the user (pre-filled with a slugified version of the issue title). Create the branch off the repo's default branch if it doesn't already exist, then create a worktree.

**Rationale**: PR-check-first is correct because a PR number is always also a valid issue-like number in GitHub's numbering scheme; checking PR first gives the right semantic.

## Risks / Trade-offs

- **`gh` not installed** → werx falls back gracefully: `werx repo add` skips fork detection (logs a warning), `werx wt create #N` errors with a helpful message, sync runs without upstream-aware logic.
- **Rate limiting** → `gh` handles token auth and will hit GitHub's rate limit on heavy automated use. Mitigation: all API calls are scoped to explicit user actions; no polling.
- **`werx-repo.toml` staleness** → if a repo is un-forked or the upstream changes on GitHub, the cached metadata becomes wrong. Mitigation: acceptable for now given the single-user context; a `werx repo refresh` command can be added later.
- **Branch naming prompt friction** → creating a worktree from an issue requires the user to confirm/edit the topic slug. Mitigation: the issue title is used as the default, so the user only needs to press Enter for typical cases.
- **Interaction with existing sync logic** → the upstream-fast-forward step is inserted before the existing push logic. Branches that only exist locally (no upstream counterpart) are unaffected.

## Open Questions

- Should `werx wt create #N` for a PR also set up the remote tracking branch to the PR contributor's fork? Deferred — for now we only check out the HEAD branch from origin.
- Should `werx repo add` prompt to refresh fork metadata if `werx-repo.toml` already exists but looks stale? Deferred — no migration concerns per the proposal.
