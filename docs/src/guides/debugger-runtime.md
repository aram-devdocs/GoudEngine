# Debugger Runtime

The shared debugger runtime is for local desktop and headless development flows.

What is in scope:

- Native Rust, C#, Python, and TypeScript Node targets
- Route-scoped attach over local IPC
- Pause, resume, frame stepping, tick stepping, time scale, debug-draw toggle, and input injection
- Frame capture, replay recording, replay playback, and metrics trace export
- The out-of-process `goudengine-mcp` bridge

What is out of scope in this batch:

- Remote attach
- Browser or WASM debugger runtime support
- Engine-wide determinism claims

`goudengine/web` exposes explicit unsupported errors for the new debugger runtime methods.

## Enable It Before Startup

Enable debugger mode in config before creating the game or context. The runtime publishes one local manifest per process when `publish_local_attach` is enabled and at least one route is attachable.

Shipped config surfaces:

- C# headless: `new GoudContext(new ContextConfig(new DebuggerConfig(true, true, "...")))`
- Python headless: `GoudContext(ContextConfig(debugger=DebuggerConfig(...)))`
- Rust headless: `Context::create_with_config(ContextConfig { debugger: ... })`
- TypeScript desktop: use `./dev.sh --sdk typescript --game feature_lab` as the shipped reference route in this batch

The runtime is Rust-owned. SDKs only forward control calls and return raw JSON or byte envelopes.

## Control and Debug Draw

The public control plane uses the same route-scoped contract across FFI, SDKs, and MCP:

- pause or resume
- step by frame or tick
- set time scale
- toggle runtime-owned debug draw
- inject keyboard, mouse button, mouse position, and scroll events

Debug draw is also runtime-owned. The enabled path supports 2D and 3D primitives, color, layering, and optional lifetime. When debugger mode is off, the runtime keeps the disabled path close to zero cost.

## Capture Artifacts

`captureDebuggerFrame()` returns a `DebuggerCapture` envelope:

- `imagePng`
- `metadataJson`
- `snapshotJson`
- `metricsTraceJson`

Capture is on-demand. The renderer readback stays inside the graphics backend, and unsupported routes report a clean failure instead of a partial artifact.

## Replay Artifacts and Determinism Limits

`stopDebuggerRecording()` returns a `DebuggerReplayArtifact` envelope:

- `manifestJson`
- `data`

Replay records normalized input, timing, and available runtime facts, then feeds playback through the same debugger control path.

Replay does not promise full engine determinism. Expect differences when behavior depends on:

- wall-clock time
- external network traffic or services
- platform or driver behavior
- providers that do not expose every source of nondeterminism

Treat replay as a debugger aid for desktop and headless development, not as a strict simulation proof.

## Metrics and Trace Export

`getDebuggerMetricsTraceJson()` exports versioned JSON from a bounded runtime buffer. The trace includes:

- frame timing
- per-system timing
- render counters
- memory summaries
- network and service-health state
- debugger events

The runtime keeps 300 frames of history per route so export cost stays predictable.

## MCP Bridge Workflow

`goudengine-mcp` is a separate local process. It reads manifests, attaches to one route over local IPC, and exposes MCP tools, prompt bundles, SDK knowledge resources, and artifact resources.

Start it from the workspace root:

```bash
cargo run -p goudengine-mcp
```

## Feature Lab Reference Routes

The owned Feature Lab examples are the reference rollout for local attach smoke
coverage. Each one enables debugger mode and prints the same attach workflow
locally. The C#, Python, and TypeScript desktop paths also exercise raw
manifest/snapshot access through the SDK helpers they already expose.

| SDK | Target | Run | Stable route label |
|---|---|---|---|
| C# | Headless | `./dev.sh --game feature_lab` | `feature-lab-csharp-headless` |
| Python | Headless | `python3 examples/python/feature_lab.py` | `feature-lab-python-headless` |
| Rust | Headless | `cargo run -p feature-lab` | `feature-lab-rust-headless` |
| TypeScript | Desktop | `./dev.sh --sdk typescript --game feature_lab` | `feature-lab-typescript-desktop` |

`feature_lab_web` stays in the repo for browser/WASM smoke coverage only. It
does not publish a debugger route.

Typical workflow:

1. Call `goudengine.list_contexts`.
2. Call `goudengine.attach_context`.
3. Use snapshot, control, capture, replay, and metrics tools against that route.
4. Use prompt bundles such as:
   - `goudengine.safe_attach`
   - `goudengine.inspect_runtime`
   - `goudengine.troubleshoot_attach`
5. Read knowledge resources such as:
   - `goudengine://knowledge/sdk-contract`
   - `goudengine://knowledge/mcp-workflow`
   - `goudengine://knowledge/sdk-rust`
   - `goudengine://knowledge/sdk-csharp`
   - `goudengine://knowledge/sdk-python`
   - `goudengine://knowledge/sdk-typescript-desktop`
6. Read stored artifacts through:
   - `goudengine://capture/{id}`
   - `goudengine://metrics/{id}`
   - `goudengine://recording/{id}`

The bridge does not run inside the game process and does not switch routes globally. Each attach session is bound to one route.
