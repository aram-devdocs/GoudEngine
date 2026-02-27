# sdks/ — SDK Development

## Purpose

Multi-language SDK wrappers over the Rust engine's FFI layer.

## Principle: Rust-First

ALL logic lives in Rust. SDKs are thin wrappers that marshal data and call FFI functions.
If you find logic in an SDK that should be in Rust, move it.

## Structure

- `GoudEngine/` — C# SDK (.NET 8.0, NuGet package)
- `GoudEngine.Tests/` — C# SDK test suite (xUnit)
- `python/` — Python SDK (ctypes-based FFI bindings)

## Feature Parity

Every FFI export MUST have wrappers in BOTH C# AND Python:
1. Check `goud_engine/src/ffi/` for all `#[no_mangle] extern "C"` functions
2. Verify matching `DllImport` in C# `NativeMethods.g.cs`
3. Verify matching ctypes declaration in `python/goud_engine/bindings.py`
4. Verify SDK wrapper classes expose the functionality

## After Adding FFI Functions

1. `cargo build` triggers csbindgen → updates C# bindings automatically
2. Manually update `python/goud_engine/bindings.py` with ctypes signatures
3. Add wrapper methods to C# classes in `GoudEngine/`
4. Add wrapper methods to Python classes in `python/goud_engine/`
5. Run tests for both SDKs

## Anti-Patterns

- NEVER implement game logic in SDK code — call FFI instead
- NEVER add an FFI function without updating BOTH SDKs
- NEVER duplicate Rust types manually — use codegen (csbindgen for C#)
