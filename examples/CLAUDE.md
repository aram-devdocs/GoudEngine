# examples/ — Example Games

## Purpose

Standalone example games demonstrating GoudEngine features, organized by SDK language.

## Structure

- `csharp/` — C# example games (flappy_goud, 3d_cube, goud_jumper, isometric_rpg, hello_ecs)
- `python/` — Python example games (main.py demo, flappy_bird.py)
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
```

## Rules

- Each example MUST be a standalone project with its own project file
- Examples MUST work with the latest SDK version
- Examples demonstrate engine features — keep code readable and well-commented
- Python Flappy Bird mirrors C# flappy_goud for SDK parity testing

## Dependencies

Layer 5 (Apps). Examples depend on SDKs. Never import engine internals directly.
