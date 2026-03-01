# collision/ -- SDK Collision Detection API

## Purpose

Ergonomic 2D collision detection functions for Rust SDK users. Pure math with
no engine state required. Delegates to the internal `ecs::collision` module
and re-exports `Contact` and `CollisionResponse` so users do not reach into
engine internals.

## Files

- `mod.rs` -- `Collision` struct with `#[goud_api]` methods, free-function wrappers, re-exports
- `tests.rs` -- comprehensive tests for all collision functions

## API Surface

| Function | Description |
|----------|-------------|
| `aabb_aabb` | AABB vs AABB, returns `Option<Contact>` |
| `circle_circle` | Circle vs circle, returns `Option<Contact>` |
| `circle_aabb` | Circle vs AABB, returns `Option<Contact>` |
| `point_in_rect` | Point inside rectangle (origin + size) |
| `point_in_circle` | Point inside circle |
| `aabb_overlap` | Fast boolean AABB overlap (min/max corners) |
| `circle_overlap` | Fast boolean circle overlap |
| `distance` | Euclidean distance between two points |
| `distance_squared` | Squared distance (avoids sqrt) |

## FFI Generation

The `Collision` struct uses `#[goud_engine_macros::goud_api(module = "collision")]`
to auto-generate `#[no_mangle] extern "C"` FFI wrappers. The public free
functions are convenience wrappers that delegate to the struct methods.

## Patterns

- All functions take `Vec2` and scalar parameters, return `Option<Contact>` or `bool`
- `Contact` provides penetration depth, collision normal, and contact point
- `distance_squared` is preferred over `distance` in hot paths to skip the sqrt

## Dependencies

Layer 2 (Engine/SDK). Imports from `crate::core::math::Vec2` and
`crate::ecs::collision`. NEVER imports from ffi/.
