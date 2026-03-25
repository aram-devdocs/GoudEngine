---
globs:
  - "**/assets/**"
---

# Asset Management Patterns

## AssetServer

- Central coordinator for loading, storing, and hot-reloading assets
- Returns generational `Handle<T>` values — never pass raw pointers or paths as asset references
- Handles are safe: using a stale handle after the asset is unloaded returns an error, not undefined behavior

## AssetLoader Trait

Each asset type has a dedicated loader implementing the `AssetLoader` trait:

See `goud_engine/src/assets/loaders/` for the full list of registered loaders and their supported formats.

## Error Handling

- Loaders MUST return `Result`, never `panic!()` or `unwrap()` on missing files
- Missing assets produce a descriptive error with the file path that failed
- Callers decide whether a missing asset is fatal or recoverable

## Hot-Reload

- Filesystem watcher detects changes to asset files
- Changed assets are automatically reloaded and existing handles updated
- Hot-reload is development-only; release builds may disable the watcher

## Adding a New Asset Type

1. Define the asset data struct
2. Implement `AssetLoader` for the new type
3. Register the loader with `AssetServer`
4. Expose via FFI if SDKs need access
