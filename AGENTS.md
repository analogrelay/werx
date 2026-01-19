# Agent Instructions

This document defines the primary workflows for AI agents working in this project.

## Core Philosophy

**Beads is the memory.** All work, planning, state tracking, and agent-to-agent communication happens through beads issues. Keep them updated religiously.

**Branch per unit of work.** Every spec change and every work item gets its own branch with a clear naming convention.

**Commit conventions matter.** Always include the bead ID in commit messages: `feat: implemented xyz ({{BEAD_PREFIX}}-abcd)`

**Push before you finish.** Work isn't done until it's pushed to the remote.

---

## Workflow Overview

This project uses three primary workflows:

1. **Spec Workflow** - Creating OpenSpec change proposals for large projects
2. **Work Workflow** - Implementing beads issues (tasks, features, bugs)
3. **Chore Workflow** - User-directed maintenance work
4. **Bug Reporting** - Capturing and tracking bugs

---

## 1. Spec Workflow

**When to use:** Creating proposals for large projects, new capabilities, breaking changes, architecture shifts, or complex features.

**Triggers:**

- User asks to create a spec, proposal, or change
- Request involves new capabilities or breaking changes
- Work needs design discussion before implementation

**Process:**

1. **Create the change proposal** using `@.github/prompts/spec.prompt.md`
   - This handles scaffolding proposal.md, design.md (if needed), tasks.md, and spec deltas
   - See `@openspec/AGENTS.md` for detailed OpenSpec conventions

2. **IMMEDIATELY switch to an `openspec/<change-id>` branch**

   ```bash
   git checkout -b openspec/<change-id>
   ```

3. **Iterate with the user** on the spec content until approved

4. **Create beads work items** using `@.github/prompts/spec-complete.prompt.md`
   - Creates one `epic` type issue for the overall change
   - Creates child `task` issues for each item in tasks.md
   - Links epic to the OpenSpec change via `--external-ref openspec:<change-id>`

5. **Commit and push everything**

   ```bash
   git add openspec/changes/<change-id>/ .beads/
   git commit -m "spec: <change description> (openspec/<change-id>)"
   bd sync
   git push -u origin openspec/<change-id>
   ```

6. **Report the branch name** to the user for review

**Key Rules:**

- Always work in `openspec/<change-id>` branches
- Don't start implementation during spec workflow
- Specs should produce 1 epic with child/grandchild tasks below it
- Push to remote before considering spec workflow complete

---

## 2. Work Workflow

**When to use:** Implementing planned work from beads issues.

**Triggers:**

- User asks to "do work", "complete outstanding work", "work on tasks"
- User references a specific bead ID to implement

**Process:**

1. **Find work** using `bd ready` - shows unblocked, prioritized work

2. **Select high-priority work** and create a branch named `bd/<bead-id>`

   ```bash
   git checkout -b bd/<bead-id>
   ```

3. **Complete the primary goal** of the issue
   - Make individual commits as you progress
   - Each commit MUST include the bead ID: `feat: add login form ({{BEAD_PREFIX}}-1234)`
   - Implementing multiple child issues within a parent branch IS permitted
   - Each issue should have at least one commit

4. **Don't yakshave** - stay focused
   - Newly-identified tasks → create new beads with `--deps discovered-from:<bead-id>` and prioritize appropriately
   - Recommended refactorings → create beads and prioritize appropriately
   - Use `bd create`, `--deps` as needed (with `parent:<bead-id>`, `blocks:<bead-id>`, `discovered-from:<bead-id>`, etc)

5. **Update bead state**
   - Close completed beads: `bd close <id>`
   - Update in-progress beads with notes, blockers, or discoveries
   - Ensure next agent session can pick up seamlessly

6. **Commit and push your branch**

   ```bash
   git add .
   git commit -m "feat: description ({{BEAD_PREFIX}}-<bead-id>)"
   bd sync
   git push -u origin bd/<bead-id>
   ```

**Key Rules:**

- Always work in `bd/<bead-id>` branches
- Include bead ID in every commit message: `({{BEAD_PREFIX}}-abcd)`
- Create new beads instead of scope creep
- Link beads liberally using `--deps`. See `bd create --help` for more.
- Push to remote before considering work complete

---

## 3. Chore Workflow

**When to use:** User asks for maintenance work like dependency updates, refactoring, cleanup.

**Triggers:**

- "Update dependencies"
- "Clean up the codebase"
- Any user-directed chore work

**Process:**

1. **Create a bead** for the chore

   ```bash
   bd create "Update npm dependencies" --type chore --priority <appropriate-level>
   ```

2. **Create branch and commit the new bead**

   ```bash
   git checkout -b bd/<bead-id>
   bd sync
   git commit -m "chore: track dependency update ({{BEAD_PREFIX}}-<bead-id>)"
   ```

3. **Follow the Work Workflow** to complete the chore
   - You may immediately work on this chore instead of selecting from `bd ready`
   - Same commit conventions apply
   - Same push requirements apply

**Key Rules:**

- Always create a bead first, even for user-directed chores
- Use appropriate priority (don't default everything to P0)
- Still use `bd/<bead-id>` branch naming

---

## 4. Bug Reporting Workflow

**When to use:** User reports a bug or issue.

**Prompt:** Use `@.github/prompts/bug.prompt.md` to start this workflow.

**Process:**

1. **Interview the user** about:
   - What they expected
   - What actually happened
   - Steps to reproduce
   - Impact/severity

2. **Create a bead** for the bug

   ```bash
   bd create "Login fails with special characters" --type bug --priority <appropriate>
   ```

3. **Link to related beads** if applicable
   - Use `--deps` as needed when creating or updating the bead

4. **Create a `report/<bead-id>` branch** and commit

   ```bash
   git checkout -b report/<bead-id>
   git add .beads/
   git commit -m "bug: report login failure ({{BEAD_PREFIX}}-<bead-id>)"
   bd sync
   git push -u origin report/<bead-id>
   ```

**Key Rules:**

- Use `report/<bead-id>` branch naming for bug reports
- Gather full context before creating the bead
- Link to related work items

---

## General Rules for All Workflows

### Bead Linking

Use `--deps <type>:<bead-id>[,<type>:<bead-id>...]` to link beads:

- `--deps parent:<bead-id>` for hierarchical relationships (tasks under epics)
- `--deps child:<bead-id>` to add a child to the current bead
- `--deps blocks:<bead-id>` to indicate this work is blocking another bead
- `--deps discovered-from:<bead-id>` when work is identified during another task
- `--deps related:<bead-id>` for loosely related beads

- Use `--external-ref` for linking to external systems (e.g., `openspec:<change-id>`)

### When to Create Beads

- Work is more complex than trivial (>2 minutes)
- Work isn't part of the current task's primary goal
- You need to track state for future sessions
- You discover new requirements or bugs

### Beads as Communication

- Beads is how agents communicate with each other
- Beads is how you communicate with the user asynchronously
- Keep beads updated with current state, blockers, and discoveries
- Prompting is temporary; beads is the permanent record

### Commit Message Format

Always include bead ID in parentheses:

- `feat: add user authentication ({{BEAD_PREFIX}}-1234)`
- `fix: resolve login bug ({{BEAD_PREFIX}}-5678)`
- `chore: update dependencies ({{BEAD_PREFIX}}-9012)`
- `spec: add payment processing proposal (openspec/add-payment)`

---

## Branch Naming Conventions

| Workflow | Branch Pattern | Example |
|----------|----------------|---------|
| Spec | `openspec/<change-id>` | `openspec/add-two-factor-auth` |
| Work | `bd/<bead-id>` | `bd/{{BEAD_PREFIX}}-1234` |
| Chore | `bd/<bead-id>` | `bd/{{BEAD_PREFIX}}-5678` |
| Bug Report | `report/<bead-id>` | `report/{{BEAD_PREFIX}}-9012` |

---

## Landing the Plane (Session Completion)

**When ending a work session**, you MUST complete ALL steps below. Work is NOT complete until `git push` succeeds.

**MANDATORY WORKFLOW:**

1. **File issues for remaining work** - Create beads for anything that needs follow-up
2. **Run quality gates** (if code changed) - Tests, linters, builds
3. **Update bead status** - Close finished work, update in-progress items with state
4. **PUSH TO REMOTE** - This is MANDATORY:

   ```bash
   git pull --rebase
   bd sync
   git push
   git status  # MUST show "up to date with origin"
   ```

5. **Clean up** - Clear stashes, prune remote branches
6. **Verify** - All changes committed AND pushed
7. **Hand off** - Provide context for next session

**CRITICAL RULES:**

- Work is NOT complete until `git push` succeeds
- NEVER stop before pushing - that leaves work stranded locally
- NEVER say "ready to push when you are" - YOU must push
- If push fails, resolve and retry until it succeeds

---

## Documentation Reference

**OpenSpec:**
- `@openspec/AGENTS.md` - Detailed OpenSpec conventions

---

## Quick Reference

**Workflow Prompts:**
- `@.github/prompts/spec.prompt.md` - Create OpenSpec change proposal
- `@.github/prompts/spec-complete.prompt.md` - Finalize spec and create beads
- `@.github/prompts/work.prompt.md` - Find and implement work
- `@.github/prompts/work-on.prompt.md` - Work on specific bead
- `@.github/prompts/chore.prompt.md` - Create and complete chore
- `@.github/prompts/bug.prompt.md` - Report bug

**Beads Commands:**
- `bd ready` - Find unblocked work
- `bd create "Title" --type task --priority 2` - Create issue
- `bd close <id>` - Complete work
- `bd sync` - Sync with git (run before push)
- `bd prime` - Get full workflow context

For full beads workflow details: `bd prime`
