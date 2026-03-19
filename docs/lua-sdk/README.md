# GoudEngine Lua SDK

The Lua SDK embeds a Lua 5.4 runtime (via [mlua](https://github.com/mlua-rs/mlua)) directly inside GoudEngine. Lua scripts have access to the same engine capabilities exposed through the C#/Python FFI layer -- entity management, rendering, audio, collision, physics, animation, and more -- without any external bindings or shared libraries.

> **Alpha status.** The Lua SDK is functional but under active development. APIs may change between releases. Input handling (keyboard/mouse queries) is not yet exposed to Lua scripts.

## Documentation

| Page | Description |
|------|-------------|
| [Getting Started](getting-started.md) | Build, run, and write your first Lua script |
| [API Reference](api-reference.md) | Complete listing of types, enums, and functions |
| [Hot Reload](hot-reload.md) | Live-reload Lua scripts during development |

## Quick Example

```lua
-- hello.lua
-- A minimal GoudEngine Lua script that prints to the console.

function on_init()
    print("Hello from GoudEngine Lua!")
end

function on_update(dt)
    -- Called every frame with the delta time in seconds.
end

function on_draw()
    -- Called every frame after update, for rendering.
end
```

## How It Works

1. GoudEngine creates an embedded Lua VM at startup.
2. Auto-generated bindings register type constructors, enum constants, and tool functions into the Lua global scope.
3. Hand-written bridge functions supplement the generated code for FFI calls that require string parameters (texture/font loading, text drawing).
4. Your script defines callback functions (`on_init`, `on_update`, `on_draw`) that the engine invokes each frame.

## Feature Flags

The Lua SDK requires two Cargo feature flags:

- `lua` -- compiles the embedded Lua VM and binding registration.
- `native` -- enables the tool functions and bridge layer that call into the native FFI (rendering, audio, window management).

Build with both:

```bash
cargo build --features lua,native
```

## Architecture

```
codegen/gen_lua.py          -- generates .g.rs binding files from the schema
    |
    v
lua_bindings/
  types.g.rs                -- type constructors (Color, Vec2, Rect, etc.)
  enums.g.rs                -- enum constant tables (Key, MouseButton, etc.)
  tools.g.rs                -- tool functions on global tables (goud_game, audio, etc.)
  register.g.rs             -- single entry point that wires everything up
    |
    v
lua_bridge.rs               -- hand-written wrappers for string-param FFI calls
lua_runtime.rs              -- LuaRuntime: VM lifecycle, script execution
lua_hot_reload.rs           -- file-system watcher for .lua changes
```
