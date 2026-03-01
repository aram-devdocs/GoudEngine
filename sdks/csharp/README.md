# GoudEngine

Rust-powered game engine for C# developers. Build 2D and 3D games with a familiar .NET API backed by a high-performance Rust core.

## Install

```bash
dotnet add package GoudEngine
```

## Quick start

```csharp
using GoudEngine;

// Create a 2D game
var game = new GoudGame(800, 600, "My Game", RendererType.Renderer2D);

// Spawn an entity with a transform
var entity = game.Spawn();
entity.Transform.Position = new Vec2(100, 50);

// Game loop handled by the engine
game.Run();
```

## Features

- 2D and 3D rendering with runtime renderer selection
- Entity Component System (ECS) with Transform2D, Sprite, and more
- Tiled map support for 2D worlds
- Audio playback (WAV, OGG)
- Input handling (keyboard, mouse)
- Asset hot-reloading during development

## Platform support

| OS | Architecture | Status |
|----|-------------|--------|
| Windows | x64 | Supported |
| macOS | x64 | Supported |
| macOS | ARM64 (Apple Silicon) | Supported |
| Linux | x64 | Supported |

Native libraries are bundled in the NuGet package and copied to your output directory automatically.

## Links

- [Repository](https://github.com/aram-devdocs/GoudEngine)
- [Examples](https://github.com/aram-devdocs/GoudEngine/tree/main/examples/csharp)
- [License: MIT](https://github.com/aram-devdocs/GoudEngine/blob/main/LICENSE)
