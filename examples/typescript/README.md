# TypeScript Examples

Example games built on the GoudEngine TypeScript SDK, demonstrating both
desktop (Node.js + napi-rs) and web (WASM in browser) targets from the
same game logic.

## Available Examples

### Flappy Bird (`flappy_bird/`)

A Flappy Bird clone that mirrors the C# and Python versions for SDK parity
testing. Uses a shared `game.ts` with platform-specific entry points.

Controls: Space/Left Click to jump, R to restart, Escape to quit (desktop).

## Running -- Desktop

Prerequisites: build the TypeScript SDK native addon first.

```bash
cd sdks/typescript && npm run build:native
```

Then run any example:

```bash
cd examples/typescript/flappy_bird
npm install
npm run desktop
```

Or use the dev script from the repository root:

```bash
./dev.sh --sdk typescript --game flappy_bird
```

## Running -- Web

Prerequisites: build the WASM module first.

```bash
cd sdks/typescript && npm run build:web
```

Then compile and serve the example:

```bash
cd examples/typescript/flappy_bird
npm run build:web        # Compile TS to dist/
npm run web              # Start local server on port 8765
```

Open `http://localhost:8765/examples/typescript/flappy_bird/web/index.html`
in a browser.

## Architecture

Game logic lives in a shared `.ts` file (e.g., `game.ts`) that accepts any
`IGoudGame` instance. Desktop and web entry points import the shared logic
and pass their platform-specific GoudGame implementation. The game code is
identical across both targets.

## Adding a New Example

1. Create a directory under `examples/typescript/`
2. Add a `package.json` with a file dependency on `@goudengine/sdk`
3. Write shared game logic in a platform-agnostic `.ts` file
4. Add `desktop.ts` and `web/index.html` entry points
5. Match constants and physics with C#/Python versions for parity testing
