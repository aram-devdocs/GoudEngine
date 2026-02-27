# Core/ — C# SDK Core Classes

## Purpose

Core engine wrappers providing the main API surface for C# game developers.

## Files

- `GoudContext.cs` — Main engine context; wraps the FFI context pointer
- `Entity.cs` — Entity wrapper; provides component access methods
- `GoudInput.cs` — Input state queries (keyboard, mouse)
- `GoudWindow.cs` — Window management (size, title, fullscreen)
- `GoudRenderer.cs` — Rendering operations (draw sprites, set camera)
- `Exceptions.cs` — Exception types mapping to Rust error codes
- `ERROR_HANDLING.md` — Error handling documentation

## Patterns

- `GoudContext` owns the native context pointer and manages its lifetime
- `Entity` wraps an entity ID and provides `GetComponent<T>()` access
- Exception types in `Exceptions.cs` map to Rust error codes from FFI
- All native calls go through `NativeMethods.g.cs`

## Anti-Patterns

- NEVER store raw pointers outside `GoudContext` — use Entity wrappers
- NEVER catch and swallow exceptions from FFI calls — let them propagate
- NEVER implement game logic here — this is a thin FFI wrapper layer
