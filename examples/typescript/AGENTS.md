# typescript/ -- TypeScript Example Games

## Purpose

Example games built on the GoudEngine TypeScript SDK, demonstrating both desktop (napi-rs)
and web (WASM) targets from the same game logic.

## Structure

- `flappy_bird/` -- Flappy Bird clone, mirrors C#/Python versions for SDK parity testing
- `feature_lab/` -- ALPHA-001 wrapper surface lab (desktop + web entrypoints)

## Running

### Desktop (Node.js + native addon)

```bash
cd examples/typescript/flappy_bird
npm install
npm run desktop
```

Requires the native addon to be built first: `cd sdks/typescript && npm run build:native`

### Web (WASM in browser)

```bash
cd examples/typescript/flappy_bird
npm run build:web        # Compile TS to dist/
npm run web              # Start local server on port 8765
# Open http://localhost:8765/examples/typescript/flappy_bird/web/index.html
```

Requires the WASM build: see `sdks/typescript/wasm/`.

## Conventions

- Game logic lives in a shared `.ts` file (e.g., `game.ts`) that is platform-agnostic
- Desktop and web entry points import the shared logic and pass their platform's GoudGame
- Examples depend on `goudengine` via file reference to `sdks/typescript/`
- Constants and physics match the C#/Python Flappy Bird implementations exactly

## Dependencies

Layer 5 (Apps). Examples depend on the TypeScript SDK only. Never import engine internals.
