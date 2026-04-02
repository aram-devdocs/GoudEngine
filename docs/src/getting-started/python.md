# Getting Started: Python SDK

> **Alpha** — APIs change frequently. [Report issues](https://github.com/aram-devdocs/GoudEngine/issues)

This guide covers installing the Python SDK, opening a window, drawing a sprite, and handling input.

See also: [C# guide](csharp.md) · [TypeScript guide](typescript.md) · [Rust guide](rust.md) · [Go guide](go.md) · [Kotlin guide](kotlin.md) · [Lua guide](lua.md)

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

{{#include ../generated/snippets/python/first-project.md}}

Run it:

```bash
python main.py
```

A window opens at 800x600 and closes when you press Escape.

`begin_frame()` polls events and clears the screen. `end_frame()` presents the frame. Everything you draw goes between those two calls.

## Debugger Runtime

Enable debugger mode before creating the headless context:

```python
from goudengine import (
    GoudContext,
)
from goudengine.generated._types import ContextConfig, DebuggerConfig

ctx = GoudContext(
    ContextConfig(
        debugger=DebuggerConfig(
            enabled=True,
            publish_local_attach=True,
            route_label="getting-started-python",
        )
    )
)

ctx.set_debugger_profiling_enabled(True)
snapshot_json = ctx.get_debugger_snapshot_json()
manifest_json = ctx.get_debugger_manifest_json()
ctx.destroy()
```

For a ready-made headless route, run `python3 examples/python/feature_lab.py`.
The example publishes `feature-lab-python-headless`, confirms manifest and
snapshot access, and prints the manual attach steps:

1. start `cargo run -p goudengine-mcp`
2. call `goudengine.list_contexts`
3. call `goudengine.attach_context`

## Drawing a Sprite

Load textures once before the game loop, then draw each frame.

{{#include ../generated/snippets/python/drawing-a-sprite.md}}

`draw_sprite` takes the center position of the sprite, not the top-left corner.

An optional sixth argument sets rotation in radians:

```python
import math
game.draw_sprite(player_tex, 400, 300, 64, 64, math.pi / 4)
```

## Handling Input

### Keyboard

Two modes are available: pressed this frame, or held continuously.

{{#include ../generated/snippets/python/keyboard.md}}

`delta_time` is the elapsed seconds since the last frame. Use it to make movement frame-rate independent.

Common key constants: `Key.ESCAPE`, `Key.SPACE`, `Key.ENTER`, `Key.W`, `Key.A`, `Key.S`, `Key.D`, `Key.LEFT`, `Key.RIGHT`, `Key.UP`, `Key.DOWN`.

### Mouse

{{#include ../generated/snippets/python/mouse.md}}

Mouse button constants: `MouseButton.LEFT`, `MouseButton.RIGHT`, `MouseButton.MIDDLE`.

## Running an Example Game

The repository includes a complete Flappy Bird clone in Python. Clone the repo and run it with `dev.sh`:

```bash
git clone https://github.com/aram-devdocs/GoudEngine.git
cd GoudEngine
./dev.sh --sdk python --game python_demo    # Basic demo
./dev.sh --sdk python --game flappy_bird    # Flappy Bird clone
./dev.sh --sdk python --game sandbox        # Full feature sandbox
python3 examples/python/feature_lab.py      # Supplemental smoke coverage
```

`dev.sh` builds the native library and launches the example. It requires a Rust toolchain (`cargo`) to be installed.

### Running Examples from Source (Without dev.sh)

If you have the repository checked out and the native library built, add the SDK path manually:

```python
import sys
from pathlib import Path

sdk_path = Path(__file__).parent.parent.parent / "sdks" / "python"
sys.path.insert(0, str(sdk_path))

from goudengine import GoudGame, Key
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

- [Python examples](https://github.com/aram-devdocs/GoudEngine/tree/main/examples/python/) — source code for `main.py`, `flappy_bird.py`, and `sandbox.py`
- [Python SDK README](https://github.com/aram-devdocs/GoudEngine/tree/main/sdks/python/) — full API reference
- [Build Your First Game](../guides/build-your-first-game.md) — end-to-end minimal game walkthrough
- [Debugger Runtime](../guides/debugger-runtime.md) — local attach, capture, replay, and metrics workflow
- [Example Showcase](../guides/showcase.md) — current cross-language parity matrix
- [Cross-Platform Deployment](../guides/deployment.md) — packaging and release workflow
- [FAQ and Troubleshooting](../guides/faq.md) — common runtime and build issues
- [Architecture overview](../architecture/sdk-first.md) — how the Rust core and Python SDK connect
- [Development guide](../development/guide.md) — building from source, running tests
