# tools/ -- Developer Tooling

## Purpose

Internal development tools for maintaining GoudEngine code quality. Not part of the
engine runtime or any SDK.

## Key Files

- `Cargo.toml` -- binary crate `lint-layers`
- `lint_layers.rs` -- single-file tool that enforces the dependency hierarchy

## lint-layers

Scans all `.rs` files under `goud_engine/src/` and checks `use crate::` imports against
the five-layer architecture:

| Layer | Directories | May Import From |
|-------|-------------|-----------------|
| 1 (Foundation) | `core/` | nothing internal |
| 2 (Libs) | `libs/` | Foundation only |
| 3 (Services) | `ecs/`, `assets/` | Foundation, Libs |
| 4 (Engine) | `sdk/`, `rendering/`, `component_ops/`, `context_registry/` | Foundation, Libs, Services |
| 5 (FFI) | `ffi/`, `wasm/` | All layers above |

Dependencies flow DOWN only. An import from a lower layer to a higher layer is a violation.
Critically, `libs/` MUST NOT import from `ecs/` or `assets/` — a violation the old
three-layer model could not detect.

### Running

```bash
cargo run -p lint-layers
```

MUST be run from the workspace root -- the tool looks for `goud_engine/src/` relative to cwd.

### Output

- Exit 0: no violations found
- Exit 1: prints each violation with file path, line number, and the offending import
- Exit 2: `goud_engine/src/` directory not found

### How It Works

1. Recursively collects all `.rs` files under `goud_engine/src/`
2. Classifies each file into a layer based on its path
3. Scans each `use crate::` line and classifies the import target
4. Reports any import where the target layer is above the source layer

Skips comment lines and files not in a recognized layer (e.g., `lib.rs` at the root).
