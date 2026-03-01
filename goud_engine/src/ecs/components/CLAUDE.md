# ecs/components/ — Built-in ECS Components

## Purpose

All built-in component types used by the engine's ECS.

## Files

- `transform2d.rs` — Primary 2D transform (position, rotation, scale)
- `global_transform2d.rs` — Computed world-space 2D transform
- `transform.rs` — 3D transform
- `global_transform.rs` — Computed world-space 3D transform
- `sprite.rs` — 2D sprite rendering component
- `hierarchy.rs` — Parent/child entity relationships
- `propagation.rs` — Transform inheritance through hierarchy
- `collider.rs` — Collision shape definitions
- `rigidbody.rs` — Physics rigid body properties
- `audiosource.rs` — Audio playback component
- `mod.rs` — Component re-exports

## Patterns

- All components derive `Debug`, `Clone`
- Components are plain data structs — no side effects in methods
- `Transform2D` is the primary 2D transform; `GlobalTransform2D` is system-computed
- Hierarchy component enables parent/child; propagation system handles inheritance
- FFI exports for components live in `ffi/component_*.rs` files

## Adding a New Component

1. Create `new_component.rs` in this directory
2. Derive standard traits (`Debug`, `Clone`)
3. Register in `mod.rs`
4. Add FFI exports in `ffi/component_new_component.rs`
5. Update C# SDK (`sdks/csharp/generated/Components/`)
6. Update Python SDK (`sdks/python/goud_engine/generated/_ffi.py`)
