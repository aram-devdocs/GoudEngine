# GoudEngine Python SDK

[![PyPI](https://img.shields.io/pypi/v/goudengine.svg)](https://pypi.org/project/goudengine/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

> **Alpha** — This SDK is under active development. APIs change frequently. [Report issues](https://github.com/aram-devdocs/GoudEngine/issues) · [Contact](mailto:aram.devdocs@gmail.com)

Python SDK for GoudEngine. Build 2D and 3D games powered by a Rust core.

## Install

```bash
pip install goudengine
```

## Quick Start

```python
from goud_engine import GoudGame, Key

game = GoudGame(800, 600, "My Game")

player_tex = game.load_texture("assets/player.png")

while not game.should_close():
    game.begin_frame()
    dt = game.delta_time

    if game.is_key_just_pressed(Key.ESCAPE):
        game.close()

    game.draw_sprite(player_tex, 400, 300, 64, 64)

game.end_frame()

game.destroy()
```

## Networking Wrapper API

Use constructor-based wrappers over `GoudGame` or `GoudContext`:

```python
from goud_engine import GoudContext, NetworkManager, NetworkProtocol

host_ctx = GoudContext()
client_ctx = GoudContext()

host_net = NetworkManager(host_ctx)
client_net = NetworkManager(client_ctx)

host = host_net.host(NetworkProtocol.TCP, 40000)  # no default peer ID
client = client_net.connect(NetworkProtocol.TCP, "127.0.0.1", 40000)  # default peer ID is set

client.send(b"hello")            # uses default peer ID from connect()
packet = host.receive()          # Optional[NetworkPacket]
if packet is not None:
    host.send_to(packet.peer_id, b"world")

host.disconnect()
client.disconnect()
host_ctx.destroy()
client_ctx.destroy()
```

## Features

- 2D and 3D rendering with runtime renderer selection
- Entity Component System (ECS) with Transform2D, Sprite, and more
- Physics simulation (Rapier2D/3D): rigid bodies, colliders, raycasting, collision events
- Audio playback with per-channel volume (Music, SFX, Ambience, UI, Voice) and spatial audio
- Text rendering with TrueType/bitmap fonts, alignment, and word-wrapping
- Sprite animation with state machine controller, multi-layer blending, and tweening
- Scene management with transitions (instant, fade, custom)
- UI component system with hierarchical node tree
- Tiled map support for 2D worlds
- Input handling (keyboard, mouse)
- Asset hot-reloading during development
- Structured error diagnostics with error codes and recovery hints

## Flappy Bird Example

Here's a condensed version of the [complete Flappy Bird example](https://github.com/aram-devdocs/GoudEngine/tree/main/examples/python/flappy_bird.py):

```python
import math
import random
from goud_engine import GoudGame, Key, MouseButton

# Constants
SCREEN_W, SCREEN_H = 288, 512
GRAVITY = 9.8
JUMP_STRENGTH = -3.5
PIPE_SPEED = 1.0
PIPE_SPAWN_INTERVAL = 1.5
PIPE_GAP = 100
TARGET_FPS = 120

game = GoudGame(SCREEN_W, SCREEN_H + 112, "Flappy Bird")

# Load textures
bg_tex   = game.load_texture("assets/sprites/background-day.png")
bird_frames = [
    game.load_texture("assets/sprites/bluebird-downflap.png"),
    game.load_texture("assets/sprites/bluebird-midflap.png"),
    game.load_texture("assets/sprites/bluebird-upflap.png"),
]
pipe_tex  = game.load_texture("assets/sprites/pipe-green.png")
base_tex  = game.load_texture("assets/sprites/base.png")
digit_tex = [game.load_texture(f"assets/sprites/{i}.png") for i in range(10)]

# Bird state
bird_x, bird_y = SCREEN_W / 4, SCREEN_H / 2
velocity = 0.0
rotation = 0.0
frame_idx = 0
frame_timer = 0.0
jump_cooldown = 0.0

# Pipe state
pipes = []          # list of dicts: {x, top_y, bottom_y, counted}
pipe_timer = 0.0
score = 0

def reset():
    global bird_x, bird_y, velocity, rotation, frame_idx, frame_timer
    global jump_cooldown, pipes, pipe_timer, score
    bird_x, bird_y = SCREEN_W / 4, SCREEN_H / 2
    velocity = rotation = frame_idx = frame_timer = jump_cooldown = 0.0
    pipes.clear()
    pipe_timer = score = 0

def spawn_pipe():
    gap_y = random.randint(PIPE_GAP, SCREEN_H - PIPE_GAP)
    pipes.append({
        "x": SCREEN_W,
        "top_y": gap_y - PIPE_GAP - 320,   # 320 = pipe image height
        "bottom_y": gap_y + PIPE_GAP,
        "counted": False,
    })

def aabb(ax, ay, aw, ah, bx, by, bw, bh):
    return ax < bx + bw and ax + aw > bx and ay < by + bh and ay + ah > by

reset()

while not game.should_close():
    game.begin_frame()
    dt = game.delta_time

    # --- Input ---
    if game.is_key_just_pressed(Key.ESCAPE):
        game.close()
    if game.is_key_just_pressed(Key.R):
        reset()

    jump = (game.is_key_just_pressed(Key.SPACE) or
            game.is_mouse_button_just_pressed(MouseButton.LEFT))
    if jump and jump_cooldown <= 0:
        velocity = JUMP_STRENGTH * TARGET_FPS
        jump_cooldown = 0.30
    jump_cooldown = max(0.0, jump_cooldown - dt)

    # --- Physics ---
    velocity += GRAVITY * dt * TARGET_FPS
    bird_y += velocity * dt
    target_rot = max(-45, min(45, velocity * 3))
    rotation += (target_rot - rotation) * 0.03

    # --- Bird animation ---
    frame_timer += dt
    if frame_timer >= 0.1:
        frame_idx = (frame_idx + 1) % 3
        frame_timer = 0.0

    # --- Pipes ---
    pipe_timer += dt
    if pipe_timer >= PIPE_SPAWN_INTERVAL:
        spawn_pipe()
        pipe_timer = 0.0

    survived = []
    for p in pipes:
        p["x"] -= PIPE_SPEED * dt * TARGET_FPS
        if p["x"] + 52 < 0:            # pipe scrolled off screen
            score += 1
            continue
        if (aabb(bird_x, bird_y, 34, 24, p["x"], p["top_y"],    52, 320) or
            aabb(bird_x, bird_y, 34, 24, p["x"], p["bottom_y"], 52, 320) or
            bird_y < 0 or bird_y > SCREEN_H):
            reset()
            break
        survived.append(p)
    else:
        pipes = survived

    # --- Draw ---
    game.draw_sprite(bg_tex,   SCREEN_W / 2, SCREEN_H / 2, SCREEN_W, SCREEN_H)

    for p in pipes:
        game.draw_sprite(pipe_tex, p["x"] + 26, p["top_y"]    + 160, 52, 320, math.pi)
        game.draw_sprite(pipe_tex, p["x"] + 26, p["bottom_y"] + 160, 52, 320)

    game.draw_sprite(
        bird_frames[frame_idx],
        bird_x + 17, bird_y + 12, 34, 24,
        math.radians(rotation)
    )

    # Score digits
    digits = [int(d) for d in str(max(score, 0))]
    start_x = (SCREEN_W - len(digits) * 24) / 2 + 12
    for i, d in enumerate(digits):
        game.draw_sprite(digit_tex[d], start_x + i * 24, 50, 24, 36)

    game.draw_sprite(base_tex, SCREEN_W / 2, SCREEN_H + 56, SCREEN_W, 112)

    game.end_frame()

game.destroy()
```

## API Overview

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

## Platform Support

| OS | Architecture | Status |
|----|-------------|--------|
| Windows | x64 | Supported |
| macOS | x64 | Supported |
| macOS | ARM64 (Apple Silicon) | Supported |
| Linux | x64 | Supported |

Native libraries are bundled in the PyPI package.

## Development

For contributors building from source:

```bash
cargo build --release
python3 sdks/python/test_bindings.py
```

## Links

- [Repository](https://github.com/aram-devdocs/GoudEngine)
- [Examples](https://github.com/aram-devdocs/GoudEngine/tree/main/examples/python)
- [License: MIT](https://github.com/aram-devdocs/GoudEngine/blob/main/LICENSE)
