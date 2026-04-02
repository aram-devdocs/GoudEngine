# ffi/ — Foreign Function Interface Layer

## Purpose

C-compatible FFI exports consumed by the C# and Python SDKs. csbindgen auto-generates
C# bindings from these exports on `cargo build`.

## Files

- `context.rs` — Engine context creation/destruction
- `entity.rs` — Entity creation, deletion, component attachment
- `renderer.rs` — 2D rendering FFI functions
- `renderer3d.rs` — 3D rendering FFI functions
- `component.rs` — Generic component operations
- `component_transform2d.rs` — Transform2D component FFI
- `component_sprite.rs` — Sprite component FFI
- `collision.rs` — Collision detection FFI
- `input.rs` — Input state queries
- `window.rs` — Window management
- `types.rs` — `#[repr(C)]` type definitions shared across FFI
- `mod.rs` — FFI module registration

## Rules (MANDATORY)

- All public functions MUST be `#[no_mangle] extern "C"`
- Structs shared across FFI MUST be `#[repr(C)]`
- Every pointer parameter MUST have a null check
- Every `unsafe` block MUST have a `// SAFETY:` comment
- Error codes returned as `i32` (0 = success, negative = error)
- Memory ownership: document who allocates and who frees

## After Changes

1. Run `cargo build` to trigger csbindgen (updates C# `NativeMethods.g.cs`)
2. Manually update `sdks/python/goudengine/generated/_ffi.py`
3. Update both C# and Python SDK wrappers

## Dependencies

Layer 3 (FFI). May import from core/, ecs/, assets/, libs/. NEVER import from sdk/.
