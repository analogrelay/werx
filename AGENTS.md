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

## Agent Workflow

When writing code to implement a spec, follow these steps:

1. Check out a feature branch from `main`, `feat/<short-name>`.
2. Read the relevant specs in `openspec/changes/` to understand requirements.
3. Read the `tasks.md` file for the spec to see implementation tasks.
4. Write code to implement the tasks, committing frequently with clear messages, following the Conventional Commits pattern.
5. Write tests to cover new functionality and edge cases.
6. Run all tests to ensure nothing is broken.
7. Your work is finished when the spec is fully implemented and all tests pass.
8. Report to the user that the implementation is complete and ready for review.
