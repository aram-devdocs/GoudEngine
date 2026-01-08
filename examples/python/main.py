#!/usr/bin/env python3
"""
GoudEngine Python Demo

A simple demo showing the Python SDK capabilities including:
- Window creation
- Input handling  
- Game loop structure
- Transform2D and Sprite component usage

Usage:
    cd examples/python
    python main.py
    
Or run the Flappy Bird demo:
    python flappy_bird.py
"""

import sys
import math
from pathlib import Path

# Add the SDK to the Python path
sdk_path = Path(__file__).parent.parent.parent / "sdks" / "python"
sys.path.insert(0, str(sdk_path))

from goud_engine import (
    GoudGame, GoudEngineError,
    GoudContext, Entity,
    Transform2D, Sprite,
    Vec2, Color,
    Keys, MouseButtons
)


def demo_components():
    """Demonstrates Transform2D and Sprite component usage."""
    print("\n=== Component Demo ===\n")
    
    # Transform2D factory methods
    print("Transform2D Examples:")
    t1 = Transform2D.from_position(100, 50)
    print(f"  from_position(100, 50): {t1}")
    
    t2 = Transform2D.from_rotation(math.pi / 4)
    print(f"  from_rotation(π/4): {t2}")
    
    # Mutation and chaining
    t3 = Transform2D()
    t3.translate(10, 20).rotate(0.5).scale_by(2, 2)
    print(f"  Chained ops: {t3}")
    
    # Direction vectors
    t4 = Transform2D.from_rotation(math.pi / 4)
    print(f"  forward at 45°: {t4.forward()}")
    
    # Sprite builder pattern
    print("\nSprite Examples:")
    s1 = Sprite(texture_handle=42)
    print(f"  Basic sprite: {s1}")
    
    s2 = (Sprite(texture_handle=42)
        .with_color(1.0, 0.0, 0.0, 1.0)
        .with_flip_x(True)
        .with_anchor(0.5, 1.0))
    print(f"  Built sprite: {s2}")
    
    print("\n✓ Component demo complete!")


def demo_context():
    """Demonstrates context and entity management."""
    print("\n=== Context Demo ===\n")
    
    # Create context
    ctx = GoudContext.create()
    print(f"Created context: {ctx}")
    
    # Spawn entities
    e1 = ctx.spawn_entity()
    e2 = ctx.spawn_entity()
    print(f"Spawned entities: {e1}, {e2}")
    print(f"Entity count: {ctx.entity_count()}")
    
    # Batch spawn
    batch = ctx.spawn_entities(10)
    print(f"Batch spawned {len(batch)} entities")
    print(f"Total entities: {ctx.entity_count()}")
    
    # Despawn
    ctx.despawn_entity(e1)
    print(f"After despawning e1: {ctx.entity_count()} entities")
    
    # Cleanup
    ctx.destroy()
    print("Context destroyed")
    
    print("\n✓ Context demo complete!")


def demo_game_window():
    """Demonstrates windowed game with input handling."""
    print("\n=== Game Window Demo ===\n")
    print("Opening game window...")
    print("  Press SPACE or click to see input detection")
    print("  Press WASD to see movement keys")
    print("  Press ESC to close")
    print()
    
    try:
        game = GoudGame(640, 480, "GoudEngine Python Demo")
        
        frame_count = 0
        last_print_frame = 0
        
        while game.is_running():
            dt = game.begin_frame()
            frame_count += 1
            
            # Check for quit
            if game.key_just_pressed(Keys.ESCAPE):
                print("  ESC pressed - closing")
                game.close()
            
            # Check for space
            if game.key_just_pressed(Keys.SPACE):
                print(f"  [Frame {frame_count}] SPACE pressed!")
            
            # Check for mouse click
            if game.mouse_button_just_pressed(MouseButtons.LEFT):
                pos = game.get_mouse_position()
                print(f"  [Frame {frame_count}] Click at ({pos.x:.0f}, {pos.y:.0f})")
            
            # Check for WASD (continuous)
            keys_held = []
            if game.key_pressed(Keys.W):
                keys_held.append("W")
            if game.key_pressed(Keys.A):
                keys_held.append("A")
            if game.key_pressed(Keys.S):
                keys_held.append("S")
            if game.key_pressed(Keys.D):
                keys_held.append("D")
            
            if keys_held and frame_count - last_print_frame > 30:
                print(f"  [Frame {frame_count}] Holding: {', '.join(keys_held)}")
                last_print_frame = frame_count
            
            game.end_frame()
        
        game.destroy()
        print(f"\nGame closed after {frame_count} frames")
        
    except GoudEngineError as e:
        print(f"\n❌ Could not create window: {e}")
        print("Make sure the native library is built:")
        print("  cd goud_engine && cargo build --release")
        return False
    
    print("\n✓ Game window demo complete!")
    return True


def main():
    """Run all demos."""
    print("=" * 60)
    print(" GoudEngine Python SDK Demo")
    print("=" * 60)
    
    # Run component demos (no window needed)
    demo_components()
    demo_context()
    
    # Run window demo automatically
    print("\n" + "-" * 60)
    print("Opening game window demo...")
    print("Press ESC to close the window.")
    demo_game_window()
    
    print("\n" + "=" * 60)
    print(" All demos completed!")
    print("=" * 60)
    print("\nTry the Flappy Bird demo:")
    print("  python flappy_bird.py")
    
    return 0


if __name__ == "__main__":
    sys.exit(main())
