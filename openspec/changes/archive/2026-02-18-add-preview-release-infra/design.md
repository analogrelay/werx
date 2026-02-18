## Context

Werx is preparing for its initial preview release. This requires establishing CI/CD pipelines and distribution channels so users can easily install and trust the tool. The project uses Rust and targets macOS as the primary platform, with Linux as secondary.

## Goals / Non-Goals

**Goals:**
- Automate testing and linting to catch issues before merge
- Automate release process to reduce manual toil and errors
- Provide multiple installation methods for different user preferences
- Support both x86_64 and aarch64 architectures for macOS and Linux

**Non-Goals:**
- Windows support (not in scope for preview release)
- Automated changelog generation (manual for now)
- Signed binaries or notarization (future consideration)
- Crates.io publishing (deferred to future release)

## Decisions

### CI Platform: GitHub Actions
GitHub Actions is chosen because the repository is hosted on GitHub, it's free for public repositories, and has excellent Rust toolchain support.

**Alternatives considered:**
- CircleCI: More configuration overhead, no significant advantage for this use case
- Self-hosted runners: Unnecessary complexity for a preview release

### Release Trigger: Manual Workflow Dispatch
Releases are triggered via manual workflow dispatch, where the user specifies the version number. The workflow creates the git tag, ensuring consistency between the tag and the release.

**Alternatives considered:**
- Release on every push to main: Too aggressive, creates noise
- Tag-triggered releases: Requires manual tagging step, prone to human error in tag naming

### Binary Distribution: Pre-built binaries on GitHub Releases
Pre-built binaries are attached to GitHub Releases for direct download. This is the simplest approach and works for most users.

### Nix Packaging: Flake with rustPlatform
Using `rustPlatform.buildRustPackage` is the standard approach for Rust projects in Nix. A flake provides reproducibility and easy consumption.

## Risks / Trade-offs

- **Cross-compilation complexity** → Mitigated by using GitHub-hosted runners for native builds on each platform

## Open Questions

- What minimum supported Rust version (MSRV) should be documented?
