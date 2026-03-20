# examples/ — Example Games

## Purpose

Standalone example games demonstrating GoudEngine features, organized by SDK language.

## Structure

- `c/` — C smoke examples for the native wrapper layer
- `cpp/` — C++ examples: smoke test, CMake integration demo, Flappy Bird game, sandbox, feature_lab
- `csharp/` — C# example games (flappy_goud, 3d_cube, goud_jumper, isometric_rpg, hello_ecs, feature_lab, sandbox)
- `python/` — Python example games (main.py demo, flappy_bird.py, sandbox.py)
- `swift/` — Swift example games (flappy_bird, sandbox, feature_lab)
- `typescript/` — TypeScript example games (flappy_bird, feature_lab, sandbox — desktop + web targets)
- `go/` — Go example games (flappy_bird, sandbox, feature_lab)
- `kotlin/` — Kotlin example games (flappy_bird, sandbox, feature_lab)
- `lua/` — Lua example games (flappy_bird, sandbox, feature_lab)
- `rust/` — Rust example games (flappy_bird, feature_lab, sandbox)

## Running

```bash
./dev.sh --game flappy_goud          # C# Flappy Bird clone
./dev.sh --game 3d_cube              # C# 3D rendering demo
./dev.sh --game goud_jumper          # C# platformer
./dev.sh --game isometric_rpg        # C# isometric RPG
./dev.sh --game hello_ecs            # C# ECS basics
./dev.sh --game feature_lab          # C# headless feature-lab smoke checks
./dev.sh --game sandbox              # C# sandbox parity app
./dev.sh --sdk python --game python_demo    # Python demo
./dev.sh --sdk python --game flappy_bird    # Python Flappy Bird
./dev.sh --sdk python --game sandbox        # Python sandbox parity app
./dev.sh --sdk typescript --game flappy_bird  # TypeScript Flappy Bird (desktop)
./dev.sh --sdk typescript --game feature_lab  # TypeScript Feature Lab (desktop)
./dev.sh --sdk typescript --game sandbox      # TypeScript Sandbox (desktop)
./dev.sh --sdk cpp --game flappy_bird          # C++ Flappy Bird
./dev.sh --sdk cpp --game cmake_example       # C++ CMake integration demo
./dev.sh --sdk cpp --game sandbox             # C++ Sandbox
./dev.sh --sdk cpp --game feature_lab         # C++ Feature Lab (headless)
./dev.sh --sdk swift --game flappy_bird        # Swift Flappy Bird
./dev.sh --sdk swift --game sandbox            # Swift Sandbox
./dev.sh --sdk swift --game feature_lab        # Swift Feature Lab (headless)
./dev.sh --sdk go --game flappy_bird          # Go Flappy Bird
./dev.sh --sdk go --game sandbox              # Go Sandbox
./dev.sh --sdk go --game feature_lab          # Go Feature Lab (headless)
./dev.sh --sdk kotlin --game flappy_bird      # Kotlin Flappy Bird
./dev.sh --sdk kotlin --game sandbox          # Kotlin Sandbox
./dev.sh --sdk kotlin --game feature_lab      # Kotlin Feature Lab (headless)
./dev.sh --sdk lua --game flappy_bird         # Lua Flappy Bird
./dev.sh --sdk lua --game sandbox             # Lua Sandbox
./dev.sh --sdk lua --game feature_lab         # Lua Feature Lab (headless)
cargo run -p flappy-bird                      # Rust Flappy Bird
cargo run -p sandbox                          # Rust Sandbox
```

### TypeScript Web Target

```bash
cd examples/typescript/sandbox
npm run build:web        # Compile TS to dist/
npm run web              # Start local server on port 8765
# Open http://localhost:8765/examples/typescript/sandbox/web/index.html
```

## Rules

- Each example MUST be a standalone project with its own project file
- Examples MUST work with the latest SDK version
- Examples demonstrate engine features — keep code readable and well-commented
- C smoke examples stay small; C++ has a full Flappy Bird parity game
- Python Flappy Bird mirrors C# flappy_goud for SDK parity testing
- TypeScript Flappy Bird mirrors C# and Python versions with shared game logic across desktop/web
- Sandbox is the public parity app across all supported targets
- TypeScript Feature Lab extends wrapper/API smoke coverage for ALPHA-001 verification
- Rust Flappy Bird (`examples/rust/flappy_bird/`) mirrors all other implementations

## Game constants

These constants MUST stay identical across all Flappy Bird implementations (C#, Python, TypeScript, Rust). The source of truth is `examples/rust/flappy_bird/src/constants.rs`.

| Constant | Value | Purpose |
|----------|-------|---------|
| SCREEN_WIDTH | 288 | Window width in pixels |
| SCREEN_HEIGHT | 512 | Window height in pixels |
| BASE_HEIGHT | 112 | Height of the ground strip |
| TARGET_FPS | 120 | Frame rate used to scale physics |
| BIRD_WIDTH | 34 | Bird sprite collision width |
| BIRD_HEIGHT | 24 | Bird sprite collision height |
| GRAVITY | 9.8 | Downward acceleration per frame (scaled by dt * TARGET_FPS) |
| JUMP_STRENGTH | -3.5 | Upward velocity on flap (scaled by TARGET_FPS) |
| JUMP_COOLDOWN | 0.3 | Minimum seconds between flaps |
| PIPE_GAP | 100 | Vertical gap between top and bottom pipe |
| PIPE_SPAWN_INTERVAL | 1.5 | Seconds between new pipe pairs |
| PIPE_SPEED | 1.0 | Pipe scroll speed (scaled by TARGET_FPS * dt) |
| PIPE_COLLISION_WIDTH | 60 | Pipe collision box width |

## Dependencies

Layer 5 (Apps). Examples depend on SDKs. Never import engine internals directly.
