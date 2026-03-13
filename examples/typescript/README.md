# TypeScript Examples

Example games built on the GoudEngine TypeScript SDK, demonstrating both
desktop (Node.js + napi-rs) and web (WASM in browser) targets from the
same game logic.

## Available Examples

### Flappy Bird (`flappy_bird/`)

A Flappy Bird clone that mirrors the C# and Python versions for SDK parity
testing. Uses a shared `game.ts` with platform-specific entry points.

Controls: Space/Left Click to jump, R to restart, Escape to quit (desktop).

### Feature Lab (`feature_lab/`)

ALPHA-001 wrapper surface lab with shared desktop/web layout. It intentionally
probes wrapper APIs beyond Flappy Bird:

- Provider capability queries (`get*Capabilities`)
- Scene wrapper calls (`loadScene`, `setActiveScene`, `unloadScene`)
- Desktop-only UI wrapper usage (`UiManager`)
- Networking wrapper access (`NetworkManager`)
- Desktop debugger enablement with a stable local attach route label
- Browser networking smoke coverage via `npm run smoke:web-network`

The desktop entry now publishes `feature-lab-typescript-desktop`, confirms
manifest and snapshot access through the shipped debugger APIs, and prints the
manual `goudengine-mcp` attach workflow. The web entry stays browser-only and
does not publish a debugger route in this batch.

### Sandbox (`sandbox/`)

Sandbox is the main parity app for the TypeScript SDK.

- Desktop and web use the same shared sandbox module
- Shared asset pack under `examples/shared/sandbox/`
- Visible 2D, 3D, hybrid, diagnostics, capability, and networking panels
- Web keeps unsupported desktop-only paths visible and capability-gated

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
./dev.sh --sdk typescript --game sandbox
./dev.sh --sdk typescript --game feature_lab
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

Feature Lab web entry:

```bash
cd examples/typescript/feature_lab
npm run build:web
npm run web
```

Open `http://localhost:8765/examples/typescript/feature_lab/web/index.html`.

Sandbox web entry:

```bash
cd examples/typescript/sandbox
npm run build:web
npm run web
```

Open `http://localhost:8765/examples/typescript/sandbox/web/index.html`.

Dedicated browser networking smoke:

```bash
cd examples/typescript/feature_lab
npx playwright install chromium
npm run smoke:web-network
```

This opens headless Chromium, connects the web SDK to a local WebSocket echo server,
waits for `peerCount() > 0`, sends `ping`, and requires `pong` before passing.

## Architecture

Game logic lives in a shared `.ts` file (e.g., `game.ts`) that accepts any
`IGoudGame` instance. Desktop and web entry points import the shared logic
and pass their platform-specific GoudGame implementation. The game code is
identical across both targets.

## Adding a New Example

1. Create a directory under `examples/typescript/`
2. Add a `package.json` with a file dependency on `goudengine`
3. Write shared game logic in a platform-agnostic `.ts` file
4. Add `desktop.ts` and `web/index.html` entry points
5. Match constants and physics with C#/Python versions for parity testing
