---
description: Start the Chore Workflow - create and complete a maintenance task.
---

$ARGUMENTS
<!-- AGENTS:START -->
**Guardrails**
- Favor straightforward, minimal implementations first and add complexity only when it is requested or clearly required.
- Keep changes tightly scoped to the requested outcome.
- Refer to `@AGENTS.md` for full workflow context.
- Use `bd prime` to get full context on how to use beads.
- This prompt expects a chore description in $ARGUMENTS or conversation context.

**Steps**
Track these steps as TODOs and complete them one by one.

1. **Extract chore description**
   - Get description from $ARGUMENTS or conversation
   - If not clear, ask user to describe the chore

2. **Create chore bead**
   - Create bead: `bd create "<description>" --type chore --priority <appropriate-level>`
   - Use priority 2 for normal chores, 1 for urgent, 3 for low priority
   - Note the bead ID returned

3. **Create chore branch**
   - Create and switch to branch: `git checkout -b bd/<bead-id>`
   - Verify branch: `git branch --show-current`

4. **Commit the new bead**
   ```bash
   git add .beads/
   git commit -m "chore: track <description> ({{BEAD_PREFIX}}-<bead-id>)"
   ```

5. **Perform the chore**
   - Keep edits minimal and focused on the chore
   - Make individual commits as you progress
   - Include bead ID in every commit: `chore: description ({{BEAD_PREFIX}}-<bead-id>)`

6. **Don't yakshave**
   - Newly-identified tasks → create new beads with `--discovered-from <bead-id>`
   - Stay focused on the primary chore

7. **Update bead state**
   - Close the chore when complete: `bd close <id>`
   - Document any discoveries or follow-up needed

8. **Run quality gates**
   - Run tests, linters, builds as appropriate
   - Fix any issues related to your changes

9. **Commit, sync, and push**
   ```bash
   git add .
   git commit -m "chore: complete <description> ({{BEAD_PREFIX}}-<bead-id>)"
   bd sync
   git push -u origin bd/<bead-id>
   git status  # Verify pushed successfully
   ```

10. **Report completion**
    - Tell user what was completed
    - Report branch name: `bd/<bead-id>`
    - Mention any new beads created for follow-up work

**Reference**
- Use `bd list` to review all beads
- Reference `@AGENTS.md` for the full Chore Workflow
<!-- AGENTS:END -->
