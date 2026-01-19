# Template Index

Welcome to the Agent Workflow Template! This index helps you navigate all the template files.

## 📚 Documentation Files

### Start Here
- **[QUICKSTART.md](QUICKSTART.md)** - Get running in 5 minutes
- **[README.md](README.md)** - Overview and basic usage
- **[INSTALLATION.md](INSTALLATION.md)** - Tool installation requirements

### Reference
- **[TEMPLATE_OVERVIEW.md](TEMPLATE_OVERVIEW.md)** - Complete template documentation
- **[AGENTS.md](AGENTS.md)** - Main workflow guide (copy to repo root)
- **[OPENSPEC_AGENTS_TEMPLATE.md](OPENSPEC_AGENTS_TEMPLATE.md)** - OpenSpec instructions template

## 🔧 Template Files

### Configuration
- **[.gitignore](.gitignore)** - Git ignore patterns

### Workflow Prompts (`.github/prompts/`)
- **[init.prompt.md](.github/prompts/init.prompt.md)** - ⭐ Initialize new repository
- **[spec.prompt.md](.github/prompts/spec.prompt.md)** - Create OpenSpec proposals
- **[spec-complete.prompt.md](.github/prompts/spec-complete.prompt.md)** - Finalize specs
- **[work.prompt.md](.github/prompts/work.prompt.md)** - Find and implement work
- **[work-on.prompt.md](.github/prompts/work-on.prompt.md)** - Work on specific bead
- **[chore.prompt.md](.github/prompts/chore.prompt.md)** - Handle maintenance tasks
- **[bug.prompt.md](.github/prompts/bug.prompt.md)** - Report bugs

## 🚀 Usage Flow

### First Time Setup
1. Read [INSTALLATION.md](INSTALLATION.md) - Install required tools
2. Read [QUICKSTART.md](QUICKSTART.md) - Copy files and initialize
3. Run `@.github/prompts/init.prompt.md` - Initialize your project

### Daily Workflow
1. **Design features**: `@.github/prompts/spec.prompt.md`
2. **Create work items**: `@.github/prompts/spec-complete.prompt.md`
3. **Implement tasks**: `@.github/prompts/work.prompt.md`
4. **Report bugs**: `@.github/prompts/bug.prompt.md`
5. **Do maintenance**: `@.github/prompts/chore.prompt.md`

## 📖 Learn More

### After Initialization
Once you've run the init prompt, you'll have:

- `AGENTS.md` in your repo root - Main workflow guide
- `openspec/AGENTS.md` - OpenSpec conventions
- `.beads/` directory - Issue tracking data
- `openspec/` directory - Specification management

### Key Commands
```bash
# Beads commands
bd ready              # Find work
bd list               # List all issues
bd show <id>          # View details
bd create "Title"     # Create issue
bd close <id>         # Complete work

# OpenSpec commands
openspec list         # Active changes
openspec list --specs # Existing specs
openspec show <id>    # View details
openspec validate     # Check specs
```

## 🎯 Quick Reference

### Branch Naming
- `openspec/<change-id>` - Specification work
- `bd/<bead-id>` - Implementation work  
- `report/<bead-id>` - Bug reports

### Commit Format
- `feat: add login (prefix-123)` - Features
- `fix: resolve bug (prefix-456)` - Fixes
- `chore: update deps (prefix-789)` - Maintenance
- `spec: add auth (openspec/add-auth)` - Specs

### Workflow Types
- **Spec** → Design new features
- **Work** → Implement tasks
- **Chore** → Maintenance work
- **Bug** → Report issues

## 🔑 Key Concepts

### Beads
- Permanent memory for work
- Types: epic, task, bug, chore
- Priorities: 0 (critical) to 3 (low)
- Links: parent/child, blocks, discovered-from

### OpenSpec
- **Specs** = Current truth (what IS)
- **Changes** = Proposals (what SHOULD be)
- **Deltas** = ADDED/MODIFIED/REMOVED
- **Archive** = Completed changes

### Philosophy
1. Beads is the memory
2. Branch per unit of work
3. Commit conventions matter
4. Push before finishing
5. Specs before code

## 📋 File Checklist

When copying to a new repo, you need:

- [ ] `.github/prompts/` directory (all 7 prompt files)
- [ ] `AGENTS.md` (copy to root)
- [ ] `OPENSPEC_AGENTS_TEMPLATE.md` (temporary, moved during init)
- [ ] `.gitignore` entries (merge with existing)

Do NOT copy (created by init):
- ❌ `.beads/` - Created by `bd init`
- ❌ `openspec/` - Created by `openspec init`
- ❌ Project-specific files

## 🛠️ Customization

After initialization, customize:

1. `openspec/project.md` - Project conventions
2. Quality gates in project.md
3. Workflow prompts if needed
4. Additional .gitignore patterns

## ❓ Getting Help

If something isn't clear:

1. Check [QUICKSTART.md](QUICKSTART.md) for common tasks
2. Read [TEMPLATE_OVERVIEW.md](TEMPLATE_OVERVIEW.md) for concepts
3. Review specific workflow prompts in `.github/prompts/`
4. Run `bd --help` or `openspec --help`

## 📦 What You Get

This template provides:

- ✅ Complete agent workflow system
- ✅ Issue tracking via beads
- ✅ Specification management via OpenSpec
- ✅ Branch and commit conventions
- ✅ 7 workflow prompts for AI agents
- ✅ Initialization automation
- ✅ Documentation and guides

## 🎉 Ready to Start?

1. Install tools: [INSTALLATION.md](INSTALLATION.md)
2. Quick setup: [QUICKSTART.md](QUICKSTART.md)
3. First task: `@.github/prompts/init.prompt.md`

Happy coding! 🚀
