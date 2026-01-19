---
description: Initialize project workflows - setup beads and OpenSpec with project-specific configuration.
---

<!-- AGENTS:START -->
**Purpose**

This prompt initializes a new repository with the agent workflow system, including beads issue tracking and OpenSpec specification management.

**Steps**

Track these steps as TODOs and complete them one by one.

1. **Interview the user**
   - Project name
   - Project description
   - Bead ID prefix (2-4 lowercase letters, e.g., "sf", "api", "web")
   - Primary programming language(s)
   - Repository type (library, application, service, etc.)
   - Any specific quality gates (linters, tests, build commands)

2. **Initialize beads**
   - Run `bd init` to create the `.beads/` directory structure
   - Verify initialization completed successfully
   - Note the default bead prefix if one was assigned

3. **Initialize OpenSpec**
   - Run `openspec init` to create the `openspec/` directory structure
   - This creates `project.md`, `specs/`, `changes/`, and instruction files
   - Verify initialization completed successfully

4. **Replace OpenSpec AGENTS.md**
   - The generic OpenSpec AGENTS.md needs project-specific customization
   - Read `@OPENSPEC_AGENTS_TEMPLATE.md` from the template directory
   - Replace placeholders:
     - `{{BEAD_PREFIX}}` with the bead prefix
     - `{{PROJECT_NAME}}` with the project name
   - Write the customized content to `openspec/AGENTS.md`

5. **Update root AGENTS.md**
   - Read `@AGENTS.md` in the repository root
   - Replace placeholders:
     - `{{BEAD_PREFIX}}` with the bead prefix
     - `{{PROJECT_NAME}}` with the project name
   - Update the file in place

6. **Update project.md**
   - Edit `openspec/project.md` with project-specific information:
     - Project name and description
     - Key conventions
     - Technology stack
     - Quality gates (test commands, linters, build process)
   - Keep it concise but informative

7. **Create .gitignore entries**
   - Ensure `.gitignore` includes common patterns for the project's language/framework
   - Add `.beads/.cache` if not already present
   - Add `.temp` for temporary agent files

8. **Commit initialization**
   ```bash
   git add .beads/ openspec/ AGENTS.md .gitignore
   git commit -m "chore: initialize agent workflows"
   git push
   ```

9. **Create initial project structure bead** (optional)
   - If the project needs basic scaffolding, create a bead to track it
   - Example: `bd create "Set up initial project structure" --type task --priority 2`
   - This becomes the first work item for the project

10. **Report completion**
    - Confirm all systems initialized
    - Show bead prefix
    - Explain where the user can find workflow documentation
    - Suggest next steps (e.g., create first spec, set up initial structure)

**Reference**
- Use `bd --help` to see available beads commands
- Use `openspec --help` to see available OpenSpec commands
- Reference `@AGENTS.md` for the full workflow overview
- Reference `@openspec/AGENTS.md` for OpenSpec-specific conventions
<!-- AGENTS:END -->
