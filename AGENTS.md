<!-- OPENSPEC:START -->
# OpenSpec Instructions

These instructions are for AI assistants working in this project.

Always open `@/openspec/AGENTS.md` when the request:
- Mentions planning or proposals (words like proposal, spec, change, plan)
- Introduces new capabilities, breaking changes, architecture shifts, or big performance/security work
- Sounds ambiguous and you need the authoritative spec before coding

Use `@/openspec/AGENTS.md` to learn:
- How to create and apply change proposals
- Spec format and conventions
- Project structure and guidelines

Keep this managed block so 'openspec update' can refresh the instructions.

<!-- OPENSPEC:END -->

## Agent Expectations

- You are responsible for committing code, commit frequently and use Conventional Commit Messages.
- When starting new work, make sure you're in a feature branch with the name `feat/[feature-name]`.

## Terminal UI

We are building a CLI tool for developers, who live on the command prompt. It should be a joy to use. Use `ratatui` to create rich terminal UIs with colors, layouts, and interactivity. However, keep the UI simple and fast, avoiding unnecessary complexity that could slow down the user experience. Use interactive selectors for choosing options, and use color effectively to highlight important information without overwhelming the user. And always remember that commands may be piped, run in scripts, or logged, so ensure that interactive elements degrade gracefully in non-interactive contexts.
