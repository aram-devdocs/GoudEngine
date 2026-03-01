# GoudEngine Rust SDK

Standalone crate that re-exports the GoudEngine SDK module for Rust consumers.
Unlike the C#, Python, and TypeScript SDKs which go through FFI, this crate
links directly against the engine with zero overhead.

## Usage

Add the dependency to your `Cargo.toml`:

```toml
[dependencies]
goud-engine-sdk = { path = "path/to/GoudEngine/sdks/rust" }
```

Then use it in your code:

```rust
use goud_engine_sdk::*;

// All types from goud_engine::sdk are available directly:
// GoudGame, Transform2D, Sprite, Vec2, Color, etc.
```

## Why a Separate Crate

A standalone crate lets downstream Rust projects depend on `goud-engine-sdk`
without pulling in FFI exports, codegen build scripts, or napi dependencies.
It also provides a clean versioned package boundary for crates.io publishing.

## Design

This crate contains a single re-export:

```rust
pub use goud_engine::sdk::*;
```

All implementation lives in `goud_engine/src/sdk/`. This crate is a pass-through.
