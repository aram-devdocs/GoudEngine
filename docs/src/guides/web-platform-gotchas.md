# Web Platform Gotchas

Browser builds are close to the desktop TypeScript API, but they are not identical in behavior.
This page collects the current rough edges so developers do not rediscover them by trial and error.

## Keep `game.run()` synchronous

The Web target uses wasm-bindgen plus internal `RefCell` state. Do not `await` inside the
`game.run()` callback and do not hand it an `async` function.

Bad:

```typescript
game.run(async (_dt) => {
  await loadLevel();
});
```

Good:

```typescript
const levelPromise = loadLevel();

game.run((_dt) => {
  // poll or react to already-started async work here
});
```

If you need async setup, do it before `game.run()` starts.

## Key state is frame-based

Use `isKeyPressed(...)` and `isMouseButtonPressed(...)` from inside the frame loop.
Do not assume browser DOM events map one-to-one with engine input state outside the loop.

Recommended pattern:

```typescript
game.run((_dt) => {
  if (game.isKeyPressed(32)) {
    // Space
  }
});
```

This avoids the stale-key-state bugs that motivated the earlier WASM input fixes.

## Asset paths resolve through the page origin

Web asset loading goes through `fetch()`. Relative paths are resolved from the page URL,
not from the TypeScript source file.

Recommended:

```typescript
const texture = await game.loadTexture("/assets/player.png");
```

or serve your page and assets from a directory layout where the relative URLs are obvious and stable.

Avoid opening the page with `file://`; use a real HTTP server such as `npx serve .`.

## Mid-loop asset loading still has visible cost

Texture and font loading on the web path can stall or hitch if you start large loads after the
frame loop is already running. The safest pattern is:

1. Create the game.
2. Call `await game.preload([...])` for the path-based textures/fonts you know you need.
3. Start `game.run()`.

Example:

```typescript
await game.preload(
  [
    '/assets/background-day.png',
    '/assets/pipe-green.png',
    '/assets/flappy.ttf',
  ],
  {
    onProgress(update) {
      console.log(update.progress);
    },
  },
);
```

Current limitation:

- The generated preloader currently covers the TypeScript SDK's path-based texture/font loaders.
- It is not a generic preload system for every possible asset class yet.

## Web networking is client-only

`goudengine/web` now supports browser WebSocket client connections.

Current state:

- `goudengine/node`: host + client workflows, loopback/headless tests
- `goudengine/web`: browser WebSocket client connections

Current limitation:

- Browser hosting is not supported.
- Use `NetworkProtocol.WebSocket` on the web target.
- `connect()` returns before the socket is fully open, so wait for `peerCount() > 0` before sending your first packet.

Recommended pattern:

```typescript
const endpoint = new NetworkManager(game).connect(
  NetworkProtocol.WebSocket,
  "ws://127.0.0.1:9001",
  9001,
);

game.run(() => {
  endpoint.poll();
  if (endpoint.peerCount() > 0) {
    endpoint.send(new TextEncoder().encode("ping"));
  }
});
```

## Touch maps to primary mouse input

The web backend maps touch input to mouse button `0`. That is enough for the current examples,
but multi-touch gestures are not exposed as a richer engine input model yet.

## Recommended smoke path

Use these before debugging your own browser build:

```bash
./dev.sh --sdk typescript --game flappy_bird_web
./dev.sh --sdk typescript --game feature_lab_web
```

If both run cleanly, your environment, asset serving, and WASM packaging path are probably fine.
