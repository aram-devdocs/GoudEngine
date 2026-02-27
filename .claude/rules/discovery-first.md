---
alwaysApply: true
---

# Discovery-First Protocol

Before modifying any file, you MUST read the relevant source files to understand the current state. Never trust assumptions about code structure, naming, or behavior.

## Required Steps

1. **Read before writing** — Open and read every file you intend to modify, plus its direct dependencies
2. **Check existing tests** — Before writing new tests, read the existing test module (`#[cfg(test)]`) or `tests/` directory for the area you're changing
3. **Verify compilation** — Run `cargo check` before and after making changes to confirm you haven't broken the build
4. **Inspect neighbors** — When adding a new function or type, read the surrounding module to match conventions (naming, error handling, visibility)

## Anti-Patterns

- Guessing at function signatures or type definitions without reading them
- Assuming a module's public API based on its name alone
- Writing tests that duplicate existing coverage
- Editing a file based on outdated context from a previous session
