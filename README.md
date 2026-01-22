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
