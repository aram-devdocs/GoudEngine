---
globs:
  - "goud_engine/src/ffi/**"
---

# FFI Boundary Patterns

The FFI layer is the bridge between Rust and external SDKs (C#, Python). Every function and type here must be safe, well-documented, and compatible with C ABI.

## Function Requirements

- All public functions MUST be `#[no_mangle] extern "C"`
- Return errors as `i32` (0 = success, negative = error code)
- Every pointer parameter MUST have a null check before dereferencing
- Every `unsafe` block MUST have a `// SAFETY:` comment explaining why it's sound

## Type Requirements

- Structs shared across FFI MUST be `#[repr(C)]`
- Use C-compatible types only: primitive integers, floats, `*const T`, `*mut T`, `bool`
- No `String`, `Vec`, `Option`, or other Rust-only types in FFI signatures

## Memory Ownership

- Document who allocates and who frees for every pointer parameter
- Default convention: caller allocates, caller frees (unless explicit transfer)
- Box-allocated values returned to callers require a corresponding `_free` function

## After FFI Changes

1. Run `cargo build` — triggers csbindgen to regenerate C# bindings
2. Manually update `sdks/python/goud_engine/bindings.py` for ctypes declarations
3. Update BOTH the C# and Python SDK wrappers to expose the new functionality
4. Run SDK tests for both languages

## File Organization

Each domain has its own FFI file: `component_*.rs`, `renderer.rs`, `audio.rs`, etc. Keep related functions grouped together.
