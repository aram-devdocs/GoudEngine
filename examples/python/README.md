# Python Demo - GoudEngine

This directory contains Python examples demonstrating the GoudEngine Python SDK.

## Prerequisites

1. **Build the native library:**
   ```bash
   cd goud_engine
   cargo build --release
   ```

2. **Python 3.8+** with no additional dependencies (uses stdlib `ctypes`)

## Running the Demos

### Basic SDK Demo
```bash
cd examples/python
python main.py
```

This demonstrates:
- Transform2D component operations
- Sprite component builder patterns
- Context and entity management
- Windowed game with input handling

### Flappy Bird Game
```bash
cd examples/python
python flappy_bird.py
```

A complete Flappy Bird clone demonstrating:
- Game loop architecture
- Physics-based movement
- Collision detection
- Score tracking
- Input handling

**Controls:**
- `SPACE` or `Left Click` - Flap / Jump
- `R` - Restart
- `ESC` - Quit

## Architecture

These demos showcase the **Rust-First SDK Architecture**:

```
┌─────────────────────────────────────────────────────────────┐
│                      Python Game Code                        │
│  (flappy_bird.py, main.py)                                  │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                   Python SDK (goud_engine)                   │
│  Pure bindings - no game logic                               │
│  sdks/python/goud_engine/                                   │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                  Rust FFI Layer (ctypes)                     │
│  goud_window_*, goud_input_*, goud_transform2d_*, etc.      │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                   Rust Engine Core                           │
│  All game logic lives here                                   │
│  goud_engine/src/                                           │
└─────────────────────────────────────────────────────────────┘
```

## API Examples

### Transform2D
```python
from goud_engine import Transform2D, Vec2
import math

# Factory methods
t = Transform2D.from_position(100, 50)
t = Transform2D.from_rotation(math.pi / 4)
t = Transform2D.look_at(0, 0, 100, 100)

# Chained operations
t = Transform2D()
t.translate(10, 20).rotate(0.5).scale_by(2, 2)

# Direction vectors
forward = t.forward()
right = t.right()
```

### Sprite
```python
from goud_engine import Sprite

# Builder pattern
sprite = (Sprite(texture_handle=42)
    .with_color(1.0, 0.0, 0.0, 1.0)  # Red tint
    .with_flip_x(True)
    .with_anchor(0.5, 1.0))  # Bottom-center
```

### Game Loop
```python
from goud_engine import GoudGame, Keys, MouseButtons

game = GoudGame(800, 600, "My Game")

while game.is_running():
    dt = game.begin_frame()
    
    if game.key_just_pressed(Keys.ESCAPE):
        game.close()
    
    if game.key_pressed(Keys.SPACE):
        # Jump!
        pass
    
    if game.mouse_button_pressed(MouseButtons.LEFT):
        pos = game.get_mouse_position()
        # Handle click at pos
    
    # Update game logic...
    
    game.end_frame()

game.destroy()
```

## Comparison with C# Version

The Python Flappy Bird demo mirrors the C# version (`examples/csharp/flappy_goud/`):

| C# Class | Python Class | Purpose |
|----------|--------------|---------|
| `GameConstants` | `GameConstants` | Game configuration |
| `Movement` | `Movement` | Physics & jumping |
| `Bird` | `Bird` | Player character |
| `BirdAnimator` | `BirdAnimator` | Animation state |
| `Pipe` | `Pipe` | Obstacles |
| `ScoreCounter` | `ScoreCounter` | Score tracking |
| `GameManager` | `GameManager` | Game orchestration |

This demonstrates that the same game architecture works across different SDK languages.

## Notes

- **Rendering**: The current FFI exposes window/input/texture operations. Full sprite rendering 
  requires additional FFI bindings (`goud_renderer_draw_sprite`). The demo uses console output 
  to show game state until sprite rendering is implemented.

- **Performance**: While Python is slower than C#/Rust for game logic, the heavy lifting 
  (rendering, physics calculations) happens in Rust. Python is suitable for prototyping 
  and games where update logic isn't the bottleneck.

- **SDK Design**: The Python SDK is a thin wrapper - all component methods delegate to Rust FFI. 
  This ensures consistent behavior across all language bindings.
