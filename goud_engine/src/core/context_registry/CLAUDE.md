# context_registry/ — Engine Context Management

## Purpose

Manages isolated engine contexts. Each context owns its own ECS World, entities,
components, and resources. Multiple contexts can coexist (e.g., multiple game instances,
editor viewports, or test isolation).

## Files

- `mod.rs` — Module declaration and re-exports
- `context_id.rs` — `GoudContextId` type (u64) and `GOUD_INVALID_CONTEXT_ID` sentinel
- `context.rs` — `GoudContext` struct: holds a `World`, generation counter, and
  debug-only thread ownership tracking
- `registry.rs` — `GoudContextRegistry` with generational slot allocation,
  `GoudContextHandle` (Arc-based access), and `get_context_registry()` global accessor
- `tests.rs` — Unit tests for context lifecycle and registry operations

## Context Lifecycle

1. `get_context_registry().lock().create()` allocates a new context, returns a `GoudContextId`
2. All engine operations take `GoudContextId` as their first parameter
3. `registry.destroy(id)` frees the context; the slot's generation increments
4. Reusing a stale ID after destruction is detected via generation mismatch

## Generational Indexing

`GoudContextId` encodes both a slot index and a generation counter. When a context
is destroyed and its slot reused, the generation increments. Old IDs fail validation
instead of silently accessing the wrong context.

## Thread Safety

- The registry itself is behind `Mutex` -- thread-safe for create/destroy
- Individual contexts are NOT `Send`/`Sync` -- use from the creating thread only
- `GoudContextHandle` uses `Arc<RwLock<...>>` for shared access within a thread

## Dependencies

Layer 2 (Engine). Imports from `ecs::World` and `core::error`. Every FFI function
and most `component_ops` functions depend on this module for context lookup.
