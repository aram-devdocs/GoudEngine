# typescript/ — TypeScript Example Games

Desktop (Node + napi-rs) and web (WASM) examples using the TypeScript SDK.

## Examples

- `flappy_bird/` — Flappy Bird parity game
- `sandbox/` — Feature parity tester (desktop + web)
- `feature_lab/` — API smoke coverage (desktop + web)

## Running

```bash
./dev.sh --sdk typescript --game flappy_bird
./dev.sh --sdk typescript --game sandbox
./dev.sh --sdk typescript --game feature_lab
```

Or manually:

```bash
cd examples/typescript/flappy_bird
npm install
npm run desktop      # Node.js (requires native addon)
npm run build:web && npm run web  # Browser on port 8765
```

## Conventions

- Game logic in shared `.ts` file (platform-agnostic)
- Desktop and web entry points pass their platform's `GoudGame` to shared logic
- Constants match C#/Python implementations exactly

See `.agents/rules/examples.md`.
