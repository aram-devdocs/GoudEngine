#!/usr/bin/env python3
"""
GoudEngine Python SDK Test Suite

Basic tests to verify the Python bindings work correctly.
Run after building the native library:
    cd goud_engine && cargo build --release
    python sdks/python/test_bindings.py
"""

import sys
import math
from pathlib import Path

# Ensure goud_engine package is importable
sys.path.insert(0, str(Path(__file__).parent))


def test_imports():
    """Test that all public types can be imported."""
    print("Testing imports...")
    from goud_engine import (
        GoudContext,
        GoudResult,
        GoudEntityId,
        Transform2D,
        Sprite,
        Vec2,
        Color,
        Rect,
        Entity,
        GoudGame,
    )
    print("  ✓ All imports successful")
    return True


def test_vec2():
    """Test Vec2 operations."""
    print("Testing Vec2...")
    from goud_engine import Vec2
    
    # Construction
    v = Vec2(3.0, 4.0)
    assert v.x == 3.0, f"Expected x=3.0, got {v.x}"
    assert v.y == 4.0, f"Expected y=4.0, got {v.y}"
    
    # Static constructors
    assert Vec2.zero().x == 0.0 and Vec2.zero().y == 0.0
    assert Vec2.one().x == 1.0 and Vec2.one().y == 1.0
    
    # Operations
    a = Vec2(1.0, 2.0)
    b = Vec2(3.0, 4.0)
    
    c = a + b
    assert c.x == 4.0 and c.y == 6.0, f"Addition failed: {c}"
    
    c = a - b
    assert c.x == -2.0 and c.y == -2.0, f"Subtraction failed: {c}"
    
    c = a * 2.0
    assert c.x == 2.0 and c.y == 4.0, f"Multiplication failed: {c}"
    
    c = b / 2.0
    assert c.x == 1.5 and c.y == 2.0, f"Division failed: {c}"
    
    # Length
    v = Vec2(3.0, 4.0)
    assert v.length() == 5.0, f"Length failed: {v.length()}"
    
    # Normalize
    n = v.normalize()
    assert abs(n.length() - 1.0) < 0.001, f"Normalize failed: {n.length()}"
    
    print("  ✓ Vec2 tests passed")
    return True


def test_color():
    """Test Color operations."""
    print("Testing Color...")
    from goud_engine import Color
    
    def approx(a, b, eps=0.001):
        return abs(a - b) < eps
    
    # Construction
    c = Color(0.5, 0.6, 0.7, 0.8)
    assert approx(c.r, 0.5) and approx(c.g, 0.6) and approx(c.b, 0.7) and approx(c.a, 0.8)
    
    # Static colors
    white = Color.white()
    assert approx(white.r, 1.0) and approx(white.g, 1.0) and approx(white.b, 1.0) and approx(white.a, 1.0)
    
    red = Color.red()
    assert approx(red.r, 1.0) and approx(red.g, 0.0) and approx(red.b, 0.0)
    
    # From hex
    c = Color.from_hex(0xFF0000)
    assert approx(c.r, 1.0) and approx(c.g, 0.0) and approx(c.b, 0.0) and approx(c.a, 1.0)
    
    c = Color.from_hex(0xFF000080)  # With alpha (0x80 = 128/255 ≈ 0.502)
    assert approx(c.r, 1.0) and approx(c.a, 128/255, eps=0.01)
    
    print("  ✓ Color tests passed")
    return True


def test_rect():
    """Test Rect operations."""
    print("Testing Rect...")
    from goud_engine import Rect, Vec2
    
    r = Rect(10, 20, 100, 50)
    assert r.x == 10 and r.y == 20
    assert r.width == 100 and r.height == 50
    
    # Contains
    assert r.contains(Vec2(50, 30))
    assert not r.contains(Vec2(0, 0))
    assert not r.contains(Vec2(111, 71))  # Outside
    
    print("  ✓ Rect tests passed")
    return True


def test_transform2d():
    """Test Transform2D operations."""
    print("Testing Transform2D...")
    from goud_engine import Transform2D, Vec2
    
    # Factory methods
    t = Transform2D.from_position(100, 50)
    assert t.position.x == 100 and t.position.y == 50
    
    t = Transform2D.from_rotation(math.pi / 2)
    assert abs(t.rotation - math.pi / 2) < 0.001
    
    t = Transform2D.from_scale(2.0, 3.0)
    assert t.scale.x == 2.0 and t.scale.y == 3.0
    
    # Mutations
    t = Transform2D()
    t.translate(10, 20)
    assert t.position.x == 10 and t.position.y == 20
    
    t = Transform2D()
    t.rotate(math.pi / 4)
    assert abs(t.rotation - math.pi / 4) < 0.001
    
    # Direction vectors
    t = Transform2D.from_rotation(0)  # Facing right
    fwd = t.forward()
    assert abs(fwd.x - 1.0) < 0.001 and abs(fwd.y) < 0.001
    
    t = Transform2D.from_rotation(math.pi / 2)  # Facing up
    fwd = t.forward()
    assert abs(fwd.x) < 0.001 and abs(fwd.y - 1.0) < 0.001
    
    # Lerp
    a = Transform2D.from_position(0, 0)
    b = Transform2D.from_position(100, 100)
    mid = a.lerp(b, 0.5)
    assert mid.position.x == 50 and mid.position.y == 50
    
    # Chaining
    t = Transform2D().translate(10, 20).rotate(0.5).scale_by(2, 2)
    assert t.position.x == 10 and t.position.y == 20
    
    print("  ✓ Transform2D tests passed")
    return True


def test_sprite():
    """Test Sprite operations."""
    print("Testing Sprite...")
    from goud_engine import Sprite, Color, Vec2
    
    # Basic creation
    s = Sprite(texture_handle=42)
    assert s.texture_handle == 42
    
    # Builder pattern
    s = (Sprite(texture_handle=100)
        .with_color(1.0, 0.0, 0.0, 1.0)
        .with_flip_x(True)
        .with_anchor(0.5, 1.0))
    
    assert s.color.r == 1.0 and s.color.g == 0.0
    assert s.flip_x == True
    assert s.anchor.x == 0.5 and s.anchor.y == 1.0
    
    # Mutable operations
    s = Sprite(texture_handle=50)
    s.color = Color.green()
    assert s.color.g == 1.0
    
    s.flip_x = True
    assert s.flip_x == True
    
    s.anchor = Vec2(0, 0)
    assert s.anchor.x == 0 and s.anchor.y == 0
    
    print("  ✓ Sprite tests passed")
    return True


def test_context():
    """Test context and entity operations."""
    print("Testing Context...")
    from goud_engine import GoudContext
    
    # Creation
    ctx = GoudContext.create()
    assert ctx.is_valid()
    
    # Entity spawn
    e1 = ctx.spawn_entity()
    assert e1 != 0xFFFFFFFFFFFFFFFF  # Not invalid
    assert ctx.is_entity_alive(e1)
    assert ctx.entity_count() == 1
    
    e2 = ctx.spawn_entity()
    e3 = ctx.spawn_entity()
    assert ctx.entity_count() == 3
    
    # Batch spawn
    batch = ctx.spawn_entities(10)
    assert len(batch) == 10
    assert ctx.entity_count() == 13
    
    # Despawn
    assert ctx.despawn_entity(e1) == True
    assert not ctx.is_entity_alive(e1)
    assert ctx.entity_count() == 12
    
    # Batch despawn
    despawned = ctx.despawn_entities(batch[:5])
    assert despawned == 5
    assert ctx.entity_count() == 7
    
    # Cleanup
    ctx.destroy()
    assert not ctx.is_valid()
    
    print("  ✓ Context tests passed")
    return True


def test_context_manager():
    """Test context manager pattern."""
    print("Testing Context Manager...")
    from goud_engine import GoudContext
    
    with GoudContext.create() as ctx:
        e = ctx.spawn_entity()
        assert ctx.is_entity_alive(e)
        assert ctx.entity_count() == 1
    
    # Context should be destroyed after exiting the block
    # (can't check is_valid because the object itself may be garbage collected)
    
    print("  ✓ Context Manager tests passed")
    return True


def test_game():
    """Test high-level game API."""
    print("Testing Game API...")
    from goud_engine import GoudGame
    
    game = GoudGame(800, 600, "Test Game")
    assert game.is_valid()
    
    entity = game.spawn()
    assert entity.id != 0xFFFFFFFFFFFFFFFF
    assert entity.is_alive()
    
    entities = game.spawn_batch(5)
    assert len(entities) == 5
    
    entity.despawn()
    assert not entity.is_alive()
    
    game.close()
    
    print("  ✓ Game API tests passed")
    return True


def main():
    """Run all tests."""
    print("=" * 60)
    print(" GoudEngine Python SDK Tests")
    print("=" * 60)
    
    tests = [
        test_imports,
        test_vec2,
        test_color,
        test_rect,
        test_transform2d,
        test_sprite,
        test_context,
        test_context_manager,
        test_game,
    ]
    
    passed = 0
    failed = 0
    
    for test in tests:
        try:
            if test():
                passed += 1
            else:
                failed += 1
        except Exception as e:
            print(f"  ✗ {test.__name__} failed with exception: {e}")
            import traceback
            traceback.print_exc()
            failed += 1
    
    print("\n" + "=" * 60)
    print(f" Results: {passed} passed, {failed} failed")
    print("=" * 60)
    
    return 0 if failed == 0 else 1


if __name__ == "__main__":
    sys.exit(main())
