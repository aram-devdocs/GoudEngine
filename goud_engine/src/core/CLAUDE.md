# core/ — Core Utilities

## Purpose

Foundation types used across the engine: error handling, events, math, and resource handles.

## Files

- `error.rs` — Error types using `thiserror`; error codes map to categories
- `event.rs`, `events.rs` — Typed event system (no stringly-typed events)
- `handle.rs` — Generational handles (`Handle<T>`) for safe resource references
- `math.rs` — Math types wrapping `glam`, exposed via FFI
- `mod.rs` — Module re-exports

## Patterns

- Error types derive `thiserror::Error` with descriptive messages
- Handles are generational: index + generation counter prevents use-after-free
- Math types wrap `glam` vectors/matrices — never expose `glam` directly in public API
- Events use typed channels, not string-based dispatch

## Anti-Patterns

- NEVER use string-based event names
- NEVER store raw indices as resource references — use `Handle<T>`
- NEVER expose `glam` types directly across FFI — use `#[repr(C)]` wrappers in `math.rs`
- NEVER use `panic!` for recoverable errors — return `Result`

## Dependencies

This is Layer 1 (Core). No imports from ecs/, assets/, ffi/, sdk/, or libs/.
