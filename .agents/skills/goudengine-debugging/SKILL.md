---
name: goudengine-debugging
description: Debugging workflow and diagnostic checklists for GoudEngine runtime via MCP tools
user-invocable: true
---

# GoudEngine Debugging

Use this skill to diagnose runtime issues in a running GoudEngine game through the MCP debugger bridge. The workflow relies on the 18 MCP tools exposed by `goudengine-mcp`.

## When to Use

Use when a game is exhibiting unexpected behavior at runtime: performance problems, visual glitches, crashes, physics anomalies, or audio issues. The game must have debugger mode enabled and `publish_local_attach = true` set in its config.

## Core Debugging Workflow

Follow these steps in order. Stop and investigate if any step returns an error.

1. **Discover** -- call `goudengine.list_contexts` to find running debugger routes.
2. **Attach** -- call `goudengine.attach_context` with the target `contextId` and `processNonce`.
3. **Assess** -- call `goudengine.get_diagnostics` to get the full provider diagnostics map. This gives an immediate overview of render, physics, audio, input, sprite batch, assets, and window subsystems.
4. **Capture** -- call `goudengine.capture_frame` to get the current framebuffer image plus snapshot and metrics attachments.
5. **Analyze** -- use the scenario checklists below to drill into the specific problem area.

## Scenario Checklists

### Laggy Game / Low FPS

1. `goudengine.get_subsystem_diagnostics` with key `render` -- check `draw_calls` and `fps`.
2. `goudengine.get_subsystem_diagnostics` with key `sprite_batch` -- check `batch_ratio` (draw calls / sprite count). A ratio close to 1.0 means batching is not working.
3. `goudengine.get_metrics_trace` -- export the full profiler trace and look for frame time outliers.
4. `goudengine.get_snapshot` -- check entity count. A very high entity count may indicate leaked entities.
5. If needed, `goudengine.set_paused` to freeze the game, then `goudengine.step` frame-by-frame to isolate which frame is slow.

### Wrong Visuals / Rendering Bugs

1. `goudengine.capture_frame` -- inspect the framebuffer image for visible artifacts.
2. `goudengine.inspect_entity` on suspect entities -- check `Transform2D`, `Sprite`, `Visibility`, and `GlobalTransform2D` components.
3. `goudengine.get_scene_hierarchy` -- verify parent/child relationships. Transform propagation depends on correct hierarchy.
4. `goudengine.get_subsystem_diagnostics` with key `render` -- confirm the correct renderer type is active.
5. `goudengine.get_subsystem_diagnostics` with key `window` -- check resolution and DPI.

### Crash or Error

1. `goudengine.get_logs` -- retrieve recent engine log entries. Use `sinceFrame` to narrow to the frame range around the crash.
2. `goudengine.get_diagnostics` -- check all subsystems for error states or unexpected values.
3. `goudengine.get_snapshot` -- look for entities with missing or corrupted components.
4. If the process is still alive, `goudengine.set_paused` to stabilize it before deeper inspection.

### Physics Issues

1. `goudengine.get_subsystem_diagnostics` with key `physics_2d` -- check `body_count`, `gravity`, and `step_rate`.
2. `goudengine.inspect_entity` on affected entities -- check `RigidBody2D`, `Collider2D`, and `Transform2D`.
3. `goudengine.set_time_scale` to slow down physics for observation.
4. `goudengine.step` with kind `Tick` to advance physics one tick at a time.
5. `goudengine.start_recording` before reproducing the issue, then `goudengine.stop_recording` and `goudengine.start_replay` to replay deterministically.

### Audio Issues

1. `goudengine.get_subsystem_diagnostics` with key `audio` -- check `active_playbacks` and `master_volume`.
2. `goudengine.get_logs` -- look for audio-related warnings or errors.
3. `goudengine.get_diagnostics` -- confirm the audio subsystem is initialized.

### Input Not Working

1. `goudengine.get_subsystem_diagnostics` with key `input` -- check which devices are detected.
2. `goudengine.inject_input` to send synthetic input events and verify the game responds.
3. `goudengine.get_logs` -- check for input-related warnings.

## Tool Reference

All 18 MCP tools available through the `goudengine-mcp` server:

| Tool | Purpose |
|------|---------|
| `goudengine.list_contexts` | Discover debugger contexts from local processes |
| `goudengine.attach_context` | Attach to a specific route-scoped context |
| `goudengine.get_snapshot` | Fetch current debugger snapshot (entities, selection, frame) |
| `goudengine.inspect_entity` | Get expanded detail for one entity |
| `goudengine.get_metrics_trace` | Export versioned profiler/metrics trace |
| `goudengine.capture_frame` | Capture framebuffer image plus metadata attachments |
| `goudengine.set_paused` | Pause or resume the game loop |
| `goudengine.set_time_scale` | Adjust debugger-owned time scale |
| `goudengine.step` | Advance by N frames or ticks while paused |
| `goudengine.inject_input` | Send synthetic input events |
| `goudengine.start_recording` | Begin input recording |
| `goudengine.stop_recording` | End recording and export replay artifact |
| `goudengine.start_replay` | Start replaying from a stored or base64 recording |
| `goudengine.stop_replay` | Stop an active replay |
| `goudengine.get_diagnostics` | Full provider diagnostics map |
| `goudengine.get_subsystem_diagnostics` | Diagnostics for one subsystem key |
| `goudengine.get_logs` | Recent engine log entries, optionally filtered by frame |
| `goudengine.get_scene_hierarchy` | Entity tree with parent/child relationships |

## Prerequisites

- The game must enable debugger mode before startup (via `DebuggerConfig` or `ContextConfig`).
- `publish_local_attach` must be `true` for the route to be discoverable.
- The `goudengine-mcp` server binary must be built: `cargo build -p goudengine-mcp`.
- Desktop/native target only. Browser/WASM debugger attach is not supported.
