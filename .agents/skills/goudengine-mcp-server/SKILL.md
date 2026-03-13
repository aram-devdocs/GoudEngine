---
name: goudengine-mcp-server
description: Setup, tool reference, and troubleshooting for the GoudEngine MCP debugger server
user-invocable: true
---

# GoudEngine MCP Server

The `goudengine-mcp` server is a Model Context Protocol bridge that translates MCP tool calls into local IPC requests against a running GoudEngine debugger runtime. It runs as a stdio-based MCP server and stays out of the game process entirely.

## When to Use

Use when setting up the MCP server for IDE integration, when referencing tool signatures, or when diagnosing connection issues between the MCP bridge and a running game.

## Setup

### Build

```bash
cargo build -p goudengine-mcp --release
```

The binary is produced at `target/release/goudengine-mcp`.

### Game-Side Configuration

The game must enable debugger mode before startup. Example in Rust:

```rust
let config = ContextConfig {
    debugger: Some(DebuggerConfig {
        publish_local_attach: true,
        ..Default::default()
    }),
    ..Default::default()
};
```

Equivalent configuration exists for C#, Python, and TypeScript desktop SDKs. Set `publish_local_attach` / `publishLocalAttach` to `true` so the route appears in `list_contexts`.

### IDE Configuration

**Claude Code** (`~/.claude/mcp.json` or project `.mcp.json`):

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

**Cursor** (`.cursor/mcp.json`):

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

**VS Code** (`.vscode/mcp.json`):

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

## Tool Reference

### Discovery and Attach

| # | Tool | Parameters | Description |
|---|------|-----------|-------------|
| 1 | `goudengine.list_contexts` | none | Discover debugger contexts published by local GoudEngine processes. Returns context list and currently attached context if any. |
| 2 | `goudengine.attach_context` | `contextId` (u64), `processNonce` (u64, optional) | Attach the bridge to one route-scoped debugger context. Required before using any other tool. |

### Snapshot and Inspection

| # | Tool | Parameters | Description |
|---|------|-----------|-------------|
| 3 | `goudengine.get_snapshot` | none | Fetch the current debugger snapshot for the attached route. Includes entities, selection state, and frame number. |
| 4 | `goudengine.inspect_entity` | `entityId` (u64) | Select one entity, fetch its expanded snapshot entry with all components, then restore the previous selection. |
| 5 | `goudengine.get_metrics_trace` | none | Export the versioned debugger metrics trace. Returns profiler samples, render stats, and memory counters. Includes a `resource_uri` for later retrieval. |
| 6 | `goudengine.capture_frame` | none | Capture the current framebuffer as PNG plus debugger metadata attachments (snapshot, metrics). Includes a `resource_uri`. |

### Control Plane

| # | Tool | Parameters | Description |
|---|------|-----------|-------------|
| 7 | `goudengine.set_paused` | `paused` (bool) | Pause or resume the game loop on the attached route. |
| 8 | `goudengine.set_time_scale` | `scale` (f32) | Set the debugger-owned time scale. Values less than 1.0 slow down; greater than 1.0 speed up. |
| 9 | `goudengine.step` | `kind` ("Frame" or "Tick"), `count` (u32) | Advance by N frames or ticks while the game is paused. |
| 10 | `goudengine.inject_input` | `events` (array of input event objects) | Inject synthetic input events. Each event has `device`, `action`, and optional `key`, `button`, `position`, `delta`. |

### Recording and Replay

| # | Tool | Parameters | Description |
|---|------|-----------|-------------|
| 11 | `goudengine.start_recording` | none | Start normalized input recording on the attached route. |
| 12 | `goudengine.stop_recording` | none | Stop recording and export the replay artifact. Returns an `artifact_id` and `resource_uri`. |
| 13 | `goudengine.start_replay` | `artifactId` (string, optional), `resourceUri` (string, optional), `dataBase64` (string, optional) | Start replay from a stored recording artifact, a resource URI, or raw base64 data. Exactly one source must be provided. |
| 14 | `goudengine.stop_replay` | none | Stop an active replay on the attached route. |

### Diagnostics (Phase 6)

| # | Tool | Parameters | Description |
|---|------|-----------|-------------|
| 15 | `goudengine.get_diagnostics` | none | Return the full provider diagnostics map from the current snapshot. Covers all registered subsystems. |
| 16 | `goudengine.get_subsystem_diagnostics` | `key` (string) | Return diagnostics for a single subsystem. Known keys: `render`, `physics_2d`, `audio`, `input`, `sprite_batch`, `assets`, `window`. |
| 17 | `goudengine.get_logs` | `sinceFrame` (u64, optional) | Return recent engine log entries. When `sinceFrame` is provided, only entries from that frame onward are returned. |
| 18 | `goudengine.get_scene_hierarchy` | none | Return entities with parent/child relationships for the attached route. |

### Resources

The server also exposes MCP resources:

| Resource | Description |
|----------|-------------|
| `goudengine://knowledge/sdk-contract` | Debugger runtime contract and SDK scope notes |
| `goudengine://knowledge/mcp-workflow` | Bridge-first workflow for discovery, attach, and artifact reads |
| `goudengine://knowledge/sdk-rust` | Rust SDK debugger enablement guidance |
| `goudengine://knowledge/sdk-csharp` | C# SDK debugger enablement guidance |
| `goudengine://knowledge/sdk-python` | Python SDK debugger enablement guidance |
| `goudengine://knowledge/sdk-typescript-desktop` | TypeScript desktop debugger guidance |
| `goudengine://capture/{id}` | Stored frame capture artifact (image + metadata + snapshot + metrics) |
| `goudengine://metrics/{id}` | Stored metrics trace export |
| `goudengine://recording/{id}` | Stored replay recording artifact (manifest + binary data) |

### Prompts

| Prompt | Description |
|--------|-------------|
| `goudengine.safe_attach` | Guided attach workflow that reads knowledge resources first |
| `goudengine.inspect_runtime` | Inspection workflow using snapshots, metrics, capture, and replay |
| `goudengine.troubleshoot_attach` | Checklist for diagnosing attach failures |

## Troubleshooting

### No contexts found in `list_contexts`

- Verify the game is running with debugger mode enabled.
- Confirm `publish_local_attach` is set to `true` in the game config.
- Check the runtime manifest directory. On macOS with `TMPDIR` set, manifests appear at `$TMPDIR/goudengine/runtime-*.json`. On Linux, check `$XDG_RUNTIME_DIR/goudengine/` or `/tmp/goudengine/`.
- Stale manifest files from previous runs may confuse discovery. Delete old `runtime-*.json` files and restart the game.

### Attach fails with "route_not_found"

- The game process may have exited. Verify the PID from the manifest is still running.
- The `processNonce` may not match if the game restarted. Re-run `list_contexts` to get the current nonce.
- Confirm the route label matches what the game registers (e.g., `sandbox-rust`).

### Attach fails with "attach_disabled"

- The route exists but `attachable` is `false` in the manifest. The game config must set the route as attachable.

### Tool call returns "no context is attached"

- You must call `attach_context` before any other tool (except `list_contexts`).
- If the game process died after attach, the bridge auto-detaches. Re-run `list_contexts` and `attach_context`.

### MCP server does not start

- Verify the binary exists: `ls target/release/goudengine-mcp`.
- Rebuild if needed: `cargo build -p goudengine-mcp --release`.
- The server communicates over stdio. It will not produce visible output when started directly; it waits for JSON-RPC input on stdin.

### Timeout reading MCP responses

- The game may be under heavy load and slow to respond to IPC requests.
- Check if the IPC socket exists at the endpoint location from the manifest.
- On macOS, verify the socket path is not too long (108-character limit for Unix domain sockets).
