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

  # Create a temp file for directives and register cleanup
  local directive_file
  directive_file=$(mktemp)

  # Run the binary directly; stdout/stderr flow to the terminal unchanged
  WERX_DIRECTIVE_FILE="$directive_file" command "$werx_bin" "$@"
  local exit_code=$?

  # Process directives written by the binary
  while IFS= read -r directive; do
    if [[ "$directive" =~ ^@werx:change_directory:(.+)$ ]]; then
      local target_dir="${BASH_REMATCH[1]}"
      if [ -d "$target_dir" ]; then
        cd "$target_dir" || true
      else
        echo "werx: directory does not exist: $target_dir" >&2
      fi
    fi
  done < "$directive_file"

  rm -f "$directive_file"
  return $exit_code
}
