## 1. CLI Structure

- [ ] 1.1 Add `create` subcommand to `repos` command group
- [ ] 1.2 Add `create` top-level alias for `repos create`
- [ ] 1.3 Parse `<owner>/<repo>` argument and validate format

## 2. Repository Creation

- [ ] 2.1 Compute storage directory name using existing progressive qualification rules
- [ ] 2.2 Create bare repository with `git init --bare`
- [ ] 2.3 Configure repository remote origin URL based on provider conventions (for future push)
- [ ] 2.4 Initialize `main` branch with empty commit

## 3. Worktree Creation

- [ ] 3.1 Create repository directory in Forge root for workspaces
- [ ] 3.2 Create worktree on `main` branch using existing workspace creation logic
- [ ] 3.3 Change working directory to new worktree (or display path for user)

## 4. User Feedback

- [ ] 4.1 Display success message with repository location
- [ ] 4.2 Display worktree location for immediate use
- [ ] 4.3 Provide next steps guidance (future: `forge publish` to create remote)

## 5. Error Handling

- [ ] 5.1 Validate Forge exists before creating repository
- [ ] 5.2 Handle directory name conflicts with existing repositories
- [ ] 5.3 Clean up partial state on failure

## 6. Testing

- [ ] 6.1 Unit tests for repository name parsing
- [ ] 6.2 Integration tests for full create flow
- [ ] 6.3 Test error cases (invalid name, existing repo, no forge)
