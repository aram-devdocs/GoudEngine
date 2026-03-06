# docs/ — Documentation

## Purpose

Reference documentation, generated diagrams, and development guides. Published as a static site via mdBook.

## Structure

The `src/` directory contains the mdBook source files. `book.toml` at the repo root configures the build.

- `src/SUMMARY.md` — Table of contents (defines book structure)
- `src/introduction.md` — Landing page
- `src/getting-started/` — SDK-specific quickstart guides (C#, Python, Rust, TypeScript)
- `src/development/` — Building, development workflow, AI agent setup
- `src/architecture/` — Layer architecture, adding new language targets
- `src/reference/` — Tooling reference (husky-rs, cbindgen)
- `diagrams/` — Auto-generated module dependency graphs

## Building the Book

```bash
mdbook build        # Output to docs/book/
mdbook serve        # Local preview at http://localhost:3000
```

## Diagrams

- `diagrams/mods.dot` — GraphViz source for module dependency graph
- `diagrams/module_graph.png` — Rendered PNG dependency graph
- `diagrams/module_graph.pdf` — Rendered PDF dependency graph

Regenerate diagrams:

```bash
./graph.sh
```

## Patterns

- Documentation is for developers working on GoudEngine itself
- Diagrams are auto-generated — edit the generation script, not the output
- Keep docs focused on architecture and tooling, not API reference (that lives in doc comments)
- Add new pages to `src/SUMMARY.md` for them to appear in the book

## Anti-Patterns

- NEVER edit generated files (`module_graph.png`, `.pdf`) — regenerate instead
- NEVER write marketing copy or AI-sounding prose in docs
- NEVER edit files under `docs/book/` — that directory is build output (gitignored)
