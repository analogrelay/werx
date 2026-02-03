# Werx shell integration for Bash
#
# This function wraps the werx binary to enable shell-level features
# like changing directories in response to directives from the binary.
#
# To install, add this to your ~/.bashrc:
#   eval "$(werx shell init bash)"

werx() {
  # Use WERX_BIN if set, otherwise 'werx' from PATH
  local werx_bin="${WERX_BIN:-werx}"

  # Capture combined output (stdout + stderr)
  local output
  output=$("$werx_bin" "$@" 2>&1)
  local exit_code=$?

  # Extract directives (lines starting with @werx:)
  local directives
  directives=$(echo "$output" | grep "^@werx:")

  # Print non-directive output
  echo "$output" | grep -v "^@werx:"

  # Process directives
  while IFS= read -r directive; do
    if [[ "$directive" =~ ^@werx:change_directory:(.+)$ ]]; then
      local target_dir="${BASH_REMATCH[1]}"
      if [ -d "$target_dir" ]; then
        cd "$target_dir" || true
      else
        echo "werx: directory does not exist: $target_dir" >&2
      fi
    fi
  done <<< "$directives"

  return $exit_code
}
