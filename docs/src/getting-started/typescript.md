# Getting Started — TypeScript SDK

> **Alpha** — This SDK is under active development. APIs change frequently.
> [Report issues](https://github.com/aram-devdocs/GoudEngine/issues)

The TypeScript SDK ships a single npm package (`goudengine`) with two backends:

- **Node.js** — native addon via napi-rs, uses GLFW + OpenGL, near-native performance
- **Web** — WASM module via wasm-bindgen, runs in any browser with WebAssembly support

Both backends expose the same TypeScript API. Game logic written for one target works on the other.

Other getting-started guides: [C#](csharp.md) · [Python](python.md) · [Rust](rust.md) · [Go](go.md) · [Kotlin](kotlin.md) · [Lua](lua.md)

---

## Prerequisites

- Node.js 16 or later
- npm 7 or later
- For web: a browser with WebAssembly support (all modern browsers qualify)

---

## Installation

```bash
npm install goudengine
```

The package uses conditional exports. Node.js projects get the napi-rs backend automatically.
For the browser, import from the `goudengine/web` sub-path (see the web section below).

---

## First Project: Desktop (Node.js)

Create `game.ts`:

{{#include ../generated/snippets/typescript/first-project-desktop.md}}

Run it:

```bash
npx tsx game.ts
```

This opens an 800x600 GLFW window. The loop calls `beginFrame` to clear the screen,
runs your logic, then calls `endFrame` to swap buffers. `deltaTime` returns seconds
elapsed since the last frame.

---

## First Project: Web (WASM)

Create `index.html`:

{{#include ../generated/snippets/typescript/first-project-web.md}}

Serve the directory (the importmap requires a real HTTP server, not `file://`):

```bash
npx serve .
```

Key differences from the Node.js version:

| | Node.js | Web |
|---|---|---|
| Constructor | `new GoudGame({...})` | `await GoudGame.create({...})` |
| Extra parameters | — | `canvas`, `wasmUrl` |
| Game loop | `while (!game.shouldClose())` | `game.run((dt) => { ... })` |
| Clear color | `beginFrame(r, g, b, a)` | `setClearColor(r, g, b, a)` |

Networking note:

- Desktop supports the full current wrapper path.
- Web supports browser WebSocket client connections only.
- On the web target, use `NetworkProtocol.WebSocket` and wait until `peerCount() > 0` before sending your first packet.
- The debugger runtime is desktop-only in this batch. `goudengine/web` does not support debugger attach, snapshot, or manifest APIs.

---

## Debugger Runtime (Desktop Only)

For this rollout, the shipped TypeScript desktop reference path is the Feature
Lab example:

```bash
./dev.sh --sdk typescript --game feature_lab
```

That example publishes `feature-lab-typescript-desktop`, confirms manifest and
snapshot access, and prints the manual attach steps:

1. start `cargo run -p goudengine-mcp`
2. call `goudengine.list_contexts`
3. call `goudengine.attach_context`

The web entry `feature_lab_web` stays outside the debugger rollout in this
batch.

---

## Drawing a Sprite

`loadTexture` is asynchronous on both targets — it returns a `Promise<number>`.
The returned number is a texture handle you pass to `drawSprite`.

{{#include ../generated/snippets/typescript/drawing-a-sprite.md}}

The `x` and `y` coordinates are the center of the sprite. All numeric parameters
are `f64` at the JavaScript boundary; the engine converts to `f32` internally.

**Node.js example:**

```typescript
import { GoudGame } from 'goudengine';

const game = new GoudGame({ width: 800, height: 600, title: 'Sprite Demo' });

const playerTex = await game.loadTexture('assets/player.png');

while (!game.shouldClose()) {
    game.beginFrame(0.1, 0.1, 0.1, 1.0);
    game.drawSprite(playerTex, 400, 300, 64, 64);
    game.endFrame();
}

game.destroy();
```

**Web example** — same logic, different setup:

```javascript
const game = await GoudGame.create({
    width: 800, height: 600, title: 'Sprite Demo',
    canvas, wasmUrl: '/node_modules/goudengine/wasm/goud_engine_bg.wasm'
});

const playerTex = await game.loadTexture('assets/player.png');

game.setClearColor(0.1, 0.1, 0.1, 1.0);
game.run((_dt) => {
    game.drawSprite(playerTex, 400, 300, 64, 64);
});
```

---

## Handling Input

Use `isKeyPressed` with GLFW key codes. Common codes:

| Key | Code |
|---|---|
| Escape | 256 |
| Space | 32 |
| R | 82 |
| Arrow Left | 263 |
| Arrow Right | 262 |
| Arrow Up | 265 |
| Arrow Down | 264 |

For mouse input, use `isMouseButtonPressed`. Button `0` is the left mouse button.

{{#include ../generated/snippets/typescript/handling-input.md}}

On the web target, `isKeyPressed` and `isMouseButtonPressed` work the same way.
The WASM backend handles browser keyboard and mouse events internally.

---

## Running an Example Game

Clone the repository and run the Flappy Bird example:

```bash
git clone https://github.com/aram-devdocs/GoudEngine.git
cd GoudEngine

# Desktop (Node.js)
./dev.sh --sdk typescript --game flappy_bird

# Web (browser, serves on localhost)
./dev.sh --sdk typescript --game flappy_bird_web

# Sandbox parity app (desktop + web)
./dev.sh --sdk typescript --game sandbox
./dev.sh --sdk typescript --game sandbox_web

# Supplemental smoke coverage
./dev.sh --sdk typescript --game feature_lab
./dev.sh --sdk typescript --game feature_lab_web
```

`dev.sh` handles building the SDK and running the example in one step.

To run the example manually:

```bash
# Build the native addon first
cd sdks/typescript && npm run build:native && cd ../..

# Desktop
cd examples/typescript/flappy_bird
npm install
npm run desktop

# Web
npm run build:web   # Compile TS to dist/
npm run web         # Start HTTP server on port 8765
# Open http://localhost:8765/examples/typescript/flappy_bird/web/index.html
```

Controls: `Space` or left-click to flap, `R` to restart, `Escape` to quit (desktop only).

---

## Next Steps

- [TypeScript SDK README](https://github.com/aram-devdocs/GoudEngine/tree/main/sdks/typescript/) — full API reference and build instructions
- [TypeScript examples](https://github.com/aram-devdocs/GoudEngine/tree/main/examples/typescript/) — Flappy Bird source with shared desktop/web logic
- [Build Your First Game](../guides/build-your-first-game.md) — end-to-end minimal game walkthrough
- [Example Showcase](../guides/showcase.md) — current cross-language parity matrix
- [Cross-Platform Deployment](../guides/deployment.md) — packaging and release workflow
- [Web Platform Gotchas](../guides/web-platform-gotchas.md) — browser-specific limitations and workarounds
- [FAQ and Troubleshooting](../guides/faq.md) — common runtime and build issues
- [Architecture overview](../architecture/sdk-first.md) — layer design and codegen pipeline
- [Development guide](../development/guide.md) — build system, git hooks, version management
