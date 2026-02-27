# ecs/ — Entity Component System

## Purpose

Bevy-inspired ECS architecture: World → Entities → Components, with type-safe queries and systems.

## Files

- `world.rs` — Central ECS container; owns all entities and components
- `entity.rs` — Entity identifiers (generational)
- `component.rs` — Component trait and registration
- `archetype.rs` — Archetype-based storage layout
- `sparse_set.rs` — Sparse set storage for components
- `storage.rs` — Storage abstraction layer
- `resource.rs` — Singleton resources accessible by systems
- `schedule.rs` — System execution ordering
- `input_manager.rs` — Input state management
- `collision.rs`, `broad_phase.rs` — Collision detection
- `physics_world.rs` — Physics simulation
- `query/` — Type-safe component queries (`fetch.rs`, `mod.rs`)
- `system/` — System trait and function systems
- `systems/` — Built-in systems (rendering, transform propagation)
- `components/` — Built-in component types

## Patterns

- Components MUST be plain data (no methods with side effects)
- Systems are functions that query components via type-safe generics
- Entity IDs are generational — never store raw integers
- Queries use the `fetch.rs` / `mod.rs` pattern for type-safe access

## Anti-Patterns

- NEVER store entity IDs as raw `u32` — use generational entity types
- NEVER put logic in components — logic belongs in systems
- NEVER access components outside the query system

## Dependencies

Layer 1 (Core). May import from `core/` only.
