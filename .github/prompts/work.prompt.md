---
description: Start the Work Workflow - find and implement unblocked work.
---

$ARGUMENTS
<!-- AGENTS:START -->
**Guardrails**
- Favor straightforward, minimal implementations first and add complexity only when it is requested or clearly required.
- Keep changes tightly scoped to the requested outcome.
- Refer to `@AGENTS.md` for full workflow context.
- Use `bd prime` to get full context on how to use beads.

**Steps**
Track these steps as TODOs and complete them one by one.

1. **Find and start highest priority work**
   - Run `bd ready` to see prioritized, unblocked work
   - IMMEDIATELY select the highest priority item and begin work
   - Take initiative - do NOT ask the user which bead to work on

2. **Create work branch**
   - Create and switch to branch: `git checkout -b bd/<bead-id>`
   - Verify branch: `git branch --show-current`

3. **Understand the work**
   - Read the bead description and context
   - If this is implementing an OpenSpec change, read `openspec/changes/<id>/proposal.md`, `design.md`, and `tasks.md`
   - Identify any blockers or missing information

4. **Implement the work**
   - Keep edits minimal and focused on the primary goal
   - Make individual commits as you progress
   - Include bead ID in every commit: `feat: description ({{BEAD_PREFIX}}-<bead-id>)`
   - Implementing multiple child beads within a parent branch IS permitted

5. **Don't yakshave**
   - Newly-identified tasks → create new beads with `--discovered-from <bead-id>`
   - Recommended refactorings → create beads and prioritize appropriately
   - Stay focused on the primary goal

6. **Update bead state**
   - Close completed beads: `bd close <id>`
   - Update in-progress beads with notes, blockers, or discoveries
   - Ensure next agent session can pick up seamlessly

7. **Run quality gates**
   - Run tests, linters, builds as appropriate
   - Fix any issues related to your changes

8. **Archive OpenSpec change if complete**
   - If this bead completes an OpenSpec change, archive it:
   - Check if all tasks in the epic are complete
   - If yes, run: `openspec archive <change-id> --yes`
   - This moves the change to archive and updates canonical specs

9. **Commit, sync, and push**
   ```bash
   git add .
   git commit -m "feat: description ({{BEAD_PREFIX}}-<bead-id>)"
   bd sync
   git push -u origin bd/<bead-id>
   git status  # Verify pushed successfully
   ```

10. **Report completion**
    - Tell user what was completed
    - Report branch name: `bd/<bead-id>`
    - Mention any new beads created or blockers discovered

**Reference**
- Use `bd list` to review all beads
- Use `bd show <id>` to view bead details
- Reference `@AGENTS.md` for the full Work Workflow
<!-- AGENTS:END -->
