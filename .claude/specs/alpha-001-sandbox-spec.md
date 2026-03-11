# Alpha-001 Sandbox Contract

Recovery note for this branch:

- `.claude/specs/alpha-001-sandbox-execution.csv` is the current source of truth for live runtime status, blocking bugs, and verification evidence while Sandbox recovery is in progress.
- `.claude/specs/alpha-001-sandbox-matrix.csv` remains the target contract ledger and must not be treated as verified implementation truth until the blocking execution rows are closed.

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

1. top-left `Overview` panel
2. top-right `Live status` panel
3. bottom `Try this next` panel
4. `2D` scene
5. `3D` scene
6. `Hybrid` scene

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

Current Rust-specific truth for this branch:

- Rust desktop now uses the public `GoudGame::draw_text(...)` path for the shared HUD copy and scene badge.
- Rust interactive rendering no longer collapses to a clear-only window after the first frame; the immediate-mode state is restored after Rust text draws.
- Remaining Rust parity work, if any, is now typography/layout polish rather than missing text API support or a frame-to-frame render collapse.

## Networking Contract

- Native wire payload is UTF-8 text with the exact shape:
  `sandbox|v1|<role>|<mode>|<x>|<y>|<label>`
- A single instance shows `host/waiting` or `client/connected`, not a false positive offline success state.
- A second instance on localhost shows peer discovery or connection state and a visible peer sprite/label.
- Cross-language pairing is required for native targets.
- Web keeps the same panel and wording, but unsupported controls must be disabled with explicit explanation.

## Example-Only Smoke Controls

Native targets support the same environment variables:

- `GOUD_SANDBOX_NETWORK_PORT`
- `GOUD_SANDBOX_NETWORK_ROLE=auto|host|client`
- `GOUD_SANDBOX_EXIT_ON_PEER=0|1`
- `GOUD_SANDBOX_EXPECT_PEER=0|1`

## Web Contract

- Web keeps the same layout, labels, and asset paths as desktop.
- Unsupported features are visible and explained via manifest-driven capability-gating copy.
- Unsupported functionality must not be faked.

## Shared Assets

The shared asset pack currently includes:

- `examples/shared/sandbox/sprites/background-day.png`
- `examples/shared/sandbox/sprites/yellowbird-midflap.png`
- `examples/shared/sandbox/sprites/pipe-green.png`
- `examples/shared/sandbox/textures/default_grey.png`
- `examples/shared/sandbox/audio/sandbox-tone.wav`

If more assets are required, they should be added here instead of under language-specific example folders.
