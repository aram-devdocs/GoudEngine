# GoudEngine

[![NuGet](https://img.shields.io/nuget/v/GoudEngine.svg)](https://www.nuget.org/packages/GoudEngine/)

> **Alpha** — This SDK is under active development. APIs change frequently. [Report issues](https://github.com/aram-devdocs/GoudEngine/issues) · [Contact](mailto:aram.devdocs@gmail.com)

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

## Networking

Use `new NetworkManager(gameOrContext)` to create wrapper endpoints. `Host(...)` and `Connect(...)` return `NetworkEndpoint`. `Connect(...)` stores a default peer ID, so clients can call `Send(...)`. Host endpoints do not have a default peer, so they reply with `SendTo(...)`.

```csharp
using System.Text;
using GoudEngine;

using var hostContext = new GoudContext();
using var clientContext = new GoudContext();

var host = new NetworkManager(hostContext).Host(NetworkProtocol.Tcp, 9000);
var client = new NetworkManager(clientContext).Connect(NetworkProtocol.Tcp, "127.0.0.1", 9000);

client.Send(Encoding.UTF8.GetBytes("ping"));

while (true)
{
    host.Poll();
    client.Poll();

    var packet = host.Receive();
    if (packet is null)
    {
        continue;
    }

    host.SendTo(packet.Value.PeerId, Encoding.UTF8.GetBytes("pong"));
    break;
}
```

## Debugger Runtime

The desktop C# SDK can opt into the shared Rust-owned debugger runtime before startup. Once enabled, you can read the raw snapshot/manifest JSON, toggle profiling, and inspect aggregate memory totals without creating any SDK-local debugger state.

```csharp
using GoudEngine;

using var config = new EngineConfig()
    .SetTitle("Debugger Demo")
    .SetDebugger(new DebuggerConfig(true, true, "csharp-demo"));

var game = config.Build();
game.SetDebuggerProfilingEnabled(true);

using var snapshot = game.ParseDebuggerSnapshot();
string manifestJson = game.GetDebuggerManifestJson();
MemorySummary memory = game.GetMemorySummary();

game.SetDebuggerSelectedEntity(42);
game.ClearDebuggerSelectedEntity();
game.Destroy();
```

## Flappy Bird Example

A condensed view of the Flappy Bird clone showing how the main patterns fit together.
See the [full source](https://github.com/aram-devdocs/GoudEngine/tree/main/examples/csharp/flappy_goud) for the complete implementation.

```csharp
using GoudEngine;

// --- Constants ---
const uint ScreenWidth = 288, ScreenHeight = 512;
const float Gravity = 9.8f, JumpStrength = -3.5f;
const float PipeSpeed = 1.0f, PipeSpawnInterval = 1.5f, PipeGap = 100f;

var game = new GoudGame(ScreenWidth, ScreenHeight, "Flappy Goud", RendererType.Renderer2D);

// Load textures once at startup (engine returns 64-bit handles)
ulong bgTex     = game.LoadTexture("assets/sprites/background-day.png");
ulong pipeTex   = game.LoadTexture("assets/sprites/pipe-green.png");
ulong baseTex   = game.LoadTexture("assets/sprites/base.png");
ulong[] birdTex = {
    game.LoadTexture("assets/sprites/bluebird-downflap.png"),
    game.LoadTexture("assets/sprites/bluebird-midflap.png"),
    game.LoadTexture("assets/sprites/bluebird-upflap.png"),
};

// --- Bird state ---
float birdX = ScreenWidth / 4f, birdY = ScreenHeight / 2f;
float velocity = 0f;

// --- Pipe state ---
var pipes = new List<(float x, float gapY)>();
float spawnTimer = 0f;
var rng = new Random();

game.Update(dt =>
{
    if (game.IsKeyPressed(Keys.Escape)) { game.Close(); return; }
    if (game.IsKeyPressed(Keys.R))      { Reset(); return; }

    // Bird physics — gravity + jump
    if (game.IsKeyPressed(Keys.Space) || game.IsMouseButtonPressed(MouseButtons.Left))
        velocity = JumpStrength * 120f;           // JumpStrength scaled to target FPS
    velocity += Gravity * dt * 120f;
    birdY    += velocity * dt;

    // Ground / ceiling collision
    if (birdY + 24f > ScreenHeight || birdY < 0f) { Reset(); return; }

    // Pipe updates and AABB collision
    foreach (var (px, gapY) in pipes)
    {
        float topY    = gapY - PipeGap - 320f;    // top pipe rect (52 × 320 sprite)
        float bottomY = gapY + PipeGap;
        bool hitTop    = birdX < px + 52f && birdX + 34f > px && birdY < topY + 320f && birdY + 24f > topY;
        bool hitBottom = birdX < px + 52f && birdX + 34f > px && birdY < bottomY + 320f && birdY + 24f > bottomY;
        if (hitTop || hitBottom) { Reset(); return; }
    }

    // Spawn pipes on a timer
    spawnTimer += dt;
    if (spawnTimer > PipeSpawnInterval)
    {
        spawnTimer = 0f;
        pipes.Add((ScreenWidth, rng.Next((int)PipeGap, (int)ScreenHeight - (int)PipeGap)));
    }
    pipes.RemoveAll(p => p.x + 60f < 0);
    for (int i = 0; i < pipes.Count; i++)
        pipes[i] = (pipes[i].x - PipeSpeed * dt * 120f, pipes[i].gapY);

    // Render — draw order sets depth (later = on top)
    game.DrawSprite(bgTex,   144f, 256f, 288f, 512f);          // background
    foreach (var (px, gapY) in pipes)
    {
        float topY    = gapY - PipeGap - 320f;
        game.DrawSprite(pipeTex, px + 26f, topY    + 160f, 52f, 320f, MathF.PI); // top (flipped)
        game.DrawSprite(pipeTex, px + 26f, gapY + PipeGap + 160f, 52f, 320f);    // bottom
    }
    game.DrawSprite(birdTex[(int)(game.TotalTime / 0.1f) % 3], birdX + 17f, birdY + 12f, 34f, 24f);
    game.DrawSprite(baseTex, 168f, ScreenHeight + 56f, 336f, 112f);              // ground
});

void Reset() { birdX = ScreenWidth / 4f; birdY = ScreenHeight / 2f; velocity = 0f; pipes.Clear(); spawnTimer = 0f; }

game.Run();
```

**Controls:** `Space` or left-click to flap, `R` to restart, `Escape` to quit.

## Features

- 2D and 3D rendering with runtime renderer selection
- Entity Component System (ECS) with Transform2D, Sprite, and more
- Tiled map support for 2D worlds
- Audio playback (WAV, OGG)
- Input handling (keyboard, mouse)
- Asset hot-reloading during development
- Physics simulation (Rapier2D/3D): rigid bodies, colliders, raycasting, collision events
- Per-channel audio mixing (Music, SFX, Ambience, UI, Voice) with spatial positioning
- Text rendering with TrueType/bitmap fonts, alignment, and word-wrapping
- Sprite animation with state machine controller, blending, and tweening
- Scene management with transitions (instant, fade, custom)
- UI component system with hierarchical node tree
- Structured error diagnostics with error codes and recovery hints

## Platform support

| OS | Architecture | Status |
|----|-------------|--------|
| Windows | x64 | Supported |
| macOS | x64 | Supported |
| macOS | ARM64 (Apple Silicon) | Supported |
| Linux | x64 | Supported |

Native libraries are bundled in the NuGet package and copied to your output directory automatically.

## Running Tests

Run the C# SDK tests locally:

```bash
DOTNET_ROOT_X64=/usr/local/share/dotnet/x64 /usr/local/share/dotnet/x64/dotnet test sdks/csharp.tests/GoudEngine.Tests.csproj -v minimal
```

Generate a local coverage artifact:

```bash
DOTNET_ROOT_X64=/usr/local/share/dotnet/x64 /usr/local/share/dotnet/x64/dotnet test sdks/csharp.tests/GoudEngine.Tests.csproj -c Release -v minimal /p:CollectCoverage=true /p:CoverletOutput=sdks/csharp.tests/TestResults/coverage/ /p:CoverletOutputFormat=cobertura
```

Coverage reports are written under `sdks/csharp.tests/TestResults/`.

## Links

- [Repository](https://github.com/aram-devdocs/GoudEngine)
- [Examples](https://github.com/aram-devdocs/GoudEngine/tree/main/examples/csharp)
- [License: MIT](https://github.com/aram-devdocs/GoudEngine/blob/main/LICENSE)
