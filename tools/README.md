# GoudEngine Developer Tools

Internal development tools for maintaining code quality. Not part of the
engine runtime or any SDK.

## lint-layers

Enforces the dependency hierarchy by scanning `use crate::` imports across
all Rust source files in `goud_engine/src/`.

### What It Checks

Dependencies must flow DOWN the layer hierarchy. The tool fails if any
upward import is detected. Layers are ordered from lowest to highest; a
layer MUST only import from layers below it.

| Layer | Directories | May Import From |
|-------|-------------|-----------------|
| 1 (Foundation) | core/ | Nothing |
| 2 (Libs) | libs/ | Foundation |
| 3 (Services) | ecs/, assets/ | Foundation, Libs |
| 4 (Engine) | sdk/, rendering/, component_ops/, context_registry/ | Foundation, Libs, Services |
| 5 (FFI) | ffi/, wasm/ | All lower layers |

The canonical definition lives in `tools/lint_layers.rs`; see
`.agents/rules/dependency-hierarchy.md` for the rationale.

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

## goudengine-mcp

`goudengine-mcp` is a stdio-first bridge for the shared debugger runtime. It discovers local debugger manifests, attaches to one route over local IPC, exposes MCP tools for snapshot, control, capture, replay, and metrics, and serves stored artifacts through `goudengine://...` resources.

Run it from the workspace root:

```bash
cargo run -p goudengine-mcp
```

The bridge stays out of the game process. Use the debugger runtime guide for scope, platform limits, and artifact details.
