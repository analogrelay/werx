---
description: Complete the Spec Workflow - create beads and finalize the spec branch.
---

$ARGUMENTS
<!-- OPENSPEC:START -->
**Guardrails**
- Favor straightforward, minimal implementations first and add complexity only when it is requested or clearly required.
- Keep changes tightly scoped to the requested outcome.
- Refer to `@openspec/AGENTS.md` for additional OpenSpec conventions.
- Use `bd prime` to get full context on how to use beads.
- This prompt is used DURING the spec workflow, while on an `openspec/<change-id>` branch.

**Steps**
Track these steps as TODOs and complete them one by one.

1. **Verify you're on the correct branch**
   - Run `git branch --show-current` to confirm you're on `openspec/<change-id>`
   - If not, stop and switch to the correct branch first

2. **Read the spec materials**
   - Read `changes/<id>/proposal.md`, `design.md` (if present), and `tasks.md`
   - Confirm scope and acceptance criteria

3. **Create epic in beads**
   - Create an epic to track the implementation of this change
   - Link to OpenSpec using `--external-ref openspec:<change-id>`
   - Example: `bd create "Implement two-factor authentication" --type epic --priority 2 --external-ref openspec:add-2fa`

4. **Create child task issues**
   - Create individual issues in beads for each task in `tasks.md`
   - ALWAYS use `--parent <epic-id>` to link each task to the epic
   - Example: `bd create "Add OTP generation logic" --type task --parent {{BEAD_PREFIX}}-1234 --priority 2`

5. **Update tasks.md checklist**
   - Mark each task as `- [*]` with the corresponding bead ID
   - Example: `- [*] 1.1 Create database schema ({{BEAD_PREFIX}}-1235)`

6. **Commit all changes**
   - Commit the spec changes and updated beads
   - Use format: `spec: <description> (openspec/<change-id>)`
   - Example: `git add . && git commit -m "spec: add two-factor auth proposal and implementation plan (openspec/add-2fa)"`

7. **Sync and push to remote**
   ```bash
   bd sync
   git push -u origin openspec/<change-id>
   git status  # Verify pushed successfully
   ```

8. **Report branch name to user**
   - Inform user that spec is ready for review on branch `openspec/<change-id>`

**Reference**
- Use `openspec show <id> --json --deltas-only` for additional context
- Use `bd list` to review created issues
- Reference `@AGENTS.md` for the full Spec Workflow
<!-- OPENSPEC:END -->
