# Getting Started — Rust SDK

The Rust SDK links directly against the engine with no FFI overhead. It re-exports
`goud_engine::sdk::*` from a single crate, so all engine types are available through
`use goudengine::*;`.

Other SDKs: [C#](csharp.md) · [Python](python.md) · [TypeScript](typescript.md)

---

## Prerequisites

### Rust toolchain

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup update stable
```

### System dependencies

Linux:
```bash
sudo apt-get install libglfw3-dev libgl1-mesa-dev
```

macOS:
```bash
brew install glfw
# OpenGL is provided by the OS — no extra package needed.
```

Windows: Install GLFW via [vcpkg](https://vcpkg.io/) or download the pre-built
binaries from [glfw.org](https://www.glfw.org/download.html).

---

## Installation

Create a new project and add the dependency:

```bash
cargo new my-game
cd my-game
cargo add goud-engine
```

Or set the version directly in `Cargo.toml`:

```toml
[package]
name = "my-game"
version = "0.1.0"
edition = "2021"

[dependencies]
goud-engine = "0.0.825"
```

---

## First Project

This opens a window, clears it to a blue-grey color each frame, and exits when
the window is closed.

{{#include ../generated/snippets/rust/first-project.md}}

`GameConfig::new` takes window title, width, and height. `GoudGame::with_platform`
creates the window and OpenGL context. `poll_events` returns the elapsed time in
seconds since the last frame, which you use to scale physics and animations.

---

## Drawing a Sprite

Load a texture once before the loop, then call `draw_sprite` each frame.

{{#include ../generated/snippets/rust/drawing-a-sprite.md}}

Positions are in pixels from the top-left corner. The `center_x`/`center_y`
arguments are the sprite's center, not its top-left corner.

---

## Handling Input

Query key state inside the game loop with `is_key_pressed`.

{{#include ../generated/snippets/rust/handling-input.md}}

`is_key_pressed` returns `true` as long as the key is held down. Mouse buttons
use `is_mouse_button_pressed(MouseButton::Button1)`.

---

## Running the Example Game

The repository includes a complete Flappy Bird clone in `examples/rust/flappy_bird/`.
It demonstrates texture loading, sprite drawing, physics, collision detection, and
input handling across multiple modules.

```bash
git clone https://github.com/aram-devdocs/GoudEngine.git
cd GoudEngine
cargo run -p flappy-bird
cargo run -p feature-lab
```

Controls: Space or left click to flap, R to restart, Escape to quit.

The example must be run from the repository root so asset paths resolve correctly.
The game reuses the shared asset directory at `examples/csharp/flappy_goud/assets/`.

---

## Next Steps

- [Rust SDK README](https://github.com/aram-devdocs/GoudEngine/tree/main/sdks/rust/) — crate design and re-export structure
- [Rust examples](https://github.com/aram-devdocs/GoudEngine/tree/main/examples/rust/) — flappy_bird source code
- [Build Your First Game](../guides/build-your-first-game.md) — end-to-end minimal game walkthrough
- [Example Showcase](../guides/showcase.md) — current cross-language parity matrix
- [Cross-Platform Deployment](../guides/deployment.md) — packaging and release workflow
- [FAQ and Troubleshooting](../guides/faq.md) — common runtime and build issues
- [Architecture](../architecture/sdk-first.md) — layer design and engine internals
- [Development guide](../development/guide.md) — building from source, running tests
