## 1. CLI Structure

- [x] 1.1 Add `create` subcommand to `repos` command group
- [x] 1.2 Add `create` top-level alias for `repos create`
- [x] 1.3 Parse `<owner>/<repo>` argument and validate format

## 2. Repository Creation

- [x] 2.1 Compute storage directory name using existing progressive qualification rules
- [x] 2.2 Create bare repository with `git init --bare`
- [x] 2.3 Configure repository remote origin URL based on provider conventions (for future push)
- [x] 2.4 Initialize `main` branch with empty commit

## 3. Worktree Creation

- [x] 3.1 Create repository directory in Forge root for workspaces
- [x] 3.2 Create worktree on `main` branch using existing workspace creation logic
- [x] 3.3 Change working directory to new worktree (or display path for user)

## 4. User Feedback

- [x] 4.1 Display success message with repository location
- [x] 4.2 Display worktree location for immediate use
- [x] 4.3 Provide next steps guidance (future: `forge publish` to create remote)

## 5. Error Handling

- [x] 5.1 Validate Forge exists before creating repository
- [x] 5.2 Handle directory name conflicts with existing repositories
- [x] 5.3 Clean up partial state on failure

## 6. Testing

- [x] 6.1 Unit tests for repository name parsing
- [x] 6.2 Integration tests for full create flow
- [x] 6.3 Test error cases (invalid name, existing repo, no forge)
