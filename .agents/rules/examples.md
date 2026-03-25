---
globs:
  - "examples/**"
---

# Example Game Conventions

## Structure

Examples are organized by SDK language:

```
examples/
├── c/               # C SDK examples
├── cpp/             # C++ SDK examples
├── csharp/          # C# SDK examples
├── go/              # Go SDK examples
├── kotlin/          # Kotlin SDK examples
├── lua/             # Lua SDK examples
├── python/          # Python SDK examples
├── rust/            # Rust SDK examples
├── shared/          # Shared assets across SDKs
├── swift/           # Swift SDK examples
└── typescript/      # TypeScript SDK examples
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
