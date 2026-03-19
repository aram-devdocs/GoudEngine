--- GoudEngine Lua SDK entry point.
-- Loads the native library and re-exports the engine API.
-- @module goudengine

local M = {}

-- Platform-specific library loading
local ffi_path
local uname = io.popen("uname -s"):read("*l")
if uname == "Darwin" then
    ffi_path = "libgoud_engine.dylib"
elseif uname == "Linux" then
    ffi_path = "libgoud_engine.so"
else
    ffi_path = "goud_engine.dll"
end

-- The native library is loaded by the Rust runtime via mlua.
-- When using the embedded runtime (lua-runner), bindings are
-- automatically registered. This module provides the standalone
-- entry point for external Lua interpreters.
M.VERSION = "0.0.832"
M.ALPHA = true

-- Re-export constants
M.constants = require("goudengine.constants")

return M
