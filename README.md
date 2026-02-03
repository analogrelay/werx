# Werx

Werx is a tool for managing your code repositories and workspaces in a centralized location.

## Installation

### From Source

```bash
git clone <repository-url>
cd werx
cargo build --release
```

The binary will be available at `target/release/werx`. You can copy it to a location in your PATH:

```bash
cp target/release/werx /usr/local/bin/
```

## Usage

### Initialize a Werx

Create a new Werx at the default location (`~/werx`):

```bash
werx init
```

Create a Werx at a custom location:

```bash
werx init /path/to/werx
```

Use environment variable for custom location:

```bash
export WERX_DIR=/path/to/werx
werx init
```

Priority order for location:

1. Command-line argument
2. `WERX_DIR` environment variable
3. Default: `~/werx`

### Command-Line Options

```bash
werx init [PATH] [OPTIONS]

Arguments:
  [PATH]  Path where the Werx should be created

Options:
  -f, --force    Force re-initialization of an existing Werx
  -h, --help     Print help
  -V, --version  Print version
```

### Examples

Initialize at default location:

```bash
werx init
```

Initialize at custom location:

```bash
werx init ~/my-projects
```

Re-initialize an existing Werx (preserves content):

```bash
werx init --force
```

## Directory Structure

When you initialize a Werx, the following structure is created:

```
~/werx/                   # Werx root (for workspaces)
├── .werx/               # Internal directory (hidden)
│   └── repos/           # Repository storage
└── [workspaces...]      # Your workspace directories (non-hidden)
```

Workspaces are created directly in the Werx root (`~/werx/`) as regular (non-hidden) directories, making them easy to access. All internal Werx data, including repository clones, is stored in the hidden `.werx/` directory.

## Navigation

Werx provides a fast navigation system to jump between workspaces using fuzzy search.

### Using `werx go`

Navigate to any workspace with fuzzy search:

```bash
werx go
```

Pre-fill the search with a query:

```bash
werx go feature
```

Direct navigation (if query matches exactly one workspace):

```bash
werx go myrepo/main
```

**Note:** The `werx go` command requires shell integration to be set up (see below) in order to change your shell's current directory.

## Shell Integration

To enable directory navigation with `werx go`, you need to set up shell integration. This wraps the `werx` command with a shell function that can process directory change directives.

### Setup for Bash

Add this line to your `~/.bashrc`:

```bash
eval "$(werx shell init bash)"
```

Then reload your shell:

```bash
source ~/.bashrc
```

### Setup for Zsh

Add this line to your `~/.zshrc`:

```bash
eval "$(werx shell init zsh)"
```

Then reload your shell:

```bash
source ~/.zshrc
```

### How It Works

The shell integration works by:

1. Wrapping the `werx` binary with a shell function
2. Capturing output from the binary that includes special directives
3. Executing shell commands (like `cd`) based on those directives
4. Displaying normal output to the user

This is similar to how tools like `direnv`, `zoxide`, and `starship` integrate with your shell.

### Environment Variables

- **`WERX_BIN`**: Override the path to the werx binary (useful for testing or custom installations)

  ```bash
  export WERX_BIN=/path/to/custom/werx
  ```

- **`WERX_DIR`**: Set a custom location for your Werx (default: `~/werx`)

  ```bash
  export WERX_DIR=/path/to/werx
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
