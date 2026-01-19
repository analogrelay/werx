---
description: Start the Bug Reporting Workflow - capture and track a bug.
---

$ARGUMENTS
<!-- AGENTS:START -->
**Guardrails**
- Favor straightforward, minimal implementations first and add complexity only when it is requested or clearly required.
- Keep changes tightly scoped to the requested outcome.
- Refer to `@AGENTS.md` for full workflow context.
- Use `bd prime` to get full context on how to use beads.
- This prompt is for REPORTING bugs, not fixing them. Use Work Workflow to fix bugs.

**Steps**
Track these steps as TODOs and complete them one by one.

1. **Interview the user**
   - What did they expect to happen?
   - What actually happened?
   - What are the steps to reproduce?
   - What is the impact/severity?
   - Are there any error messages or logs?

2. **Create bug bead**
   - Create bead: `bd create "<bug title>" --type bug --priority <appropriate-level>`
   - Use priority 0 for critical bugs, 1 for high, 2 for medium, 3 for low
   - Note the bead ID returned

3. **Add context to bead**
   - Update the bead with reproduction steps, impact, and any relevant details
   - Use `bd update <id>` or edit `.beads/beads/<id>.json` directly

4. **Link to related beads**
   - If this bug blocks other work: note which beads it blocks
   - If discovered during other work: `--discovered-from <bead-id>`
   - If related to other bugs: note the relationship

5. **Create bug report branch**
   - Create and switch to branch: `git checkout -b report/<bead-id>`
   - Verify branch: `git branch --show-current`

6. **Commit the bug bead**
   ```bash
   git add .beads/
   git commit -m "bug: report <bug title> ({{BEAD_PREFIX}}-<bead-id>)"
   bd sync
   git push -u origin report/<bead-id>
   git status  # Verify pushed successfully
   ```

7. **Report completion**
   - Tell user the bug has been reported
   - Report bead ID: `{{BEAD_PREFIX}}-<bead-id>`
   - Report branch name: `report/<bead-id>`
   - Mention if it blocks any other work

**Reference**
- Use `bd list --type bug` to review all bugs
- Use `bd show <id>` to view bug details
- Reference `@AGENTS.md` for the full Bug Reporting Workflow
- To FIX the bug, use `@.github/prompts/work-on.prompt.md` with the bug's bead ID
<!-- AGENTS:END -->
