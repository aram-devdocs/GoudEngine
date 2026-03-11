# examples/csharp/ — C# Example Games

## Purpose

C# games demonstrating GoudEngine SDK features. Each is a standalone .NET project.

## Games

- `flappy_goud/` — Flappy Bird clone: 2D sprites, collision, input, scoring
- `3d_cube/` — 3D rendering demo: cube rendering, 3D camera
- `goud_jumper/` — Platformer: physics, tile maps, sprite animation
- `isometric_rpg/` — Complex RPG: combat, NPCs, dialogue, UI, isometric rendering
- `hello_ecs/` — ECS basics: entity creation, components, systems
- `feature_lab/` — Headless smoke app: ECS components, builders, and batch entity operations

## Patterns

- Each game has its own `.csproj` referencing the GoudEngine NuGet package
- Entry point is `Program.cs` with `GoudGame` initialization
- Game logic in separate classes (e.g., `Bird.cs`, `GameManager.cs`)
- Assets in per-game `assets/` directories

## Running

```bash
./dev.sh --game flappy_goud          # Run with published NuGet
./dev.sh --game flappy_goud --local  # Run with local NuGet feed
./dev.sh --game feature_lab          # Run headless feature-lab smoke checks
```

## Adding a New Example

1. Create directory under `examples/csharp/`
2. Add `.csproj` referencing GoudEngine package
3. Create `Program.cs` with `GoudGame` setup
4. Add to `dev.sh` game list if needed
5. Include `assets/` directory for any game resources
