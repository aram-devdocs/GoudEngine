# Getting Started: Python SDK

> **Alpha** — APIs change frequently. [Report issues](https://github.com/aram-devdocs/GoudEngine/issues)

This guide covers installing the Python SDK, opening a window, drawing a sprite, and handling input.

See also: [C# guide](csharp.md) · [TypeScript guide](typescript.md) · [Rust guide](rust.md)

## Prerequisites

- Python 3.9 or later
- A supported OS: Windows x64, macOS x64, macOS ARM64, or Linux x64

## Installation

```bash
pip install goudengine
```

The package bundles the native Rust library (`.so`, `.dylib`, or `.dll`). No separate build step is needed when installing from PyPI.

## First Project

Create `main.py`:

```python
from goud_engine import GoudGame, Key

game = GoudGame(800, 600, "My Game")

while not game.should_close():
    game.begin_frame()

    if game.is_key_just_pressed(Key.ESCAPE):
        game.close()

    game.end_frame()

game.destroy()
```

Run it:

```bash
python main.py
```

A window opens at 800x600 and closes when you press Escape.

`begin_frame()` polls events and clears the screen. `end_frame()` presents the frame. Everything you draw goes between those two calls.

## Drawing a Sprite

Load textures once before the game loop, then draw each frame.

```python
from goud_engine import GoudGame, Key

game = GoudGame(800, 600, "My Game")

# Load returns an integer handle. Store it and reuse it every frame.
player_tex = game.load_texture("assets/player.png")

while not game.should_close():
    game.begin_frame()

    if game.is_key_just_pressed(Key.ESCAPE):
        game.close()

    # draw_sprite(texture_id, center_x, center_y, width, height)
    game.draw_sprite(player_tex, 400, 300, 64, 64)

    game.end_frame()

game.destroy()
```

`draw_sprite` takes the center position of the sprite, not the top-left corner.

An optional sixth argument sets rotation in radians:

```python
import math
game.draw_sprite(player_tex, 400, 300, 64, 64, math.pi / 4)
```

## Handling Input

### Keyboard

Two modes are available: pressed this frame, or held continuously.

```python
from goud_engine import GoudGame, Key

game = GoudGame(800, 600, "My Game")

x = 400.0

while not game.should_close():
    game.begin_frame()
    dt = game.delta_time

    # is_key_just_pressed: true only on the frame the key is first pressed
    if game.is_key_just_pressed(Key.ESCAPE):
        game.close()

    # is_key_pressed: true every frame the key is held down
    if game.is_key_pressed(Key.LEFT):
        x -= 200 * dt
    if game.is_key_pressed(Key.RIGHT):
        x += 200 * dt

    game.end_frame()

game.destroy()
```

`delta_time` is the elapsed seconds since the last frame. Use it to make movement frame-rate independent.

Common key constants: `Key.ESCAPE`, `Key.SPACE`, `Key.ENTER`, `Key.W`, `Key.A`, `Key.S`, `Key.D`, `Key.LEFT`, `Key.RIGHT`, `Key.UP`, `Key.DOWN`.

### Mouse

```python
from goud_engine import GoudGame, MouseButton

game = GoudGame(800, 600, "My Game")

while not game.should_close():
    game.begin_frame()

    # is_mouse_button_just_pressed: true on the frame the button is first clicked
    if game.is_mouse_button_just_pressed(MouseButton.LEFT):
        pos = game.get_mouse_position()
        print(f"Click at ({pos.x:.0f}, {pos.y:.0f})")

    game.end_frame()

game.destroy()
```

Mouse button constants: `MouseButton.LEFT`, `MouseButton.RIGHT`, `MouseButton.MIDDLE`.

## Running an Example Game

The repository includes a complete Flappy Bird clone in Python. Clone the repo and run it with `dev.sh`:

```bash
git clone https://github.com/aram-devdocs/GoudEngine.git
cd GoudEngine
./dev.sh --sdk python --game python_demo    # Basic demo
./dev.sh --sdk python --game flappy_bird    # Flappy Bird clone
```

`dev.sh` builds the native library and launches the example. It requires a Rust toolchain (`cargo`) to be installed.

### Running Examples from Source (Without dev.sh)

If you have the repository checked out and the native library built, add the SDK path manually:

```python
import sys
from pathlib import Path

sdk_path = Path(__file__).parent.parent.parent / "sdks" / "python"
sys.path.insert(0, str(sdk_path))

from goud_engine import GoudGame, Key
```

Build the native library first:

```bash
cargo build --release
```

## Available Types

| Import | Description |
|--------|-------------|
| `GoudGame` | Window, game loop, rendering, input |
| `Key` | Keyboard key constants (GLFW values) |
| `MouseButton` | Mouse button constants |
| `Vec2` | 2D vector with arithmetic methods |
| `Color` | RGBA color (`Color.red()`, `Color.from_hex(0xFF0000)`) |
| `Transform2D` | 2D position, rotation, scale |
| `Sprite` | Sprite rendering component |
| `Entity` | ECS entity handle |

## Next Steps

- [Python examples](https://github.com/aram-devdocs/GoudEngine/tree/main/examples/python/) — source code for `main.py` and `flappy_bird.py`
- [Python SDK README](https://github.com/aram-devdocs/GoudEngine/tree/main/sdks/python/) — full API reference
- [Architecture overview](../architecture/sdk-first.md) — how the Rust core and Python SDK connect
- [Development guide](../development/guide.md) — building from source, running tests
