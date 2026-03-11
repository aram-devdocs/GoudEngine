# Alpha-001 Sandbox Contract

## Purpose

Sandbox is the flagship parity example for Alpha-001.

- Flappy Bird remains the beginner tutorial baseline.
- Feature Lab remains a supplemental smoke harness.
- Sandbox is the "show me everything the engine can do" example.

## Targets

Sandbox must exist for:

- Rust desktop
- C# desktop
- Python desktop
- TypeScript desktop
- TypeScript web

All targets must use the shared asset root:

- `examples/shared/sandbox/`

## Shared Layout

Every target presents the same top-level structure:

1. `2D` scene
2. `3D` scene
3. `Hybrid` scene
4. `Diagnostics` panel
5. `Networking` panel
6. `Capability` panel

The default launch view should make it obvious that the app is interactive, not a smoke-only pass/fail runner.

## Interaction Model

- Number keys or on-screen buttons switch between `2D`, `3D`, and `Hybrid`.
- A visible focus entity moves with keyboard input.
- Pointer input updates a visible cursor/debug marker.
- An on-screen status panel lists FPS, active scene, backend/platform info, and peer count.
- An audio trigger is exposed from the UI and from keyboard input.
- A diagnostics area shows unsupported features, runtime errors, and network state.

## Required Feature Coverage

Sandbox must demonstrate or explicitly capability-gate:

- scene creation and switching
- ECS entity spawn/despawn and component mutation
- 2D transforms and sprite rendering
- runtime-created colored shapes or quads
- text rendering
- UI controls
- 3D primitives and camera movement
- combined 2D + 3D rendering in one app flow
- asset loading from shared paths
- animation
- physics or physics capability reporting
- audio playback
- input polling and mapping
- diagnostics/error reporting
- networking state and peer visibility

## Networking Contract

- A single instance shows `solo` or equivalent local-node state.
- A second instance on localhost shows peer discovery or connection state.
- Cross-language pairing is required for native targets.
- Web keeps the same panel and wording, but unsupported controls must be disabled with explicit explanation.

## Web Contract

- Web keeps the same layout, labels, and asset paths as desktop.
- Unsupported features are visible and explained.
- Unsupported functionality must not be faked.

## Shared Assets

The shared asset pack currently includes:

- `examples/shared/sandbox/sprites/background-day.png`
- `examples/shared/sandbox/sprites/yellowbird-midflap.png`
- `examples/shared/sandbox/sprites/pipe-green.png`
- `examples/shared/sandbox/textures/default_grey.png`
- `examples/shared/sandbox/audio/sandbox-tone.wav`

If more assets are required, they should be added here instead of under language-specific example folders.
