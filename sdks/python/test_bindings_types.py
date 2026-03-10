#!/usr/bin/env python3
"""Datatype binding tests for the Python SDK."""

import math

from test_bindings_common import Color, Entity, Key, MouseButton, Rect, Sprite, Transform2D, Vec2


def test_vec2():
    """Test Vec2 construction, factories, arithmetic, and math methods."""
    print("Testing Vec2...")

    def approx(a, b, eps=0.001):
        return abs(a - b) < eps

    v = Vec2(3.0, 4.0)
    assert v.x == 3.0, f"Expected x=3.0, got {v.x}"
    assert v.y == 4.0, f"Expected y=4.0, got {v.y}"

    v_default = Vec2()
    assert v_default.x == 0.0 and v_default.y == 0.0, "Default Vec2 should be (0, 0)"

    assert Vec2.zero().x == 0.0 and Vec2.zero().y == 0.0, "zero() should return (0, 0)"
    assert Vec2.one().x == 1.0 and Vec2.one().y == 1.0, "one() should return (1, 1)"
    assert Vec2.up().x == 0.0 and Vec2.up().y == -1.0, "up() should return (0, -1)"
    assert Vec2.down().x == 0.0 and Vec2.down().y == 1.0, "down() should return (0, 1)"
    assert Vec2.left().x == -1.0 and Vec2.left().y == 0.0, "left() should return (-1, 0)"
    assert Vec2.right().x == 1.0 and Vec2.right().y == 0.0, "right() should return (1, 0)"

    a = Vec2(1.0, 2.0)
    b = Vec2(3.0, 4.0)

    result = a.add(b)
    assert result.x == 4.0 and result.y == 6.0, f"add() failed: {result}"

    result = a.sub(b)
    assert result.x == -2.0 and result.y == -2.0, f"sub() failed: {result}"

    result = a.scale(3.0)
    assert result.x == 3.0 and result.y == 6.0, f"scale() failed: {result}"

    result = a + b
    assert result.x == 4.0 and result.y == 6.0, f"__add__ failed: {result}"

    result = b - a
    assert result.x == 2.0 and result.y == 2.0, f"__sub__ failed: {result}"

    result = a * 2.0
    assert result.x == 2.0 and result.y == 4.0, f"__mul__ failed: {result}"

    result = b / 2.0
    assert result.x == 1.5 and result.y == 2.0, f"__truediv__ failed: {result}"

    result = -a
    assert result.x == -1.0 and result.y == -2.0, f"__neg__ failed: {result}"

    v = Vec2(3.0, 4.0)
    assert v.length() == 5.0, f"length() expected 5.0, got {v.length()}"

    n = v.normalize()
    assert approx(n.length(), 1.0), f"normalize() result has non-unit length: {n.length()}"

    a = Vec2(1.0, 0.0)
    b = Vec2(0.0, 1.0)
    assert a.dot(b) == 0.0, f"dot() of perpendicular vectors should be 0, got {a.dot(b)}"
    assert a.dot(a) == 1.0, f"dot() of unit vector with itself should be 1, got {a.dot(a)}"

    p = Vec2(0.0, 0.0)
    q = Vec2(3.0, 4.0)
    assert p.distance(q) == 5.0, f"distance() expected 5.0, got {p.distance(q)}"

    start = Vec2(0.0, 0.0)
    end = Vec2(10.0, 20.0)
    mid = start.lerp(end, 0.5)
    assert mid.x == 5.0 and mid.y == 10.0, f"lerp(0.5) failed: {mid}"
    at_start = start.lerp(end, 0.0)
    assert at_start.x == 0.0 and at_start.y == 0.0, f"lerp(0.0) should equal start: {at_start}"
    at_end = start.lerp(end, 1.0)
    assert at_end.x == 10.0 and at_end.y == 20.0, f"lerp(1.0) should equal end: {at_end}"

    print("  Vec2 tests passed")
    return True


def test_color():
    """Test Color construction, factories, and with_alpha."""
    print("Testing Color...")

    def approx(a, b, eps=0.001):
        return abs(a - b) < eps

    c = Color(0.5, 0.6, 0.7, 0.8)
    assert approx(c.r, 0.5), f"Expected r=0.5, got {c.r}"
    assert approx(c.g, 0.6), f"Expected g=0.6, got {c.g}"
    assert approx(c.b, 0.7), f"Expected b=0.7, got {c.b}"
    assert approx(c.a, 0.8), f"Expected a=0.8, got {c.a}"

    c_default = Color()
    assert c_default.r == 0.0 and c_default.g == 0.0 and c_default.b == 0.0 and c_default.a == 0.0, \
        "Default Color should be (0, 0, 0, 0)"

    white = Color.white()
    assert approx(white.r, 1.0) and approx(white.g, 1.0) and approx(white.b, 1.0) and approx(white.a, 1.0), \
        f"white() should be (1,1,1,1), got {white}"

    black = Color.black()
    assert approx(black.r, 0.0) and approx(black.g, 0.0) and approx(black.b, 0.0) and approx(black.a, 1.0), \
        f"black() should be (0,0,0,1), got {black}"

    red = Color.red()
    assert approx(red.r, 1.0) and approx(red.g, 0.0) and approx(red.b, 0.0) and approx(red.a, 1.0), \
        f"red() should be (1,0,0,1), got {red}"

    green = Color.green()
    assert approx(green.r, 0.0) and approx(green.g, 1.0) and approx(green.b, 0.0) and approx(green.a, 1.0), \
        f"green() should be (0,1,0,1), got {green}"

    blue = Color.blue()
    assert approx(blue.r, 0.0) and approx(blue.g, 0.0) and approx(blue.b, 1.0) and approx(blue.a, 1.0), \
        f"blue() should be (0,0,1,1), got {blue}"

    c = Color.rgb(0.2, 0.4, 0.6)
    assert approx(c.r, 0.2) and approx(c.g, 0.4) and approx(c.b, 0.6) and approx(c.a, 1.0), \
        f"rgb() should set alpha to 1.0, got {c}"

    c = Color.rgba(0.1, 0.2, 0.3, 0.4)
    assert approx(c.r, 0.1) and approx(c.g, 0.2) and approx(c.b, 0.3) and approx(c.a, 0.4), \
        f"rgba() failed: {c}"

    c = Color.from_hex(0xFF0000)
    assert approx(c.r, 1.0) and approx(c.g, 0.0) and approx(c.b, 0.0) and approx(c.a, 1.0), \
        f"from_hex(0xFF0000) should be red with full alpha, got {c}"

    c = Color.from_hex(0x00FF00)
    assert approx(c.r, 0.0) and approx(c.g, 1.0) and approx(c.b, 0.0), \
        f"from_hex(0x00FF00) should be green, got {c}"

    c = Color.from_hex(0x0000FF)
    assert approx(c.r, 0.0) and approx(c.g, 0.0) and approx(c.b, 1.0), \
        f"from_hex(0x0000FF) should be blue, got {c}"

    base = Color.red()
    semi = base.with_alpha(0.5)
    assert approx(semi.r, 1.0) and approx(semi.g, 0.0) and approx(semi.b, 0.0) and approx(semi.a, 0.5), \
        f"with_alpha(0.5) on red should give (1,0,0,0.5), got {semi}"
    assert approx(base.a, 1.0), "with_alpha should not mutate the original"

    print("  Color tests passed")
    return True


def test_rect():
    """Test Rect creation, contains, and intersects."""
    print("Testing Rect...")

    r = Rect(10, 20, 100, 50)
    assert r.x == 10, f"Expected x=10, got {r.x}"
    assert r.y == 20, f"Expected y=20, got {r.y}"
    assert r.width == 100, f"Expected width=100, got {r.width}"
    assert r.height == 50, f"Expected height=50, got {r.height}"

    r_default = Rect()
    assert r_default.x == 0.0 and r_default.y == 0.0 and r_default.width == 0.0 and r_default.height == 0.0, \
        "Default Rect should be (0, 0, 0, 0)"

    assert r.contains(Vec2(50, 40)), "Point (50,40) should be inside Rect(10,20,100,50)"
    assert r.contains(Vec2(10, 45)), "Left edge point should be inside"
    assert r.contains(Vec2(50, 20)), "Top edge point should be inside"
    assert not r.contains(Vec2(9, 40)), "Point (9,40) should be outside left edge"
    assert not r.contains(Vec2(111, 45)), "Point (111,45) should be outside right edge"
    assert not r.contains(Vec2(50, 19)), "Point above rect should be outside"
    assert not r.contains(Vec2(0, 0)), "Origin should be outside Rect(10,20,...)"

    r2 = Rect(50, 40, 100, 50)
    assert r.intersects(r2), "Overlapping rects should intersect"

    r3 = Rect(110, 20, 100, 50)
    assert not r.intersects(r3), "Adjacent (touching) rects should not intersect"

    r4 = Rect(200, 200, 50, 50)
    assert not r.intersects(r4), "Non-overlapping rects should not intersect"

    r5 = Rect(0, 0, 500, 500)
    r6 = Rect(50, 50, 10, 10)
    assert r5.intersects(r6), "Contained rect should intersect its container"

    print("  Rect tests passed")
    return True


def test_transform2d():
    """Test Transform2D flat fields and construction."""
    print("Testing Transform2D...")

    t = Transform2D(position_x=10.0, position_y=20.0, rotation=0.5, scale_x=2.0, scale_y=3.0)
    assert t.position_x == 10.0, f"Expected position_x=10.0, got {t.position_x}"
    assert t.position_y == 20.0, f"Expected position_y=20.0, got {t.position_y}"
    assert t.rotation == 0.5, f"Expected rotation=0.5, got {t.rotation}"
    assert t.scale_x == 2.0, f"Expected scale_x=2.0, got {t.scale_x}"
    assert t.scale_y == 3.0, f"Expected scale_y=3.0, got {t.scale_y}"

    t.position_x = 99.0
    assert t.position_x == 99.0, "Direct field assignment to position_x should work"
    t.rotation = math.pi
    assert abs(t.rotation - math.pi) < 0.001, "Direct field assignment to rotation should work"

    try:
        t = Transform2D.default()
        assert t.position_x == 0.0 and t.position_y == 0.0, \
            f"default() position should be (0,0), got ({t.position_x},{t.position_y})"
        assert t.rotation == 0.0, f"default() rotation should be 0.0, got {t.rotation}"
        assert t.scale_x == 1.0 and t.scale_y == 1.0, \
            f"default() scale should be (1,1), got ({t.scale_x},{t.scale_y})"

        t = Transform2D.from_position(100.0, 50.0)
        assert t.position_x == 100.0, f"from_position() x failed: {t.position_x}"
        assert t.position_y == 50.0, f"from_position() y failed: {t.position_y}"
        assert t.rotation == 0.0, "from_position() should set rotation to 0"
        assert t.scale_x == 1.0 and t.scale_y == 1.0, "from_position() should set scale to (1,1)"

        t = Transform2D.from_rotation(math.pi / 2)
        assert abs(t.rotation - math.pi / 2) < 0.001, \
            f"from_rotation(pi/2) failed: {t.rotation}"
        assert t.position_x == 0.0 and t.position_y == 0.0, \
            "from_rotation() should set position to (0,0)"
        assert t.scale_x == 1.0 and t.scale_y == 1.0, \
            "from_rotation() should set scale to (1,1)"

        t = Transform2D.from_scale(3.0, 4.0)
        assert t.scale_x == 3.0, f"from_scale() x failed: {t.scale_x}"
        assert t.scale_y == 4.0, f"from_scale() y failed: {t.scale_y}"
        assert t.position_x == 0.0 and t.position_y == 0.0, \
            "from_scale() should set position to (0,0)"
        assert t.rotation == 0.0, "from_scale() should set rotation to 0"
    except ImportError:
        print("    (skipped FFI-backed factories: native library not available)")

    print("  Transform2D tests passed")
    return True


def test_sprite():
    """Test Sprite creation and flat field access."""
    print("Testing Sprite...")

    s = Sprite()
    assert s.texture_handle == 0, f"Default texture_handle should be 0, got {s.texture_handle}"
    assert s.flip_x == False, f"Default flip_x should be False, got {s.flip_x}"
    assert s.flip_y == False, f"Default flip_y should be False, got {s.flip_y}"
    assert s.anchor_x == 0.0, f"Default anchor_x should be 0.0, got {s.anchor_x}"
    assert s.anchor_y == 0.0, f"Default anchor_y should be 0.0, got {s.anchor_y}"

    s = Sprite(texture_handle=42)
    assert s.texture_handle == 42, f"Expected texture_handle=42, got {s.texture_handle}"

    s = Sprite(texture_handle=7, flip_x=True, flip_y=False, anchor_x=0.5, anchor_y=1.0)
    assert s.texture_handle == 7, f"Expected texture_handle=7, got {s.texture_handle}"
    assert s.flip_x == True, f"Expected flip_x=True, got {s.flip_x}"
    assert s.flip_y == False, f"Expected flip_y=False, got {s.flip_y}"
    assert s.anchor_x == 0.5, f"Expected anchor_x=0.5, got {s.anchor_x}"
    assert s.anchor_y == 1.0, f"Expected anchor_y=1.0, got {s.anchor_y}"

    s = Sprite(texture_handle=10)
    s.flip_x = True
    assert s.flip_x == True, "flip_x field assignment should work"
    s.flip_y = True
    assert s.flip_y == True, "flip_y field assignment should work"
    s.anchor_x = 0.25
    assert s.anchor_x == 0.25, "anchor_x field assignment should work"
    s.anchor_y = 0.75
    assert s.anchor_y == 0.75, "anchor_y field assignment should work"
    s.texture_handle = 99
    assert s.texture_handle == 99, "texture_handle field assignment should work"

    print("  Sprite tests passed")
    return True


def test_entity():
    """Test Entity bits encoding, index, generation, is_placeholder, and to_bits."""
    print("Testing Entity...")

    index_val = 5
    generation_val = 2
    bits = (generation_val << 32) | index_val
    e = Entity(bits)

    assert e.index == index_val, f"Expected index={index_val}, got {e.index}"
    assert e.generation == generation_val, f"Expected generation={generation_val}, got {e.generation}"
    assert e.is_placeholder == False, "Non-sentinel entity should not be a placeholder"
    assert e.to_bits() == bits, f"to_bits() should return original bits, got {e.to_bits()}"

    e_zero = Entity(0)
    assert e_zero.index == 0, "Entity(0).index should be 0"
    assert e_zero.generation == 0, "Entity(0).generation should be 0"
    assert e_zero.is_placeholder == False, "Entity(0) is not the placeholder sentinel"

    placeholder_bits = 0xFFFFFFFFFFFFFFFF
    e_placeholder = Entity(placeholder_bits)
    assert e_placeholder.is_placeholder == True, \
        f"Entity(0xFFFFFFFFFFFFFFFF) should be a placeholder, got {e_placeholder.is_placeholder}"
    assert e_placeholder.to_bits() == placeholder_bits, \
        "to_bits() on placeholder should return 0xFFFFFFFFFFFFFFFF"

    e_large = Entity((1000 << 32) | 999)
    assert e_large.index == 999, f"Expected index=999, got {e_large.index}"
    assert e_large.generation == 1000, f"Expected generation=1000, got {e_large.generation}"

    print("  Entity tests passed")
    return True


def test_enums():
    """Test Key and MouseButton enum constant values."""
    print("Testing enums...")

    assert Key.ESCAPE == 256, f"Key.ESCAPE should be 256, got {Key.ESCAPE}"
    assert Key.SPACE == 32, f"Key.SPACE should be 32, got {Key.SPACE}"
    assert Key.W == 87, f"Key.W should be 87, got {Key.W}"
    assert Key.A == 65, f"Key.A should be 65, got {Key.A}"
    assert Key.S == 83, f"Key.S should be 83, got {Key.S}"
    assert Key.D == 68, f"Key.D should be 68, got {Key.D}"
    assert Key.ENTER == 257, f"Key.ENTER should be 257, got {Key.ENTER}"
    assert Key.LEFT == 263, f"Key.LEFT should be 263, got {Key.LEFT}"
    assert Key.RIGHT == 262, f"Key.RIGHT should be 262, got {Key.RIGHT}"
    assert Key.UP == 265, f"Key.UP should be 265, got {Key.UP}"
    assert Key.DOWN == 264, f"Key.DOWN should be 264, got {Key.DOWN}"

    assert MouseButton.LEFT == 0, f"MouseButton.LEFT should be 0, got {MouseButton.LEFT}"
    assert MouseButton.RIGHT == 1, f"MouseButton.RIGHT should be 1, got {MouseButton.RIGHT}"
    assert MouseButton.MIDDLE == 2, f"MouseButton.MIDDLE should be 2, got {MouseButton.MIDDLE}"

    print("  Enum tests passed")
    return True
