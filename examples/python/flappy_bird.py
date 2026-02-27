#!/usr/bin/env python3
"""
Flappy Bird Clone - GoudEngine Python Demo

This is a Python port of the flappy_goud C# example, demonstrating the
multi-language SDK architecture. All game logic patterns mirror the C# version.

This demo uses immediate-mode rendering with draw_sprite() calls each frame,
which is the recommended approach for the Python/C# SDKs.

Usage:
    cd examples/python
    python flappy_bird.py

Requirements:
    - GoudEngine native library built (cargo build --release)
    - Python 3.8+
"""

import sys
import math
import random
from pathlib import Path
from dataclasses import dataclass, field
from typing import List, Optional

# Add the SDK to the Python path
sdk_path = Path(__file__).parent.parent.parent / "sdks" / "python"
sys.path.insert(0, str(sdk_path))

from goud_engine import (
    GoudGame, GoudEngineError,
    Keys, MouseButtons,
    Transform2D, Sprite,
    Vec2, Color
)


# =============================================================================
# Game Constants (mirrors GameConstants.cs)
# =============================================================================

@dataclass(frozen=True)
class GameConstants:
    """Game configuration constants matching the C# version."""
    TARGET_FPS: int = 120
    BASE_HEIGHT: int = 112
    
    SCREEN_WIDTH: int = 288
    SCREEN_HEIGHT: int = 512
    
    GRAVITY: float = 9.8
    JUMP_STRENGTH: float = -3.5
    JUMP_COOLDOWN: float = 0.30
    
    PIPE_SPEED: float = 1.0
    PIPE_SPAWN_INTERVAL: float = 1.5
    PIPE_WIDTH: int = 60
    PIPE_GAP: int = 100
    
    # Bird dimensions (from sprite)
    BIRD_WIDTH: int = 34
    BIRD_HEIGHT: int = 24
    
    # Pipe dimensions (from sprite)
    PIPE_IMG_WIDTH: int = 52
    PIPE_IMG_HEIGHT: int = 320


CONSTANTS = GameConstants()


# =============================================================================
# Texture Manager (handles loading and storing textures)
# =============================================================================

class Textures:
    """
    Container for all game textures.
    
    Loaded once at startup and reused throughout the game loop.
    Uses immediate-mode rendering with draw_sprite() calls.
    """
    
    def __init__(self):
        # Background
        self.background: int = 0
        
        # Bird animation frames
        self.bird_frames: List[int] = []
        
        # Pipe texture
        self.pipe: int = 0
        
        # Base/ground texture
        self.base: int = 0
        
        # Score digit textures (0-9)
        self.digits: List[int] = []
    
    def load_all(self, game: 'GoudGame', asset_base: str):
        """
        Loads all game textures.
        
        Args:
            game: The GoudGame instance for loading textures
            asset_base: Base path to the assets folder
        """
        print("  Loading textures...")
        
        # Background
        self.background = game.load_texture(f"{asset_base}/sprites/background-day.png")
        print(f"    Loaded background texture: {self.background}")
        
        # Bird frames (bluebird)
        bird_frame_names = [
            "bluebird-downflap.png",
            "bluebird-midflap.png",
            "bluebird-upflap.png",
        ]
        for name in bird_frame_names:
            tex = game.load_texture(f"{asset_base}/sprites/{name}")
            self.bird_frames.append(tex)
        print(f"    Loaded {len(self.bird_frames)} bird animation frames")
        
        # Pipe
        self.pipe = game.load_texture(f"{asset_base}/sprites/pipe-green.png")
        print(f"    Loaded pipe texture: {self.pipe}")
        
        # Base/ground
        self.base = game.load_texture(f"{asset_base}/sprites/base.png")
        print(f"    Loaded base texture: {self.base}")
        
        # Score digits (0-9)
        for i in range(10):
            tex = game.load_texture(f"{asset_base}/sprites/{i}.png")
            self.digits.append(tex)
        print(f"    Loaded {len(self.digits)} digit textures")
        
        print("  All textures loaded successfully!")


# =============================================================================
# Movement (mirrors Movement.cs)
# =============================================================================

class Movement:
    """
    Physics-based movement system for the bird.
    
    Handles gravity, jumping, and rotation based on velocity.
    All logic mirrors the C# implementation.
    """
    
    ROTATION_SMOOTHING = 0.03
    
    def __init__(self, gravity: float, jump_strength: float):
        self.velocity: float = 0.0
        self.rotation: float = 0.0
        self._gravity = gravity
        self._jump_strength = jump_strength
        self._jump_cooldown_timer: float = 0.0
    
    def apply_gravity(self, delta_time: float):
        """Applies gravity to velocity."""
        self.velocity += self._gravity * delta_time * CONSTANTS.TARGET_FPS
        self._jump_cooldown_timer = max(0, self._jump_cooldown_timer - delta_time)
    
    def try_jump(self, delta_time: float):
        """Attempts to jump if cooldown has elapsed."""
        if self._jump_cooldown_timer <= 0:
            self._jump()
            self._jump_cooldown_timer = CONSTANTS.JUMP_COOLDOWN
    
    def _jump(self):
        """Performs the jump."""
        self.velocity = 0  # Reset velocity before jump
        self.velocity = self._jump_strength * CONSTANTS.TARGET_FPS
    
    def update_position(self, position_y: float, delta_time: float) -> float:
        """Updates Y position and rotation based on velocity."""
        new_y = position_y + self.velocity * delta_time
        
        # Smoothly update rotation based on velocity
        target_rotation = max(-45, min(45, self.velocity * 3))
        self.rotation += (target_rotation - self.rotation) * self.ROTATION_SMOOTHING
        
        return new_y


# =============================================================================
# Bird Animator (mirrors BirdAnimator.cs)
# =============================================================================

class BirdAnimator:
    """
    Handles bird sprite animation with frame cycling.
    
    In the C# version, this manages texture swapping for wing flaps.
    Here we track animation state that is used for rendering.
    """
    
    FRAME_DURATION = 0.1
    FRAME_COUNT = 3  # downflap, midflap, upflap
    
    def __init__(self, initial_x: float, initial_y: float):
        self.x = initial_x
        self.y = initial_y
        self.rotation = 0.0
        self.frame_index = 0  # Index into textures.bird_frames
        self._animation_time = 0.0
        
        # Frame names (for reference)
        self.frame_names = [
            "bluebird-downflap",
            "bluebird-midflap",
            "bluebird-upflap",
        ]
    
    @property
    def current_frame_name(self) -> str:
        """Returns the current animation frame name."""
        return self.frame_names[self.frame_index]
    
    def update(self, delta_time: float, x: float, y: float, rotation: float):
        """Updates animation state."""
        self.x = x
        self.y = y
        self.rotation = rotation
        
        # Advance animation
        self._animation_time += delta_time
        if self._animation_time >= self.FRAME_DURATION:
            self.frame_index = (self.frame_index + 1) % self.FRAME_COUNT
            self._animation_time = 0.0
    
    def reset(self, x: float, y: float):
        """Resets to initial state."""
        self.x = x
        self.y = y
        self.rotation = 0.0
        self.frame_index = 0
        self._animation_time = 0.0


# =============================================================================
# Bird (mirrors Bird.cs)
# =============================================================================

class Bird:
    """
    The player-controlled bird.
    
    Combines movement physics and animation, handling input
    to control the bird's flight.
    """
    
    def __init__(self):
        self.x = CONSTANTS.SCREEN_WIDTH / 4
        self.y = CONSTANTS.SCREEN_HEIGHT / 2
        self._movement = Movement(CONSTANTS.GRAVITY, CONSTANTS.JUMP_STRENGTH)
        self._animator = BirdAnimator(self.x, self.y)
    
    @property
    def rotation(self) -> float:
        """Gets the current rotation in degrees."""
        return self._movement.rotation
    
    @property
    def frame_index(self) -> int:
        """Gets the current animation frame index."""
        return self._animator.frame_index
    
    @property
    def current_frame(self) -> str:
        """Gets the current animation frame name."""
        return self._animator.current_frame_name
    
    def reset(self):
        """Resets bird to starting position."""
        self.x = CONSTANTS.SCREEN_WIDTH / 4
        self.y = CONSTANTS.SCREEN_HEIGHT / 2
        self._movement.velocity = 0
        self._animator.reset(self.x, self.y)
    
    def update(self, delta_time: float, jump_pressed: bool):
        """Updates bird state for this frame."""
        if jump_pressed:
            self._movement.try_jump(delta_time)
        
        self._movement.apply_gravity(delta_time)
        self.y = self._movement.update_position(self.y, delta_time)
        
        self._animator.update(delta_time, self.x, self.y, self._movement.rotation)
    
    def get_bounds(self) -> tuple:
        """Returns (x, y, width, height) bounding box."""
        return (self.x, self.y, CONSTANTS.BIRD_WIDTH, CONSTANTS.BIRD_HEIGHT)


# =============================================================================
# Pipe (mirrors Pipe.cs)
# =============================================================================

class Pipe:
    """
    A pair of pipes (top and bottom) forming an obstacle.
    
    Pipes move left across the screen, and the bird must fly
    through the gap between them.
    """
    
    def __init__(self):
        self.x = CONSTANTS.SCREEN_WIDTH
        
        # Random gap Y position
        self.gap_y = random.randint(
            CONSTANTS.PIPE_GAP,
            CONSTANTS.SCREEN_HEIGHT - CONSTANTS.PIPE_GAP
        )
        
        # Calculate pipe positions
        self.top_y = self.gap_y - CONSTANTS.PIPE_GAP - CONSTANTS.PIPE_IMG_HEIGHT
        self.bottom_y = self.gap_y + CONSTANTS.PIPE_GAP
        
        self._passed = False
    
    def update(self, delta_time: float):
        """Moves the pipe left."""
        self.x -= CONSTANTS.PIPE_SPEED * delta_time * CONSTANTS.TARGET_FPS
    
    def is_off_screen(self) -> bool:
        """Returns True if the pipe has moved off the left edge."""
        return self.x + CONSTANTS.PIPE_WIDTH < 0
    
    def is_passed(self, bird_x: float) -> bool:
        """Returns True if the bird has passed this pipe."""
        return bird_x > self.x + CONSTANTS.PIPE_WIDTH
    
    def get_top_bounds(self) -> tuple:
        """Returns (x, y, width, height) for top pipe collision."""
        return (self.x, self.top_y, CONSTANTS.PIPE_IMG_WIDTH, CONSTANTS.PIPE_IMG_HEIGHT)
    
    def get_bottom_bounds(self) -> tuple:
        """Returns (x, y, width, height) for bottom pipe collision."""
        return (self.x, self.bottom_y, CONSTANTS.PIPE_IMG_WIDTH, CONSTANTS.PIPE_IMG_HEIGHT)


# =============================================================================
# Score Counter (mirrors ScoreCounter.cs)
# =============================================================================

class ScoreCounter:
    """
    Tracks and displays the player's score.
    
    In the C# version, this manages digit sprites for display.
    Here we track the numeric value and provide display helpers.
    """
    
    def __init__(self):
        self.score = 0
    
    def increment(self):
        """Adds one to the score."""
        self.score += 1
    
    def reset(self):
        """Resets score to zero."""
        self.score = 0
    
    def get_digits(self) -> List[int]:
        """Returns list of individual digits for display."""
        if self.score == 0:
            return [0]
        
        digits = []
        n = self.score
        while n > 0:
            digits.append(n % 10)
            n //= 10
        return list(reversed(digits))


# =============================================================================
# Game Manager (mirrors GameManager.cs)
# =============================================================================

class GameManager:
    """
    Main game controller managing all game state and logic.
    
    Coordinates the bird, pipes, scoring, and collision detection.
    This mirrors the C# GameManager class structure.
    
    Uses immediate-mode rendering with draw_sprite() calls each frame.
    """
    
    def __init__(self, game: GoudGame, textures: Textures):
        self._game = game
        self._textures = textures
        self._bird = Bird()
        self._pipes: List[Pipe] = []
        self._score_counter = ScoreCounter()
        self._pipe_spawn_timer = 0.0
        self._game_over = False
        
        # Track if we've printed the welcome message
        self._welcome_printed = False
    
    def start(self):
        """Starts/restarts the game."""
        self._bird.reset()
        self._pipes.clear()
        self._score_counter.reset()
        self._pipe_spawn_timer = 0.0
        self._game_over = False
        
        if not self._welcome_printed:
            print("\n" + "=" * 50)
            print("  🐦 Flappy Bird - GoudEngine Python Demo")
            print("=" * 50)
            print("\n  Controls:")
            print("    SPACE or Left Click  - Flap / Jump")
            print("    R                    - Restart")
            print("    ESC                  - Quit")
            print("\n" + "-" * 50)
            self._welcome_printed = True
    
    def update(self, delta_time: float):
        """Main update loop."""
        # Handle quit
        if self._game.key_just_pressed(Keys.ESCAPE):
            self._game.close()
            return
        
        # Handle restart
        if self._game.key_just_pressed(Keys.R):
            self._reset_game()
            return
        
        # Check for jump input
        jump_pressed = (
            self._game.key_just_pressed(Keys.SPACE) or
            self._game.mouse_button_just_pressed(MouseButtons.LEFT)
        )
        
        # Update bird
        self._bird.update(delta_time, jump_pressed)
        
        # Check if bird hit the ground/ceiling
        if self._bird.y < 0 or self._bird.y > CONSTANTS.SCREEN_HEIGHT:
            self._reset_game()
            return
        
        # Check if bird hit the base
        if self._bird.y + CONSTANTS.BIRD_HEIGHT > CONSTANTS.SCREEN_HEIGHT:
            self._reset_game()
            return
        
        # Update pipes and check collisions
        bird_bounds = self._bird.get_bounds()
        for pipe in self._pipes:
            pipe.update(delta_time)
            
            # Check collision with top pipe
            if self._check_collision(bird_bounds, pipe.get_top_bounds()):
                self._reset_game()
                return
            
            # Check collision with bottom pipe
            if self._check_collision(bird_bounds, pipe.get_bottom_bounds()):
                self._reset_game()
                return
        
        # Spawn new pipes
        self._pipe_spawn_timer += delta_time
        if self._pipe_spawn_timer > CONSTANTS.PIPE_SPAWN_INTERVAL:
            self._pipe_spawn_timer = 0.0
            self._pipes.append(Pipe())
        
        # Remove off-screen pipes and count score
        pipes_to_remove = []
        for pipe in self._pipes:
            if pipe.is_off_screen():
                pipes_to_remove.append(pipe)
                self._score_counter.increment()
                print(f"  Score: {self._score_counter.score}")
        
        for pipe in pipes_to_remove:
            self._pipes.remove(pipe)
    
    def _check_collision(self, bounds_a: tuple, bounds_b: tuple) -> bool:
        """Simple AABB collision check."""
        ax, ay, aw, ah = bounds_a
        bx, by, bw, bh = bounds_b
        
        return (ax < bx + bw and
                ax + aw > bx and
                ay < by + bh and
                ay + ah > by)
    
    def _reset_game(self):
        """Resets the game state."""
        if self._score_counter.score > 0:
            print(f"\n  💀 Game Over! Final Score: {self._score_counter.score}")
            print("-" * 50)
        self.start()
    
    @property
    def bird(self) -> Bird:
        """Gets the bird for rendering."""
        return self._bird
    
    @property
    def pipes(self) -> List[Pipe]:
        """Gets all pipes for rendering."""
        return self._pipes
    
    @property
    def score(self) -> int:
        """Gets the current score."""
        return self._score_counter.score
    
    def draw(self):
        """
        Renders all game objects using immediate-mode draw calls.
        
        This method should be called every frame after update() and before end_frame().
        All sprites are drawn using draw_sprite() calls - nothing is retained between frames.
        """
        tex = self._textures
        game = self._game
        
        # === LAYER 0: Background ===
        game.draw_sprite(
            tex.background,
            CONSTANTS.SCREEN_WIDTH / 2,  # x center
            CONSTANTS.SCREEN_HEIGHT / 2,  # y center
            CONSTANTS.SCREEN_WIDTH,
            CONSTANTS.SCREEN_HEIGHT
        )
        
        # === LAYER 1: Score (behind pipes, in front of background) ===
        self._draw_score()
        
        # === LAYER 2: Pipes ===
        for pipe in self._pipes:
            # Draw top pipe (flipped vertically, hanging from top)
            top_center_y = pipe.top_y + CONSTANTS.PIPE_IMG_HEIGHT / 2
            game.draw_sprite(
                tex.pipe,
                pipe.x + CONSTANTS.PIPE_IMG_WIDTH / 2,
                top_center_y,
                CONSTANTS.PIPE_IMG_WIDTH,
                CONSTANTS.PIPE_IMG_HEIGHT,
                math.pi  # Rotate 180 degrees for top pipe
            )
            
            # Draw bottom pipe (normal orientation)
            bottom_center_y = pipe.bottom_y + CONSTANTS.PIPE_IMG_HEIGHT / 2
            game.draw_sprite(
                tex.pipe,
                pipe.x + CONSTANTS.PIPE_IMG_WIDTH / 2,
                bottom_center_y,
                CONSTANTS.PIPE_IMG_WIDTH,
                CONSTANTS.PIPE_IMG_HEIGHT
            )
        
        # === LAYER 3: Bird ===
        bird = self._bird
        bird_texture = tex.bird_frames[bird.frame_index]
        rotation_rad = math.radians(bird.rotation)
        game.draw_sprite(
            bird_texture,
            bird.x + CONSTANTS.BIRD_WIDTH / 2,
            bird.y + CONSTANTS.BIRD_HEIGHT / 2,
            CONSTANTS.BIRD_WIDTH,
            CONSTANTS.BIRD_HEIGHT,
            rotation_rad
        )
        
        # === LAYER 4: Base/ground (on top of everything in game area) ===
        game.draw_sprite(
            tex.base,
            CONSTANTS.SCREEN_WIDTH / 2,
            CONSTANTS.SCREEN_HEIGHT + CONSTANTS.BASE_HEIGHT / 2,
            CONSTANTS.SCREEN_WIDTH,
            CONSTANTS.BASE_HEIGHT
        )
    
    def _draw_score(self):
        """Draws the current score at the top of the screen."""
        tex = self._textures
        game = self._game
        
        digits = self._score_counter.get_digits()
        digit_width = 24  # Approximate width of digit sprites
        digit_height = 36  # Approximate height of digit sprites
        
        # Calculate total width to center the score
        total_width = len(digits) * digit_width
        start_x = (CONSTANTS.SCREEN_WIDTH - total_width) / 2 + digit_width / 2
        y = 50  # Distance from top
        
        for i, digit in enumerate(digits):
            x = start_x + i * digit_width
            game.draw_sprite(
                tex.digits[digit],
                x, y,
                digit_width, digit_height
            )


# =============================================================================
# Main Entry Point
# =============================================================================

def main():
    """Main entry point for the Flappy Bird demo."""
    print("\nGoudEngine Python SDK - Flappy Bird Demo")
    print("-" * 40)
    
    try:
        # Create game window
        game = GoudGame(
            CONSTANTS.SCREEN_WIDTH,
            CONSTANTS.SCREEN_HEIGHT + CONSTANTS.BASE_HEIGHT,
            "Flappy Bird - Python"
        )
        
        print(f"Window created: {CONSTANTS.SCREEN_WIDTH}x{CONSTANTS.SCREEN_HEIGHT}")
        
        # Load textures
        # Find assets directory (relative to this script -> csharp/flappy_goud example)
        script_dir = Path(__file__).parent
        assets_base = script_dir.parent / "csharp" / "flappy_goud" / "assets"
        
        if not assets_base.exists():
            print(f"\n❌ Assets directory not found: {assets_base}")
            print("Make sure you're running from the examples/python directory")
            return 1
        
        textures = Textures()
        textures.load_all(game, str(assets_base))
        
        # Create game manager with textures
        manager = GameManager(game, textures)
        manager.start()
        
        # Game loop
        while game.is_running():
            # Begin frame (polls events, clears screen)
            dt = game.begin_frame()
            
            # Update game logic
            manager.update(dt)
            
            # Render all game objects (immediate-mode rendering)
            manager.draw()
            
            # End frame (presents to screen)
            game.end_frame()
        
        # Cleanup
        game.destroy()
        print("\nGame closed successfully!")
        
    except GoudEngineError as e:
        print(f"\n❌ Engine Error: {e}")
        print("\nMake sure the native library is built:")
        print("  cd goud_engine && cargo build --release")
        return 1
    except Exception as e:
        print(f"\n❌ Unexpected Error: {e}")
        import traceback
        traceback.print_exc()
        return 1
    
    return 0


if __name__ == "__main__":
    sys.exit(main())
