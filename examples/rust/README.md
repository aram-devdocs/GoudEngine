# Rust Examples

Rust examples are standalone Cargo packages in the workspace.

| Example | Purpose | Run Command |
|---------|---------|-------------|
| `flappy_bird/` | Playable cross-SDK parity example | `cargo run -p flappy-bird` |
| `sandbox/` | Interactive parity sandbox using the shared sandbox asset pack | `cargo run -p sandbox` |
| `feature_lab/` | Headless Rust SDK smoke example for scenes, ECS, animation, input, and headless fallbacks | `cargo run -p feature-lab` |

Windowed examples default to `winit + wgpu`.
To request the explicit legacy pair, set `WindowBackendKind::GlfwLegacy` and
`RenderBackendKind::OpenGlLegacy` on `GameConfig` or `EngineConfig`.

## Debugger Attach Reference

Rust examples follow the same shared debugger contract as every other desktop SDK.
To make a Rust app discoverable by `goudengine-mcp`, enable debugger mode before
startup with `DebuggerConfig { enabled: true, publish_local_attach: true, ... }`.

- Windowed flow: use `EngineConfig::with_debugger(...)`.
- Headless flow: use `Context::create_with_config(ContextConfig { debugger: ... })`.
- Bridge workflow: run `cargo run -p goudengine-mcp`, call `goudengine.list_contexts`,
  then `goudengine.attach_context`.

The current example binaries do not auto-enable debugger mode by default. Use the
Rust SDK README and debugger runtime guide as the reference path when you want to
turn debugger attach on for a local example build.
