---
description: Start the Spec Workflow - create a new OpenSpec change proposal.
---

$ARGUMENTS
<!-- OPENSPEC:START -->
**Guardrails**
- Favor straightforward, minimal implementations first and add complexity only when it is requested or clearly required.
- Keep changes tightly scoped to the requested outcome.
- Refer to `@openspec/AGENTS.md` for additional OpenSpec conventions.
- Identify any vague or ambiguous details and ask the necessary follow-up questions before editing files.
- Do not write any code during the proposal stage. Only create design documents (proposal.md, tasks.md, design.md, and spec deltas).

**Steps**

1. **Research and understand context**
   - Review `openspec/project.md`, run `openspec list` and `openspec list --specs`
   - Inspect related code or docs (e.g., via `rg`/`ls`) to ground the proposal
   - Note any gaps that require clarification

2. **Choose change ID and create branch**
   - Choose a unique verb-led `change-id` (e.g., `add-two-factor-auth`)
   - IMMEDIATELY create and switch to branch: `git checkout -b openspec/<change-id>`
   - Verify branch: `git branch --show-current`

3. **Scaffold proposal files**
   - Create `openspec/changes/<id>/proposal.md`, `tasks.md`
   - Create `design.md` when needed (see OpenSpec AGENTS.md for criteria)

4. **Map to capabilities and create spec deltas**
   - Break multi-scope efforts into distinct spec deltas
   - Draft deltas in `changes/<id>/specs/<capability>/spec.md` (one folder per capability)
   - Use `## ADDED|MODIFIED|REMOVED Requirements` with at least one `#### Scenario:` per requirement
   - Cross-reference related capabilities when relevant

5. **Draft tasks.md**
   - Create ordered list of small, verifiable work items
   - Include validation (tests, tooling)
   - Highlight dependencies or parallelizable work

6. **Validate strictly**
   - Run: `openspec validate <id> --strict --no-interactive`
   - Resolve every issue before sharing the proposal

7. **Iterate with user**
   - Share proposal for feedback
   - Make revisions as needed on the same branch
   - Re-validate after changes

**Reference**
- Use `openspec show <id> --json --deltas-only` or `openspec show <spec> --type spec` to inspect details
- Search existing requirements: `rg -n "Requirement:|Scenario:" openspec/specs`
- Explore codebase with `rg <keyword>`, `ls`, or direct file reads
- See `@openspec/AGENTS.md` for full OpenSpec conventions
- After approval, use `@.github/prompts/spec-complete.prompt.md` to create implementation beads
<!-- OPENSPEC:END -->
