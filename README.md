# Forge

Forge is a tool for managing your code repositories and workspaces in a centralized location.

## Installation

### From Source

```bash
git clone <repository-url>
cd forge
cargo build --release
```

The binary will be available at `target/release/forge`. You can copy it to a location in your PATH:

```bash
cp target/release/forge /usr/local/bin/
```

## Usage

### Initialize a Forge

Create a new Forge at the default location (`~/forge`):

```bash
forge init
```

Create a Forge at a custom location:

```bash
forge init /path/to/forge
```

Use environment variable for custom location:

```bash
export FORGE_DIR=/path/to/forge
forge init
```

Priority order for location:
1. Command-line argument
2. `FORGE_DIR` environment variable
3. Default: `~/forge`

### Command-Line Options

```bash
forge init [PATH] [OPTIONS]

Arguments:
  [PATH]  Path where the Forge should be created

Options:
  -f, --force    Force re-initialization of an existing Forge
  -h, --help     Print help
  -V, --version  Print version
```

### Examples

Initialize at default location:
```bash
forge init
```

Initialize at custom location:
```bash
forge init ~/my-projects
```

Re-initialize an existing Forge (preserves content):
```bash
forge init --force
```

## Directory Structure

When you initialize a Forge, the following structure is created:

```
~/forge/                  # Forge root (for workspaces)
├── .forge/              # Internal directory (hidden)
│   ├── marker           # Forge marker file
│   └── repos/           # Repository storage
└── [workspaces...]      # Your workspace directories (non-hidden)
```

Workspaces are created directly in the Forge root (`~/forge/`) as regular (non-hidden) directories, making them easy to access. All internal Forge data, including repository clones, is stored in the hidden `.forge/` directory.

## Navigation

Forge provides a fast navigation system to jump between workspaces using fuzzy search.

### Using `forge go`

Navigate to any workspace with fuzzy search:

```bash
forge go
```

Pre-fill the search with a query:

```bash
forge go feature
```

Direct navigation (if query matches exactly one workspace):

```bash
forge go myrepo/main
```

**Note:** The `forge go` command requires shell integration to be set up (see below) in order to change your shell's current directory.

## Shell Integration

To enable directory navigation with `forge go`, you need to set up shell integration. This wraps the `forge` command with a shell function that can process directory change directives.

### Setup for Bash

Add this line to your `~/.bashrc`:

```bash
eval "$(forge shell init bash)"
```

Then reload your shell:

```bash
source ~/.bashrc
```

### Setup for Zsh

Add this line to your `~/.zshrc`:

```bash
eval "$(forge shell init zsh)"
```

Then reload your shell:

```bash
source ~/.zshrc
```

### How It Works

The shell integration works by:

1. Wrapping the `forge` binary with a shell function
2. Capturing output from the binary that includes special directives
3. Executing shell commands (like `cd`) based on those directives
4. Displaying normal output to the user

This is similar to how tools like `direnv`, `zoxide`, and `starship` integrate with your shell.

### Environment Variables

- **`FORGE_BIN`**: Override the path to the forge binary (useful for testing or custom installations)
  ```bash
  export FORGE_BIN=/path/to/custom/forge
  ```

- **`FORGE_DIR`**: Set a custom location for your Forge (default: `~/forge`)
  ```bash
  export FORGE_DIR=/path/to/forge
  ```

## Development

### Running Tests

```bash
cargo test
```

### Code Quality

```bash
cargo fmt
cargo clippy
```

## License

[Add your license here]
