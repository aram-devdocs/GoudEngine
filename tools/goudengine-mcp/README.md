# goudengine-mcp

MCP (Model Context Protocol) server that bridges AI agent tool calls to a running GoudEngine debugger runtime over local IPC.

## Overview

`goudengine-mcp` is a stdio-based MCP server. It discovers local GoudEngine processes that have published debugger manifests, attaches to one route-scoped context, and translates MCP tool calls into debugger protocol requests. The server runs entirely outside the game process and never modifies game state except through explicit control-plane tools.

The server exposes 18 tools, 6 static knowledge resources, 3 artifact resource templates, and 3 guided prompts.

## Building

```bash
cargo build -p goudengine-mcp --release
```

Binary output: `target/release/goudengine-mcp`

## IDE Configuration

### Claude Code

Add to `~/.claude/mcp.json` or project-level `.mcp.json`:

```json
{
  "mcpServers": {
    "goudengine": {
      "command": "/absolute/path/to/target/release/goudengine-mcp",
      "args": []
    }
  }
}
```

### Cursor

Add to `.cursor/mcp.json`:

```json
{
  "mcpServers": {
    "goudengine": {
      "command": "/absolute/path/to/target/release/goudengine-mcp",
      "args": []
    }
  }
}
```

### VS Code

Add to `.vscode/mcp.json`:

```json
{
  "servers": {
    "goudengine": {
      "type": "stdio",
      "command": "/absolute/path/to/target/release/goudengine-mcp"
    }
  }
}
```

### Generic stdio client

The server reads JSON-RPC messages from stdin and writes responses to stdout. Any MCP-compatible client that supports stdio transport can use it directly.

## Game-Side Requirements

The game must enable debugger mode and local attach publishing before startup:

```rust
// Rust
let config = ContextConfig {
    debugger: Some(DebuggerConfig {
        publish_local_attach: true,
        ..Default::default()
    }),
    ..Default::default()
};
```

Equivalent options exist for C#, Python, and TypeScript desktop SDKs. Browser/WASM targets are not supported for debugger attach.

## Tools

### Discovery and Attach

#### `goudengine.list_contexts`

Discover debugger contexts published by local GoudEngine processes.

- **Parameters**: none
- **Returns**: array of discovered contexts, plus the currently attached context if any

#### `goudengine.attach_context`

Attach the MCP bridge to one route-scoped debugger context.

- **Parameters**:
  - `contextId` (u64, required) -- numeric context identifier from `list_contexts`
  - `processNonce` (u64, optional) -- process nonce for disambiguation when multiple processes share a PID
- **Returns**: context details and session info

### Snapshot and Inspection

#### `goudengine.get_snapshot`

Fetch the current debugger snapshot for the attached route.

- **Parameters**: none
- **Returns**: entities, selection state, frame number, and route metadata

#### `goudengine.inspect_entity`

Select one entity, fetch its expanded snapshot entry with all components, then restore the previous selection.

- **Parameters**:
  - `entityId` (u64, required) -- the entity to inspect
- **Returns**: entity detail with full component data, plus the snapshot

#### `goudengine.get_metrics_trace`

Export the versioned debugger metrics trace.

- **Parameters**: none
- **Returns**: profiler samples, render stats, memory counters, and a `resource_uri` for artifact retrieval

#### `goudengine.capture_frame`

Capture the current framebuffer as PNG plus debugger metadata attachments.

- **Parameters**: none
- **Returns**: capture metadata including `artifact_id` and `resource_uri`. The full capture (image, snapshot, metrics) is available via the resource URI.

### Control Plane

#### `goudengine.set_paused`

Pause or resume the game loop.

- **Parameters**:
  - `paused` (bool, required) -- `true` to pause, `false` to resume

#### `goudengine.set_time_scale`

Set the debugger-owned time scale on the attached route.

- **Parameters**:
  - `scale` (f32, required) -- multiplier for game time. 1.0 is normal speed, 0.5 is half speed, 2.0 is double speed.

#### `goudengine.step`

Advance by N frames or ticks while the game is paused.

- **Parameters**:
  - `kind` (string, required) -- `"Frame"` or `"Tick"`
  - `count` (u32, required) -- number of frames or ticks to advance

#### `goudengine.inject_input`

Inject synthetic input events into the attached route.

- **Parameters**:
  - `events` (array, required) -- each event object contains:
    - `device` (string, required) -- e.g., `"keyboard"`, `"mouse"`, `"gamepad"`
    - `action` (string, required) -- e.g., `"press"`, `"release"`, `"move"`
    - `key` (string, optional) -- key name for keyboard events
    - `button` (string, optional) -- button name for mouse/gamepad events
    - `position` (array of 2 floats, optional) -- cursor position
    - `delta` (array of 2 floats, optional) -- movement delta

### Recording and Replay

#### `goudengine.start_recording`

Start normalized input recording on the attached route.

- **Parameters**: none
- **Returns**: confirmation that recording has started

#### `goudengine.stop_recording`

Stop recording and export the replay artifact.

- **Parameters**: none
- **Returns**: `artifact_id` and `resource_uri` for the stored recording

#### `goudengine.start_replay`

Start replay from a stored recording. Exactly one of the three source parameters must be provided.

- **Parameters**:
  - `artifactId` (string, optional) -- artifact ID from a previous `stop_recording`
  - `resourceUri` (string, optional) -- resource URI of a stored recording
  - `dataBase64` (string, optional) -- raw recording data as base64

#### `goudengine.stop_replay`

Stop an active replay on the attached route.

- **Parameters**: none

### Diagnostics

#### `goudengine.get_diagnostics`

Return the full provider diagnostics map from the current snapshot.

- **Parameters**: none
- **Returns**: object with keys for each registered subsystem (e.g., `render`, `physics_2d`, `audio`, `input`, `sprite_batch`, `assets`, `window`)

#### `goudengine.get_subsystem_diagnostics`

Return diagnostics for a single subsystem.

- **Parameters**:
  - `key` (string, required) -- subsystem identifier. Known keys: `render`, `physics_2d`, `audio`, `input`, `sprite_batch`, `assets`, `window`
- **Returns**: diagnostics object for the requested subsystem

#### `goudengine.get_logs`

Return recent engine log entries.

- **Parameters**:
  - `sinceFrame` (u64, optional) -- when provided, only entries from this frame onward are returned
- **Returns**: object with an `entries` array of log records

#### `goudengine.get_scene_hierarchy`

Return entities with parent/child relationships for the attached route.

- **Parameters**: none
- **Returns**: hierarchical entity tree

## Resources

### Static Knowledge Resources

| URI | Description |
|-----|-------------|
| `goudengine://knowledge/sdk-contract` | Debugger runtime contract and SDK scope notes |
| `goudengine://knowledge/mcp-workflow` | Bridge-first workflow documentation |
| `goudengine://knowledge/sdk-rust` | Rust SDK debugger enablement guidance |
| `goudengine://knowledge/sdk-csharp` | C# SDK debugger enablement guidance |
| `goudengine://knowledge/sdk-python` | Python SDK debugger enablement guidance |
| `goudengine://knowledge/sdk-typescript-desktop` | TypeScript desktop debugger guidance |

### Artifact Resource Templates

| URI Template | Description |
|--------------|-------------|
| `goudengine://capture/{id}` | Frame capture artifact (PNG image + metadata JSON + snapshot JSON + metrics JSON) |
| `goudengine://metrics/{id}` | Metrics trace export (JSON) |
| `goudengine://recording/{id}` | Replay recording artifact (manifest JSON + binary data) |

## Prompts

| Name | Description |
|------|-------------|
| `goudengine.safe_attach` | Guided attach workflow -- reads knowledge resources first, then discovers, attaches, and takes initial snapshot |
| `goudengine.inspect_runtime` | Inspection workflow using snapshots, metrics, capture, and replay |
| `goudengine.troubleshoot_attach` | Checklist for diagnosing why discovery or attach is failing |

## E2E Testing

An end-to-end test script exercises the full pipeline: build, start a sandbox game with debugger enabled, discover the manifest, start the MCP server as a coprocess, and run JSON-RPC tool calls.

### Running the E2E test

```bash
./scripts/test-mcp-e2e.sh rust
```

### Requirements

- A display is required (the sandbox creates a GL context). The test cannot run in headless CI.
- Both `sandbox` and `goudengine-mcp` must build successfully.

### Environment variables

| Variable | Default | Description |
|----------|---------|-------------|
| `GOUD_MCP_E2E_TIMEOUT` | `15` | Seconds to wait for the runtime manifest to appear |
| `GOUD_MCP_E2E_GAME_SEC` | `20` | How long the sandbox game runs before auto-exit |

### What the test covers

1. Sandbox publishes a runtime manifest with the expected route label, capabilities, and endpoint.
2. MCP server initializes via JSON-RPC handshake.
3. `attach_context` succeeds against the live sandbox.
4. `get_diagnostics` returns an object-shaped response.
5. `get_subsystem_diagnostics` works for both valid and unknown keys.
6. `get_logs` returns an `entries` array.
7. `get_scene_hierarchy` returns a valid response.

## Troubleshooting

### No contexts found

- Verify the game is running with debugger mode enabled and `publish_local_attach = true`.
- Check the runtime manifest directory:
  - macOS: `$TMPDIR/goudengine/runtime-*.json`
  - Linux: `$XDG_RUNTIME_DIR/goudengine/runtime-*.json` or `/tmp/goudengine/runtime-*.json`
- Delete stale manifest files from previous runs and restart the game.

### Attach fails with "route_not_found"

- The game process may have exited. Check that the PID from the manifest is still running.
- The `processNonce` may be stale. Re-run `list_contexts` to refresh.

### Attach fails with "attach_disabled"

- The route exists but is not marked as attachable. Update the game config to enable attach.

### Tool call returns "no context is attached"

- Call `attach_context` before any other tool (except `list_contexts`).
- If the game exited, the bridge auto-detaches. Re-attach after the game restarts.

### Socket path too long (macOS)

- Unix domain socket paths are limited to 108 characters. If `TMPDIR` is deeply nested, the socket path may exceed this limit. Set a shorter `TMPDIR` or use `XDG_RUNTIME_DIR`.
