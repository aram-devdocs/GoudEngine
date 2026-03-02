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

```rust
use goudengine::{GameConfig, GoudGame};

fn main() {
    let config = GameConfig::new("My Game", 800, 600);
    let mut game = GoudGame::with_platform(config).expect("Failed to create game");

    game.enable_blending();

    while !game.should_close() {
        let _dt = game.poll_events().unwrap_or(0.016);
        game.begin_render();
        game.clear(0.2, 0.3, 0.4, 1.0); // RGBA: dark blue-grey

        game.end_render();
        game.swap_buffers().expect("swap_buffers failed");
    }
}
```

`GameConfig::new` takes window title, width, and height. `GoudGame::with_platform`
creates the window and OpenGL context. `poll_events` returns the elapsed time in
seconds since the last frame, which you use to scale physics and animations.

---

## Drawing a Sprite

Load a texture once before the loop, then call `draw_sprite` each frame.

```rust
use goudengine::{GameConfig, GoudGame};

fn main() {
    let config = GameConfig::new("Sprite Demo", 800, 600);
    let mut game = GoudGame::with_platform(config).expect("Failed to create game");

    game.enable_blending();

    // Load returns an opaque u64 handle. Keep it for the lifetime of the game.
    let texture = game.load("assets/sprite.png");

    while !game.should_close() {
        let _dt = game.poll_events().unwrap_or(0.016);
        game.begin_render();
        game.clear(0.2, 0.3, 0.4, 1.0);

        // draw_sprite(texture, center_x, center_y, width, height,
        //             rotation_radians, scale_x, scale_y, r, g, b, a)
        game.draw_sprite(
            texture,
            400.0, 300.0, // center position
            64.0,  64.0,  // size
            0.0,          // rotation (radians)
            1.0, 1.0,     // scale
            1.0, 1.0, 1.0, 1.0, // color tint (white = no tint)
        );

        game.end_render();
        game.swap_buffers().expect("swap_buffers failed");
    }
}
```

Positions are in pixels from the top-left corner. The `center_x`/`center_y`
arguments are the sprite's center, not its top-left corner.

---

## Handling Input

Query key state inside the game loop with `is_key_pressed`.

```rust
use goudengine::{GameConfig, GoudGame};
use goudengine::input::Key;

fn main() {
    let config = GameConfig::new("Input Demo", 800, 600);
    let mut game = GoudGame::with_platform(config).expect("Failed to create game");

    game.enable_blending();

    let mut x = 400.0_f32;

    while !game.should_close() {
        let dt = game.poll_events().unwrap_or(0.016);
        game.begin_render();
        game.clear(0.2, 0.3, 0.4, 1.0);

        if game.is_key_pressed(Key::Escape) {
            break;
        }
        if game.is_key_pressed(Key::Left) {
            x -= 200.0 * dt;
        }
        if game.is_key_pressed(Key::Right) {
            x += 200.0 * dt;
        }

        game.end_render();
        game.swap_buffers().expect("swap_buffers failed");
    }
}
```

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
```

Controls: Space or left click to flap, R to restart, Escape to quit.

The example must be run from the repository root so asset paths resolve correctly.
The game reuses the shared asset directory at `examples/csharp/flappy_goud/assets/`.

---

## Next Steps

- [Rust SDK README](../../../sdks/rust/README.md) — crate design and re-export structure
- [Rust examples](../../../examples/rust/) — flappy_bird source code
- [Architecture](../architecture/sdk-first.md) — layer design and engine internals
- [Development guide](../development/guide.md) — building from source, running tests
