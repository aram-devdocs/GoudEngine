-- Type factory and field access tests for Lua bindings.

-- Color
local c = Color({ r = 0.5, g = 0.6, b = 0.7, a = 1.0 })
assert(c.r == 0.5, "Color.r mismatch: " .. tostring(c.r))
assert(c.g > 0.59 and c.g < 0.61, "Color.g mismatch: " .. tostring(c.g))
assert(c.b > 0.69 and c.b < 0.71, "Color.b mismatch: " .. tostring(c.b))
assert(c.a == 1.0, "Color.a mismatch: " .. tostring(c.a))

-- Color field set
c.r = 0.1
assert(c.r > 0.09 and c.r < 0.11, "Color.r set mismatch: " .. tostring(c.r))

-- Vec2
local v = Vec2({ x = 3.0, y = 4.0 })
assert(v.x == 3.0, "Vec2.x mismatch: " .. tostring(v.x))
assert(v.y == 4.0, "Vec2.y mismatch: " .. tostring(v.y))

-- Vec2 field set
v.x = 10.0
assert(v.x == 10.0, "Vec2.x set mismatch: " .. tostring(v.x))

-- Rect
local r = Rect({ x = 1.0, y = 2.0, width = 100.0, height = 50.0 })
assert(r.x == 1.0, "Rect.x mismatch")
assert(r.y == 2.0, "Rect.y mismatch")
assert(r.width == 100.0, "Rect.width mismatch")
assert(r.height == 50.0, "Rect.height mismatch")

-- Transform2D
local t = Transform2D({ position_x = 5.0, position_y = 10.0, rotation = 1.5, scale_x = 2.0, scale_y = 2.0 })
assert(t.position_x == 5.0, "Transform2D.position_x mismatch")
assert(t.position_y == 10.0, "Transform2D.position_y mismatch")
assert(t.rotation == 1.5, "Transform2D.rotation mismatch")
assert(t.scale_x == 2.0, "Transform2D.scale_x mismatch")
assert(t.scale_y == 2.0, "Transform2D.scale_y mismatch")

-- Sprite (default fields)
local s = Sprite({ color_r = 1.0, color_g = 1.0, color_b = 1.0, color_a = 1.0 })
assert(s.color_r == 1.0, "Sprite.color_r mismatch")
assert(s.flip_x == false, "Sprite.flip_x should default to false")

-- Text (default fields)
local txt = Text({ font_size = 24.0, color_r = 1.0, color_g = 1.0, color_b = 1.0, color_a = 1.0 })
assert(txt.font_size == 24.0, "Text.font_size mismatch")
