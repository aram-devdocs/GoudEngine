# docs/ — Documentation

## Purpose

Reference documentation, generated diagrams, and development guides.

## Files

- `husky-rs.docs.md` — Reference for the husky-rs git hooks system
- `cbindgen.docs.md` — Reference for C header generation tooling
- `diagrams/` — Auto-generated module dependency graphs

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

## Anti-Patterns

- NEVER edit generated files (`module_graph.png`, `.pdf`) — regenerate instead
- NEVER write marketing copy or AI-sounding prose in docs
