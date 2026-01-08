# GoudEngine Python SDK

Python bindings for the GoudEngine game engine, providing a Pythonic interface
to the Rust core through ctypes FFI.

## Design Philosophy

**All logic lives in Rust.** This SDK is a thin wrapper that marshals data and
calls FFI functions. This ensures consistent behavior across all language
bindings (C#, Python, Rust native).

## Installation

### Prerequisites

1. Build the GoudEngine native library:
   ```bash
   cd goud_engine
   cargo build --release
   ```

2. The Python SDK will automatically locate the library in:
   - `target/release/libgoud_engine.dylib` (macOS)
   - `target/release/libgoud_engine.so` (Linux)
   - `target/release/goud_engine.dll` (Windows)

### Using the SDK

```python
# Add the SDK to your Python path
import sys
sys.path.insert(0, '/path/to/GoudEngine/sdks/python')

from goud_engine import GoudContext, Transform2D, Sprite, Vec2
```

## Quick Start

### Context Management

```python
from goud_engine import GoudContext

# Create a context (manages an ECS world)
ctx = GoudContext.create()

# Spawn entities
entity_id = ctx.spawn_entity()
print(f"Spawned entity: {entity_id}")

# Check entity status
print(f"Entity alive: {ctx.is_entity_alive(entity_id)}")
print(f"Entity count: {ctx.entity_count()}")

# Despawn when done
ctx.despawn_entity(entity_id)

# Clean up
ctx.destroy()
```

### Using Context Manager

```python
from goud_engine import GoudContext

with GoudContext.create() as ctx:
    entity_id = ctx.spawn_entity()
    # Context automatically destroyed when exiting the block
```

### Transform2D Component

```python
import math
from goud_engine import Transform2D, Vec2

# Factory methods (all delegate to Rust FFI)
transform = Transform2D.from_position(100, 50)
transform = Transform2D.from_rotation(math.pi / 4)  # 45 degrees
transform = Transform2D.look_at(0, 0, 100, 100)  # Position looking at target

# Mutation methods (chainable)
transform = Transform2D()
transform.translate(10, 20).rotate(0.5).scale_by(2, 2)

# Direction vectors
forward = transform.forward()  # Returns Vec2
right = transform.right()

# Point transformation
world_point = transform.transform_point(5, 5)  # Local to world
local_point = transform.inverse_transform_point(105, 55)  # World to local

# Interpolation
transform_a = Transform2D.from_position(0, 0)
transform_b = Transform2D.from_position(100, 100)
mid = transform_a.lerp(transform_b, 0.5)  # Midpoint
```

### Sprite Component

```python
from goud_engine import Sprite, Color

# Create a sprite
sprite = Sprite(texture_handle=42)

# Builder pattern (returns new instances)
sprite = (Sprite(texture_handle=42)
    .with_color(1.0, 0.0, 0.0, 1.0)  # Red tint
    .with_flip_x(True)
    .with_anchor(0.5, 1.0)  # Bottom-center
    .with_source_rect(0, 0, 32, 32)  # Sprite sheet
    .with_custom_size(64, 64))  # Scale to 64x64

# Mutable operations (modify in place)
sprite.color = Color.red()
sprite.flip_x = True
sprite.set_source_rect(32, 0, 32, 32)  # Next frame
```

### High-Level Game API

```python
from goud_engine import GoudGame, Transform2D, Sprite

# Create game (placeholder - full implementation coming)
game = GoudGame(800, 600, "My Game")

# Spawn entities
entity = game.spawn()

# Batch spawn
entities = game.spawn_batch(100)

# Clean up
game.close()
```

## API Reference

### Types

| Type | Description |
|------|-------------|
| `GoudContext` | Engine context managing an ECS world |
| `GoudResult` | FFI result type for operations that can fail |
| `GoudEntityId` | FFI entity identifier |
| `Vec2` | 2D vector |
| `Color` | RGBA color |
| `Rect` | 2D rectangle |
| `Transform2D` | 2D transformation component |
| `Sprite` | 2D sprite rendering component |
| `Entity` | High-level entity wrapper |
| `GoudGame` | High-level game abstraction |

### GoudContext Methods

| Method | Description |
|--------|-------------|
| `create()` | Creates a new context |
| `destroy()` | Destroys the context |
| `is_valid()` | Checks if context is valid |
| `spawn_entity()` | Spawns an empty entity |
| `spawn_entities(count)` | Spawns multiple entities |
| `despawn_entity(id)` | Despawns an entity |
| `is_entity_alive(id)` | Checks if entity is alive |
| `entity_count()` | Returns alive entity count |

### Transform2D Methods

| Method | Description |
|--------|-------------|
| `from_position(x, y)` | Factory: position |
| `from_rotation(radians)` | Factory: rotation |
| `from_scale(sx, sy)` | Factory: scale |
| `look_at(px, py, tx, ty)` | Factory: look at target |
| `translate(dx, dy)` | Translate in world space |
| `translate_local(dx, dy)` | Translate in local space |
| `rotate(radians)` | Rotate by angle |
| `scale_by(fx, fy)` | Multiply scale |
| `forward()` | Get forward direction |
| `right()` | Get right direction |
| `transform_point(x, y)` | Local to world |
| `inverse_transform_point(x, y)` | World to local |
| `lerp(other, t)` | Interpolate |

### Sprite Methods

| Method | Description |
|--------|-------------|
| `with_color(r, g, b, a)` | Builder: color tint |
| `with_flip_x(flip)` | Builder: horizontal flip |
| `with_flip_y(flip)` | Builder: vertical flip |
| `with_anchor(x, y)` | Builder: anchor point |
| `with_source_rect(x, y, w, h)` | Builder: sprite sheet rect |
| `with_custom_size(w, h)` | Builder: render size |
| `set_source_rect(...)` | Mutate source rect |
| `clear_source_rect()` | Clear source rect |
| `set_custom_size(...)` | Mutate size |
| `clear_custom_size()` | Clear size |

## Error Handling

```python
from goud_engine import GoudContext, GoudEngineError

try:
    ctx = GoudContext.create()
    # ... operations
except GoudEngineError as e:
    print(f"Engine error: {e}")
```

## Thread Safety

- Context creation/destruction is thread-safe
- Individual contexts are NOT thread-safe
- Use one context per thread, or synchronize access

## Building from Source

```bash
# Build the native library
cd goud_engine
cargo build --release

# The Python SDK automatically finds the library in target/release/
```

## License

MIT License - same as GoudEngine core.
