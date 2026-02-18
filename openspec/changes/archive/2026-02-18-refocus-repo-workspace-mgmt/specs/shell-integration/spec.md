# Spec Delta: shell-integration

_No spec-level changes required._

The shell integration spec defines a generic directive protocol (`@forge:change_directory`) and shell wrapper functions. These have no agent-specific behavior. The directive protocol remains unchanged.

Agent commands were invoked through the same CLI binary and shell wrapper, but the wrapper itself has no agent-specific logic — it simply forwards all commands and processes directives. Removing agent commands from the binary requires no changes to the shell integration spec.
