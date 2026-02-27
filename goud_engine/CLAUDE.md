# goud_engine/ — Main Rust Crate

## Purpose

Primary Cargo workspace member containing all engine code. This is the core of GoudEngine.

## Module Organization

- `src/core/` — Error types, events, math wrappers, generational handles
- `src/ecs/` — Entity Component System (World, entities, components, systems, queries)
- `src/assets/` — Asset loading, storage, hot-reload, audio
- `src/libs/` — Subsystems (graphics rendering, platform)
- `src/ffi/` — C-compatible FFI exports (auto-generates C# bindings via csbindgen)
- `src/sdk/` — Native Rust SDK (zero FFI overhead)

## Build

```bash
cargo build              # Debug build
cargo build --release    # Release build
cargo test               # Run all tests
cargo test -- --nocapture # Show test output
cargo bench              # Run benchmarks
```

## Key Files

- `Cargo.toml` — Crate dependencies and build configuration
- `src/lib.rs` — All public API goes through this entry point
- `build.rs` — Build script (triggers csbindgen for C# bindings)

## Rules

- All public API MUST be re-exported through `lib.rs`
- New modules MUST be registered in `mod.rs` or `lib.rs`
- Dependencies flow DOWN the layer hierarchy only
- `cargo check` MUST pass before committing
- `cargo clippy -- -D warnings` MUST be clean
