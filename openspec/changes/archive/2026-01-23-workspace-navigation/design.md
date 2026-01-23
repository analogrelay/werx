# Design: Workspace Navigation with Shell Integration

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────┐
│                         User's Shell                        │
│                                                             │
│  ┌───────────────────────────────────────────────────────┐ │
│  │  forge() {                                            │ │
│  │    output=$(FORGE_BIN="${FORGE_BIN:-forge}" \         │ │
│  │             "$FORGE_BIN" "$@" 2>&1)                   │ │
│  │    directives=$(echo "$output" | grep "^@forge:")     │ │
│  │    echo "$output" | grep -v "^@forge:"                │ │
│  │                                                        │ │
│  │    for directive in $directives; do                   │ │
│  │      case "$directive" in                             │ │
│  │        @forge:change_directory:*)                     │ │
│  │          cd "${directive#@forge:change_directory:}"   │ │
│  │          ;;                                            │ │
│  │      esac                                              │ │
│  │    done                                                │ │
│  │  }                                                     │ │
│  └───────────────────────────────────────────────────────┘ │
│                            ↓                                │
└────────────────────────────┼────────────────────────────────┘
                             ↓
                  ┌──────────────────────┐
                  │   forge binary       │
                  │                      │
                  │  ┌────────────────┐  │
                  │  │  forge go      │  │
                  │  │  - List        │  │
                  │  │  - Filter      │  │
                  │  │  - Select      │  │
                  │  │  - Emit        │  │
                  │  └────────────────┘  │
                  │           ↓          │
                  │  ┌────────────────┐  │
                  │  │  Directives    │  │
                  │  │  @forge:...    │  │
                  │  └────────────────┘  │
                  └──────────────────────┘
                             ↓
                    stderr with directives
                    stdout with regular output
```

## Component Design

### 1. Workspace Navigation (`forge go`)

#### Data Flow

```
User Input: "forge go feat"
       ↓
1. List all workspaces (via list_workspaces())
       ↓
2. Build fuzzy search items:
   - Primary text: "repo-name/workspace-name"
   - Secondary text: "branch: <branch> | <full-path>"
       ↓
3. Launch fuzzy finder with "feat" pre-filled
       ↓
4. User selects or query matches single workspace
       ↓
5. Emit directive: "@forge:change_directory:/full/path/to/workspace"
       ↓
6. Exit with success code
```

#### Fuzzy Matcher Choice: `skim`

**Rationale**:
- Full interactive TUI matching fzf UX (familiar to users)
- Maintained and actively used (used by helix, others)
- Rich features: preview, multi-line, custom display
- Pure Rust (no external dependencies)

**Alternative considered**: `nucleo` (helix's new matcher)
- Pros: Faster, lighter
- Cons: More manual TUI construction, less fzf-like OOTB

#### Workspace Display Format

```
torvalds-linux/main                    branch: main | ~/forge/torvalds-linux/main
torvalds-linux/feature-x11             branch: feature/x11 | ~/forge/torvalds-linux/feature-x11
greg-linux/development                 branch: dev | ~/forge/greg-linux/development
```

**Search optimizations**:
- Fuzzy match on full string: "repo/workspace"
- Show branch and path as context
- Order results by relevance (skim handles this)

#### Direct Navigation Logic

```rust
let query = args.query.unwrap_or_default();
let workspaces = list_workspaces(&forge)?;

if !query.is_empty() {
    // Pre-filter workspaces
    let matches: Vec<_> = workspaces.iter()
        .filter(|ws| fuzzy_match(&format!("{}/{}", ws.repository, ws.name), &query))
        .collect();

    if matches.len() == 1 {
        // Direct navigation - single match
        emit_change_directory(matches[0].path);
        return Ok(());
    }
}

// Launch interactive search with query pre-filled
interactive_select(workspaces, query)?;
```

### 2. Shell Integration (`forge shell`)

#### Shell Init Command

```
forge shell init bash   → outputs bash function
forge shell init zsh    → outputs zsh function
forge shell init fish   → error: "not yet supported"
```

**Output is pure shell code** that user evals:

```bash
# ~/.bashrc or ~/.zshrc
eval "$(forge shell init bash)"
```

#### Generated Shell Function (Bash)

```bash
forge() {
  # Use FORGE_BIN if set, otherwise 'forge' from PATH
  local forge_bin="${FORGE_BIN:-forge}"

  # Capture combined output (stdout + stderr)
  local output
  output=$("$forge_bin" "$@" 2>&1)
  local exit_code=$?

  # Extract directives (lines starting with @forge:)
  local directives
  directives=$(echo "$output" | grep "^@forge:")

  # Print non-directive output
  echo "$output" | grep -v "^@forge:"

  # Process directives
  while IFS= read -r directive; do
    if [[ "$directive" =~ ^@forge:change_directory:(.+)$ ]]; then
      local target_dir="${BASH_REMATCH[1]}"
      if [ -d "$target_dir" ]; then
        cd "$target_dir" || true
      else
        echo "forge: directory does not exist: $target_dir" >&2
      fi
    fi
  done <<< "$directives"

  return $exit_code
}
```

#### Generated Shell Function (Zsh)

```zsh
forge() {
  # Use FORGE_BIN if set, otherwise 'forge' from PATH
  local forge_bin="${FORGE_BIN:-forge}"

  # Capture combined output
  local output
  output=$($forge_bin "$@" 2>&1)
  local exit_code=$?

  # Extract and process directives
  local directives
  directives=$(echo "$output" | grep "^@forge:")

  # Print non-directive output
  echo "$output" | grep -v "^@forge:"

  # Process directives
  while IFS= read -r directive; do
    if [[ "$directive" =~ ^@forge:change_directory:(.+)$ ]]; then
      local target_dir="${match[1]}"
      if [ -d "$target_dir" ]; then
        cd "$target_dir" 2>/dev/null || true
      else
        echo "forge: directory does not exist: $target_dir" >&2
      fi
    fi
  done <<< "$directives"

  return $exit_code
}
```

**Key differences**:
- Bash uses `BASH_REMATCH`, zsh uses `match`
- Both versions identical in structure
- Both support `FORGE_BIN` for testing/custom paths

### 3. Directive Protocol

#### Format

```
@forge:<directive_name>:<argument>
```

**Rules**:
- Emitted on **stderr** (keeps stdout clean for normal output)
- One directive per line
- Directive name is `[a-z_]+`
- Argument is everything after second `:`
- No newlines in arguments (use URL encoding if needed)

#### Defined Directives

| Directive | Argument | Purpose | Example |
|-----------|----------|---------|---------|
| `change_directory` | absolute path | Change shell's cwd | `@forge:change_directory:/Users/me/forge/repo/workspace` |

**Future directives** (not implemented now):
- `set_env:<key>=<value>` - Set environment variable
- `unset_env:<key>` - Unset environment variable
- `prompt_update:<text>` - Update shell prompt

#### Why Stderr?

1. **Separation of concerns**: Stdout is for user-facing output, stderr for control
2. **Composability**: `forge go | grep` works correctly (directives not captured)
3. **Precedent**: Similar to how `ssh-agent` outputs shell commands
4. **Simplicity**: No need for special flags like `--shell-mode`

### 4. Error Handling

#### Binary Errors

```rust
// If workspace not found or other error
eprintln!("Error: No matching workspaces found");
std::process::exit(1);
// Shell wrapper returns exit code, no cd happens
```

#### Shell Wrapper Errors

```bash
# Directory doesn't exist (binary bug or race condition)
if [ ! -d "$target_dir" ]; then
  echo "forge: directory does not exist: $target_dir" >&2
  # Don't change directory, continue with exit code
fi
```

#### User Cancellation (ESC in fuzzy finder)

```rust
// skim returns None when user cancels
if let Some(selected) = selected_item {
    emit_change_directory(selected.path);
} else {
    // User cancelled - just exit, no error
    std::process::exit(0);
}
```

## Implementation Details

### Fuzzy Search Integration

#### Dependency Addition

```toml
# Cargo.toml
[dependencies]
skim = "0.10"
```

#### Usage Pattern

```rust
use skim::prelude::*;

pub fn fuzzy_select_workspace(
    workspaces: Vec<Workspace>,
    initial_query: Option<String>,
) -> Result<Option<Workspace>> {
    // Build skim items
    let items: Vec<Arc<dyn SkimItem>> = workspaces
        .iter()
        .map(|ws| {
            Arc::new(WorkspaceItem {
                display: format!("{}/{}", ws.repository, ws.name),
                preview: format!("branch: {} | {}",
                    ws.branch.as_deref().unwrap_or("(detached)"),
                    ws.path.display()),
                workspace: ws.clone(),
            }) as Arc<dyn SkimItem>
        })
        .collect();

    let options = SkimOptionsBuilder::default()
        .height(Some("50%"))
        .multi(false)
        .query(initial_query)
        .build()?;

    let selected = Skim::run_with(&options, Some(rx))?
        .selected_items
        .first()
        .map(|item| {
            item.as_any()
                .downcast_ref::<WorkspaceItem>()
                .unwrap()
                .workspace
                .clone()
        });

    Ok(selected)
}
```

### Directive Emission

```rust
pub fn emit_change_directory<P: AsRef<Path>>(path: P) {
    eprintln!("@forge:change_directory:{}", path.as_ref().display());
}

pub fn emit_directive(name: &str, arg: &str) {
    // Validate directive name (paranoia)
    assert!(name.chars().all(|c| c.is_ascii_lowercase() || c == '_'));
    // Validate no newlines in arg
    assert!(!arg.contains('\n'));

    eprintln!("@forge:{}:{}", name, arg);
}
```

### Shell Init Implementation

```rust
pub fn cmd_shell_init(shell: &str) -> Result<()> {
    match shell {
        "bash" => {
            print!("{}", include_str!("../shell/init.bash"));
            Ok(())
        }
        "zsh" => {
            print!("{}", include_str!("../shell/init.zsh"));
            Ok(())
        }
        _ => {
            Err(anyhow!("Unsupported shell: {}\nSupported shells: bash, zsh", shell))
        }
    }
}
```

**Shell scripts stored in**: `shell/init.bash`, `shell/init.zsh` (embedded at compile time)

## Testing Strategy

### Unit Tests

1. **Fuzzy matching**:
   - Test workspace filtering by query
   - Test exact match detection (single result)

2. **Directive emission**:
   - Test directive format validation
   - Test path escaping (if needed)

3. **Workspace listing**:
   - Already tested via existing `list_workspaces()`

### Integration Tests

1. **Shell wrapper** (bash script tests):
   - Create mock binary that outputs directives
   - Source shell function
   - Call function, verify `cd` occurred
   - Test error cases (missing directory)

2. **End-to-end**:
   - Create forge with workspaces
   - Run `forge go` with query
   - Verify directive output
   - (Manual testing of full shell integration)

### Manual Testing Checklist

- [ ] `forge go` launches interactive search
- [ ] `forge go <partial>` pre-fills query
- [ ] `forge go <exact-match>` navigates directly
- [ ] `forge shell init bash` outputs valid bash
- [ ] `forge shell init zsh` outputs valid zsh
- [ ] Shell integration changes directory correctly
- [ ] ESC in fuzzy search doesn't break shell
- [ ] Directive appears on stderr, not stdout
- [ ] Works with `FORGE_BIN` environment variable

## Future Extensibility

### Additional Directives

The protocol is designed to be extended:

```
# Future possibilities
@forge:set_env:FORGE_CURRENT_WORKSPACE=/path/to/workspace
@forge:prompt_update:  (forge:my-workspace)
@forge:completion:_forge_workspaces "ws1\nws2\nws3"
```

### Additional Shells

Structure for adding new shells:

1. Create `shell/init.fish` (or other shell)
2. Implement directive parsing in that shell's syntax
3. Add case to `cmd_shell_init()`
4. Test and document

### MRU Ordering (Future)

Could track recently visited workspaces:

```
~/.forge/config.toml:
[navigation]
mru = [
  "/Users/me/forge/project/feature-x",
  "/Users/me/forge/other/main",
]
```

Fuzzy search could prioritize MRU workspaces in results.

### Shell Completions (Future)

Could generate completion scripts that use workspace list:

```bash
# bash completion
_forge_go_completions() {
  COMPREPLY=( $(forge workspaces list --format=names | grep "^${COMP_WORDS[COMP_CWORD]}") )
}
complete -F _forge_go_completions forge go
```

## Security Considerations

### Directive Injection

**Risk**: Malicious repository names creating fake directives

**Mitigation**:
- Directives only emitted by trusted binary code, not from user data
- Repository/workspace names never directly concatenated into directives
- Path arguments are validated as absolute paths before emission

### Path Traversal

**Risk**: Binary emits directive to change to attacker-controlled directory

**Mitigation**:
- Binary only emits paths it constructed from forge root
- Shell wrapper validates directory exists before `cd`
- User has already cloned the repository (trust boundary)

### Binary Replacement

**Risk**: Attacker replaces `forge` binary with malicious version

**Mitigation**:
- Same risk as any CLI tool (user's PATH is trusted)
- `FORGE_BIN` allows testing but requires user opt-in
- Shell wrapper doesn't execute arbitrary code from binary output

## Performance Considerations

### Fuzzy Search Responsiveness

- **Workspace count**: Expect < 1000 workspaces per forge
- **skim performance**: Handles 100k+ items responsively
- **Workspace listing**: Already cached/fast (`list_workspaces()`)

### Shell Wrapper Overhead

- **Directive parsing**: Simple `grep` and loop, negligible
- **Non-`go` commands**: No overhead (directives not present)
- **Impact**: ~1-2ms added to `forge go` invocation (imperceptible)

## Compatibility

### Shell Versions

- **Bash**: 4.0+ (macOS default is 3.2, will support)
- **Zsh**: 5.0+ (macOS default is 5.8+)

### Platform Support

- **macOS**: Primary platform, full support
- **Linux**: Full support (bash/zsh standard)
- **Windows**: Future work (PowerShell/Git Bash)

### Terminal Requirements

- **Interactive terminal**: Required for fuzzy search
- **Non-interactive**: `forge go` with single match works
- **CI/scripts**: No directives emitted when not TTY (safety)
