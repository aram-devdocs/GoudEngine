# flappy_bird/ -- TypeScript Flappy Bird

## Purpose

Flappy Bird clone built on the GoudEngine TypeScript SDK. Demonstrates 2D sprite rendering,
input handling, collision detection, and the shared-logic pattern for desktop + web targets.

Mirrors the C# (`examples/csharp/flappy_goud/`) and Python (`examples/python/flappy_bird.py`)
versions exactly -- same constants, same physics, same rendering order.

## Key Files

- `game.ts` -- Platform-agnostic game logic (Bird, Pipe, ScoreCounter, FlappyBirdGame)
- `desktop.ts` -- Node.js entry point using native napi-rs addon (GLFW + OpenGL)
- `web/index.html` -- Browser entry point using WASM backend (canvas + WebGL)
- `package.json` -- npm scripts: `desktop`, `build:web`, `web`
- `tsconfig.json` -- Compiles `game.ts` to `dist/game.js` for the web entry point
- `dist/game.js` -- Compiled output, imported by `web/index.html`

## Running

### Desktop

```bash
npm install
npm run desktop          # npx tsx desktop.ts
```

Prerequisite: build the native addon (`cd sdks/typescript && npm run build:native`).

### Web

```bash
npm run build:web        # Compile game.ts -> dist/game.js
npm run web              # python3 -m http.server 8765
```

Then open `http://localhost:8765/examples/typescript/flappy_bird/web/index.html`.
The web entry uses an importmap to resolve `goudengine/web` from the SDK dist.

## Architecture

`game.ts` exports `FlappyBirdGame` which accepts any `IGoudGame` instance. The desktop
entry passes a native `GoudGame`; the web entry passes a WASM-backed `GoudGame`. Game logic
is identical across both -- only the engine backend differs.

## Assets

Reuses sprites from `examples/csharp/flappy_goud/assets/sprites/`. Desktop path is relative
(`../../csharp/flappy_goud/assets/sprites`); web path is absolute from the server root.

## Controls

- Space / Left Click: jump
- R: restart
- Escape: quit (desktop only)
