---
name: wasm-web
description: Build and debug the WASM browser target and the TypeScript web SDK, including the wasm-pack path and the debugger WebSocket relay
user-invocable: true
---

# WASM / Web Target

The browser build runs the engine as WebAssembly. `goud_engine/src/wasm/` is a Layer 5
(FFI) module — the browser boundary, parallel to the C FFI and JNI bridges — exposed to
JavaScript via `wasm-bindgen`. It backs the web half of the TypeScript SDK
(`sdks/typescript/`).

## When to Use

Read this when you touch `goud_engine/src/wasm/`, the TypeScript web SDK, or need to run
or debug a game in the browser. For the wider FFI/WASM patterns see
`.agents/rules/wasm-web.md`.

## Target and Boundary

- Target is `wasm32-unknown-unknown` only. Code under `goud_engine/src/wasm/` MUST compile
  without native-only deps; guard native code behind `cfg`.
- Rendering uses the wgpu backend attached to an HTML canvas — never the legacy OpenGL
  path. The WASM sprite batcher is separate from the native renderer.
- `WasmGame` is the JS handle: `WasmGame::new(w, h, title)` (headless ECS) or
  `WasmGame::create_with_canvas(canvas, w, h, title)` (rendering). Input is push-based:
  JS calls `press_key()` / `release_key()`.

## TypeScript Web SDK

- The SDK has two targets: Node.js via napi-rs (`sdks/typescript/src/node/`) and browser
  via WASM (`sdks/typescript/src/web/`). It is a thin wrapper — no game logic in TS.
- The web wrapper is generated: `codegen/gen_ts_web.py` emits
  `sdks/typescript/src/generated/web/`. Do not hand-edit `.g.ts` output.
- The WASM path uses `f32` directly (the Node napi path bridges JS `f64` ↔ engine `f32`).

## Build Path

`wasm-pack` is required (see `docs/src/development/dev-setup.md`); it is optional for
non-web work. From `sdks/typescript/`:

- **`npm run build:wasm`** →
  `wasm-pack build ../../goud_engine --target web --out-dir ../sdks/typescript/wasm --features web --no-default-features`.
  Output lands in `sdks/typescript/wasm/` (`goud_engine_bg.wasm`, `goud_engine.js`, typings).
- **`npm run build:web`** → `build:wasm` then `build:ts:web` (compiles the web TS with
  `tsconfig.web.json`).

## Steps (run a web example)

1. `./dev.sh --sdk typescript --game flappy_bird_web` — regenerates the web SDK, runs
   `npm run build:web`, then serves the repo root over `http://localhost:8765` (the port
   falls back if occupied). Other web games: `feature_lab_web`, `sandbox_web`.
2. `dev.sh` skips the wasm rebuild when `sdks/typescript/wasm/goud_engine_bg.wasm` is
   fresher than the TypeScript sources, so repeated runs are fast.

## Debugger Attach (browser)

Native debugger attach uses a local IPC socket; the browser cannot, so the WASM debugger
attaches over a **WebSocket relay** hosted by the `goudengine-mcp` server.

1. The relay listens on port `9229` by default (`ws_relay::DEFAULT_WS_PORT`, override with
   `GOUDENGINE_WS_PORT`); the MCP server spawns it at startup
   (`tools/goudengine-mcp/src/main.rs`, relay in `tools/goudengine-mcp/src/ws_relay.rs`).
2. In the browser game call `initDebugger(routeLabel)` once after constructing `WasmGame`.
   It registers the context with `publish_local_attach: false` and connects to the relay.
3. The relay forwards IPC verbs; `dispatchDebuggerRequest(json)` handles each and returns
   a JSON response. From there the standard MCP tools work — see the `goudengine-debugging`
   and `goudengine-mcp-server` skills.

## Verification

- After any FFI/WASM export change, rerun `./codegen.sh` and the SDK smoke tests, and keep
  parity with the other SDKs.
- Confirm the web example loads and renders in the browser tab that `dev.sh` prints.
- Never commit the auto-built napi loaders (`index.js` / `index.d.ts`); they are gitignored.
