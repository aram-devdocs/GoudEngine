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
Layer 1 (Core):    libs/          — graphics, platform, ecs, logger
Layer 2 (Engine):  goud_engine/src/ — core, assets, sdk (uses Layer 1)
Layer 3 (FFI):     goud_engine/src/ffi/  (uses Layer 2)
Layer 4 (SDKs):    sdks/          — C#, Python (uses Layer 3 via FFI)
Layer 5 (Apps):    examples/      (uses Layer 4)
```

## Rules

- **No upward imports** — a lower layer MUST NOT depend on a higher layer (e.g., `libs/` cannot import from `goud_engine/src/`)
- **No same-layer cross-imports** — modules within a layer should not import from sibling modules at the same layer unless explicitly designed as shared utilities
- **FFI is the boundary** — SDKs interact with the engine exclusively through FFI; no direct Rust API calls from C#/Python
- **Examples depend on SDKs** — example games use the SDK API, never internal engine types

## Validation

Check `use` statements for violations. A `use goud_engine::` in `libs/` or a `use crate::ffi::` in `libs/graphics/` indicates a hierarchy violation.
