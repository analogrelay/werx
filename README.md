# Agent Workflow Template

This directory contains a reusable template for setting up AI agent workflows in any repository.

## What's Included

- **Root AGENTS.md**: Main workflow documentation with beads and OpenSpec integration
- **.github/prompts/**: Workflow-specific prompts for common tasks
  - `init.prompt.md` - Initialize workflows in a new repository
  - `spec.prompt.md` - Create OpenSpec change proposals
  - `spec-complete.prompt.md` - Finalize specs and create beads
  - `work.prompt.md` - Find and implement work
  - `work-on.prompt.md` - Work on a specific bead
  - `chore.prompt.md` - Handle maintenance tasks
  - `bug.prompt.md` - Report bugs
- **OPENSPEC_AGENTS_TEMPLATE.md**: Template for OpenSpec-specific instructions (will be moved to `openspec/AGENTS.md` during init)

## How to Use

1. **Copy the template to your repository:**
   ```bash
   cp -r template/.github /path/to/your/repo/
   cp template/AGENTS.md /path/to/your/repo/
   cp template/OPENSPEC_AGENTS_TEMPLATE.md /path/to/your/repo/
   ```

2. **Run the init prompt:**
   - Use `@.github/prompts/init.prompt.md` in your AI assistant
   - Answer questions about your project (name, bead prefix, languages, etc.)
   - The agent will:
     - Run `bd init` to set up beads
     - Run `openspec init` to set up OpenSpec
     - Replace placeholders in templates with your project details
     - Commit and push the initialization

3. **Start using workflows:**
   - `@.github/prompts/spec.prompt.md` - Create specifications
   - `@.github/prompts/work.prompt.md` - Work on tasks
   - `@.github/prompts/chore.prompt.md` - Handle chores
   - `@.github/prompts/bug.prompt.md` - Report bugs

## Placeholders

The template uses these placeholders that get replaced during init:

- `{{PROJECT_NAME}}` - Your project name
- `{{BEAD_PREFIX}}` - Your bead ID prefix (e.g., "sf", "api", "web")

## Requirements

- `bd` (beads) CLI tool installed
- `openspec` CLI tool installed
- Git repository

## Philosophy

- **Beads is the memory** - All work tracking through beads issues
- **Branch per unit of work** - Clear branching conventions
- **Commit conventions** - Include bead IDs in commits
- **Push before finishing** - Work isn't done until pushed

## Learn More

After initialization, see:
- `AGENTS.md` in your repo root for workflow overview
- `openspec/AGENTS.md` for OpenSpec-specific conventions
- `.github/prompts/` for individual workflow details
