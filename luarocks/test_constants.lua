--- GoudEngine Lua SDK constant tests.
-- Validates that the constants module loads and contains expected values.

local constants = require("goudengine.constants")

local pass = 0
local fail = 0

local function check(name, condition)
    if condition then
        pass = pass + 1
    else
        fail = fail + 1
        io.stderr:write("FAIL: " .. name .. "\n")
    end
end

-- Key codes
check("key table exists", type(constants.key) == "table")
check("key.space == 32", constants.key.space == 32)
check("key.escape == 256", constants.key.escape == 256)
check("key.a == 65", constants.key.a == 65)
check("key.unknown == -1", constants.key.unknown == -1)

-- Mouse button codes
check("mouse_button table exists", type(constants.mouse_button) == "table")
check("mouse_button.left == 0", constants.mouse_button.left == 0)
check("mouse_button.right == 1", constants.mouse_button.right == 1)
check("mouse_button.middle == 2", constants.mouse_button.middle == 2)

-- Module metadata
local goud = require("goudengine")
check("VERSION is a string", type(goud.VERSION) == "string")
check("ALPHA is boolean", type(goud.ALPHA) == "boolean")
check("constants re-exported", goud.constants == constants)

print(string.format("\nLua SDK tests: %d passed, %d failed", pass, fail))
if fail > 0 then
    os.exit(1)
end
