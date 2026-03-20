"""Shared helper functions for Kotlin SDK code generation."""

from __future__ import annotations

import json
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

from sdk_common import (
    HEADER_COMMENT,
    SDKS_DIR,
    ROOT_DIR,
    load_schema,
    load_ffi_mapping,
    to_pascal,
    to_camel,
    to_snake,
    write_generated,
)

schema = load_schema()
mapping = load_ffi_mapping(schema)

KOTLIN_OUT = SDKS_DIR / "kotlin" / "src" / "main" / "kotlin" / "com" / "goudengine"
JAVA_SRC = ROOT_DIR / "goud_engine" / "tests" / "jni" / "java" / "com" / "goudengine" / "internal"
JAVA_DST = SDKS_DIR / "kotlin" / "src" / "main" / "java" / "com" / "goudengine" / "internal"

JAVA_METHOD_RENAMES = {
    "new": "create",
    "default": "defaultValue",
}

JAVA_TOOL_NATIVE_NAMES = {
    "GoudGame": "GoudGameNative",
    "EngineConfig": "EngineConfigNative",
    "GoudContext": "GoudContextNative",
    "PhysicsWorld2D": "PhysicsWorld2DNative",
    "PhysicsWorld3D": "PhysicsWorld3DNative",
    "AnimationController": "AnimationControllerNative",
    "Tween": "TweenNative",
    "Skeleton": "SkeletonNative",
    "AnimationEvents": "AnimationEventsNative",
    "AnimationLayerStack": "AnimationLayerStackNative",
    "Network": "NetworkNative",
    "Plugin": "PluginNative",
    "Audio": "AudioNative",
    "UiManager": "UiManagerNative",
}

TYPE_NATIVE_NAMES = {
    "Color": "ColorNative",
    "Transform2D": "Transform2DNative",
    "Sprite": "SpriteNative",
    "Text": "TextNative",
    "SpriteAnimator": "SpriteAnimatorNative",
}

BUILDER_NATIVE_NAMES = {
    "Transform2D": "Transform2DBuilderNative",
    "Sprite": "SpriteBuilderNative",
    "SpriteAnimator": "SpriteAnimatorBuilderNative",
}

KOTLIN_TYPES = {
    "f32": "Float", "f64": "Double",
    "u8": "Int", "u16": "Int", "u32": "Int", "u64": "Long",
    "i8": "Int", "i16": "Int", "i32": "Int", "i64": "Long",
    "usize": "Long", "ptr": "Long",
    "bool": "Boolean", "string": "String", "void": "Unit", "bytes": "ByteArray",
}

ENUM_SUBDIRS = {
    "Key": "input", "MouseButton": "input",
    "RendererType": "core", "OverlayCorner": "core", "DebuggerStepKind": "core",
    "PlaybackMode": "animation", "BodyType": "physics", "ShapeType": "physics",
    "PhysicsBackend2D": "physics", "RenderBackendKind": "core", "WindowBackendKind": "core",
    "EasingType": "animation", "NetworkProtocol": "network", "TransitionType": "animation",
    "TextAlignment": "core", "TextDirection": "core", "BlendMode": "core",
    "EventPayloadType": "animation",
    "RpcDirection": "network",
}

JAVA_CARRIERS = {
    "Color", "Vec2", "Vec3", "Rect", "Mat3x3",
    "Transform2D", "Sprite", "Text", "SpriteAnimator",
    "Contact", "RenderStats", "FpsStats",
    "PhysicsRaycastHit2D", "PhysicsCollisionEvent2D",
    "AnimationEventData", "NetworkStats", "NetworkSimulationConfig",
    "NetworkConnectResult", "NetworkPacket",
    "AudioCapabilities", "InputCapabilities",
    "RenderCapabilities", "PhysicsCapabilities", "NetworkCapabilities",
    "DebuggerConfig", "MemoryCategoryStats", "MemorySummary",
    "DebuggerCapture", "DebuggerReplayArtifact",
    "UiStyle", "UiEvent",
    "P2pMeshConfig", "RollbackConfig",
}


def kt_type(t: str) -> str:
    nullable = t.endswith("?")
    base = t.rstrip("?")
    mapped = KOTLIN_TYPES.get(base, to_pascal(base))
    return f"{mapped}?" if nullable else mapped


def java_method_name(name: str) -> str:
    return JAVA_METHOD_RENAMES.get(name, name)


def java_native_class(tool_name: str) -> str:
    return JAVA_TOOL_NATIVE_NAMES.get(tool_name, f"{tool_name}Native")


def java_type_native_class(type_name: str) -> str:
    return TYPE_NATIVE_NAMES.get(type_name, f"{type_name}Native")


def java_builder_native_class(type_name: str) -> str:
    return BUILDER_NATIVE_NAMES.get(type_name, f"{type_name}BuilderNative")


def java_carrier_import(type_name: str) -> str:
    return f"import com.goudengine.internal.{type_name}"


def java_native_import(native_class: str) -> str:
    return f"import com.goudengine.internal.{native_class}"


def kt_default_value(kt_ty: str) -> str:
    if kt_ty == "String": return '""'
    if kt_ty == "Float": return "0f"
    if kt_ty == "Double": return "0.0"
    if kt_ty in ("Int", "Long"): return "0"
    if kt_ty == "Boolean": return "false"
    return "TODO()"


def kdoc(doc, indent=""):
    """Generate KDoc comment lines from a doc string."""
    if not doc:
        return []
    lines = doc.strip().split("\n")
    if len(lines) == 1:
        return [f"{indent}/** {lines[0]} */"]
    result = [f"{indent}/**"]
    for line in lines:
        result.append(f"{indent} * {line}")
    result.append(f"{indent} */")
    return result


def write_kotlin(path: Path, content: str):
    write_generated(path, content)
