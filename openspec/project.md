# Project Context

## Purpose

Werx is a CLI tool for managing a single root directory on a user's machine that holds all their source code repositories and workspaces. The tool provides a unified interface for organizing and working with code across your entire machine.

**Core Capabilities**:

- **Repository Management**: Add existing Git repositories to the Werx, making them discoverable and manageable from anywhere on the system
- **New Repository Creation**: Create brand new Git repositories within the Werx structure
- **Workspace Management**: Create and manage workspaces, which can be:
  - Git worktrees linked to repositories in the Werx (for parallel development on different branches)
  - Scratch directories for quick tests and experiments
  - Prototype projects that may later become full Git repositories
- **Centralized Code Organization**: Your local Werx becomes the single source of truth for where all your code lives

The goal is to eliminate the scattered nature of code directories across a developer's machine and provide a consistent, discoverable structure for all development work.

## Tech Stack

- **Language**: Rust (latest stable)
- **Build System**: Cargo
- **Development Environment**: Nix/devenv for reproducible environments
- **Container Support**: Dev Containers with devenv integration

## Project Conventions

### Code Style

- Follow standard Rust conventions (`rustfmt` and `clippy`)
- Use `cargo fmt` for automatic formatting
- Address all `clippy` warnings before committing
- Prefer explicit error handling with `Result<T, E>` over panics
- Use descriptive variable and function names that reflect domain concepts
- Keep functions focused and modular
- Use `anyhow` for error handling in application code, `thiserror` for library code

### Architecture Patterns

- CLI architecture: Use structured command parsing (e.g., `clap` or similar)
- Prefer composition over inheritance
- Use traits for abstraction and testability
- Keep business logic separate from I/O and presentation
- Favor simple, direct implementations over complex abstractions
- Follow the "boring technology" principle - use proven patterns

### Testing Strategy

- **Unit Tests**: Use `#[test]` and `#[cfg(test)]` modules for unit testing
  - Test public APIs and critical logic paths
  - Mock external dependencies where appropriate
- **Integration Tests**: Place end-to-end tests in `tests/` directory
  - Test actual CLI behavior with representative inputs
  - Verify exit codes and output formats
- **Benchmarks**: Use `criterion` for performance-critical code
  - Benchmark before optimizing
  - Track performance regressions
- Run `cargo test` before every commit
- Aim for high coverage of critical paths, but prioritize meaningful tests over coverage metrics

### Git Workflow

- **Trunk-based development**: Work primarily on `main` branch
- Use short-lived feature branches for larger changes
- Keep commits atomic and focused
- Commit message format:
  - First line: Brief imperative summary (50 chars or less)
  - Optional body: Detailed explanation of why and what changed
- Run tests before pushing
- For significant architectural changes, use OpenSpec proposals before implementation

## Domain Context

Werx operates in the DevOps/tooling space, focusing on:

- Developer productivity and automation
- Command-line interfaces and scripting integration
- Cross-platform compatibility (primarily Unix-like systems, with macOS as primary development platform)
- Integration with common development workflows (git, CI/CD, etc.)

## Important Constraints

- Must work on macOS (Darwin) as primary platform
- Should follow Unix philosophy: do one thing well
- Performance matters - CLI tools should feel fast and responsive
- Binary size should be reasonable for distribution
- Must handle errors gracefully with helpful user messages

## External Dependencies

- Development environment managed via Nix/devenv
- No external runtime dependencies beyond what's vendored in the binary
- Consider dependency weight carefully - prefer lighter alternatives when functionality is similar
