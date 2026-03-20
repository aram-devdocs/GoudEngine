# GoudEngine Lua SDK

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

> **Alpha** -- This SDK is under active development. APIs change frequently. [Report issues](https://github.com/aram-devdocs/GoudEngine/issues)

Lua scripting SDK for GoudEngine. Write 2D game logic in Lua while the Rust engine handles rendering, physics, audio, and asset management.

## Architecture

Unlike the Python and C# SDKs which use FFI/ctypes to call into the engine, the Lua SDK is **embedded** inside the Rust engine via [mlua](https://github.com/khonsulabs/mlua). Lua scripts run inside the engine process and communicate through registered Lua globals (`goud_game`, `key`, etc.) rather than through a C ABI boundary.

```
Lua script  -->  mlua (embedded)  -->  Rust engine core
```

The codegen at `codegen/gen_lua.py` generates Rust-side mlua bindings (`*.g.rs`) that expose engine functionality to the Lua VM.

## Installation

### LuaRocks (recommended)

```bash
luarocks install goudengine
```

The LuaRocks package bundles the native engine library and the Lua runner binary.

### Build from source

```bash
# 1. Build the Rust engine with Lua support
cargo build --release -p lua-runner

# 2. Run a Lua game script
cargo run -p lua-runner -- examples/lua/flappy_bird/main.lua
```

## Quick Start

GoudEngine Lua games define three callback functions that the engine calls each frame:

```lua
-- main.lua

function on_init()
    -- Load assets, set up initial state.
    tex = goud_game.texture_load("assets/player.png")
    x, y = 400, 300
end

function on_update(dt)
    -- Game logic runs here every frame.
    if goud_game.input_key_just_pressed(key.escape) then
        goud_game.close()
        return
    end

    local speed = 200
    if goud_game.input_key_pressed(key.w) then y = y - speed * dt end
    if goud_game.input_key_pressed(key.s) then y = y + speed * dt end
    if goud_game.input_key_pressed(key.a) then x = x - speed * dt end
    if goud_game.input_key_pressed(key.d) then x = x + speed * dt end
end

function on_draw()
    -- Render your game world.
    goud_game.window_clear(0.2, 0.2, 0.3, 1.0)
    goud_game.draw_sprite(tex, x, y, 64, 64, 0, 1, 1, 1, 1)
end
```

Run it:

```bash
cargo run -p lua-runner -- main.lua
```

## API Overview

### Globals

| Global | Description |
|--------|-------------|
| `goud_game` | Engine interface: rendering, input, audio, assets |
| `key` | Keyboard key constants (e.g. `key.space`, `key.escape`) |

### goud_game Methods

| Method | Description |
|--------|-------------|
| `texture_load(path)` | Load a texture, returns a handle |
| `draw_sprite(tex, x, y, w, h, rot, r, g, b, a)` | Draw a sprite |
| `window_clear(r, g, b, a)` | Clear the screen with a color |
| `input_key_pressed(k)` | Check if a key is currently held |
| `input_key_just_pressed(k)` | Check if a key was pressed this frame |
| `close()` | Signal the game to exit |

### Callbacks

| Callback | Description |
|----------|-------------|
| `on_init()` | Called once at startup |
| `on_update(dt)` | Called every frame with delta time in seconds |
| `on_draw()` | Called every frame for rendering |

## Project Structure

A typical Lua game project:

```
my_game/
  constants.lua    -- Shared constants (loaded before main.lua)
  main.lua         -- Entry point with on_init, on_update, on_draw
  assets/
    sprites/       -- PNG images
    audio/         -- WAV/OGG files
```

All `.lua` files in the script directory are loaded alphabetically, with `main.lua` always loaded last. This lets you split constants and helpers into separate files.

## Hot Reload

The Lua runner watches the script directory for changes. When you save a `.lua` file, the engine reloads all scripts automatically. No restart needed during development.

## Features

- 2D rendering with sprites, text, and sprite sheet animation
- Keyboard and mouse input
- Audio playback
- Physics simulation (Rapier2D)
- Asset hot-reloading during development
- ECS entity/component management from Lua
- All game logic runs in Lua; performance-critical code stays in Rust

## Examples

- [Flappy Bird](https://github.com/aram-devdocs/GoudEngine/tree/main/examples/lua/flappy_bird) -- Full game clone demonstrating sprites, input, collision, and scoring

## Links

- [Repository](https://github.com/aram-devdocs/GoudEngine)
- [Examples](https://github.com/aram-devdocs/GoudEngine/tree/main/examples/lua)
- [License: MIT](https://github.com/aram-devdocs/GoudEngine/blob/main/LICENSE)
