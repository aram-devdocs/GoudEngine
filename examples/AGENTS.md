# examples/ — Example Games

Standalone examples organized by SDK language. Each has its own project file.

## Languages

- `csharp/` — Flappy Bird (flappy_goud), 3D cube, platformer (goud_jumper), ECS (hello_ecs), sandbox, feature_lab
- `python/` — Demo, Flappy Bird, sandbox
- `typescript/` — Flappy Bird, sandbox, feature_lab (desktop + web)
- `cpp/`, `c/`, `swift/`, `go/`, `kotlin/`, `lua/`, `rust/` — Flappy Bird, sandbox, feature_lab

## Running

```bash
./dev.sh --game flappy_goud              # C# Flappy Bird
./dev.sh --game 3d_cube                  # C# 3D demo
./dev.sh --sdk python --game flappy_bird # Python Flappy Bird
./dev.sh --sdk typescript --game flappy_bird  # TypeScript Flappy Bird
cargo run -p flappy-bird                 # Rust Flappy Bird
```

## Rules

- Each example is a standalone project
- Examples MUST work with latest SDK version
- Flappy Bird constants must match source of truth at `examples/rust/flappy_bird/src/constants.rs`
- Python/TypeScript Flappy Bird mirror C# for parity testing
- Sandbox is the public parity app across all targets

See `.agents/rules/examples.md`.
