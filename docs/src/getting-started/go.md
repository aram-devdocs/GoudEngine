# Getting Started -- Go SDK

> **Alpha** -- GoudEngine is under active development. APIs change frequently. [Report issues](https://github.com/aram-devdocs/GoudEngine/issues)

This guide covers installing the Go SDK, opening a window, drawing a sprite, and handling input.

See also: [C# guide](csharp.md) -- [Python guide](python.md) -- [TypeScript guide](typescript.md) -- [Rust guide](rust.md) -- [Swift guide](swift.md) -- [Kotlin guide](kotlin.md)

## Prerequisites

- Go 1.21 or later
- A C compiler (GCC or Clang) -- required for cgo
- [Rust toolchain](https://rustup.rs/) -- needed to build the native engine library

## Building the Native Library

The Go SDK wraps the native Rust engine through its C FFI. Build the engine first:

```bash
git clone https://github.com/aram-devdocs/GoudEngine.git
cd GoudEngine
cargo build --release
```

This produces `target/release/libgoud_engine.dylib` (macOS), `libgoud_engine.so` (Linux), or `goud_engine.dll` (Windows).

## Installation

The Go SDK uses cgo to call into the native library. Add the module to your project:

```bash
go get github.com/aram-devdocs/goud-engine-go
```

For local development from the repository, the SDK is at `sdks/go/`. Set CGO flags to find the native library:

```bash
export CGO_CFLAGS="-I$(pwd)/sdks/go/include"
export CGO_LDFLAGS="-L$(pwd)/target/release"
```

## First Project

Create `main.go`:

```go
package main

import "github.com/aram-devdocs/goud-engine-go/goud"

func main() {
    game := goud.NewGame(800, 600, "My First Game")
    defer game.Destroy()

    for !game.ShouldClose() {
        game.BeginFrame(0.2, 0.2, 0.2, 1.0)

        if game.IsKeyJustPressed(goud.KeyEscape) {
            game.Close()
        }

        game.EndFrame()
    }
}
```

Run it:

```bash
CGO_ENABLED=1 go run main.go
```

A window opens at 800x600 and closes when you press Escape.

`BeginFrame` clears the screen to the given RGBA color and prepares the frame. `EndFrame` swaps buffers and polls events. Everything you draw goes between those two calls.

## Drawing a Sprite

Load textures once before the game loop, then draw each frame.

```go
package main

import "github.com/aram-devdocs/goud-engine-go/goud"

func main() {
    game := goud.NewGame(800, 600, "Sprite Demo")
    defer game.Destroy()

    tex := game.LoadTexture("assets/player.png")

    for !game.ShouldClose() {
        game.BeginFrame(0.1, 0.1, 0.1, 1.0)

        game.DrawSprite(tex, 400, 300, 64, 64, 0, goud.ColorWhite())

        if game.IsKeyJustPressed(goud.KeyEscape) {
            game.Close()
        }

        game.EndFrame()
    }
}
```

`DrawSprite` takes the center position of the sprite, width, height, rotation in radians, and a color tint.

Put your image files in an `assets/` folder next to your project. The path is relative to the working directory.

## Handling Input

### Keyboard

Two modes are available: just pressed this frame, or held continuously.

```go
// One-shot: true only on the frame the key goes down
if game.IsKeyJustPressed(goud.KeySpace) {
    // jump
}

// Held: true every frame the key is down
if game.IsKeyPressed(goud.KeyW) {
    y -= speed * game.DeltaTime()
}
if game.IsKeyPressed(goud.KeyS) {
    y += speed * game.DeltaTime()
}
```

`DeltaTime()` is the elapsed seconds since the last frame. Use it to make movement frame-rate independent.

Common key constants: `goud.KeyEscape`, `goud.KeySpace`, `goud.KeyEnter`, `goud.KeyW`, `goud.KeyA`, `goud.KeyS`, `goud.KeyD`, `goud.KeyLeft`, `goud.KeyRight`, `goud.KeyUp`, `goud.KeyDown`.

### Mouse

```go
if game.IsMouseButtonPressed(goud.MouseButtonLeft) {
    // click action
}

mouseX := game.MouseX()
mouseY := game.MouseY()
```

Mouse button constants: `goud.MouseButtonLeft`, `goud.MouseButtonRight`, `goud.MouseButtonMiddle`.

## Available Types

| Import | Description |
|--------|-------------|
| `goud.NewGame` | Create a windowed game instance |
| `goud.Color` | RGBA color (`goud.ColorWhite()`, `goud.ColorRGB(r, g, b)`) |
| `goud.Vec2` | 2D vector with arithmetic methods |
| `goud.Vec3` | 3D vector |
| `goud.Rect` | Rectangle (x, y, width, height) |
| `goud.Transform2D` | 2D position, rotation, scale |
| `goud.EntityID` | ECS entity handle |

## Running an Example Game

The repository includes a complete Flappy Bird clone in Go:

```bash
git clone https://github.com/aram-devdocs/GoudEngine.git
cd GoudEngine
cargo build --release
./dev.sh --sdk go --game flappy_bird
```

`dev.sh` builds the native library and launches the example. Source is in [`examples/go/flappy_bird/`](https://github.com/aram-devdocs/GoudEngine/tree/main/examples/go/flappy_bird/).

## Code Generation

All files in `sdks/go/goud/` and `sdks/go/internal/ffi/helpers.go` are auto-generated. Do not hand-edit them. Regenerate with:

```bash
python3 codegen/gen_go_sdk.py    # Wrapper package
python3 codegen/gen_go.py        # Internal cgo bindings
```

## Next Steps

- [Go SDK README](https://github.com/aram-devdocs/GoudEngine/tree/main/sdks/go/) -- full architecture and API overview
- [Go examples source](https://github.com/aram-devdocs/GoudEngine/tree/main/examples/go/) -- complete game source code
- [Build Your First Game](../guides/build-your-first-game.md) -- end-to-end minimal game walkthrough
- [Example Showcase](../guides/showcase.md) -- current cross-language parity matrix
- [SDK-first architecture](../architecture/sdk-first.md) -- how the engine layers fit together
- [Development guide](../development/guide.md) -- building from source, version management, git hooks
- Other getting started guides: [C#](csharp.md) -- [Python](python.md) -- [TypeScript](typescript.md) -- [Rust](rust.md) -- [Swift](swift.md) -- [Kotlin](kotlin.md) -- [Lua](lua.md)
