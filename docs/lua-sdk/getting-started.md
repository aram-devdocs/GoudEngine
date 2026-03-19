# Getting Started with the Lua SDK

## Prerequisites

- Rust toolchain (stable, 1.75+)
- GoudEngine source checkout
- A C compiler (for mlua's LuaJIT/Lua compilation)

## Building

The Lua SDK is compiled into the engine when both the `lua` and `native` feature flags are enabled:

```bash
cargo build --features lua,native
```

For a release build:

```bash
cargo build --release --features lua,native
```

## Running a Lua Script

GoudEngine exposes `execute_lua(source, name)` on the `GoudGame` instance. In a Rust host program you would load and run a script like this:

```rust
use goud_engine::sdk::game::GoudGame;
use goud_engine::sdk::game_config::GameConfig;

fn main() {
    let config = GameConfig::default()
        .with_title("Lua Demo")
        .with_size(800, 600);

    let game = GoudGame::with_platform(config).expect("failed to create game");

    let source = std::fs::read_to_string("scripts/main.lua")
        .expect("failed to read script");

    game.execute_lua(&source, "main.lua")
        .expect("Lua script error");
}
```

## Hello World

Create a file `scripts/hello.lua`:

```lua
function on_init()
    print("GoudEngine Lua SDK is running!")
end

function on_update(dt)
    -- dt is the frame delta time in seconds
end

function on_draw()
    -- rendering calls go here
end
```

## Callback Model

The engine calls three global Lua functions each frame if they are defined:

| Callback | When | Arguments |
|----------|------|-----------|
| `on_init()` | Once, after the Lua VM is ready | none |
| `on_update(dt)` | Every frame, before drawing | `dt` -- delta time in seconds (number) |
| `on_draw()` | Every frame, after update | none |

All three are optional. If a callback is not defined the engine silently skips it.

## Loading Assets

Textures and fonts are loaded through the `goud_game` bridge functions:

```lua
local bird_tex
local my_font

function on_init()
    bird_tex = goud_game.texture_load("assets/bird.png")
    my_font  = goud_game.font_load("assets/font.ttf")
end
```

The returned values are integer handles used by other API calls.

## Drawing Text

```lua
function on_draw()
    -- goud_game.draw_text(font_handle, text, x, y, size, r, g, b, a)
    goud_game.draw_text(my_font, "Score: 42", 10, 10, 24, 1.0, 1.0, 1.0, 1.0)
end
```

## Querying Delta Time

```lua
function on_update(dt)
    -- dt is passed as an argument, but you can also query it explicitly:
    local delta = goud_game.delta_time()
    print("frame time: " .. delta)
end
```

## Using Entities

```lua
function on_init()
    -- Spawn an empty entity and get its handle
    local entity = goud_game.spawn_empty()

    -- Check if the entity is alive
    print(goud_game.is_alive(entity))  -- true

    -- Clone an entity
    local clone = goud_game.clone_entity(entity)

    -- Despawn
    goud_game.despawn(entity)
end
```

## Using Collision Helpers

```lua
function on_update(dt)
    -- AABB overlap test
    local hit = goud_game.aabb_overlap(
        0, 0, 32, 32,    -- rect A: x, y, w, h
        20, 20, 32, 32    -- rect B: x, y, w, h
    )
    if hit then
        print("collision!")
    end

    -- Distance between two points
    local d = goud_game.distance(0, 0, 3, 4)  -- returns 5.0
end
```

## Enabling Hot Reload

See [Hot Reload](hot-reload.md) for instructions on live-reloading Lua scripts during development.
