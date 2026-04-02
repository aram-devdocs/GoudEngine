# python/goudengine/ — Python Package

## Purpose

The installable Python package providing GoudEngine bindings.

## Files

- `__init__.py` — Public API exports
- `generated/_ffi.py` — ctypes function declarations (argtypes, restype for every FFI function)
- `ffi_metadata.json` — FFI function metadata for tooling

## Patterns

- `generated/_ffi.py` declares every FFI function with explicit `argtypes` and `restype`
- Rust `snake_case` FFI names map directly to Python `snake_case` wrappers
- Library loading handles macOS (`.dylib`) and Linux (`.so`) paths
- `__init__.py` re-exports the public API — users import from `goudengine`

## Updating Bindings

When new FFI functions are added in Rust (`goudengine/src/ffi/`):

1. Add ctypes declaration in `generated/_ffi.py`:
   - Set `argtypes` to match the C function signature
   - Set `restype` to match the return type
2. Add Python wrapper class methods as needed
3. Update `ffi_metadata.json` if used by tooling
4. Run `python3 sdks/python/test_bindings.py`

## Anti-Patterns

- NEVER guess ctypes signatures — match the `#[repr(C)]` types exactly
- NEVER add logic beyond marshalling — all logic lives in Rust
