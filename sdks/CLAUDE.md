# sdks/ — SDK Development

## Purpose

Multi-language SDK wrappers over the Rust engine's FFI layer.

## Principle: Rust-First

ALL logic lives in Rust. SDKs are thin wrappers that marshal data and call FFI functions.
If you find logic in an SDK that should be in Rust, move it.

## Structure

- `csharp/` — C# SDK (.NET 8.0, NuGet package)
- `csharp.tests/` — C# SDK test suite (xUnit)
- `python/` — Python SDK (ctypes-based FFI bindings)
- `typescript/` — TypeScript SDK: Node.js desktop (napi-rs) + Web browser (wasm-bindgen)
- `rust/` — Rust SDK re-export crate (`goud-engine-sdk`), zero FFI overhead

## Feature Parity

Every FFI export MUST have wrappers in ALL SDK languages — C#, Python, and TypeScript:
1. Check `goud_engine/src/ffi/` for all `#[no_mangle] extern "C"` functions
2. Verify matching `DllImport` in C# `NativeMethods.g.cs`
3. Verify matching ctypes declaration in `python/goud_engine/generated/_ffi.py`
4. Verify matching napi-rs bindings in `typescript/native/src/` (Node) and WASM exports (Web)
5. Verify SDK wrapper classes expose the functionality in all languages
6. The Rust SDK (`rust/`) re-exports `goud_engine::sdk::*` directly — no FFI involved

## After Adding FFI Functions

1. `cargo build` triggers csbindgen → updates C# bindings automatically
2. Manually update `python/goud_engine/generated/_ffi.py` with ctypes signatures
3. Add wrapper methods to C# classes in `csharp/`
4. Add wrapper methods to Python classes in `python/goud_engine/`
5. Run `./codegen.sh` to regenerate TypeScript SDK (Node + Web) from the schema
6. Run tests for all SDKs

## Anti-Patterns

- NEVER implement game logic in SDK code — call FFI instead
- NEVER add an FFI function without updating ALL SDKs (C#, Python, TypeScript)
- NEVER duplicate Rust types manually — use codegen (csbindgen for C#, gen_*.py for others)
