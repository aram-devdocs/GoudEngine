# components_sprite/ -- SDK Sprite Component Operations

## Purpose

FFI-ready sprite operations split into three categories: factory (create by value),
builder (heap-allocated fluent API), and pointer ops (get/set on existing sprites).
All methods are annotated with `#[goud_api]` to auto-generate C FFI wrappers.

## Files

- `mod.rs` -- Module declaration, re-exports submodules
- `factory.rs` -- `SpriteOps`: value constructors (`new_sprite`, `new_default`, `with_color`, `with_flip`, etc.)
- `builder.rs` -- `SpriteBuilderOps`: heap-allocated builder pattern (`builder_new` -> chain methods -> `builder_build`)
- `ptr_ops.rs` -- `SpritePtrOps`: get/set operations on `*mut FfiSprite` pointers

## Key Types

- `FfiSprite` -- `#[repr(C)]` struct with texture handle, color RGBA, source rect, flip flags, anchor, custom size
- `FfiSpriteBuilder` -- wrapper holding an `FfiSprite` for heap-based builder pattern
- `FfiColor`, `FfiRect`, `FfiVec2` -- C-compatible return types for getters

## Patterns

- Factory functions return `FfiSprite` by value (copy semantics, no allocation)
- Builder functions return `*mut FfiSpriteBuilder` for chaining; caller MUST call
  `builder_build` or `builder_free` to avoid leaks
- Pointer ops check for null before every dereference and return safe defaults
- `clippy::not_unsafe_ptr_arg_deref` is allowed because the `#[goud_api]` macro
  generates the outer `unsafe extern "C"` wrapper

## Anti-Patterns

- NEVER skip null checks on pointer parameters
- NEVER implement rendering logic here -- sprites are pure data
- NEVER forget to pair `builder_new` with `builder_build` or `builder_free`

## Dependencies

Layer 2 (Engine/SDK). Imports from `crate::core::types` for FFI structs.
