# Getting Started — C# SDK

> **Alpha** — GoudEngine is under active development. APIs change frequently. [Report issues](https://github.com/aram-devdocs/GoudEngine/issues)

## Prerequisites

- [.NET 8.0 SDK](https://dotnet.microsoft.com/download/dotnet/8.0)

## Installation

Create a new console project and add the NuGet package:

```bash
dotnet new console -n MyGame
cd MyGame
dotnet add package GoudEngine
```

Open `MyGame.csproj` and add `<AllowUnsafeBlocks>true</AllowUnsafeBlocks>`. The SDK uses unsafe interop internally.

```xml
<Project Sdk="Microsoft.NET.Sdk">
  <PropertyGroup>
    <OutputType>Exe</OutputType>
    <TargetFramework>net8.0</TargetFramework>
    <AllowUnsafeBlocks>true</AllowUnsafeBlocks>
  </PropertyGroup>
  <ItemGroup>
    <PackageReference Include="GoudEngine" Version="0.0.832" />
  </ItemGroup>
</Project>
```

## First Project

Replace `Program.cs` with a minimal window that closes on Escape:

{{#include ../generated/snippets/csharp/first-project.md}}

`BeginFrame` clears the screen to the given color and prepares the frame. `EndFrame` swaps buffers and polls events. `DeltaTime` gives seconds since the last frame — use it to keep movement frame-rate independent.

Run it:

```bash
dotnet run
```

For 3D rendering, the same game window supports both 2D and 3D rendering. Load a 3D model and render it within the same frame loop:

```csharp
var model = game.LoadModel("assets/model.gltf");
game.Draw3D(model, transform);
```

## Debugger Runtime

Enable debugger mode before creating the windowed game or headless context.

```csharp
using GoudEngine;

using var ctx = new GoudContext(
    new ContextConfig(new DebuggerConfig(true, true, "getting-started-csharp"))
);

ctx.SetDebuggerProfilingEnabled(true);
string snapshotJson = ctx.GetDebuggerSnapshotJson();
string manifestJson = ctx.GetDebuggerManifestJson();
```

For a ready-made headless route, run `./dev.sh --game feature_lab`. The example
publishes `feature-lab-csharp-headless`, confirms manifest and snapshot access,
and prints the manual attach steps:

1. start `cargo run -p goudengine-mcp`
2. call `goudengine.list_contexts`
3. call `goudengine.attach_context`

## Drawing a Sprite

Load textures once before the loop. Drawing happens inside the loop between `BeginFrame` and `EndFrame`.

{{#include ../generated/snippets/csharp/drawing-a-sprite.md}}

To draw a colored quad without a texture:

```csharp
game.DrawQuad(x, y, width, height, new Color(1.0f, 0.0f, 0.0f, 1.0f));
```

Put your image files in an `assets/` folder next to the project. The path is relative to the working directory when you run `dotnet run`.

## Handling Input

`IsKeyPressed` returns true every frame the key is held. Use it for movement. For one-shot actions, track state yourself.

{{#include ../generated/snippets/csharp/handling-input.md}}

Mouse input follows the same pattern:

```csharp
if (game.IsMouseButtonPressed(MouseButton.Left)) { /* click action */ }

float mouseX = game.MouseX;
float mouseY = game.MouseY;
```

## Running an Example Game

The repository includes several complete C# games. Clone and run them directly:

```bash
git clone https://github.com/aram-devdocs/GoudEngine.git
cd GoudEngine
./dev.sh --game flappy_goud     # Flappy Bird clone
./dev.sh --game sandbox         # Full feature sandbox
./dev.sh --game goud_jumper     # Platformer
./dev.sh --game 3d_cube         # 3D rendering demo
./dev.sh --game isometric_rpg   # Isometric RPG
./dev.sh --game hello_ecs       # ECS basics
./dev.sh --game feature_lab     # Supplemental smoke coverage
```

`dev.sh` builds the engine and runs the example in one step. Source for each example is in [`examples/csharp/`](https://github.com/aram-devdocs/GoudEngine/tree/main/examples/csharp/).

To use a locally built version of the engine instead of the published NuGet package:

```bash
./build.sh
./package.sh --local
./dev.sh --game flappy_goud --local
```

## Next Steps

- [C# SDK README](https://github.com/aram-devdocs/GoudEngine/tree/main/sdks/csharp/) — full API reference
- [C# examples source](https://github.com/aram-devdocs/GoudEngine/tree/main/examples/csharp/) — complete game source code
- [Build Your First Game](../guides/build-your-first-game.md) — end-to-end minimal game walkthrough
- [Debugger Runtime](../guides/debugger-runtime.md) — local attach, capture, replay, and metrics workflow
- [Example Showcase](../guides/showcase.md) — current cross-language parity matrix
- [Cross-Platform Deployment](../guides/deployment.md) — packaging and release workflow
- [FAQ and Troubleshooting](../guides/faq.md) — common runtime and build issues
- [SDK-first architecture](../architecture/sdk-first.md) — how the engine layers fit together
- [Development guide](../development/guide.md) — building from source, version management, git hooks
- Other getting started guides: [Python](python.md) · [TypeScript](typescript.md) · [Rust](rust.md) · [Go](go.md) · [Kotlin](kotlin.md) · [Lua](lua.md)
