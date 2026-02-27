---
globs:
  - "**/ecs/**"
---

# ECS Architecture Patterns

GoudEngine uses a Bevy-inspired Entity-Component-System architecture.

## Components

- Plain data structs — no methods with side effects
- Derive `Debug` and `Clone` at minimum
- `Transform2D` is the primary 2D spatial component; `GlobalTransform2D` is computed by the propagation system
- `Hierarchy` component manages parent/child relationships
- FFI exports for components live in `ffi/component_*.rs` files

## Entities

- Entity IDs use the generational `Handle` pattern — never store or compare raw `u32` values
- Create entities through the `World`, not by constructing IDs manually
- Entity deletion must handle component cleanup

## Systems

- Systems are functions that query components from the `World`
- Queries are type-safe and generic — no raw index access into component storage
- System ordering matters: transform propagation must run after hierarchy changes
- Keep systems focused — one responsibility per system

## Queries

- Use the typed query API to access components
- Never downcast or use `Any` to access component data
- Filter queries to the minimal set of components needed

## Transform Propagation

- Parent transforms propagate to children automatically via the propagation system
- Modify `Transform2D` (local), read `GlobalTransform2D` (world-space)
- The propagation system must run every frame after transform/hierarchy modifications
