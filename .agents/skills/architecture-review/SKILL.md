---
name: architecture-review
description: Dependency flow audit and layer validation for GoudEngine's 5-layer hierarchy
user-invocable: true
---

# Architecture Review

Audit the codebase for dependency hierarchy violations, circular imports, and module boundary breaches.

## When to Use

Run after structural changes, new module additions, or when adding cross-module imports. Also run periodically as part of hardening.

## 5-Layer Hierarchy

Dependencies MUST flow DOWN only. No upward imports. No same-layer cross-imports (unless explicitly documented). The canonical model lives in `tools/lint_layers.rs`.

```
Layer 1 (Foundation): goud_engine/src/core/
Layer 2 (Libs)      : goud_engine/src/libs/          — graphics, platform, logger
Layer 3 (Services)  : goud_engine/src/ecs/, assets/
Layer 4 (Engine)    : goud_engine/src/sdk/, rendering/, component_ops/, context_registry/
Layer 5 (FFI)       : goud_engine/src/ffi/, wasm/
```

SDKs (`sdks/`) and apps (`examples/`) sit outside `goud_engine/src/` and connect through the FFI boundary only. The SDK matrix is 10 languages: `c`, `cpp`, `csharp`, `go`, `kotlin`, `lua`, `python`, `rust`, `swift`, `typescript`. Examples are organized by SDK language under `examples/<lang>/`.

**Valid dependency directions:**
- Layer 5 → Layer 4 → Layer 3 → Layer 2 → Layer 1
- Any layer may depend on layers below it (not just adjacent)

**Invalid:**
- Layer 1 importing from Layer 2+ (foundation depending on libs/services/engine)
- Layer 5 (FFI) is the outermost boundary — nothing inside `goud_engine/src/` may depend on it
- SDKs reaching into engine internals instead of going through FFI
- `libs/graphics/` importing from `ecs/` (upward, since ecs is Layer 3)

## Audit Process

### Step 1: Scan `use` Statements

Check Rust `use` statements for violations:

The authoritative validator is `cargo run -p lint-layers`, which scans every `.rs` under `goud_engine/src/` and reports layer violations. Run it first; the greps below are for narrowing down a specific breach.

```bash
# Authoritative layer check
cargo run -p lint-layers

# Find all use statements in the codebase
rg "^use " goud_engine/src/ --type rust

# Layer 1 (core) MUST NOT import from any higher layer
rg "use crate::(libs|ecs|assets|sdk|rendering|component_ops|context_registry|ffi|wasm)" goud_engine/src/core/ --type rust

# Layer 2 (libs) MUST NOT import from Layer 3+
rg "use crate::(ecs|assets|sdk|rendering|component_ops|context_registry|ffi|wasm)" goud_engine/src/libs/ --type rust

# Layer 3 (services) MUST NOT import from Layer 4+
rg "use crate::(sdk|rendering|component_ops|context_registry|ffi|wasm)" goud_engine/src/ecs/ goud_engine/src/assets/ --type rust

# Layer 4 (engine) MUST NOT import from Layer 5 (FFI)
rg "use crate::(ffi|wasm)" goud_engine/src/sdk/ goud_engine/src/rendering/ --type rust
```

### Step 2: Validate Module Boundaries

Each module should have a clear public API through its `mod.rs` or `lib.rs`:

- `libs/graphics/` exposes renderer traits, not internal OpenGL types
- `libs/ecs/` exposes World, Entity, Component types, not storage internals
- `ffi/` exposes only `extern "C"` functions
- `sdk/` wraps engine types with ergonomic Rust API

### Step 3: Check Cross-Module Types

Types should not leak across boundaries:

- OpenGL types (`GLuint`, `GLint`) stay in `libs/graphics/backend/`
- GLFW types stay in `libs/platform/window/`
- ECS storage internals stay in `libs/ecs/`
- FFI-specific types (`*mut`, `*const`, `CStr`) stay in `ffi/`

### Step 4: SDK Logic Audit

All 10 SDKs MUST be thin wrappers. Check each language for logic that belongs in Rust:

```bash
# C# SDK: look for complex logic (loops, conditionals beyond null checks)
rg "for |while |if .* && |switch " sdks/csharp/ --type cs -g '!generated/*'

# Python SDK: look for logic beyond FFI calls
rg "for |while |if .* and |if .* or " sdks/python/goudengine/ --type py -g '!generated/*'

# Sweep the remaining SDKs for the same smell
rg "for |while " sdks/go/ sdks/kotlin/ sdks/lua/ sdks/swift/ sdks/rust/ sdks/typescript/ sdks/c/ sdks/cpp/ -g '!*generated*'
```

Flag any non-trivial logic found in SDK code — it should be moved to Rust and exposed via FFI so every SDK inherits it.

### Step 5: Circular Dependency Check

Generate the module dependency graph and inspect:

```bash
./graph.sh  # Creates module_graph.png
```

Look for cycles in the output. Any cycle is a hard violation.

## Output Format

```
# Architecture Review — GoudEngine

## Layer Violations
| Violation | File | Import | Direction |
|-----------|------|--------|-----------|
(none found = PASS)

## Module Boundary Breaches
| Module | Leaked Type | Found In |
|--------|-------------|----------|
(none found = PASS)

## SDK Logic Violations
| SDK | File | Logic Found | Suggested Move |
|-----|------|-------------|----------------|
(none found = PASS)

## Circular Dependencies
(none found = PASS)

## Verdict: PASS | VIOLATIONS FOUND
```

## Quick Validation Command

Run the dependency graph generation to visually verify:

```bash
./graph.sh && open module_graph.png
```
