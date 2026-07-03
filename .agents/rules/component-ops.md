---
globs:
  - "**/component_ops/**"
  - "**/context_registry/**"
alwaysApply: false
---

# Component Ops and Context Registry Patterns

`component_ops/` and `context_registry/` are top-level Layer 4 (Engine) modules — reached as `crate::component_ops` and `crate::context_registry`, not under `core/`. They MAY import from lower layers (`core/`, `ecs/`) and each other, and are consumed by Layer 5 (`ffi/`, `wasm/`, `jni/`). They MUST NOT import from `ffi/` or `sdk/`.

## Context Registry

- A `GoudContext` is one isolated engine instance: it owns its own ECS `World`, generation counter, and (in debug) thread-ownership tracking. Multiple contexts coexist (game instances, editor viewports, test isolation).
- `GoudContextId` is a generational handle encoding slot index + generation. Destroying a context increments the slot's generation, so a stale ID fails validation instead of aliasing a reused context. Never construct or compare raw IDs.
- The global registry is a singleton behind `Mutex`, reached via `get_context_registry()` (backed by `OnceLock`). Lifecycle: `get_context_registry().lock().create()` -> operate with the `GoudContextId` as the first parameter of every op -> `registry.destroy(id)` to free the slot.
- Locking: the registry Mutex protects create/destroy and slot lookup. Individual contexts are NOT `Send`/`Sync` — use a context only from the thread that created it. `GoudContextHandle` wraps `Arc<RwLock<...>>` for shared access within that thread. Hold the registry lock as briefly as possible and never call back into the registry while holding it.

## Component Ops

- This module implements type-erased component operations (`component_register_type_impl`, `component_add_impl`, `remove`, `has`, `get`, `get_mut`, plus batch variants in `batch_ops.rs`). Both the FFI `#[no_mangle]` wrappers and the Rust SDK call these `_impl` functions, so logic lives here once.
- Components cross the boundary as raw byte pointers plus a `type_id_hash` and `size`/`align` metadata, because the FFI layer does not know concrete Rust types at compile time. Storage is `RawComponentStorage`, a sparse-set-like byte store (`sparse` index -> `dense` array of entities + `data` pointers).
- Storage state lives in module-level statics guarded by `Mutex`: `CONTEXT_COMPONENT_STORAGE` (per-context storage maps) and `COMPONENT_TYPE_REGISTRY` (global type info), both `Mutex<Option<HashMap<...>>>` accessed through their guard helpers. Take the lock, operate, drop it — do not leak a `MutexGuard` across an FFI return.
- `RawComponentStorage` carries `unsafe impl Send + Sync`; that soundness relies entirely on the registry/storage Mutexes serializing access. Do not add a code path that touches the raw pointers without holding the lock.
- All `_impl` functions in `single_ops.rs`/`batch_ops.rs` are `unsafe`. Callers MUST pass pointers to valid data whose size/alignment match the registered type, and keep that memory valid for the call's duration. Every `unsafe` block needs a `// SAFETY:` comment.
