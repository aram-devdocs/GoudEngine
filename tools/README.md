# GoudEngine Developer Tools

Internal development tools for maintaining code quality. Not part of the
engine runtime or any SDK.

## lint-layers

Enforces the dependency hierarchy by scanning `use crate::` imports across
all Rust source files in `goud_engine/src/`.

### What It Checks

Dependencies must flow DOWN the layer hierarchy. The tool fails if any
upward import is detected.

| Layer | Directories | May Import From |
|-------|-------------|-----------------|
| 1 (Core) | libs/, core/, ecs/, assets/ | Nothing in Layer 2 or 3 |
| 2 (Engine) | sdk/ | Layer 1 only |
| 3 (FFI) | ffi/, wasm/ | Layer 1 and 2 |

### Usage

Run from the workspace root:

```bash
cargo run -p lint-layers
```

### Exit Codes

- **0** -- No violations found
- **1** -- Violations detected (prints file path, line number, and offending import)
- **2** -- `goud_engine/src/` directory not found

### Integration

lint-layers runs automatically in two places:

- `codegen.sh` step 2
- Pre-commit hook (via `.husky/`)
