# Change: Add Repository Creation Command

## Why

Currently, Forge can only add existing repositories via `forge add`, which clones from a remote. Developers often want to start a brand new project from scratch without first creating it on a provider like GitHub. This change enables creating new local repositories that can later be published to a provider.

## What Changes

- Add new `forge repos create <owner>/<repo>` command (with alias `forge create`)
- Create a bare repository in `.forge/repos/<name>/` following existing naming conventions
- Initialize the `main` branch with an empty commit to establish a valid branch
- Automatically create a worktree on `main` for immediate development
- Repository naming follows the `[owner]/[repo]` convention to prepare for future remote publishing

## Impact

- Affected specs: New `repo-create` capability
- Affected code: 
  - New `repos create` subcommand in CLI
  - Repository creation logic in repository management module
  - Integration with existing worktree creation logic
