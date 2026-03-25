# tools/ — Developer Tooling

Internal development tools for maintaining GoudEngine code quality.

## lint-layers

Validates the five-layer dependency hierarchy. Scans `goud_engine/src/` for `use crate::` violations.

```bash
cargo run -p lint-layers    # From workspace root
```

See `.agents/rules/dependency-hierarchy.md` for layer definitions.

Exit codes: 0 (ok), 1 (violations found), 2 (directory not found).
