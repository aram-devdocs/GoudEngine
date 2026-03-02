# Getting Started — TypeScript SDK

> **Alpha** — This SDK is under active development. APIs change frequently.
> [Report issues](https://github.com/aram-devdocs/GoudEngine/issues)

The TypeScript SDK ships a single npm package (`goudengine`) with two backends:

- **Node.js** — native addon via napi-rs, uses GLFW + OpenGL, near-native performance
- **Web** — WASM module via wasm-bindgen, runs in any browser with WebAssembly support

Both backends expose the same TypeScript API. Game logic written for one target works on the other.

Other getting-started guides: [C#](getting-started-csharp.md) · [Python](getting-started-python.md) · [Rust](getting-started-rust.md)

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

```typescript
import { GoudGame } from 'goudengine';

const game = new GoudGame({ width: 800, height: 600, title: 'My Game' });

while (!game.shouldClose()) {
    game.beginFrame(0.2, 0.3, 0.4, 1.0);  // RGBA clear color

    const dt = game.deltaTime;

    // Press Escape to exit
    if (game.isKeyPressed(256)) { break; }

    game.endFrame();
}

game.destroy();
```

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

```html
<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <title>My Game</title>
</head>
<body>
<canvas id="canvas" width="800" height="600"></canvas>

<script type="importmap">
{
  "imports": {
    "goudengine/web": "/node_modules/goudengine/dist/web/generated/web/index.g.js"
  }
}
</script>

<script type="module">
import { GoudGame } from 'goudengine/web';

const canvas = document.getElementById('canvas');

const game = await GoudGame.create({
    width: 800,
    height: 600,
    title: 'My Game',
    canvas,
    wasmUrl: '/node_modules/goudengine/wasm/goud_engine_bg.wasm'
});

game.setClearColor(0.2, 0.3, 0.4, 1.0);

game.run((dt) => {
    // Game logic here — dt is seconds since last frame
});
</script>
</body>
</html>
```

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

---

## Drawing a Sprite

`loadTexture` is asynchronous on both targets — it returns a `Promise<number>`.
The returned number is a texture handle you pass to `drawSprite`.

```typescript
// Load before the game loop
const textureId = await game.loadTexture('assets/player.png');

// Inside the game loop
game.drawSprite(textureId, x, y, width, height);

// Optional: draw with rotation (radians)
game.drawSprite(textureId, x, y, width, height, Math.PI / 4);
```

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

```typescript
while (!game.shouldClose()) {
    game.beginFrame(0.1, 0.1, 0.1, 1.0);

    if (game.isKeyPressed(256)) { break; }          // Escape: quit
    if (game.isKeyPressed(32) ||
        game.isMouseButtonPressed(0)) {
        // Space or left-click: jump
    }
    if (game.isKeyPressed(263)) { /* move left */ }
    if (game.isKeyPressed(262)) { /* move right */ }

    game.endFrame();
}
```

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

- [TypeScript SDK README](../sdks/typescript/README.md) — full API reference and build instructions
- [TypeScript examples](../examples/typescript/) — Flappy Bird source with shared desktop/web logic
- [Architecture overview](architecture/sdk-first-architecture.md) — layer design and codegen pipeline
- [Development guide](DEVELOPMENT.md) — build system, git hooks, version management
