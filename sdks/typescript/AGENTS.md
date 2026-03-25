# typescript/ — TypeScript SDK

Node.js (napi-rs) and browser (WASM) targets. Published as `goudengine` on npm.

## Key Gotcha: f64 vs f32

napi-rs uses `f64` for JS numbers; engine uses `f32`. Node SDK converts at boundary; WASM uses `f32` directly.

## Build

```bash
npm run build         # Native + TypeScript (default)
npm run build:native  # napi-rs addon
npm run build:web     # WASM + TS for browser
npm run test          # Smoke tests
```

## Generated Files

- `.g.rs` and `.g.ts` files: codegen output, never hand-edit
- `index.js` and `index.d.ts`: napi-rs loaders, gitignored, auto-built
- See `.agents/rules/sdk-development.md` for feature parity requirements

## Anti-Patterns

- Never hand-edit `.g.rs` or `.g.ts`
- Never commit `index.js` or `index.d.ts`
- Never assume f64 precision matches f32
