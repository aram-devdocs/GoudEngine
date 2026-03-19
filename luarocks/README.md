# GoudEngine LuaRocks Package

LuaRocks distribution package for GoudEngine's Lua SDK. The Lua bindings use mlua to embed Lua 5.4 inside the Rust engine, giving scripts direct access to the ECS, rendering, physics, audio, and input systems.

## Prerequisites

- Rust toolchain (stable, via rustup)
- Lua 5.4+
- LuaRocks 3.x

## Build from source

```bash
cd luarocks
luarocks make goudengine-scm-1.rockspec
```

Or build manually:

```bash
cd luarocks
make          # builds the native library via cargo
make install  # installs Lua modules and the native library
```

## Platform support

| Platform | Library          | Status |
|----------|------------------|--------|
| macOS    | libgoud_engine.dylib | Supported |
| Linux    | libgoud_engine.so    | Supported |
| Windows  | goud_engine.dll      | Experimental |

## Usage

When running scripts through the embedded `lua-runner`, all engine bindings are registered automatically as globals. The LuaRocks package provides a `require`-able module for standalone Lua interpreters:

```lua
local goud = require("goudengine")
print(goud.VERSION)  -- "0.0.832"

-- Access key constants
local Key = goud.constants.key
print(Key.space)  -- 32
```

For game development, use the embedded runtime which exposes the full engine API (entity creation, rendering, input polling, etc.) directly into the Lua global scope.
