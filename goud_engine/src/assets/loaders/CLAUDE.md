# assets/loaders/ — Asset Loader Implementations

## Purpose

Concrete implementations of the `AssetLoader` trait for each supported asset type.

## Files

- `texture.rs` — Texture loading (PNG, JPG via `image` crate)
- `shader.rs` — GLSL shader loading (vertex/fragment pairs)
- `audio.rs` — Audio file loading (WAV/OGG)
- `rodio_integration.rs` — Rodio audio backend integration
- `mod.rs` — Loader registration and re-exports

## Patterns

- Each loader implements the `AssetLoader` trait
- Loaders MUST handle missing files gracefully — return `Result`, never panic
- Texture loader supports PNG and JPG formats
- Shader loader reads GLSL vertex and fragment shader pairs
- Audio loader supports WAV and OGG via rodio

## Adding a New Loader

1. Create `new_format.rs` implementing `AssetLoader`
2. Register in `mod.rs`
3. Handle all error cases with `Result` (file not found, parse errors, format errors)
4. Add tests — loaders are pure I/O so they can be tested without GL context
