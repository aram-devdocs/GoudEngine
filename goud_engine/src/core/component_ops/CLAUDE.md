# component_ops/ — Type-Erased Component Operations

## Purpose

Implementation logic for component operations that work with raw bytes and type IDs
rather than concrete Rust types. Both the FFI layer and the Rust SDK call into these
`_impl` functions, keeping component logic in one place.

## Why Type-Erased?

The FFI boundary does not know Rust types at compile time. Components cross FFI as
raw byte pointers with a `type_id_hash` and `size`/`align` metadata. This module
bridges that gap while keeping the actual storage and validation centralized.

## Files

- `mod.rs` — Re-exports all public `_impl` functions
- `single_ops.rs` — Single-entity operations: `component_register_type_impl`,
  `component_add_impl`, `component_remove_impl`, `component_has_impl`,
  `component_get_impl`, `component_get_mut_impl`
- `batch_ops.rs` — Multi-entity operations: `component_add_batch_impl`,
  `component_has_batch_impl`, `component_remove_batch_impl`
- `storage.rs` — `RawComponentStorage` (sparse-set-like byte storage), global type
  registry, per-context storage maps
- `helpers.rs` — Shared utilities: context key construction, entity ID conversion

## Call Flow

```
FFI #[no_mangle] fn  --->  component_*_impl()  --->  RawComponentStorage
Rust SDK wrapper     --->  component_*_impl()  --->  RawComponentStorage
```

## Safety

All `_impl` functions in `single_ops.rs` and `batch_ops.rs` are `unsafe`. Callers must:
- Pass pointers to valid component data
- Ensure size/alignment match the registered type
- Keep memory valid for the duration of the call

## Dependencies

Layer 2 (Engine). Imports from `core::context_registry`, `core::error`, `core::types`,
and `ecs::Entity`. Does not import from `ffi/` or `sdk/`.
