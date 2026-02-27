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

Dependencies MUST flow DOWN only. No upward imports. No same-layer cross-imports (unless explicitly documented).

```
Layer 1 (Core)    : goud_engine/src/libs/   — graphics, platform, ecs, logger
Layer 2 (Engine)  : goud_engine/src/        — core, assets, sdk
Layer 3 (FFI)     : goud_engine/src/ffi/
Layer 4 (SDKs)    : sdks/                   — GoudEngine (C#), python
Layer 5 (Apps)    : examples/               — csharp, python, rust
```

**Valid dependency directions:**
- Layer 5 → Layer 4 → Layer 3 → Layer 2 → Layer 1
- Any layer may depend on layers below it (not just adjacent)

**Invalid:**
- Layer 1 importing from Layer 2+ (core depending on engine)
- Layer 3 importing from Layer 4 (FFI depending on SDK)
- `libs/graphics/` importing from `libs/ecs/` (same-layer cross-import without justification)

## Audit Process

### Step 1: Scan `use` Statements

Check Rust `use` statements for violations:

```bash
# Find all use statements in the codebase
rg "^use " goud_engine/src/ --type rust

# Check for upward dependencies from libs (Layer 1)
rg "use crate::(ffi|sdk|assets)" goud_engine/src/libs/ --type rust

# Check for upward dependencies from core modules (Layer 2)
rg "use crate::ffi" goud_engine/src/core/ --type rust
rg "use crate::ffi" goud_engine/src/assets/ --type rust

# Check FFI (Layer 3) doesn't import SDK (Layer 4)
rg "use crate::sdk" goud_engine/src/ffi/ --type rust
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

SDKs MUST be thin wrappers. Check for logic that belongs in Rust:

```bash
# C# SDK: look for complex logic (loops, conditionals beyond null checks)
rg "for |while |if .* && |switch " sdks/GoudEngine/ --type cs

# Python SDK: look for logic beyond FFI calls
rg "for |while |if .* and |if .* or " sdks/python/goud_engine/ --type py
```

Flag any non-trivial logic found in SDK code — it should be moved to Rust.

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
