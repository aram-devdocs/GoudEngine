---
rfc: "0004"
title: "Debugger Runtime, Snapshot Contract, and Local Attach Model"
status: proposed
created: 2026-03-12
authors: ["aram-devdocs"]
tracking-issue: "#511"
related-issues: ["#513", "#517", "#520"]
---
# RFC-0004: Debugger Runtime, Snapshot Contract, and Local Attach Model

## 1. Summary
Phase 2.5.1 needs a fixed contract for GoudEngine's debugger stack. This RFC sets the runtime topology to one Rust-owned debugger service per process in dev mode, one out-of-process `goudengine-mcp` bridge that speaks MCP over stdio, one shared snapshot and service-health schema, one local-only attach model, and one debugger enablement contract that spans `GameConfig`, `EngineConfig`, and a future config-based `GoudContext` path. The batch only sets the contract. It does not implement the debugger runtime, FFI, SDK rollout, capture, replay, or the MCP bridge.

## 2. Motivation

Phase 2.5 pulls debugger, profiling, replay, observability, and AI-agent runtime tooling into Alpha v1. The later implementation batches need a fixed contract before code lands. Without that contract, the runtime service, FFI layer, SDK wrappers, overlays, Feature Lab, and `goudengine-mcp` bridge can drift into incompatible shapes.

The current codebase already exposes several narrow debug surfaces:
- `goud_engine/src/sdk/game_config.rs` carries `show_fps_overlay`, `physics_debug`, and `diagnostic_mode`.
- `goud_engine/src/sdk/engine_config.rs` forwards part of that config model for windowed game creation.
- `goud_engine/src/sdk/context.rs` exposes a bare `GoudContext` lifecycle API with no config object.
- `goud_engine/src/ffi/debug.rs` and `goud_engine/src/ffi/window/state.rs` expose point solutions for FPS and diagnostic state.

Those pieces are useful, but they do not define:
- one process-wide debugger runtime,
- one route identity for multi-context processes,
- one semantic snapshot schema for agents and overlays,
- one local attach protocol,
- or one debugger-mode contract across init surfaces.

The document resolves four blocking issues:
- `#511` debugger singleton architecture and MCP contract,
- `#513` snapshot schema and service-health model,
- `#517` local transport, attach protocol, and local-only security policy,
- `#520` shared debugger enablement contract.

## 3. Design

### 3.1 Scope and Non-Goals

Scope:
- Process model for the debugger runtime and the `goudengine-mcp` bridge.
- Shared type contract for runtime identity, local discovery, attach, snapshots, and service health.
- Enablement contract for `GameConfig`, `EngineConfig`, and a future config-based `GoudContext` flow.
- Approval gate that later Phase 2.5 batches must follow.

Non-goals:
- Implementing the debugger runtime service (`#512`).
- Implementing the MCP bridge (`#518`).
- Adding FFI exports, codegen schema, or SDK wrappers (`#230`, `#521`-`#524`).
- Embedding an MCP stdio server inside game processes.
- Making TypeScript web or browser remote attach a Phase 2.5 gate.
- Defining a remote network protocol. This phase is local-only.

### 3.2 Runtime Topology and Ownership

The debugger stack has exactly two runtime participants in this phase:

| Component | Process | Owner | Responsibility |
|---|---|---|---|
| `DebuggerRuntime` | Game process | Rust engine | Produces snapshots, service health, stats, control hooks, capture/replay hooks, and route registration |
| `goudengine-mcp` | Separate local process | Local developer tool | Speaks MCP over stdio and forwards requests over local attach transport |

Topology rules:
- One `DebuggerRuntime` exists per process when debugger mode is enabled in a supported dev-mode flow.
- The runtime is Rust-owned and lives in Layer 2. SDKs do not create parallel debugger services.
- The runtime registers one or more debuggable routes for the contexts in that process.
- `goudengine-mcp` is thin. It translates MCP requests into debugger attach, snapshot, inspection, control, capture, and replay requests. It does not own engine state.
- The game process does not implement MCP directly and never opens a stdio MCP endpoint for agents.

Conceptual flow: `agent <-> stdio <-> goudengine-mcp <-> local socket/pipe <-> DebuggerRuntime`, with overlays, FFI exports, SDK convenience APIs, and Feature Lab all consuming the same runtime contract.

### 3.3 Route Identity and Lifecycle

Each debuggable context in a process gets a stable route identifier for the lifetime of that process:

```rust
pub struct RuntimeRouteId {
    pub process_nonce: u64,
    pub context_id: u64,
    pub surface_kind: RuntimeSurfaceKind,
}

pub enum RuntimeSurfaceKind {
    WindowedGame,
    HeadlessContext,
    ToolContext,
}
```

Rules:
- `process_nonce` is generated once at runtime-service startup and changes on each process start.
- `context_id` maps to the existing engine context identity used by runtime code and FFI.
- `surface_kind` disambiguates windowed and headless flows that may share engine infrastructure. Wire serialization of `RuntimeSurfaceKind` must use `"windowed_game"`, `"headless_context"`, and `"tool_context"`, and parsers must reject other spellings such as `WindowedGame`.
- A route is stable until that context detaches or the process exits.
- A detached route disappears from discovery and returns an attach error if selected later.

Lifecycle:
1. Debugger mode is enabled during engine/context creation.
2. The process creates or reuses the process-wide `DebuggerRuntime`.
3. Each eligible context registers a `RuntimeRouteId` with the runtime.
4. The runtime publishes discovery metadata for the process and its routes only when `publish_local_attach = true` and at least one attachable route exists.
5. Local tools attach to one route at a time.
6. Context shutdown removes that route.
7. Process shutdown removes discovery metadata and closes any published local transport endpoint.

### 3.4 Capability Surface

The debugger runtime exposes one canonical capability surface. Later batches may implement pieces incrementally, but they must preserve these categories and names:

| Capability | Consumer examples | Producer/owner |
|---|---|---|
| Snapshots | overlays, MCP, SDK helpers | debugger runtime core |
| Profiling | overlays, MCP, exports | profiler subsystem through debugger runtime |
| Render stats | overlays, MCP | renderer adapter through debugger runtime |
| Memory stats | overlays, MCP | memory/statistics adapter through debugger runtime |
| Entity inspection | overlays, MCP, SDK helpers | scene/entity inspector through debugger runtime |
| Debug draw | overlays, SDK helpers | debug-draw/control layer |
| Control plane | overlays, MCP, SDK helpers | debugger runtime control layer |
| Replay | MCP, SDK helpers | replay subsystem through debugger runtime |
| Capture | MCP, SDK helpers | capture subsystem through debugger runtime |
| SDK knowledge | MCP resources/prompts | `goudengine-mcp` bridge only; not route-scoped |

Route wire keys are `snapshots`, `profiling`, `render_stats`, `memory_stats`, `entity_inspection`, `debug_draw`, `control_plane`, `replay`, and `capture`. `sdk_knowledge` is a bridge-only capability key and is not part of runtime-published route capability maps.

Control plane scope in this phase covers pause/resume, single-step, time-scale changes, route selection, entity selection, debug toggle state, and input injection where supported by later implementation work.

### 3.5 Runtime Service, FFI Surface, SDK APIs, and Bridge

The ownership chain is fixed:

1. `DebuggerRuntime` is the source of truth.
2. FFI exports expose runtime-owned operations and snapshots to non-Rust SDKs.
3. SDK convenience APIs wrap the FFI or Rust runtime surface. They do not invent SDK-local debugger behavior.
4. `goudengine-mcp` attaches through the same local runtime contract and does not bypass the runtime with private side channels.

Implications:
- overlays, FFI, SDK helpers, Feature Lab, and the bridge all consume the same route IDs, capability names, snapshot schema, and health model;
- future public APIs may be ergonomic, but they must not redefine the underlying contract;
- later docs and SDK guides may specialize for language ergonomics, but not for topology.

### 3.6 Shared Debugger Enablement Contract

Debugger mode is a Rust-owned pre-init concept. The canonical types are:

```rust
pub struct DebuggerConfig {
    pub enabled: bool,
    pub publish_local_attach: bool,
    pub route_label: Option<String>,
}

pub struct ContextConfig {
    pub debugger: DebuggerConfig,
}
```

Contract rules:
- `DebuggerConfig` is the canonical debugger-mode value object.
- `enabled = false` means no debugger runtime startup, no route registration, and near-zero overhead outside existing debug features.
- `publish_local_attach = true` allows the runtime to publish local discovery metadata and accept local attachments.
- `route_label` is optional display metadata for tools. It is not the stable route identity.

Init-surface mapping:

| Surface | Contract |
|---|---|
| `GameConfig` | Gains `debugger: DebuggerConfig` as the source of debugger-mode settings for windowed Rust flows |
| `EngineConfig` | Wraps the same `GameConfig.debugger` contract and exposes builder helpers without redefining it |
| `GoudContext` | Gains a future config-based constructor path such as `GoudContext::with_config(ContextConfig)` while the bare constructor remains shorthand for defaults |

Compatibility rules:
- The bare `GoudContext()` / `goud_context_create()` path remains valid and defaults to debugger disabled.
- The canonical path for debugger mode is pre-init configuration, not a post-create toggle.
- Later FFI and codegen work must expose the same Rust-owned `DebuggerConfig` / `ContextConfig` model instead of per-SDK divergence.
- TypeScript desktop follows the shared config model. TypeScript web/browser remote attach remains out of gate scope.

### 3.7 Local Discovery Contract

Discovery is manifest-based and snapshot-oriented, not streaming. Each process with debugger mode enabled and `publish_local_attach = true` publishes one manifest while at least one attachable route exists.

```rust
pub struct LocalEndpointV1 {
    pub transport: String,
    pub location: String,
}

pub struct RouteSummaryV1 {
    pub route_id: RuntimeRouteId,
    pub label: Option<String>,
    pub attachable: bool,
    pub capabilities: std::collections::BTreeMap<String, CapabilityStateV1>,
}

pub struct RuntimeManifestV1 {
    pub manifest_version: u32,
    pub pid: u32,
    pub process_nonce: u64,
    pub executable: String,
    pub endpoint: LocalEndpointV1,
    pub routes: Vec<RouteSummaryV1>,
    pub published_at_unix_ms: u64,
}
```

`LocalEndpointV1.transport` is `"unix"` on macOS/Linux and `"named_pipe"` on Windows. `location` is an absolute socket path or a full pipe name. `RouteSummaryV1.route_id.surface_kind` is the only surface discriminator. `RouteSummaryV1.capabilities` must include every route-scoped wire key from Section 3.4 and use the states from Section 3.10.1; unsupported features stay present with `disabled` or `unavailable` instead of omission. `sdk_knowledge` is bridge-only and excluded from runtime-published route maps. `RuntimeManifestV1.manifest_version` is fixed to `1` in this phase.

Manifest placement:
- macOS/Linux: publish in `$XDG_RUNTIME_DIR/goudengine/` when available, otherwise `/tmp/goudengine-<uid>/`.
- Windows: publish in `%LOCALAPPDATA%\\GoudEngine\\runtime\\`.
- If a Unix socket path would exceed platform limits, implementations must fall back to a hashed basename under a short root such as `/tmp/goudengine-<uid>/s/`, while still encoding `pid` and `process_nonce` in the manifest `location`.
Operational rules:
- One manifest per process, not per context, and no manifest when `publish_local_attach = false` or no attachable routes exist.
- The manifest lists all current routes for that process and is rewritten with a new, strictly monotonic `published_at_unix_ms = max(now_ms, last_published_at_unix_ms + 1)` on any manifest field change.
- Manifest updates must use atomic replace semantics, such as write-temp-then-rename, so tools never observe partial files.
- For the same `pid` and `process_nonce`, the highest `published_at_unix_ms` is authoritative; if multiple updates fall in the same millisecond, the runtime must increment the value to preserve strict monotonicity, and older copies are stale.
- Manifest filenames and `LocalEndpointV1.location` must include both `pid` and `process_nonce`.
- Readers must treat manifests as stale when the `pid` is no longer live.
- An endpoint-open failure requires exactly one local retry after 100 ms before the manifest is ignored for the current discovery invocation. Readers SHOULD perform that retry asynchronously, and any success from that retry may appear only on the next explicit manifest-directory scan rather than mutating results already returned. Pruning is reserved for dead-`pid` cases.
- The manifest is local developer metadata only. It is never a remote discovery protocol.

### 3.8 Local Attach Transport and Handshake

Transport is OS-local IPC: Unix domain sockets on macOS/Linux and named pipes on Windows. Out of scope for this phase: TCP listen sockets, WebSocket listeners, remote host binding, and any release gate that depends on cross-machine attach.
- Frame taxonomy: v1 allows only client requests, runtime responses, `{"type":"heartbeat"}`, and `{"type":"heartbeat_ack"}`. Unsolicited runtime->client notifications are forbidden.
- Framing: every message is one 4-byte little-endian length prefix plus one UTF-8 JSON object, with a 1 MiB maximum frame size. Lengths above the limit must return `protocol_error` and immediately close the session without draining the frame.
- Versioning: `AttachHelloV1.protocol_version` and `AttachAcceptedV1.protocol_version` are fixed to `1` in this phase.
- Heartbeat sender: `goudengine-mcp` sends `{"type":"heartbeat"}` and the runtime only replies with `{"type":"heartbeat_ack"}`; heartbeat frames are out-of-band and do not count against the one in-flight request limit.
- Heartbeat interval: if `heartbeat_interval_ms` is `0`, heartbeats are disabled. Otherwise the client sends a heartbeat after one idle interval without client->runtime traffic.
- Heartbeat timeout: a sent heartbeat is satisfied only by `{"type":"heartbeat_ack"}`; other runtime->client frames do not clear it. The client closes when a sent heartbeat is not acknowledged within exactly `2 * heartbeat_interval_ms`, measured from send time on a monotonic clock rather than wall time.
- Error handling: attach sessions allow only one in-flight request at a time, and a pipelined request before the active response completes is `protocol_error` plus immediate session close. Failures use `{"type":"error","code":"...","message":"..."}` with codes from `protocol_error`, `version_mismatch`, `route_not_found`, `route_not_attachable`, and `attach_disabled`; unknown codes are extensions and still fail the active request.
The first request/response pair uses:

```rust
pub struct AttachHelloV1 {
    pub protocol_version: u32,
    pub client_name: String,
    pub client_pid: u32,
    pub route_id: RuntimeRouteId,
}

pub struct AttachAcceptedV1 {
    pub protocol_version: u32,
    pub session_id: u64,
    pub route_id: RuntimeRouteId,
    pub snapshot_schema: String,
    pub heartbeat_interval_ms: u32,
}
```

Handshake rules:
- `route_id` is required. Multi-route processes must not rely on implicit selection.
- The runtime rejects unknown or detached routes.
- The runtime rejects protocol-version mismatches.
- The runtime may reject attach when debugger mode is disabled or local attach publication is disabled.
- Accepted sessions are local only and scoped to one selected route.
- The bridge may open multiple sessions to different routes, but each session binds to one route ID.

### 3.9 Local-Only Security Policy

Security posture for this phase:
- opt-in only through debugger-mode configuration,
- local IPC only,
- no remote bind,
- no unauthenticated network-facing debugger endpoint,
- and no browser remote attach requirement.

Trust boundaries:
- The local machine user account is the trust boundary for this phase.
- `goudengine-mcp` is trusted as a local developer tool once it reaches the local IPC endpoint.
- The manifest reveals only local debugging metadata needed to choose a route.

Required behavior:
- Release-oriented builds may ignore debugger enablement or compile out discovery publication.
- Local attach is disabled unless debugger mode is enabled.
- Manifest directories must be owner-only where the OS supports it: `0700` directories and `0600` manifest files on Unix-like systems, restrictive per-user ACLs under `%LOCALAPPDATA%` on Windows.
- Unix socket endpoints must live inside an owner-only directory; the runtime must refuse to publish a socket in a broader directory.
- Windows named pipes must grant access only to the current user, `SYSTEM`, and local Administrators; the runtime must refuse broader pipe ACLs.
- Future remote attach, auth, or sandboxing work is follow-up scope and must not be backported into this phase as an implicit gate.

### 3.10 Snapshot and Service-Health Schema

#### 3.10.1 Capability state model

All route capabilities use one state enum:

| State | Meaning |
|---|---|
| `ready` | Data or control path is available and functioning |
| `disabled` | Feature exists but is turned off for this runtime |
| `unavailable` | Feature does not exist for this platform/runtime/provider combination |
| `faulted` | Feature should exist, but the runtime detected an error or degraded state |

#### 3.10.2 Service health

`ServiceHealthV1` is the shared status shape for subsystems and debugger-owned services:

```rust
pub struct ServiceHealthV1 {
    pub name: String,
    pub state: CapabilityStateV1,
    pub owner: String,
    pub detail: Option<String>,
    pub updated_frame: u64,
}
```

`services` must include exactly one entry for each of `renderer`, `memory`, `profiling`, `physics`, `audio`, `network`, `window`, `assets`, `capture`, `replay`, and `debugger`, using `disabled`, `unavailable`, or `faulted` instead of omission.

`owner` is a closed wire-literal set: `renderer_adapter`, `memory_adapter`, `physics_adapter`, `audio_adapter`, `network_adapter`, `window_adapter`, `asset_manager`, `capture_subsystem`, `replay_subsystem`, and `debugger_runtime`. Required mappings are `renderer -> renderer_adapter`, `memory -> memory_adapter`, `physics -> physics_adapter`, `audio -> audio_adapter`, `network -> network_adapter`, `window -> window_adapter`, `assets -> asset_manager`, `capture -> capture_subsystem`, `replay -> replay_subsystem`, and `debugger -> debugger_runtime`.

#### 3.10.3 Snapshot shape

`DebuggerSnapshotV1` is the canonical semantic view of one route. `snapshot_version` is fixed to `1` in this phase:

```rust
pub struct DebuggerSnapshotV1 {
    pub snapshot_version: u32,
    pub route_id: RuntimeRouteId,
    pub frame: FrameStateV1,
    pub selection: SelectionStateV1,
    pub scene: SceneStateV1,
    pub entities: Vec<EntityStateV1>,
    pub services: Vec<ServiceHealthV1>,
    pub stats: SnapshotStatsV1,
    pub diagnostics: DiagnosticsStateV1,
    pub debugger: DebuggerStateV1,
}
```

```rust
pub struct FrameStateV1 { pub index: u64, pub delta_seconds: f32, pub total_seconds: f64 }
pub struct SelectionStateV1 { pub scene_id: String, pub entity_id: Option<u64> }
pub struct SceneStateV1 { pub active_scene: String, pub entity_count: u32 }
pub struct EntityStateV1 { pub entity_id: u64, pub name: Option<String>, pub components: std::collections::BTreeMap<String, serde_json::Value> }
pub struct RenderStatsV1 { pub draw_calls: u32 }
pub struct MemoryStatsV1 { pub tracked_bytes: u64 }
pub struct NetworkStatsV1 { pub bytes_sent: u64, pub bytes_received: u64 }
pub struct SnapshotStatsV1 { pub render: RenderStatsV1, pub memory: MemoryStatsV1, pub network: NetworkStatsV1 }
pub struct DiagnosticsStateV1 { pub errors: Vec<String>, pub last_fault: Option<String> }
pub struct DebuggerStateV1 { pub paused: bool, pub time_scale: f32, pub attached_clients: u32 }
```

All fields are required unless wrapped in `Option<T>`; `entities` and `components` maps may be empty, but required `services` and capability maps must be present, complete, and unique by name/key.
Field ownership:

| Snapshot section | Produced by |
|---|---|
| `route_id`, `debugger`, `services.debugger` | debugger runtime |
| `frame` | debugger runtime frame coordinator |
| `selection`, `scene`, `entities` | scene/entity inspector |
| `services.renderer`, `stats.render` | renderer adapter |
| `services.memory`, `stats.memory` | memory/statistics adapter |
| `services.profiling` | profiler subsystem through debugger runtime |
| `services.physics` | physics adapter |
| `services.audio` | audio adapter |
| `services.network`, `stats.network` | network adapter |
| `services.window` | window/platform adapter |
| `services.assets` | asset manager |
| `services.capture` | capture subsystem |
| `services.replay` | replay subsystem |
| `diagnostics` | error/diagnostic subsystem plus debugger runtime aggregation |
Minimum semantic coverage includes frame timing and frame index, selected scene/entity state, inspected component state for the current entity selection, provider capability and health, render/memory/network stats, replay and capture status, debugger health, and current errors or diagnostics suitable for agent consumption. When one of the modeled stats producers (`render`, `memory`, or `network`) is `disabled` or `unavailable`, its stats object remains present with zero/default values and the service state is authoritative.

#### 3.10.4 JSON example: `ServiceHealthV1`

```json
{
  "name": "renderer",
  "state": "ready",
  "owner": "renderer_adapter",
  "detail": null,
  "updated_frame": 4812
}
```

#### 3.10.5 JSON excerpt: `DebuggerSnapshotV1`
The `services` array below is abbreviated for brevity; conforming snapshots still include exactly one entry for every required service name.
```json
{
  "snapshot_version": 1,
  "route_id": {
    "process_nonce": 44199288,
    "context_id": 3,
    "surface_kind": "windowed_game"
  },
  "frame": {
    "index": 4812,
    "delta_seconds": 0.0166,
    "total_seconds": 79.84
  },
  "selection": {
    "scene_id": "default",
    "entity_id": 42
  },
  "scene": {
    "active_scene": "default",
    "entity_count": 118
  },
  "entities": [
    {
      "entity_id": 42,
      "name": "Player",
      "components": {
        "Transform2D": {
          "x": 144.0,
          "y": 96.0,
          "rotation_deg": 0.0
        },
        "Sprite": {
          "texture": "player_idle"
        }
      }
    }
  ],
  "services": [
    {
      "name": "renderer",
      "state": "ready",
      "owner": "renderer_adapter",
      "detail": null,
      "updated_frame": 4812
    },
    {
      "name": "replay",
      "state": "disabled",
      "owner": "replay_subsystem",
      "detail": "replay not active for this route",
      "updated_frame": 4812
    }
  ],
  "stats": {
    "render": {
      "draw_calls": 88
    },
    "memory": {
      "tracked_bytes": 12582912
    },
    "network": {
      "bytes_sent": 0,
      "bytes_received": 0
    }
  },
  "diagnostics": {
    "errors": [],
    "last_fault": null
  },
  "debugger": {
    "paused": false,
    "time_scale": 1.0,
    "attached_clients": 1
  }
}
```

### 3.11 Approval Gate for Later Phase 2.5 Batches

Phase `2.5.2` through `2.5.5` are approved to proceed only if they preserve this RFC's fixed decisions:

| Batch | Must preserve |
|---|---|
| `2.5.2` engine substrate | one process-wide runtime, route registration, runtime-owned capabilities |
| `2.5.3` control/protocol/replay | local-only attach model, route-scoped sessions, snapshot/service-health names |
| `2.5.4` public surface rollout | one Rust-owned enablement model and no SDK-local debugger runtimes |
| `2.5.5` DX/docs/Feature Lab | bridge-first local attach workflow and TypeScript web out-of-gate wording |

Any later work that proposes embedded MCP in the game process, remote bind as a release gate, a post-create-only debugger toggle as the canonical path, or a different snapshot-schema or route-identity model must first update or supersede this RFC.

---

## 4. Alternatives Considered

1. Embedded MCP stdio server inside each game process.
Rejected because it couples agent protocol concerns to the runtime process and creates multiple MCP hosts instead of one thin bridge.

2. Separate debugger enablement models per SDK.
Rejected because the later FFI and codegen rollout would fragment the contract and create SDK-only behavior.

3. Post-create debugger enablement as the canonical `GoudContext` path.
Rejected because `GameConfig` and `EngineConfig` are already pre-init configuration surfaces. A future config-based `ContextConfig` keeps the model aligned.

4. Remote TCP attach in this phase.
Rejected because the acceptance criteria only require local developer attach and explicitly exclude remote bind as a release gate.

5. Separate schema file in this batch.
Rejected because this batch is a contract gate, not an implementation batch. A structured appendix in the RFC is enough to unblock runtime, FFI, SDK, and MCP work without adding a second source of truth yet.

## 5. Impact

- This RFC is docs only. No engine, FFI, SDK, codegen, or example behavior changes land in this batch.
- Later implementation work must add one runtime-owned debugger path instead of extending today's point debug features independently.
- `GameConfig` and `EngineConfig` will converge on `DebuggerConfig`.
- Standalone `GoudContext` will need a future config-based constructor path, while existing bare constructors remain valid with debugger disabled by default.
- Desktop native flows are the Phase 2.5 gate. TypeScript web/browser attach remains follow-up work.

## 6. Open Questions

1. Should future attach sessions expose one multiplexed connection per process or keep one session per route even after the bridge supports route switching?
