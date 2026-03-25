# flappy_bird/ — TypeScript Flappy Bird

Flappy Bird clone for desktop (Node + napi-rs) and web (WASM). Mirrors C#/Python versions exactly — same constants, physics, rendering.

## Key Files

- `game.ts` — Platform-agnostic logic
- `desktop.ts` — Node.js entry (napi-rs)
- `web/index.html` — Browser entry (WASM)

## Running

```bash
npm install
npm run desktop    # Node.js (requires built native addon)
npm run build:web && npm run web  # Browser on port 8765
```

## Architecture

`game.ts` exports `FlappyBirdGame(iGoudGame)` — accepts any `IGoudGame` instance. Same logic runs on both platforms; only backend differs.

## Controls

Space / Click: jump, R: restart, Escape: quit (desktop)
