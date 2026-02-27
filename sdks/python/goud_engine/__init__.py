"""
GoudEngine Python SDK

A Python binding for the GoudEngine game engine, providing a Pythonic interface
to the Rust core through ctypes FFI.

Example Usage:
    from goud_engine import GoudGame, Keys, MouseButtons
    
    game = GoudGame(800, 600, "My Game")
    
    # Load textures
    tex_id = game.load_texture("player.png")
    
    # Game loop
    while game.is_running():
        dt = game.begin_frame()
        
        if game.key_just_pressed(Keys.ESCAPE):
            game.close()
        
        if game.key_pressed(Keys.SPACE):
            # Jump!
            pass
        
        game.end_frame()
    
    game.destroy()
"""

from .bindings import (
    # Core types
    GoudContext,
    GoudResult,
    GoudEntityId,
    
    # Component types
    Transform2D,
    Sprite,
    
    # Math types
    Vec2,
    Color,
    Rect,
    
    # High-level API
    Entity,
    GameEntity,
    GoudGame,
    
    # Input constants
    Keys,
    MouseButtons,
    
    # Error type
    GoudEngineError,
)

__version__ = "0.0.809"
__all__ = [
    # Core
    "GoudContext",
    "GoudResult", 
    "GoudEntityId",
    
    # Components
    "Transform2D",
    "Sprite",
    
    # Math
    "Vec2",
    "Color",
    "Rect",
    
    # High-level
    "Entity",
    "GameEntity",
    "GoudGame",
    
    # Input
    "Keys",
    "MouseButtons",
    
    # Errors
    "GoudEngineError",
]
