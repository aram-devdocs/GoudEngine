"""
GoudEngine Python FFI Bindings

This module provides Python bindings to the GoudEngine Rust core using ctypes.
All FFI types and functions are defined here, wrapping the native library.

Design Philosophy:
    All logic lives in Rust. This SDK is a thin wrapper that marshals data
    and calls FFI functions. This ensures consistent behavior across all
    language bindings (C#, Python, Rust native).
"""

import ctypes
import platform
import os
from ctypes import (
    Structure, c_float, c_uint64, c_uint32, c_int32,
    c_bool, c_uint8, POINTER, byref, c_void_p, c_char_p
)
from typing import Optional, List, Callable, Any
from pathlib import Path


# =============================================================================
# Library Loading
# =============================================================================

def _get_library_path() -> str:
    """Determine the correct native library path based on platform."""
    system = platform.system()
    
    # Try to find library in common locations
    search_paths = [
        # Development paths (relative to this file)
        Path(__file__).parent.parent.parent.parent / "target" / "release",
        Path(__file__).parent.parent.parent.parent / "target" / "debug",
        # Installed paths
        Path("/usr/local/lib"),
        Path("/usr/lib"),
        # Current directory
        Path("."),
    ]
    
    if system == "Darwin":
        lib_name = "libgoud_engine.dylib"
    elif system == "Linux":
        lib_name = "libgoud_engine.so"
    elif system == "Windows":
        lib_name = "goud_engine.dll"
    else:
        raise OSError(f"Unsupported platform: {system}")
    
    for path in search_paths:
        lib_path = path / lib_name
        if lib_path.exists():
            return str(lib_path)
    
    # Fall back to letting the system find it
    return lib_name


def _load_library():
    """Load the native library."""
    lib_path = _get_library_path()
    try:
        return ctypes.CDLL(lib_path)
    except OSError as e:
        raise OSError(
            f"Failed to load GoudEngine native library from '{lib_path}'.\n"
            f"Make sure the library is built: cd goud_engine && cargo build --release\n"
            f"Error: {e}"
        ) from e


# Load the library at module import time
_lib = _load_library()


# =============================================================================
# FFI Type Definitions
# =============================================================================

class GoudContextId(Structure):
    """FFI-safe context identifier (packed index + generation)."""
    _fields_ = [("_bits", c_uint64)]
    
    @classmethod
    def invalid(cls) -> "GoudContextId":
        """Returns the invalid context ID sentinel."""
        return cls(_bits=0xFFFFFFFFFFFFFFFF)
    
    def is_invalid(self) -> bool:
        """Returns True if this is the invalid sentinel."""
        return self._bits == 0xFFFFFFFFFFFFFFFF
    
    def __repr__(self) -> str:
        if self.is_invalid():
            return "GoudContextId(INVALID)"
        index = self._bits & 0xFFFFFFFF
        generation = (self._bits >> 32) & 0xFFFFFFFF
        return f"GoudContextId(index={index}, gen={generation})"


class GoudEntityId(Structure):
    """FFI-safe entity identifier."""
    _fields_ = [("bits", c_uint64)]
    
    INVALID = 0xFFFFFFFFFFFFFFFF
    
    @classmethod
    def invalid(cls) -> "GoudEntityId":
        """Returns the invalid entity ID sentinel."""
        return cls(bits=cls.INVALID)
    
    def is_invalid(self) -> bool:
        """Returns True if this is the invalid sentinel."""
        return self.bits == self.INVALID
    
    def __repr__(self) -> str:
        if self.is_invalid():
            return "GoudEntityId(INVALID)"
        return f"GoudEntityId({self.bits})"


class GoudResult(Structure):
    """FFI-safe result type for operations that can fail."""
    _fields_ = [
        ("code", c_int32),
        ("success", c_bool),
    ]
    
    def is_ok(self) -> bool:
        """Returns True if this result indicates success."""
        return self.success
    
    def is_err(self) -> bool:
        """Returns True if this result indicates failure."""
        return not self.success
    
    def __repr__(self) -> str:
        if self.success:
            return "GoudResult(Success)"
        return f"GoudResult(Error: code={self.code})"
    
    def raise_on_error(self, message: str = "Operation failed"):
        """Raises GoudEngineError if this result is an error."""
        if self.is_err():
            raise GoudEngineError(f"{message}: error code {self.code}")


class Vec2(Structure):
    """FFI-safe 2D vector."""
    _fields_ = [
        ("x", c_float),
        ("y", c_float),
    ]
    
    def __init__(self, x: float = 0.0, y: float = 0.0):
        super().__init__()
        self.x = x
        self.y = y
    
    def __repr__(self) -> str:
        return f"Vec2({self.x}, {self.y})"
    
    def __add__(self, other: "Vec2") -> "Vec2":
        return Vec2(self.x + other.x, self.y + other.y)
    
    def __sub__(self, other: "Vec2") -> "Vec2":
        return Vec2(self.x - other.x, self.y - other.y)
    
    def __mul__(self, scalar: float) -> "Vec2":
        return Vec2(self.x * scalar, self.y * scalar)
    
    def __truediv__(self, scalar: float) -> "Vec2":
        return Vec2(self.x / scalar, self.y / scalar)
    
    def length(self) -> float:
        """Returns the length of this vector."""
        return (self.x ** 2 + self.y ** 2) ** 0.5
    
    def normalize(self) -> "Vec2":
        """Returns a normalized copy of this vector."""
        length = self.length()
        if length == 0:
            return Vec2(0, 0)
        return self / length
    
    @classmethod
    def zero(cls) -> "Vec2":
        return cls(0.0, 0.0)
    
    @classmethod
    def one(cls) -> "Vec2":
        return cls(1.0, 1.0)


class Color(Structure):
    """FFI-safe RGBA color."""
    _fields_ = [
        ("r", c_float),
        ("g", c_float),
        ("b", c_float),
        ("a", c_float),
    ]
    
    def __new__(cls, r: float = 1.0, g: float = 1.0, b: float = 1.0, a: float = 1.0):
        # Use __new__ to properly initialize ctypes Structure with our values
        instance = super().__new__(cls)
        instance.r = r
        instance.g = g
        instance.b = b
        instance.a = a
        return instance
    
    def __init__(self, r: float = 1.0, g: float = 1.0, b: float = 1.0, a: float = 1.0):
        # Fields are set in __new__, this is just for IDE completeness
        pass
    
    def __repr__(self) -> str:
        return f"Color({self.r}, {self.g}, {self.b}, {self.a})"
    
    @classmethod
    def white(cls) -> "Color":
        return cls(1.0, 1.0, 1.0, 1.0)
    
    @classmethod
    def black(cls) -> "Color":
        return cls(0.0, 0.0, 0.0, 1.0)
    
    @classmethod
    def red(cls) -> "Color":
        return cls(1.0, 0.0, 0.0, 1.0)
    
    @classmethod
    def green(cls) -> "Color":
        return cls(0.0, 1.0, 0.0, 1.0)
    
    @classmethod
    def blue(cls) -> "Color":
        return cls(0.0, 0.0, 1.0, 1.0)
    
    @classmethod
    def yellow(cls) -> "Color":
        return cls(1.0, 1.0, 0.0, 1.0)
    
    @classmethod
    def transparent(cls) -> "Color":
        return cls(0.0, 0.0, 0.0, 0.0)
    
    @classmethod
    def from_hex(cls, hex_value: int) -> "Color":
        """Creates a color from a hex value (0xRRGGBB or 0xRRGGBBAA)."""
        if hex_value > 0xFFFFFF:
            # Has alpha
            r = ((hex_value >> 24) & 0xFF) / 255.0
            g = ((hex_value >> 16) & 0xFF) / 255.0
            b = ((hex_value >> 8) & 0xFF) / 255.0
            a = (hex_value & 0xFF) / 255.0
        else:
            # No alpha
            r = ((hex_value >> 16) & 0xFF) / 255.0
            g = ((hex_value >> 8) & 0xFF) / 255.0
            b = (hex_value & 0xFF) / 255.0
            a = 1.0
        return cls(r, g, b, a)


class Rect(Structure):
    """FFI-safe rectangle."""
    _fields_ = [
        ("x", c_float),
        ("y", c_float),
        ("width", c_float),
        ("height", c_float),
    ]
    
    def __init__(self, x: float = 0.0, y: float = 0.0, 
                 width: float = 0.0, height: float = 0.0):
        super().__init__()
        self.x = x
        self.y = y
        self.width = width
        self.height = height
    
    def __repr__(self) -> str:
        return f"Rect({self.x}, {self.y}, {self.width}, {self.height})"
    
    def contains(self, point: Vec2) -> bool:
        """Returns True if the point is inside this rectangle."""
        return (self.x <= point.x < self.x + self.width and
                self.y <= point.y < self.y + self.height)


class Mat3x3(Structure):
    """FFI-safe 3x3 matrix (column-major order)."""
    _fields_ = [("m", c_float * 9)]
    
    def __repr__(self) -> str:
        return f"Mat3x3({list(self.m)})"


# =============================================================================
# Transform2D Component
# =============================================================================

class FfiTransform2D(Structure):
    """FFI-safe Transform2D representation."""
    _fields_ = [
        ("position_x", c_float),
        ("position_y", c_float),
        ("rotation", c_float),
        ("scale_x", c_float),
        ("scale_y", c_float),
    ]


class Transform2D:
    """
    2D transformation component with position, rotation, and scale.
    
    All operations delegate to the Rust FFI layer to ensure consistent
    behavior across all language bindings.
    
    Example:
        transform = Transform2D.from_position(100, 50)
        transform.translate(10, 20)
        transform.rotate(math.pi / 4)
        forward = transform.forward()
    """
    
    def __init__(self, position: Vec2 = None, rotation: float = 0.0, scale: Vec2 = None):
        """Creates a new Transform2D with the given values."""
        self._ffi = FfiTransform2D()
        if position is None:
            position = Vec2(0, 0)
        if scale is None:
            scale = Vec2(1, 1)
        self._ffi.position_x = position.x
        self._ffi.position_y = position.y
        self._ffi.rotation = rotation
        self._ffi.scale_x = scale.x
        self._ffi.scale_y = scale.y
    
    @classmethod
    def _from_ffi(cls, ffi: FfiTransform2D) -> "Transform2D":
        """Internal: Create from FFI struct."""
        t = cls.__new__(cls)
        t._ffi = ffi
        return t
    
    @property
    def position(self) -> Vec2:
        """Gets the position."""
        return Vec2(self._ffi.position_x, self._ffi.position_y)
    
    @position.setter
    def position(self, value: Vec2):
        """Sets the position."""
        self._ffi.position_x = value.x
        self._ffi.position_y = value.y
    
    @property
    def rotation(self) -> float:
        """Gets the rotation in radians."""
        return self._ffi.rotation
    
    @rotation.setter
    def rotation(self, value: float):
        """Sets the rotation in radians."""
        _lib.goud_transform2d_set_rotation(byref(self._ffi), c_float(value))
    
    @property
    def scale(self) -> Vec2:
        """Gets the scale."""
        return Vec2(self._ffi.scale_x, self._ffi.scale_y)
    
    @scale.setter
    def scale(self, value: Vec2):
        """Sets the scale."""
        self._ffi.scale_x = value.x
        self._ffi.scale_y = value.y
    
    # Factory methods (delegate to Rust FFI)
    
    @classmethod
    def from_position(cls, x: float, y: float) -> "Transform2D":
        """Creates a transform at the given position with default rotation and scale."""
        ffi = _lib.goud_transform2d_from_position(c_float(x), c_float(y))
        return cls._from_ffi(ffi)
    
    @classmethod
    def from_rotation(cls, rotation: float) -> "Transform2D":
        """Creates a transform with the given rotation (in radians)."""
        ffi = _lib.goud_transform2d_from_rotation(c_float(rotation))
        return cls._from_ffi(ffi)
    
    @classmethod
    def from_rotation_degrees(cls, degrees: float) -> "Transform2D":
        """Creates a transform with the given rotation (in degrees)."""
        ffi = _lib.goud_transform2d_from_rotation_degrees(c_float(degrees))
        return cls._from_ffi(ffi)
    
    @classmethod
    def from_scale(cls, scale_x: float, scale_y: float) -> "Transform2D":
        """Creates a transform with the given scale."""
        ffi = _lib.goud_transform2d_from_scale(c_float(scale_x), c_float(scale_y))
        return cls._from_ffi(ffi)
    
    @classmethod
    def from_scale_uniform(cls, scale: float) -> "Transform2D":
        """Creates a transform with uniform scale on both axes."""
        ffi = _lib.goud_transform2d_from_scale_uniform(c_float(scale))
        return cls._from_ffi(ffi)
    
    @classmethod
    def look_at(cls, pos_x: float, pos_y: float, target_x: float, target_y: float) -> "Transform2D":
        """Creates a transform positioned and looking at a target."""
        ffi = _lib.goud_transform2d_look_at(
            c_float(pos_x), c_float(pos_y),
            c_float(target_x), c_float(target_y)
        )
        return cls._from_ffi(ffi)
    
    # Mutation methods (delegate to Rust FFI)
    
    def translate(self, dx: float, dy: float) -> "Transform2D":
        """Translates by the given offset in world space. Returns self for chaining."""
        _lib.goud_transform2d_translate(byref(self._ffi), c_float(dx), c_float(dy))
        return self
    
    def translate_local(self, dx: float, dy: float) -> "Transform2D":
        """Translates by the given offset in local space. Returns self for chaining."""
        _lib.goud_transform2d_translate_local(byref(self._ffi), c_float(dx), c_float(dy))
        return self
    
    def rotate(self, angle: float) -> "Transform2D":
        """Rotates by the given angle in radians. Returns self for chaining."""
        _lib.goud_transform2d_rotate(byref(self._ffi), c_float(angle))
        return self
    
    def rotate_degrees(self, degrees: float) -> "Transform2D":
        """Rotates by the given angle in degrees. Returns self for chaining."""
        _lib.goud_transform2d_rotate_degrees(byref(self._ffi), c_float(degrees))
        return self
    
    def look_at_target(self, target_x: float, target_y: float) -> "Transform2D":
        """Makes the transform look at a target position. Returns self for chaining."""
        _lib.goud_transform2d_look_at_target(
            byref(self._ffi), c_float(target_x), c_float(target_y)
        )
        return self
    
    def scale_by(self, factor_x: float, factor_y: float) -> "Transform2D":
        """Multiplies the current scale. Returns self for chaining."""
        _lib.goud_transform2d_scale_by(
            byref(self._ffi), c_float(factor_x), c_float(factor_y)
        )
        return self
    
    # Direction vectors (delegate to Rust FFI)
    
    def forward(self) -> Vec2:
        """Returns the forward direction vector."""
        result = _lib.goud_transform2d_forward(byref(self._ffi))
        return Vec2(result.x, result.y)
    
    def right(self) -> Vec2:
        """Returns the right direction vector."""
        result = _lib.goud_transform2d_right(byref(self._ffi))
        return Vec2(result.x, result.y)
    
    def backward(self) -> Vec2:
        """Returns the backward direction vector."""
        result = _lib.goud_transform2d_backward(byref(self._ffi))
        return Vec2(result.x, result.y)
    
    def left(self) -> Vec2:
        """Returns the left direction vector."""
        result = _lib.goud_transform2d_left(byref(self._ffi))
        return Vec2(result.x, result.y)
    
    # Point transformation (delegate to Rust FFI)
    
    def transform_point(self, x: float, y: float) -> Vec2:
        """Transforms a point from local to world space."""
        result = _lib.goud_transform2d_transform_point(
            byref(self._ffi), c_float(x), c_float(y)
        )
        return Vec2(result.x, result.y)
    
    def inverse_transform_point(self, x: float, y: float) -> Vec2:
        """Transforms a point from world to local space."""
        result = _lib.goud_transform2d_inverse_transform_point(
            byref(self._ffi), c_float(x), c_float(y)
        )
        return Vec2(result.x, result.y)
    
    # Interpolation (delegate to Rust FFI)
    
    def lerp(self, other: "Transform2D", t: float) -> "Transform2D":
        """Linearly interpolates between this and another transform."""
        ffi = _lib.goud_transform2d_lerp(self._ffi, other._ffi, c_float(t))
        return Transform2D._from_ffi(ffi)
    
    def __repr__(self) -> str:
        return (f"Transform2D(position=({self._ffi.position_x}, {self._ffi.position_y}), "
                f"rotation={self._ffi.rotation}, "
                f"scale=({self._ffi.scale_x}, {self._ffi.scale_y}))")


# =============================================================================
# Sprite Component
# =============================================================================

class FfiSprite(Structure):
    """FFI-safe Sprite representation."""
    _fields_ = [
        ("texture_handle", c_uint64),
        ("color_r", c_float),
        ("color_g", c_float),
        ("color_b", c_float),
        ("color_a", c_float),
        ("source_rect_x", c_float),
        ("source_rect_y", c_float),
        ("source_rect_width", c_float),
        ("source_rect_height", c_float),
        ("has_source_rect", c_bool),
        ("flip_x", c_bool),
        ("flip_y", c_bool),
        ("anchor_x", c_float),
        ("anchor_y", c_float),
        ("custom_size_x", c_float),
        ("custom_size_y", c_float),
        ("has_custom_size", c_bool),
    ]


class Sprite:
    """
    Sprite component for 2D rendering.
    
    All operations delegate to the Rust FFI layer to ensure consistent
    behavior across all language bindings.
    
    Example:
        sprite = Sprite(texture_handle=tex_id)
        sprite = sprite.with_color(1.0, 0.0, 0.0, 1.0)  # Red tint
        sprite = sprite.with_flip_x(True)
        sprite = sprite.with_anchor(0.5, 1.0)  # Bottom center
    """
    
    def __init__(self, texture_handle: int = 0xFFFFFFFFFFFFFFFF):
        """Creates a new Sprite with the given texture handle."""
        self._ffi = _lib.goud_sprite_new(c_uint64(texture_handle))
    
    @classmethod
    def _from_ffi(cls, ffi: FfiSprite) -> "Sprite":
        """Internal: Create from FFI struct."""
        s = cls.__new__(cls)
        s._ffi = ffi
        return s
    
    @property
    def texture_handle(self) -> int:
        """Gets the texture handle."""
        return self._ffi.texture_handle
    
    @texture_handle.setter
    def texture_handle(self, value: int):
        """Sets the texture handle."""
        _lib.goud_sprite_set_texture(byref(self._ffi), c_uint64(value))
    
    @property
    def color(self) -> Color:
        """Gets the color tint."""
        return Color(
            self._ffi.color_r,
            self._ffi.color_g,
            self._ffi.color_b,
            self._ffi.color_a
        )
    
    @color.setter
    def color(self, value: Color):
        """Sets the color tint."""
        _lib.goud_sprite_set_color(
            byref(self._ffi),
            c_float(value.r), c_float(value.g),
            c_float(value.b), c_float(value.a)
        )
    
    @property
    def flip_x(self) -> bool:
        """Gets the horizontal flip state."""
        return self._ffi.flip_x
    
    @flip_x.setter
    def flip_x(self, value: bool):
        """Sets the horizontal flip state."""
        _lib.goud_sprite_set_flip_x(byref(self._ffi), c_bool(value))
    
    @property
    def flip_y(self) -> bool:
        """Gets the vertical flip state."""
        return self._ffi.flip_y
    
    @flip_y.setter
    def flip_y(self, value: bool):
        """Sets the vertical flip state."""
        _lib.goud_sprite_set_flip_y(byref(self._ffi), c_bool(value))
    
    @property
    def anchor(self) -> Vec2:
        """Gets the anchor point (normalized 0-1)."""
        return Vec2(self._ffi.anchor_x, self._ffi.anchor_y)
    
    @anchor.setter
    def anchor(self, value: Vec2):
        """Sets the anchor point (normalized 0-1)."""
        _lib.goud_sprite_set_anchor(byref(self._ffi), c_float(value.x), c_float(value.y))
    
    # Builder pattern methods (delegate to Rust FFI, return new instances)
    
    def with_color(self, r: float, g: float, b: float, a: float = 1.0) -> "Sprite":
        """Returns a new sprite with the given color tint."""
        ffi = _lib.goud_sprite_with_color(
            self._ffi, c_float(r), c_float(g), c_float(b), c_float(a)
        )
        return Sprite._from_ffi(ffi)
    
    def with_flip_x(self, flip: bool) -> "Sprite":
        """Returns a new sprite with the given horizontal flip."""
        ffi = _lib.goud_sprite_with_flip_x(self._ffi, c_bool(flip))
        return Sprite._from_ffi(ffi)
    
    def with_flip_y(self, flip: bool) -> "Sprite":
        """Returns a new sprite with the given vertical flip."""
        ffi = _lib.goud_sprite_with_flip_y(self._ffi, c_bool(flip))
        return Sprite._from_ffi(ffi)
    
    def with_anchor(self, x: float, y: float) -> "Sprite":
        """Returns a new sprite with the given anchor point."""
        ffi = _lib.goud_sprite_with_anchor(self._ffi, c_float(x), c_float(y))
        return Sprite._from_ffi(ffi)
    
    def with_source_rect(self, x: float, y: float, width: float, height: float) -> "Sprite":
        """Returns a new sprite with the given source rectangle for sprite sheets."""
        ffi = _lib.goud_sprite_with_source_rect(
            self._ffi, c_float(x), c_float(y), c_float(width), c_float(height)
        )
        return Sprite._from_ffi(ffi)
    
    def with_custom_size(self, width: float, height: float) -> "Sprite":
        """Returns a new sprite with a custom render size."""
        ffi = _lib.goud_sprite_with_custom_size(
            self._ffi, c_float(width), c_float(height)
        )
        return Sprite._from_ffi(ffi)
    
    def set_source_rect(self, x: float, y: float, width: float, height: float) -> "Sprite":
        """Sets the source rectangle. Returns self for chaining."""
        _lib.goud_sprite_set_source_rect(
            byref(self._ffi), c_float(x), c_float(y), c_float(width), c_float(height)
        )
        return self
    
    def clear_source_rect(self) -> "Sprite":
        """Clears the source rectangle. Returns self for chaining."""
        _lib.goud_sprite_clear_source_rect(byref(self._ffi))
        return self
    
    def set_custom_size(self, width: float, height: float) -> "Sprite":
        """Sets a custom render size. Returns self for chaining."""
        _lib.goud_sprite_set_custom_size(
            byref(self._ffi), c_float(width), c_float(height)
        )
        return self
    
    def clear_custom_size(self) -> "Sprite":
        """Clears the custom size. Returns self for chaining."""
        _lib.goud_sprite_clear_custom_size(byref(self._ffi))
        return self
    
    def __repr__(self) -> str:
        return (f"Sprite(texture={self._ffi.texture_handle}, "
                f"color=({self._ffi.color_r}, {self._ffi.color_g}, "
                f"{self._ffi.color_b}, {self._ffi.color_a}), "
                f"flip=({self._ffi.flip_x}, {self._ffi.flip_y}))")


# =============================================================================
# Context Management
# =============================================================================

class GoudEngineError(Exception):
    """Exception raised for GoudEngine errors."""
    pass


class GoudContext:
    """
    Engine context managing an ECS world and associated resources.
    
    Each context is isolated and has its own entities, components, and resources.
    
    Example:
        ctx = GoudContext.create()
        entity_id = ctx.spawn_entity()
        ctx.destroy()
    """
    
    def __init__(self, context_id: GoudContextId):
        """Internal: Create wrapper around existing context ID."""
        self._id = context_id
        self._destroyed = False
    
    @classmethod
    def create(cls) -> "GoudContext":
        """Creates a new engine context."""
        ctx_id = _lib.goud_context_create()
        if ctx_id.is_invalid():
            raise GoudEngineError("Failed to create context")
        return cls(ctx_id)
    
    @property
    def id(self) -> GoudContextId:
        """Gets the raw context ID."""
        return self._id
    
    def is_valid(self) -> bool:
        """Returns True if this context is still valid."""
        if self._destroyed:
            return False
        return _lib.goud_context_is_valid(self._id)
    
    def destroy(self):
        """Destroys this context and frees all resources."""
        if not self._destroyed:
            _lib.goud_context_destroy(self._id)
            self._destroyed = True
    
    def spawn_entity(self) -> int:
        """Spawns a new empty entity. Returns the entity ID (u64)."""
        entity_id = _lib.goud_entity_spawn_empty(self._id)
        if entity_id == GoudEntityId.INVALID:
            raise GoudEngineError("Failed to spawn entity")
        return entity_id
    
    def spawn_entities(self, count: int) -> List[int]:
        """Spawns multiple empty entities. Returns list of entity IDs."""
        out_array = (c_uint64 * count)()
        actual = _lib.goud_entity_spawn_batch(self._id, c_uint32(count), out_array)
        return list(out_array[:actual])
    
    def despawn_entity(self, entity_id: int) -> bool:
        """Despawns an entity. Returns True if successful."""
        result = _lib.goud_entity_despawn(self._id, c_uint64(entity_id))
        return result.success
    
    def despawn_entities(self, entity_ids: List[int]) -> int:
        """Despawns multiple entities. Returns count of entities despawned."""
        array = (c_uint64 * len(entity_ids))(*entity_ids)
        return _lib.goud_entity_despawn_batch(self._id, array, c_uint32(len(entity_ids)))
    
    def is_entity_alive(self, entity_id: int) -> bool:
        """Returns True if the entity is alive."""
        return _lib.goud_entity_is_alive(self._id, c_uint64(entity_id))
    
    def entity_count(self) -> int:
        """Returns the number of alive entities."""
        return _lib.goud_entity_count(self._id)
    
    def __enter__(self) -> "GoudContext":
        return self
    
    def __exit__(self, exc_type, exc_val, exc_tb):
        self.destroy()
    
    def __repr__(self) -> str:
        return f"GoudContext({self._id})"


# =============================================================================
# Entity Wrapper
# =============================================================================

class Entity:
    """
    High-level entity wrapper providing a convenient interface for entity operations.
    
    Example:
        entity = ctx.wrap_entity(entity_id)
        entity.add(Transform2D.from_position(100, 50))
        entity.add(Sprite(texture_id))
        entity.despawn()
    """
    
    def __init__(self, context: GoudContext, entity_id: int):
        self._context = context
        self._id = entity_id
    
    @property
    def id(self) -> int:
        """Gets the raw entity ID."""
        return self._id
    
    def is_alive(self) -> bool:
        """Returns True if this entity is still alive."""
        return self._context.is_entity_alive(self._id)
    
    def despawn(self):
        """Despawns this entity."""
        self._context.despawn_entity(self._id)
    
    def __repr__(self) -> str:
        return f"Entity({self._id})"


# =============================================================================
# Input Constants (GLFW key codes)
# =============================================================================

class Keys:
    """GLFW key codes for input handling."""
    SPACE = 32
    APOSTROPHE = 39
    COMMA = 44
    MINUS = 45
    PERIOD = 46
    SLASH = 47
    NUM_0 = 48
    NUM_1 = 49
    NUM_2 = 50
    NUM_3 = 51
    NUM_4 = 52
    NUM_5 = 53
    NUM_6 = 54
    NUM_7 = 55
    NUM_8 = 56
    NUM_9 = 57
    A = 65
    B = 66
    C = 67
    D = 68
    E = 69
    F = 70
    G = 71
    H = 72
    I = 73
    J = 74
    K = 75
    L = 76
    M = 77
    N = 78
    O = 79
    P = 80
    Q = 81
    R = 82
    S = 83
    T = 84
    U = 85
    V = 86
    W = 87
    X = 88
    Y = 89
    Z = 90
    ESCAPE = 256
    ENTER = 257
    TAB = 258
    BACKSPACE = 259
    INSERT = 260
    DELETE = 261
    RIGHT = 262
    LEFT = 263
    DOWN = 264
    UP = 265
    PAGE_UP = 266
    PAGE_DOWN = 267
    HOME = 268
    END = 269
    F1 = 290
    F2 = 291
    F3 = 292
    F4 = 293
    F5 = 294
    F6 = 295
    F7 = 296
    F8 = 297
    F9 = 298
    F10 = 299
    F11 = 300
    F12 = 301
    LEFT_SHIFT = 340
    LEFT_CONTROL = 341
    LEFT_ALT = 342
    RIGHT_SHIFT = 344
    RIGHT_CONTROL = 345
    RIGHT_ALT = 346


class MouseButtons:
    """GLFW mouse button codes."""
    LEFT = 0
    RIGHT = 1
    MIDDLE = 2
    BUTTON_4 = 3
    BUTTON_5 = 4


# =============================================================================
# High-Level Game API
# =============================================================================

class GoudGame:
    """
    High-level game abstraction for easy game development.
    
    This class provides a complete game loop with window, rendering, and input.
    
    Example:
        game = GoudGame(800, 600, "My Game")
        
        # Load textures
        tex_id = game.load_texture("player.png")
        
        # Game loop
        while game.is_running():
            dt = game.begin_frame()
            
            if game.key_just_pressed(Keys.ESCAPE):
                game.close()
            
            # Update game logic...
            
            game.end_frame()
        
        game.destroy()
    """
    
    def __init__(self, width: int, height: int, title: str):
        """Creates a new game instance with a window."""
        self._width = width
        self._height = height
        self._title = title
        self._context_id = GoudContextId.invalid()
        self._delta_time = 0.0
        self._textures: dict = {}
        
        # Create windowed context
        title_bytes = title.encode('utf-8') + b'\0'
        self._context_id = _lib.goud_window_create(
            c_uint32(width), c_uint32(height), title_bytes
        )
        
        if self._context_id.is_invalid():
            raise GoudEngineError("Failed to create game window")
        
        # Enable blending for sprites
        _lib.goud_renderer_enable_blending(self._context_id)
    
    @property
    def width(self) -> int:
        """Gets the window width."""
        return self._width
    
    @property
    def height(self) -> int:
        """Gets the window height."""
        return self._height
    
    @property
    def delta_time(self) -> float:
        """Gets the time since last frame in seconds."""
        return self._delta_time
    
    def is_running(self) -> bool:
        """Returns True if the game window is still open."""
        if self._context_id.is_invalid():
            return False
        return not _lib.goud_window_should_close(self._context_id)
    
    def is_valid(self) -> bool:
        """Returns True if the game context is valid."""
        return not self._context_id.is_invalid()
    
    def close(self):
        """Signals the game to close."""
        if not self._context_id.is_invalid():
            _lib.goud_window_set_should_close(self._context_id, True)
    
    def begin_frame(self) -> float:
        """
        Begins a new frame, polling events and clearing the screen.
        
        Returns the delta time since last frame.
        """
        # Poll events and get delta time
        self._delta_time = _lib.goud_window_poll_events(self._context_id)
        
        # Clear screen with sky blue color (Flappy Bird style)
        _lib.goud_window_clear(self._context_id, 0.4, 0.7, 0.9, 1.0)
        
        # Begin rendering
        _lib.goud_renderer_begin(self._context_id)
        
        return self._delta_time
    
    def end_frame(self):
        """Ends the current frame, presenting the rendered content."""
        # End rendering
        _lib.goud_renderer_end(self._context_id)
        
        # Swap buffers
        _lib.goud_window_swap_buffers(self._context_id)
    
    def destroy(self):
        """Destroys the game and releases all resources."""
        if not self._context_id.is_invalid():
            # Destroy all loaded textures
            for tex_handle in self._textures.values():
                _lib.goud_texture_destroy(self._context_id, c_uint64(tex_handle))
            self._textures.clear()
            
            # Destroy window
            _lib.goud_window_destroy(self._context_id)
            self._context_id = GoudContextId.invalid()
    
    # =========================================================================
    # Texture Management
    # =========================================================================
    
    def load_texture(self, path: str) -> int:
        """Loads a texture from file and returns its handle."""
        if path in self._textures:
            return self._textures[path]
        
        path_bytes = path.encode('utf-8') + b'\0'
        handle = _lib.goud_texture_load(self._context_id, path_bytes)
        
        if handle == 0xFFFFFFFFFFFFFFFF:
            raise GoudEngineError(f"Failed to load texture: {path}")
        
        self._textures[path] = handle
        return handle
    
    # =========================================================================
    # Input Handling
    # =========================================================================
    
    def key_pressed(self, key: int) -> bool:
        """Returns True if the key is currently pressed."""
        return _lib.goud_input_key_pressed(self._context_id, c_int32(key))
    
    def key_just_pressed(self, key: int) -> bool:
        """Returns True if the key was just pressed this frame."""
        return _lib.goud_input_key_just_pressed(self._context_id, c_int32(key))
    
    def key_just_released(self, key: int) -> bool:
        """Returns True if the key was just released this frame."""
        return _lib.goud_input_key_just_released(self._context_id, c_int32(key))
    
    def mouse_button_pressed(self, button: int) -> bool:
        """Returns True if the mouse button is currently pressed."""
        return _lib.goud_input_mouse_button_pressed(self._context_id, c_int32(button))
    
    def mouse_button_just_pressed(self, button: int) -> bool:
        """Returns True if the mouse button was just pressed this frame."""
        return _lib.goud_input_mouse_button_just_pressed(self._context_id, c_int32(button))
    
    def get_mouse_position(self) -> Vec2:
        """Returns the current mouse position."""
        x = c_float()
        y = c_float()
        _lib.goud_input_get_mouse_position(self._context_id, byref(x), byref(y))
        return Vec2(x.value, y.value)
    
    # =========================================================================
    # Collision Detection
    # =========================================================================
    
    def check_aabb_overlap(self, ax: float, ay: float, aw: float, ah: float,
                           bx: float, by: float, bw: float, bh: float) -> bool:
        """Checks if two axis-aligned bounding boxes overlap."""
        return _lib.goud_collision_aabb_overlap(
            c_float(ax), c_float(ay), c_float(ax + aw), c_float(ay + ah),
            c_float(bx), c_float(by), c_float(bx + bw), c_float(by + bh)
        )
    
    # =========================================================================
    # Immediate-Mode Rendering
    # =========================================================================
    
    def draw_sprite(self, texture: int, x: float, y: float,
                    width: float, height: float, rotation: float = 0.0,
                    color: Color = None) -> bool:
        """
        Draws a textured sprite at the given position.
        
        This is an immediate-mode draw call - the sprite is rendered immediately
        and not retained between frames. Call this each frame in your game loop.
        
        Args:
            texture: Texture handle from load_texture()
            x: X position (center of sprite)
            y: Y position (center of sprite)
            width: Width of the sprite
            height: Height of the sprite
            rotation: Rotation in radians (default 0.0)
            color: Color tint (default white/no tint)
        
        Returns:
            True on success, False on error.
        
        Example:
            tex_id = game.load_texture("player.png")
            game.draw_sprite(tex_id, 100, 100, 64, 64)
        """
        c = color or Color.white()
        return _lib.goud_renderer_draw_sprite(
            self._context_id, c_uint64(texture),
            c_float(x), c_float(y), c_float(width), c_float(height),
            c_float(rotation),
            c_float(c.r), c_float(c.g), c_float(c.b), c_float(c.a)
        )
    
    def draw_quad(self, x: float, y: float, width: float, height: float,
                  color: Color = None) -> bool:
        """
        Draws a colored quad (no texture) at the given position.
        
        This is an immediate-mode draw call - the quad is rendered immediately
        and not retained between frames. Call this each frame in your game loop.
        
        Args:
            x: X position (center of quad)
            y: Y position (center of quad)
            width: Width of the quad
            height: Height of the quad
            color: Color of the quad (default white)
        
        Returns:
            True on success, False on error.
        
        Example:
            game.draw_quad(100, 100, 50, 50, Color.red())
        """
        c = color or Color.white()
        return _lib.goud_renderer_draw_quad(
            self._context_id,
            c_float(x), c_float(y), c_float(width), c_float(height),
            c_float(c.r), c_float(c.g), c_float(c.b), c_float(c.a)
        )
    
    # =========================================================================
    # Entity Management
    # =========================================================================
    
    def spawn(self) -> "GameEntity":
        """Spawns a new empty entity and returns a GameEntity wrapper."""
        entity_id = _lib.goud_entity_spawn_empty(self._context_id)
        if entity_id == GoudEntityId.INVALID:
            raise GoudEngineError("Failed to spawn entity")
        return GameEntity(self, entity_id)
    
    def spawn_batch(self, count: int) -> List["GameEntity"]:
        """Spawns multiple empty entities and returns a list of GameEntity wrappers."""
        out_array = (c_uint64 * count)()
        actual = _lib.goud_entity_spawn_batch(self._context_id, c_uint32(count), out_array)
        return [GameEntity(self, entity_id) for entity_id in out_array[:actual]]
    
    def despawn_entity(self, entity_id: int) -> bool:
        """Despawns an entity by ID. Returns True if successful."""
        result = _lib.goud_entity_despawn(self._context_id, c_uint64(entity_id))
        return result.success
    
    def is_entity_alive(self, entity_id: int) -> bool:
        """Returns True if the entity is alive."""
        return _lib.goud_entity_is_alive(self._context_id, c_uint64(entity_id))
    
    def entity_count(self) -> int:
        """Returns the number of alive entities."""
        return _lib.goud_entity_count(self._context_id)
    
    # =========================================================================
    # Context Manager
    # =========================================================================
    
    def __enter__(self) -> "GoudGame":
        return self
    
    def __exit__(self, exc_type, exc_val, exc_tb):
        self.destroy()


class GameEntity:
    """
    Entity wrapper for use with GoudGame.
    
    Provides a convenient interface for entity operations within a game context.
    """
    
    def __init__(self, game: GoudGame, entity_id: int):
        self._game = game
        self._id = entity_id
    
    @property
    def id(self) -> int:
        """Gets the raw entity ID."""
        return self._id
    
    def is_alive(self) -> bool:
        """Returns True if this entity is still alive."""
        return self._game.is_entity_alive(self._id)
    
    def despawn(self):
        """Despawns this entity."""
        self._game.despawn_entity(self._id)
    
    def __repr__(self) -> str:
        return f"GameEntity({self._id})"


# =============================================================================
# FFI Function Signatures
# =============================================================================

def _setup_ffi_signatures():
    """Set up ctypes function signatures for type checking and proper return values."""
    
    # Context functions
    _lib.goud_context_create.argtypes = []
    _lib.goud_context_create.restype = GoudContextId
    
    _lib.goud_context_destroy.argtypes = [GoudContextId]
    _lib.goud_context_destroy.restype = c_bool
    
    _lib.goud_context_is_valid.argtypes = [GoudContextId]
    _lib.goud_context_is_valid.restype = c_bool
    
    # Entity functions
    _lib.goud_entity_spawn_empty.argtypes = [GoudContextId]
    _lib.goud_entity_spawn_empty.restype = c_uint64
    
    _lib.goud_entity_spawn_batch.argtypes = [GoudContextId, c_uint32, POINTER(c_uint64)]
    _lib.goud_entity_spawn_batch.restype = c_uint32
    
    _lib.goud_entity_despawn.argtypes = [GoudContextId, c_uint64]
    _lib.goud_entity_despawn.restype = GoudResult
    
    _lib.goud_entity_despawn_batch.argtypes = [GoudContextId, POINTER(c_uint64), c_uint32]
    _lib.goud_entity_despawn_batch.restype = c_uint32
    
    _lib.goud_entity_is_alive.argtypes = [GoudContextId, c_uint64]
    _lib.goud_entity_is_alive.restype = c_bool
    
    _lib.goud_entity_count.argtypes = [GoudContextId]
    _lib.goud_entity_count.restype = c_uint32
    
    # Transform2D factory functions
    _lib.goud_transform2d_default.argtypes = []
    _lib.goud_transform2d_default.restype = FfiTransform2D
    
    _lib.goud_transform2d_from_position.argtypes = [c_float, c_float]
    _lib.goud_transform2d_from_position.restype = FfiTransform2D
    
    _lib.goud_transform2d_from_rotation.argtypes = [c_float]
    _lib.goud_transform2d_from_rotation.restype = FfiTransform2D
    
    _lib.goud_transform2d_from_rotation_degrees.argtypes = [c_float]
    _lib.goud_transform2d_from_rotation_degrees.restype = FfiTransform2D
    
    _lib.goud_transform2d_from_scale.argtypes = [c_float, c_float]
    _lib.goud_transform2d_from_scale.restype = FfiTransform2D
    
    _lib.goud_transform2d_from_scale_uniform.argtypes = [c_float]
    _lib.goud_transform2d_from_scale_uniform.restype = FfiTransform2D
    
    _lib.goud_transform2d_look_at.argtypes = [c_float, c_float, c_float, c_float]
    _lib.goud_transform2d_look_at.restype = FfiTransform2D
    
    # Transform2D mutation functions
    _lib.goud_transform2d_translate.argtypes = [POINTER(FfiTransform2D), c_float, c_float]
    _lib.goud_transform2d_translate.restype = None
    
    _lib.goud_transform2d_translate_local.argtypes = [POINTER(FfiTransform2D), c_float, c_float]
    _lib.goud_transform2d_translate_local.restype = None
    
    _lib.goud_transform2d_set_position.argtypes = [POINTER(FfiTransform2D), c_float, c_float]
    _lib.goud_transform2d_set_position.restype = None
    
    _lib.goud_transform2d_rotate.argtypes = [POINTER(FfiTransform2D), c_float]
    _lib.goud_transform2d_rotate.restype = None
    
    _lib.goud_transform2d_rotate_degrees.argtypes = [POINTER(FfiTransform2D), c_float]
    _lib.goud_transform2d_rotate_degrees.restype = None
    
    _lib.goud_transform2d_set_rotation.argtypes = [POINTER(FfiTransform2D), c_float]
    _lib.goud_transform2d_set_rotation.restype = None
    
    _lib.goud_transform2d_look_at_target.argtypes = [POINTER(FfiTransform2D), c_float, c_float]
    _lib.goud_transform2d_look_at_target.restype = None
    
    _lib.goud_transform2d_set_scale.argtypes = [POINTER(FfiTransform2D), c_float, c_float]
    _lib.goud_transform2d_set_scale.restype = None
    
    _lib.goud_transform2d_scale_by.argtypes = [POINTER(FfiTransform2D), c_float, c_float]
    _lib.goud_transform2d_scale_by.restype = None
    
    # Transform2D direction functions
    _lib.goud_transform2d_forward.argtypes = [POINTER(FfiTransform2D)]
    _lib.goud_transform2d_forward.restype = Vec2
    
    _lib.goud_transform2d_right.argtypes = [POINTER(FfiTransform2D)]
    _lib.goud_transform2d_right.restype = Vec2
    
    _lib.goud_transform2d_backward.argtypes = [POINTER(FfiTransform2D)]
    _lib.goud_transform2d_backward.restype = Vec2
    
    _lib.goud_transform2d_left.argtypes = [POINTER(FfiTransform2D)]
    _lib.goud_transform2d_left.restype = Vec2
    
    # Transform2D point transformation
    _lib.goud_transform2d_transform_point.argtypes = [POINTER(FfiTransform2D), c_float, c_float]
    _lib.goud_transform2d_transform_point.restype = Vec2
    
    _lib.goud_transform2d_inverse_transform_point.argtypes = [POINTER(FfiTransform2D), c_float, c_float]
    _lib.goud_transform2d_inverse_transform_point.restype = Vec2
    
    # Transform2D interpolation
    _lib.goud_transform2d_lerp.argtypes = [FfiTransform2D, FfiTransform2D, c_float]
    _lib.goud_transform2d_lerp.restype = FfiTransform2D
    
    # Sprite functions
    _lib.goud_sprite_new.argtypes = [c_uint64]
    _lib.goud_sprite_new.restype = FfiSprite
    
    _lib.goud_sprite_default.argtypes = []
    _lib.goud_sprite_default.restype = FfiSprite
    
    _lib.goud_sprite_set_color.argtypes = [POINTER(FfiSprite), c_float, c_float, c_float, c_float]
    _lib.goud_sprite_set_color.restype = None
    
    _lib.goud_sprite_set_flip_x.argtypes = [POINTER(FfiSprite), c_bool]
    _lib.goud_sprite_set_flip_x.restype = None
    
    _lib.goud_sprite_set_flip_y.argtypes = [POINTER(FfiSprite), c_bool]
    _lib.goud_sprite_set_flip_y.restype = None
    
    _lib.goud_sprite_set_anchor.argtypes = [POINTER(FfiSprite), c_float, c_float]
    _lib.goud_sprite_set_anchor.restype = None
    
    _lib.goud_sprite_set_source_rect.argtypes = [POINTER(FfiSprite), c_float, c_float, c_float, c_float]
    _lib.goud_sprite_set_source_rect.restype = None
    
    _lib.goud_sprite_clear_source_rect.argtypes = [POINTER(FfiSprite)]
    _lib.goud_sprite_clear_source_rect.restype = None
    
    _lib.goud_sprite_set_custom_size.argtypes = [POINTER(FfiSprite), c_float, c_float]
    _lib.goud_sprite_set_custom_size.restype = None
    
    _lib.goud_sprite_clear_custom_size.argtypes = [POINTER(FfiSprite)]
    _lib.goud_sprite_clear_custom_size.restype = None
    
    _lib.goud_sprite_set_texture.argtypes = [POINTER(FfiSprite), c_uint64]
    _lib.goud_sprite_set_texture.restype = None
    
    # Sprite builder pattern functions
    _lib.goud_sprite_with_color.argtypes = [FfiSprite, c_float, c_float, c_float, c_float]
    _lib.goud_sprite_with_color.restype = FfiSprite
    
    _lib.goud_sprite_with_flip_x.argtypes = [FfiSprite, c_bool]
    _lib.goud_sprite_with_flip_x.restype = FfiSprite
    
    _lib.goud_sprite_with_flip_y.argtypes = [FfiSprite, c_bool]
    _lib.goud_sprite_with_flip_y.restype = FfiSprite
    
    _lib.goud_sprite_with_anchor.argtypes = [FfiSprite, c_float, c_float]
    _lib.goud_sprite_with_anchor.restype = FfiSprite
    
    _lib.goud_sprite_with_source_rect.argtypes = [FfiSprite, c_float, c_float, c_float, c_float]
    _lib.goud_sprite_with_source_rect.restype = FfiSprite
    
    _lib.goud_sprite_with_custom_size.argtypes = [FfiSprite, c_float, c_float]
    _lib.goud_sprite_with_custom_size.restype = FfiSprite
    
    # Window functions
    _lib.goud_window_create.argtypes = [c_uint32, c_uint32, c_char_p]
    _lib.goud_window_create.restype = GoudContextId
    
    _lib.goud_window_destroy.argtypes = [GoudContextId]
    _lib.goud_window_destroy.restype = c_bool
    
    _lib.goud_window_should_close.argtypes = [GoudContextId]
    _lib.goud_window_should_close.restype = c_bool
    
    _lib.goud_window_set_should_close.argtypes = [GoudContextId, c_bool]
    _lib.goud_window_set_should_close.restype = None
    
    _lib.goud_window_poll_events.argtypes = [GoudContextId]
    _lib.goud_window_poll_events.restype = c_float
    
    _lib.goud_window_swap_buffers.argtypes = [GoudContextId]
    _lib.goud_window_swap_buffers.restype = None
    
    _lib.goud_window_clear.argtypes = [GoudContextId, c_float, c_float, c_float, c_float]
    _lib.goud_window_clear.restype = None
    
    _lib.goud_window_get_delta_time.argtypes = [GoudContextId]
    _lib.goud_window_get_delta_time.restype = c_float
    
    # Renderer functions
    _lib.goud_renderer_begin.argtypes = [GoudContextId]
    _lib.goud_renderer_begin.restype = c_bool
    
    _lib.goud_renderer_end.argtypes = [GoudContextId]
    _lib.goud_renderer_end.restype = c_bool
    
    _lib.goud_renderer_enable_blending.argtypes = [GoudContextId]
    _lib.goud_renderer_enable_blending.restype = None
    
    _lib.goud_renderer_disable_blending.argtypes = [GoudContextId]
    _lib.goud_renderer_disable_blending.restype = None
    
    # Texture functions
    _lib.goud_texture_load.argtypes = [GoudContextId, c_char_p]
    _lib.goud_texture_load.restype = c_uint64
    
    _lib.goud_texture_destroy.argtypes = [GoudContextId, c_uint64]
    _lib.goud_texture_destroy.restype = c_bool
    
    # Input functions
    _lib.goud_input_key_pressed.argtypes = [GoudContextId, c_int32]
    _lib.goud_input_key_pressed.restype = c_bool
    
    _lib.goud_input_key_just_pressed.argtypes = [GoudContextId, c_int32]
    _lib.goud_input_key_just_pressed.restype = c_bool
    
    _lib.goud_input_key_just_released.argtypes = [GoudContextId, c_int32]
    _lib.goud_input_key_just_released.restype = c_bool
    
    _lib.goud_input_mouse_button_pressed.argtypes = [GoudContextId, c_int32]
    _lib.goud_input_mouse_button_pressed.restype = c_bool
    
    _lib.goud_input_mouse_button_just_pressed.argtypes = [GoudContextId, c_int32]
    _lib.goud_input_mouse_button_just_pressed.restype = c_bool
    
    _lib.goud_input_mouse_button_just_released.argtypes = [GoudContextId, c_int32]
    _lib.goud_input_mouse_button_just_released.restype = c_bool
    
    _lib.goud_input_get_mouse_position.argtypes = [GoudContextId, POINTER(c_float), POINTER(c_float)]
    _lib.goud_input_get_mouse_position.restype = c_bool
    
    # Collision functions
    _lib.goud_collision_aabb_overlap.argtypes = [
        c_float, c_float, c_float, c_float,
        c_float, c_float, c_float, c_float
    ]
    _lib.goud_collision_aabb_overlap.restype = c_bool
    
    _lib.goud_collision_point_in_rect.argtypes = [
        c_float, c_float,
        c_float, c_float, c_float, c_float
    ]
    _lib.goud_collision_point_in_rect.restype = c_bool
    
    # Immediate-mode rendering functions
    _lib.goud_renderer_draw_sprite.argtypes = [
        GoudContextId, c_uint64,  # context, texture
        c_float, c_float, c_float, c_float,  # x, y, width, height
        c_float,  # rotation
        c_float, c_float, c_float, c_float  # r, g, b, a
    ]
    _lib.goud_renderer_draw_sprite.restype = c_bool
    
    _lib.goud_renderer_draw_quad.argtypes = [
        GoudContextId,
        c_float, c_float, c_float, c_float,  # x, y, width, height
        c_float, c_float, c_float, c_float  # r, g, b, a
    ]
    _lib.goud_renderer_draw_quad.restype = c_bool


# Initialize FFI signatures when module loads
_setup_ffi_signatures()
