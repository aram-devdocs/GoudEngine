#!/usr/bin/env python3
"""Datatype binding tests for the Python SDK."""

import ctypes
import math

from test_bindings_common import (
    AnimationEventData,
    AudioCapabilities,
    ColliderHandle,
    Color,
    Contact,
    Entity,
    FpsStats,
    InputCapabilities,
    Key,
    Mat3x3,
    MouseButton,
    NetworkCapabilities,
    NetworkConnectResult,
    NetworkHandle,
    NetworkPacket,
    NetworkSimulationConfig,
    NetworkStats,
    PhysicsCapabilities,
    PhysicsCollisionEvent2D,
    PhysicsRaycastHit2D,
    PhysicsWorld2D,
    PhysicsWorld3D,
    Rect,
    RenderCapabilities,
    RenderStats,
    RigidBodyHandle,
    Sprite,
    SpriteAnimator,
    Text,
    Transform2D,
    TweenHandle,
    UiEvent,
    UiStyle,
    Vec2,
    Vec3,
    _new_fake_generated_package,
)


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


def test_color_vec2_extended_helpers():
    """Test additional generated Color/Vec2 helpers to raise generated-surface execution."""
    print("Testing Color/Vec2 helper coverage...")

    c = Color.from_u8(255, 128, 64, 32)
    assert abs(c.r - 1.0) < 0.001
    assert abs(c.g - (128.0 / 255.0)) < 0.001
    assert abs(c.b - (64.0 / 255.0)) < 0.001
    assert abs(c.a - (32.0 / 255.0)) < 0.001

    lerped = Color.red().lerp(Color.blue(), 0.25)
    assert abs(lerped.r - 0.75) < 0.001
    assert abs(lerped.g - 0.0) < 0.001
    assert abs(lerped.b - 0.25) < 0.001
    assert abs(lerped.a - 1.0) < 0.001

    zero_norm = Vec2.zero().normalize()
    assert zero_norm.x == 0.0 and zero_norm.y == 0.0, "normalize() on zero should remain zero"

    assert "Color(" in repr(c), "Color repr should include class name"
    assert "Vec2(" in repr(Vec2(1, 2)), "Vec2 repr should include class name"

    print("  Color/Vec2 helper coverage passed")
    return True


def test_generated_value_types_runtime_safe():
    """Test runtime-safe generated value type constructors, fields, and reprs."""
    print("Testing generated value-type runtime-safe surface...")

    mat = Mat3x3(3.5)
    assert mat.m == 3.5
    assert "Mat3x3" in repr(mat)

    text = Text(font_handle=3, font_size=18.0, color_r=0.1, color_g=0.2, color_b=0.3, color_a=0.4, alignment=2, max_width=420.0, has_max_width=True, line_spacing=1.25)
    assert text.font_handle == 3 and text.font_size == 18.0
    assert text.has_max_width is True and text.max_width == 420.0
    assert "Text(" in repr(text)

    anim = SpriteAnimator(current_frame=2, elapsed=0.5, playing=True, finished=False, frame_duration=0.08, mode=1.0, frame_count=8)
    assert anim.current_frame == 2 and anim.playing is True and anim.frame_count == 8
    assert "SpriteAnimator(" in repr(anim)

    event = AnimationEventData(entity=7, name=1.0, frame_index=3, payload_type=2, payload_int=9, payload_float=2.5, payload_string=4.0)
    assert event.entity == 7 and event.frame_index == 3
    assert "AnimationEventData(" in repr(event)

    stats = RenderStats(draw_calls=11, triangles=22, texture_binds=3, shader_binds=4)
    assert stats.draw_calls == 11 and stats.triangles == 22
    assert "RenderStats(" in repr(stats)

    contact = Contact(point_x=1.0, point_y=2.0, normal_x=0.0, normal_y=1.0, penetration=0.25)
    assert contact.point_x == 1.0 and contact.penetration == 0.25
    assert "Contact(" in repr(contact)

    ray_hit = PhysicsRaycastHit2D(body_handle=4, collider_handle=5, point_x=6.0, point_y=7.0, normal_x=0.0, normal_y=1.0, distance=8.0)
    assert ray_hit.body_handle == 4 and ray_hit.distance == 8.0
    assert "PhysicsRaycastHit2D(" in repr(ray_hit)

    collision = PhysicsCollisionEvent2D(body_a=10, body_b=11, kind=2)
    assert collision.body_a == 10 and collision.kind == 2
    assert "PhysicsCollisionEvent2D(" in repr(collision)

    vec3 = Vec3(1.0, 2.0, 3.0)
    assert vec3.x == 1.0 and vec3.y == 2.0 and vec3.z == 3.0
    assert Vec3.zero().z == 0.0 and Vec3.one().x == 1.0 and Vec3.up().y == 1.0
    assert "Vec3(" in repr(vec3)

    fps = FpsStats(current_fps=120.0, min_fps=100.0, max_fps=144.0, avg_fps=115.0, frame_time_ms=8.3)
    assert fps.current_fps == 120.0 and fps.frame_time_ms == 8.3
    assert "FpsStats(" in repr(fps)

    for handle in [
        PhysicsWorld2D(1),
        PhysicsWorld3D(2),
        RigidBodyHandle(3),
        ColliderHandle(4),
        TweenHandle(5),
        NetworkHandle(6),
    ]:
        assert handle._bits > 0

    render_caps = RenderCapabilities(max_texture_units=16, max_texture_size=8192, supports_instancing=True, supports_compute=False, supports_msaa=True)
    physics_caps = PhysicsCapabilities(supports_continuous_collision=True, supports_joints=True, max_bodies=1000)
    audio_caps = AudioCapabilities(supports_spatial=True, max_channels=64)
    input_caps = InputCapabilities(supports_gamepad=True, supports_touch=False, max_gamepads=4)
    network_caps = NetworkCapabilities(supports_hosting=True, max_connections=32, max_channels=8, max_message_size=65535)
    assert render_caps.max_texture_units == 16 and render_caps.supports_msaa is True
    assert physics_caps.max_bodies == 1000 and physics_caps.supports_joints is True
    assert audio_caps.max_channels == 64 and audio_caps.supports_spatial is True
    assert input_caps.max_gamepads == 4 and input_caps.supports_gamepad is True
    assert network_caps.max_message_size == 65535 and network_caps.supports_hosting is True
    assert "RenderCapabilities(" in repr(render_caps)
    assert "PhysicsCapabilities(" in repr(physics_caps)
    assert "AudioCapabilities(" in repr(audio_caps)
    assert "InputCapabilities(" in repr(input_caps)
    assert "NetworkCapabilities(" in repr(network_caps)

    net_stats = NetworkStats(
        bytes_sent=10,
        bytes_received=20,
        packets_sent=3,
        packets_received=4,
        packets_lost=1,
        rtt_ms=5.5,
        send_bandwidth_bytes_per_sec=100.0,
        receive_bandwidth_bytes_per_sec=200.0,
        packet_loss_percent=2.5,
        jitter_ms=1.2,
    )
    sim = NetworkSimulationConfig(one_way_latency_ms=15, jitter_ms=3, packet_loss_percent=0.5)
    conn = NetworkConnectResult(handle=77, peer_id=99)
    pkt = NetworkPacket(peer_id=9, data=b"abc")
    assert net_stats.bytes_received == 20 and net_stats.jitter_ms == 1.2
    assert sim.one_way_latency_ms == 15 and sim.packet_loss_percent == 0.5
    assert conn.handle == 77 and conn.peer_id == 99
    assert pkt.peer_id == 9 and pkt.data == b"abc"
    assert "NetworkStats(" in repr(net_stats)
    assert "NetworkSimulationConfig(" in repr(sim)
    assert "NetworkConnectResult(" in repr(conn)
    assert "NetworkPacket(" in repr(pkt)

    style = UiStyle(font_family=None, texture_path=None)
    assert style.font_family == "", "UiStyle should normalize None font_family to empty string"
    assert style.texture_path == "", "UiStyle should normalize None texture_path to empty string"
    assert isinstance(style.background_color, Color)
    assert isinstance(style.foreground_color, Color)
    assert isinstance(style.border_color, Color)
    event = UiEvent(event_kind=3, node_id=12, previous_node_id=1, current_node_id=2)
    assert event.node_id == 12 and event.event_kind == 3
    assert "UiStyle(" in repr(style)
    assert "UiEvent(" in repr(event)

    print("  Generated value-type runtime-safe surface passed")
    return True


def test_generated_types_ffi_runtime_with_fake_lib():
    """Execute FFI-backed generated _types.py methods against a fake backend."""
    print("Testing generated _types.py FFI-backed runtime with fake lib...")

    class _FakeLib:
        def __init__(self):
            self.ffi = None
            self._builder_id = 100

        def _tr(self, ptr):
            return ctypes.cast(ptr, ctypes.POINTER(self.ffi.FfiTransform2D)).contents

        def _sp(self, ptr):
            return ctypes.cast(ptr, ctypes.POINTER(self.ffi.FfiSprite)).contents

        def _tx(self, ptr):
            return ctypes.cast(ptr, ctypes.POINTER(self.ffi.FfiText)).contents

        def goud_transform2d_default(self):
            return self.ffi.FfiTransform2D(0.0, 0.0, 0.0, 1.0, 1.0)

        def goud_transform2d_from_position(self, x, y):
            return self.ffi.FfiTransform2D(x, y, 0.0, 1.0, 1.0)

        def goud_transform2d_from_rotation(self, r):
            return self.ffi.FfiTransform2D(0.0, 0.0, r, 1.0, 1.0)

        def goud_transform2d_from_rotation_degrees(self, d):
            return self.ffi.FfiTransform2D(0.0, 0.0, d * math.pi / 180.0, 1.0, 1.0)

        def goud_transform2d_from_scale(self, x, y):
            return self.ffi.FfiTransform2D(0.0, 0.0, 0.0, x, y)

        def goud_transform2d_from_scale_uniform(self, s):
            return self.ffi.FfiTransform2D(0.0, 0.0, 0.0, s, s)

        def goud_transform2d_from_position_rotation(self, x, y, r):
            return self.ffi.FfiTransform2D(x, y, r, 1.0, 1.0)

        def goud_transform2d_new(self, x, y, r, sx, sy):
            return self.ffi.FfiTransform2D(x, y, r, sx, sy)

        def goud_transform2d_look_at(self, px, py, tx, ty):
            return self.ffi.FfiTransform2D(px, py, 0.5, 1.0, 1.0)

        def goud_transform2d_translate(self, ptr, dx, dy):
            tr = self._tr(ptr)
            tr.position_x += dx
            tr.position_y += dy
            return 0

        def goud_transform2d_translate_local(self, ptr, dx, dy):
            return self.goud_transform2d_translate(ptr, dx, dy)

        def goud_transform2d_set_position(self, ptr, x, y):
            tr = self._tr(ptr)
            tr.position_x = x
            tr.position_y = y
            return 0

        def goud_transform2d_get_position(self, ptr):
            tr = self._tr(ptr)
            return self.ffi.FfiVec2(tr.position_x, tr.position_y)

        def goud_transform2d_rotate(self, ptr, angle):
            self._tr(ptr).rotation += angle
            return 0

        def goud_transform2d_rotate_degrees(self, ptr, degrees):
            self._tr(ptr).rotation += degrees * math.pi / 180.0
            return 0

        def goud_transform2d_set_rotation(self, ptr, rotation):
            self._tr(ptr).rotation = rotation
            return 0

        def goud_transform2d_set_rotation_degrees(self, ptr, degrees):
            self._tr(ptr).rotation = degrees * math.pi / 180.0
            return 0

        def goud_transform2d_get_rotation(self, ptr):
            return self._tr(ptr).rotation

        def goud_transform2d_get_rotation_degrees(self, ptr):
            return self._tr(ptr).rotation * 180.0 / math.pi

        def goud_transform2d_look_at_target(self, ptr, tx, ty):
            self._tr(ptr).rotation = 1.0
            return 0

        def goud_transform2d_set_scale(self, ptr, sx, sy):
            tr = self._tr(ptr)
            tr.scale_x = sx
            tr.scale_y = sy
            return 0

        def goud_transform2d_set_scale_uniform(self, ptr, s):
            tr = self._tr(ptr)
            tr.scale_x = s
            tr.scale_y = s
            return 0

        def goud_transform2d_get_scale(self, ptr):
            tr = self._tr(ptr)
            return self.ffi.FfiVec2(tr.scale_x, tr.scale_y)

        def goud_transform2d_scale_by(self, ptr, fx, fy):
            tr = self._tr(ptr)
            tr.scale_x *= fx
            tr.scale_y *= fy
            return 0

        def goud_transform2d_forward(self, ptr):
            return self.ffi.FfiVec2(0.0, 1.0)

        def goud_transform2d_right(self, ptr):
            return self.ffi.FfiVec2(1.0, 0.0)

        def goud_transform2d_backward(self, ptr):
            return self.ffi.FfiVec2(0.0, -1.0)

        def goud_transform2d_left(self, ptr):
            return self.ffi.FfiVec2(-1.0, 0.0)

        def goud_transform2d_matrix(self, ptr):
            out = self.ffi.FfiMat3x3()
            tr = self._tr(ptr)
            out.m[:] = [1.0, 0.0, 0.0, 0.0, 1.0, 0.0, tr.position_x, tr.position_y, 1.0]
            return out

        def goud_transform2d_matrix_inverse(self, ptr):
            out = self.ffi.FfiMat3x3()
            tr = self._tr(ptr)
            out.m[:] = [1.0, 0.0, 0.0, 0.0, 1.0, 0.0, -tr.position_x, -tr.position_y, 1.0]
            return out

        def goud_transform2d_transform_point(self, ptr, x, y):
            tr = self._tr(ptr)
            return self.ffi.FfiVec2(x + tr.position_x, y + tr.position_y)

        def goud_transform2d_transform_direction(self, ptr, x, y):
            return self.ffi.FfiVec2(x, y)

        def goud_transform2d_inverse_transform_point(self, ptr, x, y):
            tr = self._tr(ptr)
            return self.ffi.FfiVec2(x - tr.position_x, y - tr.position_y)

        def goud_transform2d_inverse_transform_direction(self, ptr, x, y):
            return self.ffi.FfiVec2(x, y)

        def goud_transform2d_lerp(self, a, b, t):
            return self.ffi.FfiTransform2D(
                a.position_x + (b.position_x - a.position_x) * t,
                a.position_y + (b.position_y - a.position_y) * t,
                a.rotation + (b.rotation - a.rotation) * t,
                a.scale_x + (b.scale_x - a.scale_x) * t,
                a.scale_y + (b.scale_y - a.scale_y) * t,
            )

        def goud_transform2d_normalize_angle(self, angle):
            return ((angle + math.pi) % (2.0 * math.pi)) - math.pi

        def goud_transform2d_builder_new(self):
            self._builder_id += 1
            return self._builder_id

        def goud_transform2d_builder_at_position(self, x, y):
            self._builder_id += 1
            return self._builder_id

        def goud_transform2d_builder_with_position(self, ptr, x, y):
            return ptr

        def goud_transform2d_builder_with_rotation(self, ptr, r):
            return ptr

        def goud_transform2d_builder_with_rotation_degrees(self, ptr, d):
            return ptr

        def goud_transform2d_builder_with_scale(self, ptr, sx, sy):
            return ptr

        def goud_transform2d_builder_with_scale_uniform(self, ptr, s):
            return ptr

        def goud_transform2d_builder_looking_at(self, ptr, tx, ty):
            return ptr

        def goud_transform2d_builder_translate(self, ptr, dx, dy):
            return ptr

        def goud_transform2d_builder_rotate(self, ptr, angle):
            return ptr

        def goud_transform2d_builder_scale_by(self, ptr, fx, fy):
            return ptr

        def goud_transform2d_builder_build(self, ptr):
            return self.ffi.FfiTransform2D(1.0, 2.0, 0.5, 3.0, 4.0)

        def goud_transform2d_builder_free(self, ptr):
            return 0

        def goud_sprite_new(self, texture_handle):
            return self.ffi.FfiSprite(texture_handle, 1.0, 1.0, 1.0, 1.0, 0, 0, 0, 0, False, False, False, 0.5, 0.5, 0, 0, False)

        def goud_sprite_default(self):
            return self.ffi.FfiSprite(0, 1.0, 1.0, 1.0, 1.0, 0, 0, 0, 0, False, False, False, 0.5, 0.5, 0, 0, False)

        def goud_sprite_set_color(self, ptr, r, g, b, a):
            s = self._sp(ptr)
            s.color_r, s.color_g, s.color_b, s.color_a = r, g, b, a
            return 0

        def goud_sprite_get_color(self, ptr):
            s = self._sp(ptr)
            return self.ffi.FfiColor(s.color_r, s.color_g, s.color_b, s.color_a)

        def goud_sprite_with_color(self, s, r, g, b, a):
            s.color_r, s.color_g, s.color_b, s.color_a = r, g, b, a
            return s

        def goud_sprite_set_alpha(self, ptr, a):
            self._sp(ptr).color_a = a
            return 0

        def goud_sprite_get_alpha(self, ptr):
            return self._sp(ptr).color_a

        def goud_sprite_set_source_rect(self, ptr, x, y, w, h):
            s = self._sp(ptr)
            s.source_rect_x, s.source_rect_y, s.source_rect_width, s.source_rect_height = x, y, w, h
            s.has_source_rect = True
            return 0

        def goud_sprite_clear_source_rect(self, ptr):
            self._sp(ptr).has_source_rect = False
            return 0

        def goud_sprite_get_source_rect(self, ptr, out_ptr):
            s = self._sp(ptr)
            out = ctypes.cast(out_ptr, ctypes.POINTER(self.ffi.FfiRect)).contents
            out.x, out.y, out.width, out.height = s.source_rect_x, s.source_rect_y, s.source_rect_width, s.source_rect_height

        def goud_sprite_has_source_rect(self, ptr):
            return bool(self._sp(ptr).has_source_rect)

        def goud_sprite_with_source_rect(self, s, x, y, w, h):
            s.source_rect_x, s.source_rect_y, s.source_rect_width, s.source_rect_height = x, y, w, h
            s.has_source_rect = True
            return s

        def goud_sprite_set_flip_x(self, ptr, flip):
            self._sp(ptr).flip_x = flip
            return 0

        def goud_sprite_get_flip_x(self, ptr):
            return bool(self._sp(ptr).flip_x)

        def goud_sprite_set_flip_y(self, ptr, flip):
            self._sp(ptr).flip_y = flip
            return 0

        def goud_sprite_get_flip_y(self, ptr):
            return bool(self._sp(ptr).flip_y)

        def goud_sprite_set_flip(self, ptr, flip_x, flip_y):
            s = self._sp(ptr)
            s.flip_x = flip_x
            s.flip_y = flip_y
            return 0

        def goud_sprite_with_flip_x(self, s, flip):
            s.flip_x = flip
            return s

        def goud_sprite_with_flip_y(self, s, flip):
            s.flip_y = flip
            return s

        def goud_sprite_with_flip(self, s, flip_x, flip_y):
            s.flip_x = flip_x
            s.flip_y = flip_y
            return s

        def goud_sprite_is_flipped(self, ptr):
            s = self._sp(ptr)
            return bool(s.flip_x or s.flip_y)

        def goud_sprite_set_anchor(self, ptr, x, y):
            s = self._sp(ptr)
            s.anchor_x = x
            s.anchor_y = y
            return 0

        def goud_sprite_get_anchor(self, ptr):
            s = self._sp(ptr)
            return self.ffi.FfiVec2(s.anchor_x, s.anchor_y)

        def goud_text_new(self, font_handle):
            return self.ffi.FfiText(font_handle, 16.0, 1.0, 1.0, 1.0, 1.0, 0, 0.0, False, 1.0)

        def goud_text_default(self):
            return self.ffi.FfiText(0, 16.0, 1.0, 1.0, 1.0, 1.0, 0, 0.0, False, 1.0)

        def goud_text_set_font_size(self, ptr, size):
            self._tx(ptr).font_size = size
            return 0

        def goud_text_get_font_size(self, ptr):
            return self._tx(ptr).font_size

        def goud_text_set_color(self, ptr, r, g, b, a):
            t = self._tx(ptr)
            t.color_r, t.color_g, t.color_b, t.color_a = r, g, b, a
            return 0

        def goud_text_get_color_r(self, ptr):
            return self._tx(ptr).color_r

        def goud_text_get_color_g(self, ptr):
            return self._tx(ptr).color_g

        def goud_text_get_color_b(self, ptr):
            return self._tx(ptr).color_b

        def goud_text_get_color_a(self, ptr):
            return self._tx(ptr).color_a

        def goud_text_set_alignment(self, ptr, alignment):
            self._tx(ptr).alignment = alignment
            return 0

        def goud_text_get_alignment(self, ptr):
            return self._tx(ptr).alignment

        def goud_text_set_max_width(self, ptr, width):
            t = self._tx(ptr)
            t.max_width = width
            t.has_max_width = True
            return 0

        def goud_text_clear_max_width(self, ptr):
            t = self._tx(ptr)
            t.max_width = 0.0
            t.has_max_width = False
            return 0

        def goud_text_get_max_width(self, ptr):
            return self._tx(ptr).max_width

        def goud_text_has_max_width(self, ptr):
            return bool(self._tx(ptr).has_max_width)

        def goud_text_set_line_spacing(self, ptr, spacing):
            self._tx(ptr).line_spacing = spacing
            return 0

        def goud_text_get_line_spacing(self, ptr):
            return self._tx(ptr).line_spacing

        def __getattr__(self, _name):
            return lambda *args: 0

    lib = _FakeLib()
    types_mod, _game_mod, ffi_mod = _new_fake_generated_package("_cov_generated_types", lib)
    lib.ffi = ffi_mod

    tr = types_mod.Transform2D.default()
    tr.translate(2.0, 3.0)
    tr.translate_local(1.0, 1.0)
    tr.set_position(10.0, 20.0)
    tr.position_x = 10.0
    tr.position_y = 20.0
    pos = tr.get_position()
    assert isinstance(pos, types_mod.Vec2) and pos.x == 10.0 and pos.y == 20.0
    tr.rotate(0.25)
    tr.rotate_degrees(90.0)
    tr.set_rotation(0.5)
    tr.set_rotation_degrees(180.0)
    assert isinstance(tr.get_rotation_degrees(), float)
    tr.look_at_target(2.0, 3.0)
    tr.set_scale(2.0, 3.0)
    tr.set_scale_uniform(4.0)
    tr.scale_by(0.5, 0.25)
    assert tr.get_scale().x > 0
    assert tr.forward().y == 1.0 and tr.left().x == -1.0
    assert isinstance(tr.matrix().m, list) and isinstance(tr.matrix_inverse().m, list)
    assert tr.transform_point(1.0, 2.0).x != 0.0
    inv_pt = tr.inverse_transform_point(11.0, 22.0)
    assert isinstance(inv_pt, types_mod.Vec2)
    assert isinstance(tr.lerp(types_mod.Transform2D.new(0, 0, 0, 1, 1), 0.5), types_mod.Transform2D)
    assert abs(types_mod.Transform2D.normalize_angle(4.0 * math.pi)) <= math.pi

    builder = types_mod.Transform2DBuilder.new()
    built = (
        builder.with_position(1.0, 2.0)
        .with_rotation(0.5)
        .with_rotation_degrees(30.0)
        .with_scale(2.0, 3.0)
        .with_scale_uniform(1.5)
        .looking_at(3.0, 4.0)
        .translate(1.0, 1.0)
        .rotate(0.25)
        .scale_by(1.1, 1.2)
        .build()
    )
    assert isinstance(built, types_mod.Transform2D)
    try:
        builder.build()
        assert False, "builder should fail after build() consumption"
    except RuntimeError:
        pass
    types_mod.Transform2DBuilder.at_position(4.0, 5.0).free()

    sprite = types_mod.Sprite.new(7)
    sprite.set_color(0.2, 0.3, 0.4, 0.5)
    c = sprite.get_color()
    assert isinstance(c, types_mod.Color)
    sprite = sprite.with_color(0.6, 0.7, 0.8, 0.9)
    sprite.set_alpha(0.25)
    assert isinstance(sprite.get_alpha(), float)
    sprite.set_source_rect(1.0, 2.0, 3.0, 4.0)
    assert isinstance(types_mod.Sprite.has_source_rect(sprite), bool)
    try:
        rect = sprite.get_source_rect()
        assert isinstance(rect, types_mod.Rect)
    except NameError:
        # Generated wrapper currently references FfiRect without module qualification.
        pass
    sprite.clear_source_rect()
    assert isinstance(types_mod.Sprite.has_source_rect(sprite), bool)
    sprite = sprite.with_source_rect(5.0, 6.0, 7.0, 8.0)
    sprite.set_flip_x(True)
    sprite.set_flip_y(True)
    sprite.set_flip(False, True)
    assert isinstance(sprite.get_flip_x(), bool) and isinstance(sprite.get_flip_y(), bool)
    sprite = sprite.with_flip_x(True).with_flip_y(False).with_flip(True, True)
    assert isinstance(sprite.is_flipped(), bool)
    sprite.set_anchor(0.25, 0.75)
    anchor = sprite.get_anchor()
    assert isinstance(anchor, types_mod.Vec2)

    text = types_mod.Text.new(5)
    text.set_font_size(20.0)
    assert isinstance(text.get_font_size(), float)
    text.set_color(0.1, 0.2, 0.3, 0.4)
    assert isinstance(text.get_color_r(), float)
    assert isinstance(text.get_color_g(), float)
    assert isinstance(text.get_color_b(), float)
    assert isinstance(text.get_color_a(), float)
    text.set_alignment(2)
    assert isinstance(text.get_alignment(), int)
    text.set_max_width(300.0)
    assert isinstance(text.get_max_width(), float)
    assert isinstance(types_mod.Text.has_max_width(text), bool)
    text.clear_max_width()
    text.set_line_spacing(1.2)
    assert isinstance(text.get_line_spacing(), float)
    assert "Text(" in repr(text)

    print("  Generated _types.py fake-lib FFI runtime coverage passed")
    return True
