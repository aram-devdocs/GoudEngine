# Components/ — C# Component Wrappers

## Purpose

C# wrappers for ECS components. Each class corresponds to a Rust component
exposed through FFI.

## Files

- `IComponent.cs` — Interface all components implement
- `Transform2D.cs` — 2D transform wrapper (position, rotation, scale)
- `Sprite.cs` — Sprite rendering wrapper (texture, source rect, color)
- `README.md` — Component usage documentation

## Patterns

- All components implement `IComponent`
- Properties call FFI functions to get/set values on the Rust side
- Constructor takes entity ID + context pointer for FFI calls
- Naming: PascalCase properties matching Rust field names

## Adding a New Component Wrapper

1. Create `NewComponent.cs` implementing `IComponent`
2. Add properties that call corresponding FFI functions
3. FFI functions are in `NativeMethods.g.cs` (auto-generated)
4. Match the Rust component in `goud_engine/src/ecs/components/`
5. Add tests in `sdks/GoudEngine.Tests/Components/`
6. Add equivalent Python wrapper for SDK parity
