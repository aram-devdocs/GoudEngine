# GoudEngine TypeScript SDK

TypeScript bindings for GoudEngine with two targets: Node.js desktop (via napi-rs)
and web browser (via wasm-bindgen). Published as `@goudengine/sdk` on npm.

## Installation

```bash
npm install @goudengine/sdk
```

## Usage

### Node.js (Desktop)

```typescript
import { GoudGame, Color, Transform2D } from "@goudengine/sdk";

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
import { GoudGame, Color, Transform2D } from "@goudengine/sdk/web";

const game = await GoudGame.create(800, 600, "My Game");
// same API as Node
```

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
import { GoudGame } from "@goudengine/sdk/node";  // Force Node backend
import { GoudGame } from "@goudengine/sdk/web";   // Force Web backend
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
