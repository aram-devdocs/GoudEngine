---
globs:
  - "examples/**"
---

# Example Game Conventions

## Structure

Examples are organized by SDK language:

```
examples/
├── csharp/          # C# SDK examples
│   ├── flappy_goud/    — 2D sprites, collision, input
│   ├── goud_jumper/    — platformer physics, tile maps
│   ├── 3d_cube/        — 3D rendering, camera
│   ├── isometric_rpg/  — combat, NPCs, UI
│   └── hello_ecs/      — basic ECS usage
├── python/          # Python SDK examples
│   ├── main.py         — basic SDK demo
│   └── flappy_bird.py  — mirrors C# flappy_goud for parity testing
└── rust/            # Rust SDK examples (future)
```

## Requirements

- Each example is a standalone project with its own project file (`.csproj` or script)
- Examples MUST work with the latest published SDK version
- C# examples reference the GoudEngine NuGet package
- Python examples import from the local SDK path

## Running Examples

```
./dev.sh --game flappy_goud              # C# example
./dev.sh --game 3d_cube                  # C# 3D example
./dev.sh --sdk python --game python_demo # Python example
./dev.sh --sdk python --game flappy_bird # Python example
```

## When Modifying Examples

- If you change the SDK API, update all affected examples
- Python Flappy Bird should mirror C# Flappy Goud — keep them in sync for parity testing
- Test the example end-to-end with `./dev.sh` after changes
