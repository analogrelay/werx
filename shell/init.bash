# Forge shell integration for Bash
#
# This function wraps the forge binary to enable shell-level features
# like changing directories in response to directives from the binary.
#
# To install, add this to your ~/.bashrc:
#   eval "$(forge shell init bash)"

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
