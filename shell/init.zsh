# Werx shell integration for Zsh
#
# This function wraps the werx binary to enable shell-level features
# like changing directories in response to directives from the binary.
#
# To install, add this to your ~/.zshrc:
#   eval "$(werx shell init zsh)"

werx() {
  # Use WERX_BIN if set, otherwise 'werx' from PATH
  local werx_bin="${WERX_BIN:-werx}"

  # Capture combined output (stdout + stderr)
  local output
  output=$(command "$werx_bin" "$@" 2>&1)
  local exit_code=$?

  # Extract and process directives
  local directives
  directives=$(echo "$output" | grep "^@werx:")

  # Print non-directive output
  echo "$output" | grep -v "^@werx:"

  # Process directives
  while IFS= read -r directive; do
    if [[ "$directive" =~ ^@werx:change_directory:(.+)$ ]]; then
      local target_dir="${match[1]}"
      if [ -d "$target_dir" ]; then
        cd "$target_dir" 2>/dev/null || true
      else
        echo "werx: directory does not exist: $target_dir" >&2
      fi
    fi
  done <<< "$directives"

  return $exit_code
}
