# Change: Add Preview Release Infrastructure

## Why

Werx needs CI/CD and distribution infrastructure to prepare for its initial preview release. Without automated testing, release pipelines, and distribution channels, users cannot easily install or trust the stability of the tool.

## What Changes

- Add GitHub Actions CI workflow for automated testing and linting on `main` and PRs
- Add GitHub Actions release pipeline triggered via manual workflow dispatch, which tags the release, creates a GitHub Release, and builds packages
- Add Nix flake packaging for Nix/NixDarwin/NixOS users

## Impact

- Affected specs: None existing (creates new `ci-cd` and `distribution` capabilities)
- Affected code: 
  - `.github/workflows/` - new CI and release workflows
  - `flake.nix` - new Nix flake definition
