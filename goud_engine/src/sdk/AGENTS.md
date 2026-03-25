# sdk/ — Rust Native SDK

Zero-overhead native Rust API for Rust game code. Mirrors FFI capabilities.

## Key Modules

- `game.rs`, `entity.rs`, `context.rs` — Core game API
- `components.rs`, `component_ops.rs` — Component CRUD
- `rendering.rs`, `rendering_3d.rs` — Rendering interfaces
- `collision/` — Collision detection
- `components_sprite/`, `components_transform2d/` — Component builders and factories
- `color.rs`, `input.rs`, `texture.rs`, `window.rs` — Utilities

## Patterns

- Ergonomic wrappers with builders, no FFI overhead
- MUST stay in sync with FFI exports

## Anti-Patterns

- Never duplicate core/ecs/assets logic — wrap it
- Never diverge from FFI capabilities

See `.agents/rules/sdk-development.md` and `.agents/rules/dependency-hierarchy.md`.
