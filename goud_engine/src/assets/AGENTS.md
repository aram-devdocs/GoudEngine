# assets/ — Asset Management

## Purpose

Asset loading, storage, hot-reload, and audio management.

## Files

- `server.rs` — AssetServer: loading, storage, and hot-reload coordination
- `asset.rs` — Asset trait and type definitions
- `handle.rs` — Generational asset handles (`Handle<T>`) for safe references
- `loader.rs` — AssetLoader trait for pluggable loaders
- `storage.rs` — Asset storage backend
- `hot_reload.rs` — Filesystem watcher for automatic asset re-loading
- `audio_manager.rs` — Audio playback management
- `loaders/` — Concrete loader implementations
- `mod.rs` — Module re-exports

## Patterns

- Handles are generational (`Handle<T>`) — safe to hold across frames
- Loaders implement the `AssetLoader` trait
- Hot-reload watches the filesystem and triggers automatic re-load
- Audio uses `rodio` integration for playback

## Anti-Patterns

- NEVER return raw pointers to assets — use `Handle<T>`
- NEVER panic on missing assets — return `Result` with descriptive error
- NEVER load assets synchronously on the main thread in production paths

## Dependencies

Layer 2 (Engine). May import from `core/` and `libs/`.
