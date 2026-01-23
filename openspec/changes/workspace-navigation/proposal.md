# Proposal: Workspace Navigation with Shell Integration

## Problem Statement

Currently, users must manually navigate between forge workspaces using `cd` commands with full paths. This creates friction in the developer workflow:

1. **Cognitive overhead**: Users must remember or look up workspace paths
2. **Typing burden**: Full paths like `~/forge/torvalds-linux/feature-branch` are tedious to type
3. **Context switching friction**: Moving between workspaces requires leaving the terminal to find paths
4. **Shell limitation**: The `forge` binary cannot change the user's current directory

Additionally, the binary cannot directly affect the user's shell state (like current directory), requiring a shell integration approach that many modern CLI tools (like `direnv`, `zoxide`, `starship`) successfully employ.

## Proposed Solution

Implement a two-part navigation system:

### 1. Fuzzy Workspace Navigation (`forge go`)

Add a `forge go` command with fuzzy search capabilities:

- **`forge go`**: Launch interactive fuzzy search over all workspaces
- **`forge go <query>`**: Start fuzzy search with `<query>` pre-filled
- **Direct navigation**: If query matches exactly one workspace, navigate immediately without interactive selection
- **Fuzzy matching**: Use substring/fuzzy matching to quickly filter workspaces

The fuzzy search interface should:
- Display workspace name, repository, and branch
- Support keyboard navigation (arrows, enter, escape)
- Allow backspacing to modify the initial query
- Show the full path of the selected workspace

### 2. Shell Integration (`forge shell`)

Provide shell hooks that wrap the `forge` binary to enable directory changes:

- **`forge shell init <shell>`**: Output shell-specific initialization code
- **Supported shells**: bash and zsh initially (others later)
- **Shell wrapper function**: Replace `forge` command with a function that:
  - Calls the real binary (located via PATH or `$FORGE_BIN`)
  - Parses special directives from the binary's output
  - Executes shell commands based on directives (like `cd`)
- **Protocol**: Simple directive-based protocol between binary and shell
  - Format: `@forge:<directive>:<args>` on stderr
  - Initial directive: `@forge:change_directory:<path>`
  - Extensible for future needs without code execution

### Installation Flow

1. User installs `forge` binary to PATH
2. User adds to `.bashrc` or `.zshrc`: `eval "$(forge shell init bash)"` or `eval "$(forge shell init zsh)"`
3. Shell wrapper intercepts `forge go` commands
4. Binary outputs directory change directives
5. Wrapper executes `cd` in user's shell

## Benefits

1. **Fast navigation**: Jump to any workspace with a few keystrokes
2. **Reduced cognitive load**: No need to remember paths
3. **Context-aware**: Binary knows all workspaces in the forge
4. **Ergonomic**: Natural integration with shell workflow
5. **Extensible**: Directive protocol can support future shell integrations
6. **Testable**: Binary can be tested independently of shell integration

## Trade-offs

### Advantages

- **Standard pattern**: Shell integration via eval is well-understood and widely used
- **No magic**: Users explicitly opt-in via shell config
- **Debuggable**: Users can inspect shell function and binary output
- **Portable**: Works across different shells with minimal changes
- **Safe**: No arbitrary code execution from binary

### Disadvantages

- **Setup required**: Users must add initialization to shell config
- **Shell-specific code**: Need to maintain bash and zsh implementations
- **Complexity**: Two-layer architecture (binary + shell wrapper)

### Alternative Considered: Subshell Approach

Could spawn a subshell with `cd`, but:
- Creates nested shell context (confusing)
- Loses shell history/state
- Inferior UX compared to shell integration

## Scope

This proposal covers:

1. **Binary changes**:
   - New `forge go` command with fuzzy search
   - New `forge shell init` command to output shell code
   - Directive protocol for shell communication
   - Fuzzy matching library integration

2. **Shell integration**:
   - Bash initialization script
   - Zsh initialization script
   - Shell wrapper function generation
   - Directive parser in shell code

3. **User experience**:
   - Interactive fuzzy search UI
   - Pre-filled query support
   - Direct navigation for single matches
   - Clear feedback on navigation

Out of scope (future work):

- Additional shells (fish, powershell, elvish)
- Shell completion for workspace names
- Most recently used (MRU) workspace ordering
- Workspace aliases/bookmarks
- Multi-workspace operations

## Implementation Strategy

### Phase 1: Binary Commands

1. Add fuzzy matching dependency (e.g., `skim` or `nucleo`)
2. Implement `forge go` command with interactive search
3. Implement `forge shell init` command to output shell code
4. Define and document directive protocol

### Phase 2: Shell Integration

1. Create bash wrapper function template
2. Create zsh wrapper function template
3. Implement directive parsing in shell code
4. Add tests for shell integration (if feasible)

### Phase 3: User Documentation

1. Update README with shell integration instructions
2. Add shell setup to initialization flow (suggest after `forge init`)
3. Document directive protocol for future extensibility

## Migration Path

This is a new feature with no migration concerns. Existing users:
- Continue using `forge` without shell integration (fully functional)
- Can opt-in to shell integration by adding eval to shell config
- Can remove shell integration by removing eval line

## Security Considerations

- **No code execution**: Directives are predefined, not arbitrary commands
- **Path validation**: Binary ensures directory exists before emitting directive
- **User control**: Shell integration is opt-in and user-visible
- **Binary trust**: Shell wrapper trusts binary output (same as any CLI tool)

## Open Questions

1. **Fuzzy matching library**: `skim` (full TUI), `nucleo` (helix's matcher), or `fuzzy-matcher` (lightweight)?
   - Recommendation: `skim` for full interactive experience matching `fzf` UX

2. **Directive format**: Is `@forge:<directive>:<args>` on stderr sufficient?
   - Alternative: JSON output on stdout with `--shell-mode` flag
   - Recommendation: Stderr directives for simplicity (fzf-style)

3. **Error handling**: How should shell wrapper handle binary errors?
   - Recommendation: Display error, don't change directory

4. **Shell detection**: Should binary auto-detect shell?
   - Recommendation: Explicit `forge shell init <shell>` for clarity
