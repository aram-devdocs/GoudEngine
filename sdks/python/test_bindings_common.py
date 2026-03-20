#!/usr/bin/env python3
"""Shared helpers for the Python SDK binding tests."""

import ctypes
import importlib.util
import sys
import types
from pathlib import Path

_GENERATED_DIR = Path(__file__).parent / "goud_engine" / "generated"
_PACKAGE_DIR = Path(__file__).parent / "goud_engine"
_NETWORKING_PATH = _PACKAGE_DIR / "networking.py"
_ROOT_INIT_PATH = _PACKAGE_DIR / "__init__.py"
_LEGACY_ERRORS_PATH = Path(__file__).parent / "goud_engine" / "errors.py"
_ERRORS_PATH = (
    _LEGACY_ERRORS_PATH
    if _LEGACY_ERRORS_PATH.exists()
    else _GENERATED_DIR / "_errors.py"
)


def _load_module(name, path):
    spec = importlib.util.spec_from_file_location(name, path)
    mod = importlib.util.module_from_spec(spec)
    sys.modules[name] = mod
    spec.loader.exec_module(mod)
    return mod


def _new_fake_generated_package(package_name, fake_lib):
    """Create an isolated fake generated package wired to a fake native lib."""
    pkg = types.ModuleType(package_name)
    pkg.__path__ = [str(_GENERATED_DIR)]
    sys.modules[package_name] = pkg

    ffi_mod = types.ModuleType(f"{package_name}._ffi")

    class GoudContextId(ctypes.Structure):
        _fields_ = [("_bits", ctypes.c_uint64)]

    class FfiColor(ctypes.Structure):
        _fields_ = [("r", ctypes.c_float), ("g", ctypes.c_float), ("b", ctypes.c_float), ("a", ctypes.c_float)]

    class FfiVec2(ctypes.Structure):
        _fields_ = [("x", ctypes.c_float), ("y", ctypes.c_float)]

    class FfiVec3(ctypes.Structure):
        _fields_ = [("x", ctypes.c_float), ("y", ctypes.c_float), ("z", ctypes.c_float)]

    class FfiRect(ctypes.Structure):
        _fields_ = [("x", ctypes.c_float), ("y", ctypes.c_float), ("width", ctypes.c_float), ("height", ctypes.c_float)]

    class FfiTransform2D(ctypes.Structure):
        _fields_ = [
            ("position_x", ctypes.c_float),
            ("position_y", ctypes.c_float),
            ("rotation", ctypes.c_float),
            ("scale_x", ctypes.c_float),
            ("scale_y", ctypes.c_float),
        ]

    class FfiSprite(ctypes.Structure):
        _fields_ = [
            ("texture_handle", ctypes.c_uint64),
            ("color_r", ctypes.c_float),
            ("color_g", ctypes.c_float),
            ("color_b", ctypes.c_float),
            ("color_a", ctypes.c_float),
            ("source_rect_x", ctypes.c_float),
            ("source_rect_y", ctypes.c_float),
            ("source_rect_width", ctypes.c_float),
            ("source_rect_height", ctypes.c_float),
            ("has_source_rect", ctypes.c_bool),
            ("flip_x", ctypes.c_bool),
            ("flip_y", ctypes.c_bool),
            ("z_layer", ctypes.c_int32),
            ("anchor_x", ctypes.c_float),
            ("anchor_y", ctypes.c_float),
            ("custom_size_x", ctypes.c_float),
            ("custom_size_y", ctypes.c_float),
            ("has_custom_size", ctypes.c_bool),
        ]

    class FfiText(ctypes.Structure):
        _fields_ = [
            ("font_handle", ctypes.c_uint64),
            ("font_size", ctypes.c_float),
            ("color_r", ctypes.c_float),
            ("color_g", ctypes.c_float),
            ("color_b", ctypes.c_float),
            ("color_a", ctypes.c_float),
            ("alignment", ctypes.c_uint32),
            ("max_width", ctypes.c_float),
            ("has_max_width", ctypes.c_bool),
            ("line_spacing", ctypes.c_float),
        ]

    class FfiUiStyle(ctypes.Structure):
        _fields_ = [
            ("has_background_color", ctypes.c_bool),
            ("background_color", FfiColor),
            ("has_foreground_color", ctypes.c_bool),
            ("foreground_color", FfiColor),
            ("has_border_color", ctypes.c_bool),
            ("border_color", FfiColor),
            ("has_border_width", ctypes.c_bool),
            ("border_width", ctypes.c_float),
            ("has_font_family", ctypes.c_bool),
            ("font_family_ptr", ctypes.c_void_p),
            ("font_family_len", ctypes.c_size_t),
            ("has_font_size", ctypes.c_bool),
            ("font_size", ctypes.c_float),
            ("has_texture_path", ctypes.c_bool),
            ("texture_path_ptr", ctypes.c_void_p),
            ("texture_path_len", ctypes.c_size_t),
            ("has_widget_spacing", ctypes.c_bool),
            ("widget_spacing", ctypes.c_float),
        ]

    class FfiUiEvent(ctypes.Structure):
        _fields_ = [
            ("event_kind", ctypes.c_uint32),
            ("node_id", ctypes.c_uint64),
            ("previous_node_id", ctypes.c_uint64),
            ("current_node_id", ctypes.c_uint64),
        ]

    class RenderCapabilities(ctypes.Structure):
        _fields_ = [
            ("max_texture_units", ctypes.c_uint32),
            ("max_texture_size", ctypes.c_uint32),
            ("supports_instancing", ctypes.c_bool),
            ("supports_compute", ctypes.c_bool),
            ("supports_msaa", ctypes.c_bool),
        ]

    class PhysicsCapabilities(ctypes.Structure):
        _fields_ = [
            ("supports_continuous_collision", ctypes.c_bool),
            ("supports_joints", ctypes.c_bool),
            ("max_bodies", ctypes.c_uint32),
        ]

    class AudioCapabilities(ctypes.Structure):
        _fields_ = [("supports_spatial", ctypes.c_bool), ("max_channels", ctypes.c_uint32)]

    class InputCapabilities(ctypes.Structure):
        _fields_ = [
            ("supports_gamepad", ctypes.c_bool),
            ("supports_touch", ctypes.c_bool),
            ("max_gamepads", ctypes.c_uint32),
        ]

    class NetworkCapabilities(ctypes.Structure):
        _fields_ = [
            ("supports_hosting", ctypes.c_bool),
            ("max_connections", ctypes.c_uint32),
            ("max_channels", ctypes.c_uint8),
            ("max_message_size", ctypes.c_uint32),
        ]

    class FfiNetworkStats(ctypes.Structure):
        _fields_ = [
            ("bytes_sent", ctypes.c_uint64),
            ("bytes_received", ctypes.c_uint64),
            ("packets_sent", ctypes.c_uint64),
            ("packets_received", ctypes.c_uint64),
            ("packets_lost", ctypes.c_uint64),
            ("rtt_ms", ctypes.c_float),
            ("send_bandwidth_bytes_per_sec", ctypes.c_float),
            ("receive_bandwidth_bytes_per_sec", ctypes.c_float),
            ("packet_loss_percent", ctypes.c_float),
            ("jitter_ms", ctypes.c_float),
        ]

    class NetworkSimulationConfig(ctypes.Structure):
        _fields_ = [
            ("one_way_latency_ms", ctypes.c_uint32),
            ("jitter_ms", ctypes.c_uint32),
            ("packet_loss_percent", ctypes.c_float),
        ]

    class GoudDebuggerConfig(ctypes.Structure):
        _fields_ = [
            ("enabled", ctypes.c_bool),
            ("publish_local_attach", ctypes.c_bool),
            ("route_label", ctypes.c_char_p),
        ]

    class GoudContextConfig(ctypes.Structure):
        _fields_ = [("debugger", GoudDebuggerConfig)]

    class GoudMemoryCategoryStats(ctypes.Structure):
        _fields_ = [
            ("current_bytes", ctypes.c_uint64),
            ("peak_bytes", ctypes.c_uint64),
        ]

    class GoudMemorySummary(ctypes.Structure):
        _fields_ = [
            ("rendering", GoudMemoryCategoryStats),
            ("assets", GoudMemoryCategoryStats),
            ("ecs", GoudMemoryCategoryStats),
            ("ui", GoudMemoryCategoryStats),
            ("audio", GoudMemoryCategoryStats),
            ("network", GoudMemoryCategoryStats),
            ("debugger", GoudMemoryCategoryStats),
            ("other", GoudMemoryCategoryStats),
            ("total_current_bytes", ctypes.c_uint64),
            ("total_peak_bytes", ctypes.c_uint64),
        ]

    class FfiMat3x3(ctypes.Structure):
        _fields_ = [("m", ctypes.c_float * 9)]

    class GoudRenderStats(ctypes.Structure):
        _fields_ = [
            ("draw_calls", ctypes.c_uint32),
            ("triangles", ctypes.c_uint32),
            ("texture_binds", ctypes.c_uint32),
            ("shader_binds", ctypes.c_uint32),
        ]

    class GoudContact(ctypes.Structure):
        _fields_ = [
            ("point_x", ctypes.c_float),
            ("point_y", ctypes.c_float),
            ("normal_x", ctypes.c_float),
            ("normal_y", ctypes.c_float),
            ("penetration", ctypes.c_float),
        ]

    class FpsStats(ctypes.Structure):
        _fields_ = [
            ("current_fps", ctypes.c_float),
            ("min_fps", ctypes.c_float),
            ("max_fps", ctypes.c_float),
            ("avg_fps", ctypes.c_float),
            ("frame_time_ms", ctypes.c_float),
        ]

    class RenderMetrics(ctypes.Structure):
        _fields_ = [
            ("draw_call_count", ctypes.c_uint32),
            ("sprites_submitted", ctypes.c_uint32),
            ("sprites_drawn", ctypes.c_uint32),
            ("sprites_culled", ctypes.c_uint32),
            ("batches_submitted", ctypes.c_uint32),
            ("avg_sprites_per_batch", ctypes.c_float),
            ("sprite_render_ms", ctypes.c_float),
            ("text_render_ms", ctypes.c_float),
            ("ui_render_ms", ctypes.c_float),
            ("total_render_ms", ctypes.c_float),
            ("text_draw_calls", ctypes.c_uint32),
            ("text_glyph_count", ctypes.c_uint32),
            ("ui_draw_calls", ctypes.c_uint32),
        ]

    ffi_mod.GoudContextId = GoudContextId
    ffi_mod.FfiColor = FfiColor
    ffi_mod.FfiVec2 = FfiVec2
    ffi_mod.FfiVec3 = FfiVec3
    ffi_mod.FfiRect = FfiRect
    ffi_mod.FfiTransform2D = FfiTransform2D
    ffi_mod.FfiSprite = FfiSprite
    ffi_mod.FfiText = FfiText
    ffi_mod.FfiUiStyle = FfiUiStyle
    ffi_mod.FfiUiEvent = FfiUiEvent
    ffi_mod.RenderCapabilities = RenderCapabilities
    ffi_mod.PhysicsCapabilities = PhysicsCapabilities
    ffi_mod.AudioCapabilities = AudioCapabilities
    ffi_mod.InputCapabilities = InputCapabilities
    ffi_mod.NetworkCapabilities = NetworkCapabilities
    ffi_mod.FfiNetworkStats = FfiNetworkStats
    ffi_mod.NetworkSimulationConfig = NetworkSimulationConfig
    ffi_mod.GoudDebuggerConfig = GoudDebuggerConfig
    ffi_mod.GoudContextConfig = GoudContextConfig
    ffi_mod.GoudMemoryCategoryStats = GoudMemoryCategoryStats
    ffi_mod.GoudMemorySummary = GoudMemorySummary
    ffi_mod.FfiMat3x3 = FfiMat3x3
    ffi_mod.GoudRenderStats = GoudRenderStats
    ffi_mod.GoudContact = GoudContact
    ffi_mod.FpsStats = FpsStats
    ffi_mod.RenderMetrics = RenderMetrics
    ffi_mod.get_lib = lambda: fake_lib
    sys.modules[f"{package_name}._ffi"] = ffi_mod

    keys_mod = types.ModuleType(f"{package_name}._keys")
    keys_mod.Key = type("Key", (), {"SPACE": 32})
    keys_mod.MouseButton = type("MouseButton", (), {"LEFT": 0})
    keys_mod.DebuggerStepKind = type("DebuggerStepKind", (), {"FRAME": 0, "TICK": 1})
    keys_mod.PhysicsBackend2D = type("PhysicsBackend2D", (), {"DEFAULT": 0})
    sys.modules[f"{package_name}._keys"] = keys_mod

    errors_mod = types.ModuleType(f"{package_name}._errors")

    class GoudError(RuntimeError):
        @staticmethod
        def from_last_error(_lib):
            return None

    errors_mod.GoudError = GoudError
    sys.modules[f"{package_name}._errors"] = errors_mod

    types_mod = _load_module(f"{package_name}._types", _GENERATED_DIR / "_types.py")
    game_mod = _load_module(f"{package_name}._game", _GENERATED_DIR / "_game.py")
    return types_mod, game_mod, ffi_mod


_types_mod = _load_module("_types", _GENERATED_DIR / "_types.py")
_keys_mod = _load_module("_keys", _GENERATED_DIR / "_keys.py")

Color = _types_mod.Color
Vec2 = _types_mod.Vec2
Rect = _types_mod.Rect
Transform2D = _types_mod.Transform2D
Sprite = _types_mod.Sprite
Entity = _types_mod.Entity
Mat3x3 = _types_mod.Mat3x3
Text = _types_mod.Text
SpriteAnimator = _types_mod.SpriteAnimator
AnimationEventData = _types_mod.AnimationEventData
RenderStats = _types_mod.RenderStats
Contact = _types_mod.Contact
PhysicsRaycastHit2D = _types_mod.PhysicsRaycastHit2D
PhysicsCollisionEvent2D = _types_mod.PhysicsCollisionEvent2D
Vec3 = _types_mod.Vec3
FpsStats = _types_mod.FpsStats
PhysicsWorld2D = _types_mod.PhysicsWorld2D
PhysicsWorld3D = _types_mod.PhysicsWorld3D
RigidBodyHandle = _types_mod.RigidBodyHandle
ColliderHandle = _types_mod.ColliderHandle
TweenHandle = _types_mod.TweenHandle
NetworkHandle = _types_mod.NetworkHandle
RenderCapabilities = _types_mod.RenderCapabilities
PhysicsCapabilities = _types_mod.PhysicsCapabilities
AudioCapabilities = _types_mod.AudioCapabilities
InputCapabilities = _types_mod.InputCapabilities
NetworkCapabilities = _types_mod.NetworkCapabilities
NetworkStats = _types_mod.NetworkStats
NetworkConnectResult = _types_mod.NetworkConnectResult
NetworkPacket = _types_mod.NetworkPacket
NetworkSimulationConfig = _types_mod.NetworkSimulationConfig
UiStyle = _types_mod.UiStyle
UiEvent = _types_mod.UiEvent
Key = _keys_mod.Key
MouseButton = _keys_mod.MouseButton
