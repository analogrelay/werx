# Quick Start Guide

Get up and running with agent workflows in 5 minutes.

## Prerequisites

Install required tools:
```bash
# Install beads (check latest installation method)
# Install openspec (check latest installation method)
```

## Step 1: Copy Template Files

```bash
# From this template directory, copy to your repository
cd /path/to/your/repo

# Copy workflow files
cp -r /path/to/template/.github .
cp /path/to/template/AGENTS.md .
cp /path/to/template/OPENSPEC_AGENTS_TEMPLATE.md .

# Merge .gitignore (or append if you have one)
cat /path/to/template/.gitignore >> .gitignore
```

## Step 2: Initialize Workflows

Open your AI assistant (Claude, GitHub Copilot, etc.) and run:

```
@.github/prompts/init.prompt.md
```

The agent will ask you:
- Project name
- Bead ID prefix (e.g., "myapp" → "ma")
- Primary programming language(s)
- Repository type
- Quality gates (test/lint commands)

Then it will:
- Run `bd init`
- Run `openspec init`
- Replace placeholders with your project details
- Move OPENSPEC_AGENTS_TEMPLATE.md → openspec/AGENTS.md
- Commit and push everything

## Step 3: Create Your First Spec

```
@.github/prompts/spec.prompt.md

"Create a spec for adding user authentication"
```

The agent will:
- Create `openspec/<change-id>` branch
- Draft proposal.md, tasks.md, and spec deltas
- Validate the spec
- Ask for your approval

## Step 4: Create Work Items

After approving the spec:

```
@.github/prompts/spec-complete.prompt.md
```

The agent will:
- Create an epic in beads
- Create child tasks from tasks.md
- Link everything together
- Push to remote

## Step 5: Start Working

```
@.github/prompts/work.prompt.md
```

The agent will:
- Find highest priority work with `bd ready`
- Create `bd/<bead-id>` branch
- Implement the task
- Run tests
- Push to remote

## Common Commands

### Finding Work
```
bd ready          # Show unblocked, prioritized work
bd list           # List all beads
bd show <id>      # View bead details
```

### Specs
```
openspec list              # Active changes
openspec list --specs      # Existing capabilities
openspec show <id>         # View change/spec details
openspec validate --strict # Check everything
```

### Daily Workflows

**Create a feature:**
```
@.github/prompts/spec.prompt.md
# Design it, get approval
@.github/prompts/spec-complete.prompt.md
# Create beads
@.github/prompts/work.prompt.md
# Implement it
```

**Fix a bug:**
```
@.github/prompts/bug.prompt.md
# Report it
@.github/prompts/work-on.prompt.md <bead-id>
# Fix it
```

**Do maintenance:**
```
@.github/prompts/chore.prompt.md "Update dependencies"
# Agent creates bead and does the work
```

## Tips

1. **Always push** - Work isn't done until `git push` succeeds
2. **Include bead IDs** - Every commit: `feat: xyz (prefix-123)`
3. **Use the right workflow** - Specs for design, work for implementation
4. **Don't yakshave** - Create new beads for scope creep
5. **Update beads** - Keep them current for future sessions

## Troubleshooting

**"bd: command not found"**
- Install beads CLI tool

**"openspec: command not found"**
- Install openspec CLI tool

**Placeholders not replaced**
- Run `@.github/prompts/init.prompt.md` again

**Can't find work**
- Check `bd ready` or `bd list`
- Create work with `@.github/prompts/spec.prompt.md`

**Validation errors**
- Read error message
- Fix spec file format
- Ensure scenarios use `#### Scenario:` format
- Run `openspec validate --strict --no-interactive`

## Next Steps

- Read `AGENTS.md` for full workflow documentation
- Read `openspec/AGENTS.md` for spec conventions
- Run `bd prime` for beads workflow details
- Customize `openspec/project.md` for your project

## Getting Help

- Check workflow prompts in `.github/prompts/`
- Run `bd --help` or `openspec --help`
- Review beads: `bd list` and `bd show <id>`
- Review specs: `openspec list` and `openspec show <id>`

---

**You're ready!** Start with `@.github/prompts/spec.prompt.md` to create your first feature.
