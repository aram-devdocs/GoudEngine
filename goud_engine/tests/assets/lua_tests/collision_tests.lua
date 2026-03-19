-- Collision function tests for Lua bindings.
-- These functions live on the goud_game table.

-- AABB overlap: two overlapping boxes
local overlap = goud_game.aabb_overlap(0, 0, 10, 10, 5, 5, 15, 15)
assert(overlap == true, "overlapping AABBs should return true")

-- AABB no overlap: two separated boxes
local no_overlap = goud_game.aabb_overlap(0, 0, 10, 10, 20, 20, 30, 30)
assert(no_overlap == false, "separated AABBs should return false")

-- Circle overlap: two overlapping circles
local c_overlap = goud_game.circle_overlap(0, 0, 5, 3, 0, 5)
assert(c_overlap == true, "overlapping circles should return true")

-- Circle no overlap: two separated circles
local c_no_overlap = goud_game.circle_overlap(0, 0, 1, 100, 100, 1)
assert(c_no_overlap == false, "separated circles should return false")

-- Distance
local d = goud_game.distance(0, 0, 3, 4)
assert(d > 4.99 and d < 5.01, "distance(0,0,3,4) should be ~5, got: " .. tostring(d))

-- Distance squared
local d2 = goud_game.distance_squared(0, 0, 3, 4)
assert(d2 > 24.9 and d2 < 25.1, "distance_squared(0,0,3,4) should be ~25, got: " .. tostring(d2))

-- Point in rect
local in_rect = goud_game.point_in_rect(5, 5, 0, 0, 10, 10)
assert(in_rect == true, "point (5,5) should be in rect (0,0,10,10)")

local out_rect = goud_game.point_in_rect(15, 15, 0, 0, 10, 10)
assert(out_rect == false, "point (15,15) should not be in rect (0,0,10,10)")

-- Point in circle
local in_circle = goud_game.point_in_circle(1, 1, 0, 0, 5)
assert(in_circle == true, "point (1,1) should be in circle at (0,0) r=5")

local out_circle = goud_game.point_in_circle(10, 10, 0, 0, 5)
assert(out_circle == false, "point (10,10) should not be in circle at (0,0) r=5")
