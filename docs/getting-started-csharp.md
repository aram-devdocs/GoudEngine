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
    <PackageReference Include="GoudEngine" Version="0.0.815" />
  </ItemGroup>
</Project>
```

## First Project

Replace `Program.cs` with a minimal window that closes on Escape:

```csharp
using GoudEngine;

using var game = new GoudGame(800, 600, "My Game");

while (!game.ShouldClose())
{
    game.BeginFrame(0.2f, 0.3f, 0.4f, 1.0f); // RGBA clear color

    float dt = game.DeltaTime;

    if (game.IsKeyPressed(Keys.Escape))
    {
        game.Close();
        continue;
    }

    game.EndFrame();
}
```

`BeginFrame` clears the screen to the given color and prepares the frame. `EndFrame` swaps buffers and polls events. `DeltaTime` gives seconds since the last frame — use it to keep movement frame-rate independent.

Run it:

```bash
dotnet run
```

For a 3D window, pass the renderer type:

```csharp
using var game = new GoudGame(800, 600, "3D Game", RendererType.Renderer3D);
```

## Drawing a Sprite

Load textures once before the loop. Drawing happens inside the loop between `BeginFrame` and `EndFrame`.

```csharp
using GoudEngine;

using var game = new GoudGame(800, 600, "My Game");

ulong textureId = game.LoadTexture("assets/sprite.png");

float x = 100f, y = 100f;
float width = 64f, height = 64f;

while (!game.ShouldClose())
{
    game.BeginFrame(0.1f, 0.1f, 0.1f, 1.0f);

    if (game.IsKeyPressed(Keys.Escape)) { game.Close(); continue; }

    game.DrawSprite(textureId, x, y, width, height);

    game.EndFrame();
}
```

To draw a colored quad without a texture:

```csharp
game.DrawQuad(x, y, width, height, new Color(1.0f, 0.0f, 0.0f, 1.0f));
```

Put your image files in an `assets/` folder next to the project. The path is relative to the working directory when you run `dotnet run`.

## Handling Input

`IsKeyPressed` returns true every frame the key is held. Use it for movement. For one-shot actions, track state yourself.

```csharp
float speed = 200f;

while (!game.ShouldClose())
{
    game.BeginFrame(0.1f, 0.1f, 0.1f, 1.0f);

    float dt = game.DeltaTime;

    if (game.IsKeyPressed(Keys.Escape)) { game.Close(); continue; }

    if (game.IsKeyPressed(Keys.Left))  x -= speed * dt;
    if (game.IsKeyPressed(Keys.Right)) x += speed * dt;
    if (game.IsKeyPressed(Keys.Up))    y -= speed * dt;
    if (game.IsKeyPressed(Keys.Down))  y += speed * dt;
    if (game.IsKeyPressed(Keys.Space)) { /* fire, jump, etc. */ }

    game.DrawSprite(textureId, x, y, width, height);

    game.EndFrame();
}
```

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
./dev.sh --game goud_jumper     # Platformer
./dev.sh --game 3d_cube         # 3D rendering demo
./dev.sh --game isometric_rpg   # Isometric RPG
./dev.sh --game hello_ecs       # ECS basics
```

`dev.sh` builds the engine and runs the example in one step. Source for each example is in [`examples/csharp/`](../examples/csharp/).

To use a locally built version of the engine instead of the published NuGet package:

```bash
./build.sh
./package.sh --local
./dev.sh --game flappy_goud --local
```

## Next Steps

- [C# SDK README](../sdks/csharp/README.md) — full API reference
- [C# examples source](../examples/csharp/) — complete game source code
- [SDK-first architecture](architecture/sdk-first-architecture.md) — how the engine layers fit together
- [Development guide](DEVELOPMENT.md) — building from source, version management, git hooks
- Other getting started guides: [Python](getting-started-python.md) · [TypeScript](getting-started-typescript.md) · [Rust](getting-started-rust.md)
