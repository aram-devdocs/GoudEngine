# rust/ -- Rust SDK Re-export Crate

## Purpose

Standalone crate (`goud-engine-sdk`) that re-exports `goud_engine::sdk::*` for Rust consumers.
Unlike the C#, Python, and TypeScript SDKs which go through FFI, this crate links directly
against the engine with zero overhead.

## Key Files

- `Cargo.toml` -- declares dependency on `goud_engine` via path
- `src/lib.rs` -- single line: `pub use goud_engine::sdk::*;`

## Why This Exists

A separate crate lets downstream Rust projects depend on `goud-engine-sdk` without pulling
in FFI exports, codegen build scripts, or napi dependencies. It also provides a clean
versioned package boundary if the SDK is published to crates.io.

## Documented Exception

The Rust SDK bypasses the FFI layer intentionally. Routing Rust-to-Rust calls through
C-ABI FFI would add mutex locks and context lookups for zero benefit. This is the only
SDK that does not go through `goud_engine/src/ffi/`.

## Rules

- MUST stay in sync with `goud_engine/src/sdk/` -- this crate is a pass-through
- NEVER add logic here; all implementation belongs in `goud_engine/src/sdk/`
- NEVER import from `ffi/` -- that path is for non-Rust SDKs only
