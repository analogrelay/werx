# Installation Requirements

## Required Tools

To use this agent workflow template, you need the following tools installed:

### 1. Beads (`bd`)

**Purpose:** Issue tracking and work management integrated with git.

**Installation:** 
- Check the beads repository for latest installation instructions
- Typically installed via Cargo: `cargo install beads` (verify actual package name)
- Or from source if not published

**Verify installation:**
```bash
bd --version
bd --help
```

### 2. OpenSpec (`openspec`)

**Purpose:** Specification management and change tracking.

**Installation:**
- Check the openspec repository for latest installation instructions
- Typically installed via Cargo: `cargo install openspec` (verify actual package name)
- Or from source if not published

**Verify installation:**
```bash
openspec --version
openspec --help
```

### 3. Git

**Purpose:** Version control system.

**Installation:**
- macOS: `brew install git` or Xcode Command Line Tools
- Linux: `apt install git` or `yum install git`
- Windows: Download from git-scm.com

**Verify installation:**
```bash
git --version
```

### 4. AI Assistant

**Purpose:** Run the workflow prompts.

**Options:**
- **Claude Desktop** (Anthropic) - Native prompt file support
- **GitHub Copilot Chat** - Prompt file support via `@`
- **Other AI assistants** - As long as they support reading markdown files

**Note:** You need an AI assistant that can:
- Read local files via `@` notation
- Execute bash commands
- Create and edit files

## Optional Tools

### ripgrep (`rg`)

**Purpose:** Fast code search (used in OpenSpec workflows).

**Installation:**
- macOS: `brew install ripgrep`
- Linux: `apt install ripgrep`
- Windows: `choco install ripgrep`

**Verify installation:**
```bash
rg --version
```

### jq

**Purpose:** JSON processing (useful for debugging OpenSpec output).

**Installation:**
- macOS: `brew install jq`
- Linux: `apt install jq`
- Windows: `choco install jq`

**Verify installation:**
```bash
jq --version
```

## Development Environment (Optional)

The template doesn't include devenv files, but you may want:

### Nix with devenv

**Purpose:** Reproducible development environments.

**Installation:**
- Follow instructions at: https://devenv.sh/getting-started/

**Note:** This is optional and project-specific. The template doesn't include devenv files.

## Verification Checklist

Before using the template, verify:

- [ ] `bd --version` works
- [ ] `openspec --version` works
- [ ] `git --version` works
- [ ] AI assistant can read `@filename.md` files
- [ ] AI assistant can execute bash commands
- [ ] You're in a git repository
- [ ] `rg --version` works (optional but recommended)

## Next Steps

Once all tools are installed:

1. Copy template files to your repository
2. Run `@.github/prompts/init.prompt.md` in your AI assistant
3. Follow the initialization prompts
4. Start working with `@.github/prompts/work.prompt.md`

See `QUICKSTART.md` for detailed setup instructions.

## Troubleshooting

**Command not found errors:**
- Ensure tools are in your PATH
- Restart your terminal after installation
- Check installation instructions for your OS

**Permission errors:**
- You may need to use `sudo` for system-wide installation
- Or use user-local installation (e.g., `cargo install --locked`)

**Git repository required:**
- Initialize git: `git init`
- Set up remote: `git remote add origin <url>`

**AI assistant issues:**
- Ensure you're using a compatible AI assistant
- Update to the latest version
- Check that file reading via `@` notation is supported
