# Agent Workflow Template Overview

This template provides a complete AI agent workflow system that can be applied to any repository. It combines beads issue tracking with OpenSpec specification management.

## Structure

```
template/
├── .github/
│   └── prompts/          # Workflow prompts for AI agents
│       ├── init.prompt.md
│       ├── spec.prompt.md
│       ├── spec-complete.prompt.md
│       ├── work.prompt.md
│       ├── work-on.prompt.md
│       ├── chore.prompt.md
│       └── bug.prompt.md
├── .gitignore            # Common ignores + beads cache
├── AGENTS.md             # Main workflow documentation
├── OPENSPEC_AGENTS_TEMPLATE.md  # OpenSpec-specific instructions
└── README.md             # Usage guide

NOT INCLUDED (created by init commands):
├── .beads/               # Created by `bd init`
├── openspec/             # Created by `openspec init`
├── devenv.{nix,yaml,lock}  # Project-specific dev environment
└── language-specific files   # Cargo.toml, package.json, etc.
```

## Workflows Provided

### 1. Spec Workflow (`@.github/prompts/spec.prompt.md`)
- Create OpenSpec change proposals
- Design new features before implementation
- Branch: `openspec/<change-id>`

### 2. Work Workflow (`@.github/prompts/work.prompt.md`)
- Find and implement prioritized work
- Uses `bd ready` to discover tasks
- Branch: `bd/<bead-id>`

### 3. Chore Workflow (`@.github/prompts/chore.prompt.md`)
- Handle maintenance tasks
- Track dependency updates, refactoring
- Branch: `bd/<bead-id>`

### 4. Bug Reporting (`@.github/prompts/bug.prompt.md`)
- Capture bug reports systematically
- Interview user for details
- Branch: `report/<bead-id>`

### 5. Init Workflow (`@.github/prompts/init.prompt.md`)
- **START HERE** for new repositories
- Interviews user for project details
- Runs `bd init` and `openspec init`
- Replaces all placeholders
- Commits and pushes initialization

## Key Concepts

### Beads (Issue Tracking)
- **Memory**: Beads are the permanent record of work
- **Types**: epic, task, bug, chore
- **Linking**: parent/child, blocks, discovered-from, related
- **Priority**: 0 (critical) to 3 (low)

### OpenSpec (Specification Management)
- **Specs**: Current truth (what IS built)
- **Changes**: Proposals (what SHOULD change)
- **Deltas**: ADDED/MODIFIED/REMOVED requirements
- **Archiving**: Move completed changes to archive

### Branch Strategy
- `openspec/<change-id>` - Specification work
- `bd/<bead-id>` - Implementation work
- `report/<bead-id>` - Bug reports

### Commit Conventions
- Include bead ID: `feat: add login ({{BEAD_PREFIX}}-1234)`
- Spec commits: `spec: add auth proposal (openspec/add-auth)`

## Placeholders

Replace these during initialization:

- `{{PROJECT_NAME}}` - Your project name
- `{{BEAD_PREFIX}}` - Bead ID prefix (2-4 letters, e.g., "sf", "api")

## Installation Steps

1. **Copy template files:**
   ```bash
   cp -r /path/to/template/.github /path/to/your/repo/
   cp /path/to/template/AGENTS.md /path/to/your/repo/
   cp /path/to/template/OPENSPEC_AGENTS_TEMPLATE.md /path/to/your/repo/
   cp /path/to/template/.gitignore /path/to/your/repo/.gitignore
   ```

2. **Run init prompt:**
   - Open AI assistant in your repo
   - Use: `@.github/prompts/init.prompt.md`
   - Answer questions about your project
   - Agent will initialize everything and commit

3. **Start working:**
   - Create specs: `@.github/prompts/spec.prompt.md`
   - Work on tasks: `@.github/prompts/work.prompt.md`
   - Report bugs: `@.github/prompts/bug.prompt.md`

## Philosophy

1. **Beads is the memory** - All planning and state lives in beads
2. **Branch per unit of work** - Clean, focused branches
3. **Commit conventions** - Always include bead IDs
4. **Push before finishing** - Work isn't done until pushed
5. **Specs before code** - Design complex changes first

## Requirements

- Git repository
- `bd` CLI tool (beads)
- `openspec` CLI tool
- AI assistant with prompt support (e.g., Claude, GitHub Copilot)

## Customization Points

After initialization, you can customize:

1. **openspec/project.md** - Project-specific conventions
2. **Quality gates** - Add test/lint commands to project.md
3. **Prompts** - Modify .github/prompts/ for your workflow
4. **Bead prefix** - Set during init, used in all commits

## Example Usage

### New Feature
1. `@.github/prompts/spec.prompt.md` - Design the feature
2. User approves spec
3. `@.github/prompts/spec-complete.prompt.md` - Create beads
4. `@.github/prompts/work.prompt.md` - Implement tasks

### Bug Fix
1. `@.github/prompts/bug.prompt.md` - Report bug
2. `@.github/prompts/work-on.prompt.md <bead-id>` - Fix it

### Maintenance
1. `@.github/prompts/chore.prompt.md` - Create and do chore

## Benefits

- **Consistency**: Same workflow across all projects
- **Documentation**: Beads provide automatic work history
- **Clarity**: Specs document design decisions
- **Handoff**: Easy context switching between sessions
- **Collaboration**: Clear branches and commit messages

## Learn More

After setup, read:
- `AGENTS.md` - Full workflow documentation
- `openspec/AGENTS.md` - OpenSpec conventions
- `bd prime` - Beads workflow details
- `openspec --help` - OpenSpec commands
