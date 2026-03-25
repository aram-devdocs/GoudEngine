# GoudEngine

GoudEngine is a Rust game engine with multi-language SDK support. All game logic lives in Rust. SDKs under `sdks/` provide thin wrappers over the FFI boundary.

## SDK Support

| SDK | Package | Backend |
|-----|---------|---------|
| C# | [NuGet](https://www.nuget.org/packages/GoudEngine) | DllImport (P/Invoke) |
| Python | [PyPI](https://pypi.org/project/goudengine/) | ctypes |
| TypeScript | [npm](https://www.npmjs.com/package/goudengine) | napi-rs (Node.js) + wasm-bindgen (Web) |
| Rust | [crates.io](https://crates.io/crates/goud-engine) | Direct linking (no FFI) |
| C/C++ | local | C header via cbindgen |
| Go | local | cgo |
| Kotlin | local | JNI |
| Swift | local | C interop |
| Lua | local | C FFI |

See the [Getting Started](getting-started/csharp.md) section in the sidebar for per-language guides.

## Engine Features

- **Physics** — 2D and 3D rigid body simulation via Rapier. Supports dynamic, kinematic, and static bodies, collision shapes (circle, box, capsule, polygon), raycasting, and collision events.
- **Audio** — Playback via Rodio with per-channel volume control (Music, SFX, Ambience, UI, Voice), spatial audio with distance attenuation, looping, and pitch control.
- **Text rendering** — TrueType and bitmap font loading, glyph atlas caching, text alignment (left, center, right), word-wrapping, and line spacing.
- **Animation** — Sprite sheet animation with frame events, state machine controller with parametric transitions, multi-layer blending (override and additive modes), and standalone tweening with easing functions.
- **Scene management** — Named scenes with isolated ECS worlds, transitions (instant, fade, custom), and transition progress tracking.
- **UI** — Hierarchical node tree with generational IDs, parent/child relationships, and component-based widgets (buttons, panels, text, images).
- **Error diagnostics** — Structured error codes by category, thread-local error state for FFI, backtrace capture, recovery hints, and severity levels.

## Quick Links

- **New to GoudEngine?** Start with a getting-started guide for your language in the sidebar.
- **Building from source?** See the [Building](development/building.md) and [Development Guide](development/guide.md).
- **Understanding the internals?** Read the [SDK-First Architecture](architecture/sdk-first.md) document.

## Status

GoudEngine is in alpha. APIs change frequently. [Report issues on GitHub](https://github.com/aram-devdocs/GoudEngine/issues).
