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

from goudengine import (
    GoudGame,
    Entity,
    Transform2D, Sprite,
    Vec2, Color,
    Key, MouseButton
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
    
    # Flat field access
    t3 = Transform2D()
    t3.position_x = 10
    t3.position_y = 20
    t3.rotation = 0.5
    t3.scale_x = 2
    t3.scale_y = 2
    print(f"  Flat field assignment: {t3}")

    # Sprite flat field assignment
    print("\nSprite Examples:")
    s1 = Sprite(texture_handle=42)
    print(f"  Basic sprite: {s1}")

    s2 = Sprite(texture_handle=42)
    s2.anchor_x = 0.5
    s2.anchor_y = 1.0
    s2.flip_x = True
    print(f"  Configured sprite: {s2}")
    
    print("\nComponent demo complete!")



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
        
        while not game.should_close():
            game.begin_frame()
            dt = game.delta_time
            frame_count += 1

            # Check for quit
            if game.is_key_just_pressed(Key.ESCAPE):
                print("  ESC pressed - closing")
                game.close()

            # Check for space
            if game.is_key_just_pressed(Key.SPACE):
                print(f"  [Frame {frame_count}] SPACE pressed!")

            # Check for mouse click
            if game.is_mouse_button_just_pressed(MouseButton.LEFT):
                pos = game.get_mouse_position()
                print(f"  [Frame {frame_count}] Click at ({pos.x:.0f}, {pos.y:.0f})")

            # Check for WASD (continuous)
            keys_held = []
            if game.is_key_pressed(Key.W):
                keys_held.append("W")
            if game.is_key_pressed(Key.A):
                keys_held.append("A")
            if game.is_key_pressed(Key.S):
                keys_held.append("S")
            if game.is_key_pressed(Key.D):
                keys_held.append("D")
            
            if keys_held and frame_count - last_print_frame > 30:
                print(f"  [Frame {frame_count}] Holding: {', '.join(keys_held)}")
                last_print_frame = frame_count
            
            game.end_frame()
        
        game.destroy()
        print(f"\nGame closed after {frame_count} frames")
        
    except Exception as e:
        print(f"\nCould not create window: {e}")
        print("Make sure the native library is built:")
        print("  cd goudengine && cargo build --release")
        return False

    print("\nGame window demo complete!")
    return True


def main():
    """Run all demos."""
    print("=" * 60)
    print(" GoudEngine Python SDK Demo")
    print("=" * 60)
    
    # Run component demos (no window needed)
    demo_components()
    
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
