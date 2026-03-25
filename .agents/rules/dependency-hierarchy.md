---
globs:
  - "goud_engine/**"
  - "libs/**"
  - "sdks/**"
---

# Dependency Hierarchy

GoudEngine follows a strict 5-layer architecture. Dependencies flow DOWN only.

## Layers

```
Layer 1 (Foundation): core/
Layer 2 (Libs):       libs/
Layer 3 (Services):   ecs/, assets/
Layer 4 (Engine):     sdk/, rendering/, component_ops/, context_registry/
Layer 5 (FFI):        ffi/, wasm/
```

SDKs (`sdks/`) and Apps (`examples/`) sit outside `goud_engine/src/` and connect via FFI only.

Canonical source: `tools/lint_layers.rs`. Run `cargo run -p lint-layers` to validate.

## Rules

- **No upward imports** — a lower layer MUST NOT depend on a higher layer
- **No same-layer cross-imports** — modules within a layer should not import from sibling modules unless explicitly designed as shared utilities
- **FFI is the boundary** — all SDK languages under `sdks/` interact with the engine exclusively through FFI
- **Examples depend on SDKs** — example games use the SDK API, never internal engine types

## Validation

Run `cargo run -p lint-layers` from the workspace root. It scans all `.rs` files under `goud_engine/src/` and reports any `use crate::` import that violates the layer hierarchy.
