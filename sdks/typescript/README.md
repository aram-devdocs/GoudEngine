# GoudEngine TypeScript SDK

[![npm](https://img.shields.io/npm/v/goudengine.svg)](https://www.npmjs.com/package/goudengine)

> **Alpha** — This SDK is under active development. APIs change frequently. [Report issues](https://github.com/aram-devdocs/GoudEngine/issues) · [Contact](mailto:aram.devdocs@gmail.com)

TypeScript bindings for GoudEngine with two targets: Node.js desktop (via napi-rs)
and web browser (via wasm-bindgen). Published as `goudengine` on npm.

## Installation

```bash
npm install goudengine
```

## Usage

### Node.js (Desktop)

```typescript
import { GoudGame, Color, Transform2D } from "goudengine";

const game = new GoudGame(800, 600, "My Game");

while (game.isRunning()) {
  const dt = game.beginFrame();
  // game logic here
  game.endFrame();
}

game.destroy();
```

### Web (Browser)

```typescript
import { GoudGame, Color, Transform2D } from "goudengine/web";

const game = await GoudGame.create(800, 600, "My Game");
// same API as Node
```

## Flappy Bird Example

A condensed Node.js version showing the core patterns — game loop, physics, sprite rendering,
and AABB collision. The web variant uses the same `game.ts` logic with a WASM-backed `GoudGame`.
See the [full source](https://github.com/aram-devdocs/GoudEngine/tree/main/examples/typescript/flappy_bird)
for the complete implementation including the web entry point.

```typescript
import { GoudGame } from "goudengine";

const SCREEN_W = 288, SCREEN_H = 512;
const GRAVITY = 9.8, JUMP_STRENGTH = -3.5;
const PIPE_SPEED = 1.0, PIPE_SPAWN_INTERVAL = 1.5, PIPE_GAP = 100;
const TARGET_FPS = 120;

const game = new GoudGame({ width: SCREEN_W, height: SCREEN_H + 112, title: "Flappy Goud" });

// Load textures — engine returns numeric handles
const bgTex   = await game.loadTexture("assets/sprites/background-day.png");
const pipeTex = await game.loadTexture("assets/sprites/pipe-green.png");
const baseTex = await game.loadTexture("assets/sprites/base.png");
const birdTex = [
  await game.loadTexture("assets/sprites/bluebird-downflap.png"),
  await game.loadTexture("assets/sprites/bluebird-midflap.png"),
  await game.loadTexture("assets/sprites/bluebird-upflap.png"),
];

// Bird state
let birdY = SCREEN_H / 2, velocity = 0;
const birdX = SCREEN_W / 4;

// Pipe state
type Pipe = { x: number; gapY: number };
let pipes: Pipe[] = [];
let spawnTimer = 0;

function reset() {
  birdY = SCREEN_H / 2; velocity = 0; pipes = []; spawnTimer = 0;
}

function aabb(x1: number, y1: number, w1: number, h1: number,
              x2: number, y2: number, w2: number, h2: number): boolean {
  return x1 < x2 + w2 && x1 + w1 > x2 && y1 < y2 + h2 && y1 + h1 > y2;
}

while (!game.shouldClose()) {
  game.beginFrame(0.4, 0.7, 0.9, 1.0);
  const dt = game.deltaTime;

  if (game.isKeyPressed(256)) { game.close(); break; }   // Escape
  if (game.isKeyPressed(82))  { reset(); }                // R

  // Bird physics
  if (game.isKeyPressed(32) || game.isMouseButtonPressed(0))
    velocity = JUMP_STRENGTH * TARGET_FPS;
  velocity += GRAVITY * dt * TARGET_FPS;
  birdY    += velocity * dt;

  if (birdY + 24 > SCREEN_H || birdY < 0) { reset(); game.endFrame(); continue; }

  // Pipe movement and collision
  for (const p of pipes) {
    p.x -= PIPE_SPEED * dt * TARGET_FPS;
    const topY = p.gapY - PIPE_GAP - 320;
    if (aabb(birdX, birdY, 34, 24, p.x, topY, 52, 320) ||
        aabb(birdX, birdY, 34, 24, p.x, p.gapY + PIPE_GAP, 52, 320)) {
      reset(); game.endFrame(); continue;
    }
  }
  pipes = pipes.filter(p => p.x + 60 > 0);

  spawnTimer += dt;
  if (spawnTimer > PIPE_SPAWN_INTERVAL) {
    spawnTimer = 0;
    pipes.push({ x: SCREEN_W, gapY: PIPE_GAP + Math.random() * (SCREEN_H - 2 * PIPE_GAP) });
  }

  // Render — draw order sets depth
  game.drawSprite(bgTex, 144, 256, 288, 512);
  for (const p of pipes) {
    const topY = p.gapY - PIPE_GAP - 320;
    game.drawSprite(pipeTex, p.x + 26, topY + 160, 52, 320, Math.PI);
    game.drawSprite(pipeTex, p.x + 26, p.gapY + PIPE_GAP + 160, 52, 320, 0);
  }
  game.drawSprite(birdTex[Math.floor(game.totalTime / 0.1) % 3], birdX + 17, birdY + 12, 34, 24);
  game.drawSprite(baseTex, 168, SCREEN_H + 56, 336, 112);

  game.endFrame();
}

game.destroy();
```

**Controls:** `Space` or left-click to flap, `R` to restart, `Escape` to quit.
The web variant works the same way — import from `goudengine/web` and call `await GoudGame.create(...)`.

## Web Platform Notes

A few things to keep in mind when targeting the browser:

- **Synchronous game loop callback** -- The `game.run()` callback must be
  synchronous. Passing an `async` function triggers a console warning because
  wasm-bindgen's `RefCell` borrow cannot be held across `await` points.
- **Texture loading uses `fetch()`** -- `loadTexture(path)` calls `fetch()`
  internally, so relative URLs resolve from the page's origin. Make sure your
  asset paths are correct relative to your HTML file or use absolute URLs.
- **`pause()` / `resume()`** -- You can pause the game loop without detaching
  input handlers by calling `game.pause()`. Call `game.resume()` to restart.
  `game.stop()` fully tears down the loop and input handlers.
- **Tab visibility** -- `requestAnimationFrame` automatically pauses when the
  browser tab is hidden. No extra handling is needed.
- **Touch input** -- Touch events are automatically mapped to mouse button 0.
  `touchstart` maps to `press_mouse_button(0)`, `touchend` maps to `release_mouse_button(0)`.

## Node vs Web Targets

The package uses conditional exports to select the right backend automatically:

- **Node.js** -- Uses a native addon built with napi-rs. Calls the Rust engine
  directly through N-API for near-native performance. Requires a `.node` binary
  matching your platform.
- **Web** -- Uses a WASM module built with wasm-bindgen. Runs in any modern
  browser with WebAssembly support. Smaller binary, slightly lower performance.

Both targets expose the same TypeScript API. Game code written against one target
works on the other without changes.

You can also import a specific target explicitly:

```typescript
import { GoudGame } from "goudengine/node";  // Force Node backend
import { GoudGame } from "goudengine/web";   // Force Web backend
```

## Build from Source

### Node.js native addon

```bash
cd sdks/typescript/native && npm run build
```

Or from the SDK root:

```bash
cd sdks/typescript
npm run build:native    # Build napi-rs .node addon
npm run build:ts        # Compile TypeScript sources
npm run build           # Both of the above
```

### WASM module

```bash
cd sdks/typescript
npm run build:web       # Build WASM + compile TS for web target
npm run build:all       # Node + Web
```

### Tests

```bash
cd sdks/typescript
npm test                # node --test test/*.test.mjs
npm run typecheck       # tsc --noEmit for both targets
```

## Codegen

Most source files under `native/src/` and `src/generated/` are auto-generated
by the codegen pipeline. Files with a `.g.rs` or `.g.ts` suffix should not be
edited by hand. Run `./codegen.sh` from the repository root to regenerate.

## f64 vs f32

JavaScript `number` is always 64-bit. The Node SDK converts between `f64` (JS)
and `f32` (Rust engine) at the napi boundary. The WASM target uses `f32` directly
since wasm-bindgen supports it natively.
