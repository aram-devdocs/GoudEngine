# sdk/ — Rust Native SDK

## Purpose

Zero-overhead native Rust API. Provides ergonomic wrappers over the engine internals
without FFI indirection. Mirrors the functionality exposed to C#/Python via FFI.

## Files

- `components.rs` — Ergonomic component API for Rust users
- `mod.rs` — SDK module re-exports

## Patterns

- Direct Rust API — no FFI overhead, no marshalling
- Wraps ECS components with builder patterns and convenience methods
- MUST stay in sync with FFI exports (same capabilities available)

## Anti-Patterns

- NEVER duplicate logic that exists in core/ecs/assets — wrap it
- NEVER diverge from FFI capabilities — if Rust SDK can do it, FFI should expose it too

## Dependencies

Layer 2 (Engine). May import from core/, ecs/, assets/, libs/. NEVER import from ffi/.
