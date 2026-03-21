# GoudEngine LuaRocks Package

LuaRocks distribution for GoudEngine's Lua SDK. See the [Lua SDK README](../sdks/lua/README.md) and [Getting Started guide](../docs/src/getting-started/lua.md) for full documentation.

## Installation

```bash
luarocks make goudengine-scm-1.rockspec
```

## Usage

```lua
local goud = require("goudengine")
print(goud.VERSION)

local Key = goud.constants.key
print(Key.space)  -- 32
```

For game development, use the embedded runner which exposes the full engine API. See the [Lua SDK](../sdks/lua/README.md).

## Platform Support

| OS | Library | Status |
|----|---------|--------|
| macOS | libgoud_engine.dylib | Supported |
| Linux | libgoud_engine.so | Supported |
| Windows | goud_engine.dll | Experimental |
