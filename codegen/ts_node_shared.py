#!/usr/bin/env python3
"""Shared data and type helpers for the TypeScript Node SDK generator."""

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))
from sdk_common import (
    HEADER_COMMENT,
    SDKS_DIR,
    TYPESCRIPT_TYPES,
    load_ffi_mapping,
    load_schema,
    to_camel,
    to_pascal,
    to_snake,
    write_generated,
)

TS = SDKS_DIR / "typescript"
GEN = TS / "src" / "generated"
NATIVE_SRC = TS / "native" / "src"
schema = load_schema()
mapping = load_ffi_mapping(schema)

IFACE_TYPES = {
    "Entity": "IEntity",
    "Transform2D": "ITransform2DData",
    "Sprite": "ISpriteData",
    "Vec2": "IVec2",
    "Vec3": "IVec3",
    "Color": "IColor",
    "Rect": "IRect",
    "RenderStats": "IRenderStats",
    "FpsStats": "IFpsStats",
    "DebuggerConfig": "IDebuggerConfig",
    "ContextConfig": "IContextConfig",
    "MemoryCategoryStats": "IMemoryCategoryStats",
    "MemorySummary": "IMemorySummary",
    "Contact": "IContact",
    "PhysicsRaycastHit2D": "IPhysicsRaycastHit2D",
    "PhysicsCollisionEvent2D": "IPhysicsCollisionEvent2D",
    "Entity[]": "IEntity[]",
    "RenderCapabilities": "IRenderCapabilities",
    "PhysicsCapabilities": "IPhysicsCapabilities",
    "AudioCapabilities": "IAudioCapabilities",
    "InputCapabilities": "IInputCapabilities",
    "NetworkCapabilities": "INetworkCapabilities",
    "NetworkStats": "INetworkStats",
    "NetworkSimulationConfig": "INetworkSimulationConfig",
    "NetworkConnectResult": "INetworkConnectResult",
    "NetworkPacket": "INetworkPacket",
    "UiStyle": "IUiStyle",
    "UiEvent": "IUiEvent",
    "UiManager": "IUiManager",
    "GoudContext": "IGoudContext",
    "PhysicsWorld2D": "IPhysicsWorld2D",
    "PhysicsWorld3D": "IPhysicsWorld3D",
}

TS_EXCLUDE_METHODS = {
    "componentRegisterType",
    "componentAdd",
    "componentRemove",
    "componentHas",
    "componentGet",
    "componentGetMut",
    "componentAddBatch",
    "componentRemoveBatch",
    "componentHasBatch",
    "isAliveBatch",
}

RUST_HEADER = f"// {HEADER_COMMENT}"

NAPI_RUST_TYPES = {
    "f32": "f64",
    "f64": "f64",
    "u32": "u32",
    "u64": "f64",
    "i32": "i32",
    "i64": "f64",
    "bool": "bool",
    "string": "String",
    "void": "()",
}

AUDIO_PLAYBACK_FNS = {
    "goud_audio_play",
    "goud_audio_play_on_channel",
    "goud_audio_play_with_settings",
}

AUDIO_CONTROLS_FNS = {
    "goud_audio_stop",
    "goud_audio_pause",
    "goud_audio_resume",
    "goud_audio_stop_all",
    "goud_audio_set_global_volume",
    "goud_audio_get_global_volume",
    "goud_audio_set_channel_volume",
    "goud_audio_get_channel_volume",
    "goud_audio_is_playing",
    "goud_audio_active_count",
    "goud_audio_cleanup_finished",
    "goud_audio_activate",
}

TS_RESERVED_PARAM_NAMES = {
    "debugger": "debuggerConfig",
}


def ts_type(t: str) -> str:
    base = t.rstrip("?")
    mapped = TYPESCRIPT_TYPES.get(base, base)
    if t.endswith("?"):
        return f"{mapped} | null"
    return mapped


def ts_iface_type(t: str) -> str:
    """Map schema type to TypeScript interface type for IGoudGame."""
    base = t.rstrip("?")
    mapped = IFACE_TYPES.get(base, TYPESCRIPT_TYPES.get(base, base))
    if t.endswith("?"):
        return f"{mapped} | null"
    return mapped


def ts_param_name(name: str) -> str:
    """Return a safe TypeScript parameter name for generated wrappers."""
    param_name = to_camel(name)
    return TS_RESERVED_PARAM_NAMES.get(param_name, param_name)


def _napi_type(schema_type: str) -> str:
    nullable = schema_type.endswith("?")
    base = schema_type.rstrip("?")
    mapped = NAPI_RUST_TYPES.get(base, base)
    if nullable:
        return f"Option<{mapped}>"
    return mapped


def _napi_rust_param_type(schema_type: str) -> str:
    if schema_type == "bytes":
        return "Buffer"
    return _napi_type(schema_type)


def _napi_rust_return_type(schema_type: str) -> str:
    if schema_type in {"f32", "f64", "u64", "i64"}:
        return "f64"
    return _napi_type(schema_type)


def _ffi_native_return_type(ffi_type: str) -> str:
    native_map = {
        "f32": "f32",
        "f64": "f64",
        "u8": "u8",
        "u16": "u16",
        "u32": "u32",
        "u64": "u64",
        "i32": "i32",
        "i64": "i64",
        "bool": "bool",
        "usize": "usize",
        "void": "()",
    }
    return native_map.get(ffi_type, ffi_type)


def _audio_ffi_module(ffi_name: str) -> str:
    if ffi_name in AUDIO_PLAYBACK_FNS:
        return "playback"
    if ffi_name in AUDIO_CONTROLS_FNS:
        return "controls"
    return "spatial"


def _ffi_arg_from_schema(param_name: str, param_type: str, ffi_param_type: str) -> str:
    if param_type == "bytes":
        if ffi_param_type == "*const u8":
            return f"{param_name}.as_ptr()"
        if ffi_param_type == "usize":
            return f"{param_name}.len()"

    rust_type = _napi_rust_param_type(param_type)
    if rust_type == ffi_param_type:
        return param_name

    if ffi_param_type == "f32":
        return f"{param_name} as f32"
    if ffi_param_type == "f64":
        return param_name
    if ffi_param_type == "u8":
        return f"{param_name} as u8"
    if ffi_param_type == "u16":
        return f"{param_name} as u16"
    if ffi_param_type == "u32":
        return f"{param_name} as u32"
    if ffi_param_type == "u64":
        return f"{param_name} as u64"
    if ffi_param_type == "i32":
        return f"{param_name} as i32"
    if ffi_param_type == "i64":
        return f"{param_name} as i64"
    return param_name
