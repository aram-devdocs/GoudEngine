-- Enum constant access tests for Lua bindings.

-- key table exists
assert(key ~= nil, "key table should exist")

-- Spot-check key values (GLFW-style codes)
assert(key.space == 32, "key.space should be 32, got: " .. tostring(key.space))
assert(key.w == 87, "key.w should be 87, got: " .. tostring(key.w))
assert(key.a == 65, "key.a should be 65, got: " .. tostring(key.a))
assert(key.s == 83, "key.s should be 83, got: " .. tostring(key.s))
assert(key.d == 68, "key.d should be 68, got: " .. tostring(key.d))
assert(key.escape == 256, "key.escape should be 256, got: " .. tostring(key.escape))
assert(key.enter == 257, "key.enter should be 257, got: " .. tostring(key.enter))
assert(key.up == 265, "key.up should be 265, got: " .. tostring(key.up))
assert(key.down == 264, "key.down should be 264, got: " .. tostring(key.down))
assert(key.left == 263, "key.left should be 263, got: " .. tostring(key.left))
assert(key.right == 262, "key.right should be 262, got: " .. tostring(key.right))

-- mouse_button table exists
assert(mouse_button ~= nil, "mouse_button table should exist")

-- renderer_type table exists
assert(renderer_type ~= nil, "renderer_type table should exist")

-- blend_mode table exists
assert(blend_mode ~= nil, "blend_mode table should exist")

-- text_alignment table exists
assert(text_alignment ~= nil, "text_alignment table should exist")
