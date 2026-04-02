# Python Examples

Python examples for the GoudEngine SDK live in this directory.

## Examples

| File | Purpose |
|------|---------|
| `main.py` | Basic SDK demo (window lifecycle, input polling, `Transform2D`, `Sprite`) |
| `flappy_bird.py` | Playable Flappy Bird sample with texture loading, sprite drawing, audio, and collision logic |
| `sandbox.py` | Interactive parity sandbox covering 2D, 3D, diagnostics, UI widgets, and localhost networking state |
| `feature_lab.py` | Headless feature smoke: scenes, ECS component roundtrip, network/UI wrappers, and provider-safe fallbacks |

## Prerequisites

1. Build the native library:
   ```bash
   cd goudengine
   cargo build --release
   ```
2. Use Python 3.8 or newer.

The SDK bindings are loaded from `sdks/python/goudengine` and use `ctypes`.

## Run

From repo root:

```bash
./dev.sh --sdk python --game python_demo
./dev.sh --sdk python --game flappy_bird
./dev.sh --sdk python --game sandbox
python3 examples/python/feature_lab.py
```

Or run directly:

```bash
cd examples/python
python main.py
python flappy_bird.py
python sandbox.py
python feature_lab.py
```

## What These Examples Currently Exercise

- Window creation and frame loop (`GoudGame`, `begin_frame`, `end_frame`, `should_close`)
- Keyboard and mouse input (`Key`, `MouseButton`)
- Texture loading and immediate-mode sprite rendering (`load_texture`, `draw_sprite`)
- Audio playback from in-memory bytes (`audio_play`)
- SDK value types (`Transform2D`, `Sprite`, `Vec2`, `Color`)
- Headless scene + ECS checks (`GoudContext`, `scene_create`, `scene_set_current`, `add/get/set/remove_transform2d`)
- Network/UI wrapper access with provider-safe fallback reporting (`NetworkManager`, `UiManager`, capability queries)
- Interactive parity walkthrough with shared sandbox assets and mixed 2D/3D runtime behavior

## API Snippets (Current Names)

### Frame Loop and Input

```python
from goudengine import GoudGame, Key, MouseButton

game = GoudGame(800, 600, "My Game")

while not game.should_close():
    game.begin_frame()

    if game.is_key_just_pressed(Key.ESCAPE):
        game.close()

    if game.is_mouse_button_just_pressed(MouseButton.LEFT):
        pos = game.get_mouse_position()
        print(pos.x, pos.y)

    game.end_frame()

game.destroy()
```

### Transform2D and Sprite

```python
from goudengine import Transform2D, Sprite
import math

t = Transform2D.from_position(100, 50)
t.rotate(math.pi / 4)
t.scale_by(2, 2)

sprite = (
    Sprite(texture_handle=42)
    .with_color(1.0, 0.0, 0.0, 1.0)
    .with_flip_x(True)
    .with_anchor(0.5, 1.0)
)
```

## Cross-Language Notes

- `flappy_bird.py` intentionally mirrors the C# Flappy Bird example structure (`examples/csharp/flappy_goud/`) for parity testing.
- The examples are not a claim that every SDK surface is identical.
- Engine behavior comes from the Rust core; the Python SDK calls into that core through FFI.
