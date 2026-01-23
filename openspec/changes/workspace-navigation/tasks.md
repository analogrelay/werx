# Implementation Tasks: Workspace Navigation with Shell Integration

## Task 1: Add fuzzy matching dependency

**Description**: Add `skim` crate for fuzzy search functionality.

**Files to modify**:
- `Cargo.toml`

**Changes**:
1. Add `skim = "0.10"` to `[dependencies]`
2. Run `cargo build` to verify dependency resolves
3. Check for version conflicts or build issues

**Tests**:
- Verify `cargo build` succeeds
- Verify `cargo test` still passes

**Validation**: Run `cargo tree | grep skim` to confirm dependency.

---

## Task 2: Implement directive emission utilities

**Description**: Create utility functions for emitting directives to stderr.

**Files to modify**:
- `src/directive.rs` (new file)
- `src/lib.rs`

**Changes**:
1. Create `src/directive.rs` module
2. Implement `emit_change_directory(path: &Path)` function
3. Implement `emit_directive(name: &str, arg: &str)` with validation
4. Validate directive name format (lowercase, underscores only)
5. Validate argument doesn't contain newlines
6. Export from `lib.rs`

**Tests**:
- `test_emit_directive_format()` - verify output format
- `test_emit_directive_validation()` - verify name validation
- `test_emit_change_directory()` - verify path handling

**Validation**: Run `cargo test directive` and verify all tests pass.

---

## Task 3: Implement fuzzy workspace selection

**Description**: Create interactive fuzzy search for workspaces using skim.

**Files to modify**:
- `src/workspace.rs`

**Changes**:
1. Create `WorkspaceItem` struct implementing `SkimItem` trait
2. Implement `fuzzy_select_workspace(workspaces: Vec<Workspace>, query: Option<String>) -> Result<Option<Workspace>>`
3. Configure skim with:
   - Height: 50% of terminal
   - Single selection mode
   - Initial query pre-filled
   - Display format: `<repo>/<workspace>` with branch/path context
4. Handle user cancellation (return None)
5. Return selected workspace

**Tests**:
- Manual testing required (interactive TUI)
- Unit test for `WorkspaceItem` display format

**Validation**: Create test forge with workspaces, call function, verify UI appears.

---

## Task 4: Implement direct navigation logic

**Description**: Add logic to skip fuzzy search when query has single match.

**Files to modify**:
- `src/workspace.rs`

**Changes**:
1. Implement `find_workspace_matches(workspaces: &[Workspace], query: &str) -> Vec<&Workspace>`
2. Use fuzzy matching on `format!("{}/{}", ws.repository, ws.name)`
3. Add `select_workspace_with_query()` function that:
   - Pre-filters workspaces if query provided
   - Returns immediately if exactly one match
   - Launches fuzzy search otherwise
4. Handle case-insensitive matching

**Tests**:
- `test_find_workspace_matches_single()` - one match
- `test_find_workspace_matches_multiple()` - multiple matches
- `test_find_workspace_matches_none()` - no matches
- `test_find_workspace_matches_case_insensitive()`

**Validation**: Run `cargo test workspace` and verify matching logic.

---

## Task 5: Add `forge go` command

**Description**: Implement CLI command for workspace navigation.

**Files to modify**:
- `src/main.rs`

**Changes**:
1. Add `Go` command to `Commands` enum with optional `query` parameter
2. Add `Go` command to `WorkspaceCommands` enum
3. Implement `cmd_go(query: Option<String>) -> Result<()>`:
   - Load forge
   - List all workspaces
   - Call `select_workspace_with_query()`
   - Emit change directory directive if workspace selected
   - Handle errors (no forge, no workspaces, no selection)
4. Wire up command in main match statement
5. Support both `forge go` and `forge workspace go` syntax

**Tests**:
- Integration test: Create forge, run `forge go`, verify directive output

**Validation**: Build and manually test `forge go` with test forge.

---

## Task 6: Handle non-TTY contexts

**Description**: Detect non-interactive environments and handle gracefully.

**Files to modify**:
- `src/workspace.rs`

**Changes**:
1. Check `std::io::stdin().is_terminal()` before launching fuzzy search
2. If not TTY and query has single match, allow direct navigation
3. If not TTY and query has multiple/no matches, return error
4. Error message: "Interactive terminal required for fuzzy search"

**Tests**:
- Test with `echo | forge go query` (pipe stdin)
- Verify single match works
- Verify multiple matches fail

**Validation**: Run tests with piped stdin, verify behavior.

---

## Task 7: Create bash shell initialization script

**Description**: Write bash wrapper function template.

**Files to modify**:
- `shell/init.bash` (new file)

**Changes**:
1. Create `shell/` directory
2. Write bash function that:
   - Uses `${FORGE_BIN:-forge}` to locate binary
   - Captures combined stdout/stderr
   - Extracts directives with `grep "^@forge:"`
   - Prints non-directive output
   - Parses `change_directory` directive with `BASH_REMATCH`
   - Changes directory if valid
   - Returns original exit code
3. Test with bash 3.2 (macOS default) and bash 5.x

**Tests**:
- Create test script that sources function and calls with mock binary
- Verify directory change occurs
- Verify exit code preservation

**Validation**: Manual test by sourcing script and calling function.

---

## Task 8: Create zsh shell initialization script

**Description**: Write zsh wrapper function template.

**Files to modify**:
- `shell/init.zsh` (new file)

**Changes**:
1. Write zsh function that:
   - Uses `${FORGE_BIN:-forge}` to locate binary
   - Captures combined stdout/stderr
   - Extracts directives with `grep "^@forge:"`
   - Prints non-directive output
   - Parses `change_directory` directive with `match` array
   - Changes directory if valid
   - Returns original exit code
2. Test with zsh 5.x

**Tests**:
- Create test script that sources function and calls with mock binary
- Verify directory change occurs
- Verify exit code preservation

**Validation**: Manual test by sourcing script and calling function.

---

## Task 9: Implement `forge shell init` command

**Description**: Add command to output shell integration code.

**Files to modify**:
- `src/main.rs`
- `src/shell.rs` (new file)
- `src/lib.rs`

**Changes**:
1. Create `src/shell.rs` module
2. Implement `cmd_shell_init(shell: &str) -> Result<()>`:
   - Match on shell name: `bash` or `zsh`
   - Output appropriate script with `include_str!`
   - Return error for unsupported shells
3. Add `Shell` command to `Commands` enum
4. Add `ShellCommands::Init { shell }` subcommand
5. Wire up in main match statement
6. Embed shell scripts at compile time

**Tests**:
- `test_shell_init_bash()` - verify bash output
- `test_shell_init_zsh()` - verify zsh output
- `test_shell_init_unsupported()` - verify error

**Validation**: Run `cargo test shell` and verify outputs.

---

## Task 10: Update `forge init` success message

**Description**: Suggest shell integration after forge initialization.

**Files to modify**:
- `src/init.rs`

**Changes**:
1. Add shell integration suggestion to success message
2. Detect user's shell from `$SHELL` environment variable
3. Show appropriate command: `eval "$(forge shell init bash)"`
4. Suggest adding to appropriate config file (`.bashrc`, `.zshrc`)
5. Keep message concise (optional next step)

**Tests**:
- Update tests to expect new output format

**Validation**: Run `forge init` and verify message displayed.

---

## Task 11: Add integration tests

**Description**: Create end-to-end tests for navigation and shell integration.

**Files to modify**:
- `tests/navigation_test.rs` (new file)

**Tests**:
1. **Test: Fuzzy search with pre-filled query**
   - Create forge with multiple workspaces
   - Run `forge go <query>` (capturing stderr)
   - Verify directive emitted with correct path

2. **Test: Direct navigation with single match**
   - Create forge with workspaces
   - Run `forge go` with query matching single workspace
   - Verify no interactive UI launched
   - Verify directive emitted immediately

3. **Test: Shell init outputs valid code**
   - Run `forge shell init bash`
   - Verify output contains function definition
   - Verify output is valid bash syntax (use `bash -n`)

4. **Test: Shell wrapper processes directive**
   - Source bash wrapper
   - Create mock binary that outputs directive
   - Call wrapper function
   - Verify directory changed

**Validation**: Run `cargo test --test navigation_test` and verify all pass.

---

## Task 12: Update documentation

**Description**: Document new commands and shell integration.

**Files to modify**:
- `README.md`

**Changes**:
1. Add "Navigation" section explaining `forge go`
2. Add "Shell Integration" section with setup instructions
3. Include example usage and screenshots (if applicable)
4. Document `FORGE_BIN` environment variable
5. Document supported shells and versions
6. Add troubleshooting section for common issues

**Validation**: Review README for clarity and completeness.

---

## Task 13: Update command help text

**Description**: Ensure all new commands have clear help text.

**Files to modify**:
- `src/main.rs`

**Changes**:
1. Add detailed help text to `Go` command
2. Add detailed help text to `Shell` command
3. Add examples to help output
4. Verify `forge go --help` shows usage
5. Verify `forge shell --help` shows shells and examples

**Validation**: Run `forge go --help` and `forge shell --help`, verify output.

---

## Dependencies

```
Task 1 (dependency)
  ↓
Task 2 (directives) ──→ Task 5 (go command)
  ↓                           ↓
Task 3 (fuzzy search) ──→ Task 4 (direct nav) ──→ Task 6 (TTY handling)
  ↓
Task 7 (bash script) ──→ Task 9 (shell init)
  ↓                          ↓
Task 8 (zsh script) ────────┘
  ↓
Task 10 (init message)
  ↓
Task 11 (integration tests)
  ↓
Task 12 (docs) ──→ Task 13 (help text)
```

## Parallelizable Work

- Tasks 2, 3 can be done concurrently (independent modules)
- Tasks 7, 8 can be done concurrently (separate shell scripts)
- Tasks 12, 13 can be done concurrently (documentation)

## Estimated Complexity

- **Task 1**: Low (add dependency)
- **Task 2**: Low (simple utility functions)
- **Task 3**: Medium (skim integration, TUI setup)
- **Task 4**: Medium (fuzzy matching logic)
- **Task 5**: Medium (CLI plumbing, error handling)
- **Task 6**: Low (TTY detection)
- **Task 7**: Medium (bash scripting, regex parsing)
- **Task 8**: Medium (zsh scripting, similar to Task 7)
- **Task 9**: Low (simple command, include_str)
- **Task 10**: Low (message update)
- **Task 11**: Medium (integration tests, mocking)
- **Task 12**: Low (documentation)
- **Task 13**: Low (help text)

## Acceptance Criteria

All tasks complete when:

1. `cargo build` succeeds with no warnings
2. `cargo test` passes all unit and integration tests
3. `cargo clippy` reports no issues
4. Manual testing confirms:
   - `forge go` launches fuzzy search
   - `forge go <query>` pre-fills query
   - `forge go <single-match>` navigates directly
   - `forge shell init bash` outputs valid bash code
   - `forge shell init zsh` outputs valid zsh code
   - Sourcing shell init and running `forge go` changes directory
   - ESC in fuzzy search doesn't break shell
   - Directive appears on stderr, not stdout
   - `FORGE_BIN` variable works for custom binary path
5. Documentation is complete and clear
6. Help text for all commands is accurate and helpful
