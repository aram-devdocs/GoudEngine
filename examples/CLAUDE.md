# examples/ — Example Games

## Purpose

Standalone example games demonstrating GoudEngine features, organized by SDK language.

## Structure

- `csharp/` — C# example games (flappy_goud, 3d_cube, goud_jumper, isometric_rpg, hello_ecs)
- `python/` — Python example games (main.py demo, flappy_bird.py)
- `typescript/` — TypeScript example games (flappy_bird — desktop + web targets)
- `rust/` — Rust examples (future)

## Running

```bash
./dev.sh --game flappy_goud          # C# Flappy Bird clone
./dev.sh --game 3d_cube              # C# 3D rendering demo
./dev.sh --game goud_jumper          # C# platformer
./dev.sh --game isometric_rpg        # C# isometric RPG
./dev.sh --game hello_ecs            # C# ECS basics
./dev.sh --sdk python --game python_demo    # Python demo
./dev.sh --sdk python --game flappy_bird    # Python Flappy Bird
./dev.sh --sdk typescript --game flappy_bird  # TypeScript Flappy Bird (desktop)
```

### TypeScript Web Target

```bash
cd examples/typescript/flappy_bird
npm run build:web        # Compile TS to dist/
npm run web              # Start local server on port 8765
# Open http://localhost:8765/examples/typescript/flappy_bird/web/index.html
```

## Rules

- Each example MUST be a standalone project with its own project file
- Examples MUST work with the latest SDK version
- Examples demonstrate engine features — keep code readable and well-commented
- Python Flappy Bird mirrors C# flappy_goud for SDK parity testing
- TypeScript Flappy Bird mirrors C# and Python versions with shared game logic across desktop/web

## Dependencies

Layer 5 (Apps). Examples depend on SDKs. Never import engine internals directly.
