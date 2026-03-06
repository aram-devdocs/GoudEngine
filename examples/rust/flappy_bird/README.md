# Flappy Bird — Rust

A complete Flappy Bird clone using GoudEngine's Rust SDK. Game constants and behavior match the C#, Python, and TypeScript versions exactly for SDK parity validation.

## Run

```bash
cargo run -p flappy-bird
```

The example must be run from the repository root so that asset paths resolve correctly.

## Controls

| Key / Input | Action |
|-------------|--------|
| Space | Flap |
| Left Click | Flap |
| R | Restart |
| Escape | Quit |

## File structure

| File | Purpose |
|------|---------|
| `main.rs` | Entry point, window creation, game loop |
| `engine.rs` | Thin safe wrapper over the GoudEngine FFI (window, renderer, input) |
| `game_manager.rs` | Top-level game state: loads assets, drives update/draw cycle |
| `bird.rs` | Bird position, velocity, animation frame cycling |
| `pipe.rs` | Pipe pair positioning, scrolling, AABB collision |
| `score.rs` | Score tracking and digit sprite rendering |
| `constants.rs` | All numeric constants shared with other SDK implementations |

## Assets

Sprites are loaded from `examples/csharp/flappy_goud/assets/sprites/`. The Rust example shares assets with the C# version — no separate asset copies.

## Parity with other implementations

The same game logic exists in:

- C#: `examples/csharp/flappy_goud/`
- Python: `examples/python/flappy_bird.py`
- TypeScript: `examples/typescript/flappy_bird/`

Constants in `constants.rs` must stay in sync with those files. See the constants table in `examples/AGENTS.md`.
