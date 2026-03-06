# sdk/ — Rust Native SDK

## Purpose

Zero-overhead native Rust API. Provides ergonomic wrappers over the engine internals
without FFI indirection. Mirrors the functionality exposed to C#/Python via FFI.

## Files

- `mod.rs` — SDK module re-exports
- `components.rs` — Ergonomic component API for Rust users
- `component_ops.rs` — Generic component CRUD operations
- `color.rs` — Color type and factory methods
- `context.rs` — Engine context management
- `entity.rs` — Entity handle operations
- `entity_builder.rs` — Entity builder pattern
- `game.rs` — GoudGame high-level API
- `game_config.rs` — Game configuration
- `input.rs` — Input state queries
- `rendering.rs` — 2D rendering SDK interface
- `rendering_3d.rs` — 3D rendering SDK interface
- `texture.rs` — Texture loading and management
- `window.rs` — Window management
- `collision/` — Collision detection API
  - `mod.rs` — Module root
  - `tests.rs` — Collision detection tests
- `components_sprite/` — Sprite component FFI wrappers
  - `mod.rs` — Module root
  - `builder.rs` — Sprite builder pattern
  - `factory.rs` — Sprite factory methods
  - `ptr_ops.rs` — Pointer-based operations for FFI
- `components_transform2d/` — Transform2D component FFI wrappers
  - `mod.rs` — Module root
  - `builder.rs` — Transform2D builder pattern
  - `factory.rs` — Transform2D factory methods
  - `ptr_ops.rs` — Pointer-based operations for FFI

## Patterns

- Direct Rust API — no FFI overhead, no marshalling
- Wraps ECS components with builder patterns and convenience methods
- MUST stay in sync with FFI exports (same capabilities available)

## Anti-Patterns

- NEVER duplicate logic that exists in core/ecs/assets — wrap it
- NEVER diverge from FFI capabilities — if Rust SDK can do it, FFI should expose it too

## Dependencies

Layer 2 (Engine). May import from core/, ecs/, assets/, libs/. NEVER import from ffi/.
