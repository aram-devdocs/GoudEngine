#!/usr/bin/env python3
"""Generate the internal JNI bridge and JVM fixtures from the schema."""

from __future__ import annotations

import re
from dataclasses import dataclass
from pathlib import Path
from typing import Iterable

from sdk_common import (
    HEADER_COMMENT,
    load_ffi_mapping,
    load_schema,
    to_camel,
    write_generated,
)

ROOT = Path(__file__).resolve().parent.parent
JNI_DIR = ROOT / "goud_engine" / "src" / "jni"
JNI_RS = JNI_DIR / "generated.rs"
JAVA_DIR = ROOT / "goud_engine" / "tests" / "jni" / "java" / "com" / "goudengine" / "internal"

JAVA_KEYWORDS = {
    "abstract", "assert", "boolean", "break", "byte", "case", "catch", "char", "class",
    "const", "continue", "default", "do", "double", "else", "enum", "extends", "final",
    "finally", "float", "for", "goto", "if", "implements", "import", "instanceof", "int",
    "interface", "long", "native", "new", "package", "private", "protected", "public",
    "return", "short", "static", "strictfp", "super", "switch", "synchronized", "this",
    "throw", "throws", "transient", "try", "void", "volatile", "while",
}
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
HANDLE_ARG_NAME = {
    "GoudContextId": "contextId",
    "EngineConfigHandle": "configHandle",
    "UiManagerHandle": "managerHandle",
}
PRIMITIVE_TYPES = {
    "bool", "f32", "f64", "i32", "i64", "u8", "u16", "u32", "u64", "usize", "ptr", "void",
}
JAVA_HEADER = f"// {HEADER_COMMENT}"


@dataclass
class GeneratedMethod:
    owner_name: str
    class_name: str
    method_name: str
    java_method_name: str
    params: list[dict]
    returns: str
    mapping: dict
    handle_type: str | None = None
    self_param: str | None = None
    section: str = "methods"
    kind: str = "tool"


def scanned_path_preference(path: str) -> tuple[int, int, str]:
    lowered = path.lower()
    return (
        1 if "wasm" in lowered else 0,
        1 if "test" in lowered else 0,
        path,
    )


def record_scanned_path(paths: dict[str, str], name: str, path: str) -> None:
    existing = paths.get(name)
    if existing is None:
        paths[name] = path
        return
    if existing == path:
        return
    if scanned_path_preference(path) < scanned_path_preference(existing):
        paths[name] = path
        return
    if scanned_path_preference(path) == scanned_path_preference(existing):
        raise RuntimeError(f"duplicate symbol resolution for {name}: {existing} vs {path}")


def is_test_source(path: Path, root: Path) -> bool:
    rel = path.relative_to(root)
    if any(part == "tests" for part in rel.parts):
        return True
    stem = path.stem
    return stem.startswith("test_") or stem.endswith("_tests")


def scan_rust_type_paths() -> dict[str, str]:
    root = ROOT / "goud_engine" / "src"
    pattern = re.compile(r"\bpub\s+(?:struct|enum|type)\s+([A-Za-z_][A-Za-z0-9_]*)\b")
    paths: dict[str, str] = {
        "c_char": "std::os::raw::c_char",
        "c_void": "std::ffi::c_void",
    }
    for path in sorted(root.rglob("*.rs")):
        if path.match("jni/generated.rs") or path.match("jni/generated.g.rs"):
            continue
        if is_test_source(path, root):
            continue
        text = path.read_text()
        rel = path.relative_to(root)
        parts = list(rel.parts)
        if parts[-1] == "mod.rs":
            parts = parts[:-1]
        else:
            parts[-1] = parts[-1][:-3]
        module = "crate"
        if parts and parts != ["lib"]:
            module += "::" + "::".join(parts)
        for name in pattern.findall(text):
            record_scanned_path(paths, name, f"{module}::{name}")
    return paths


TYPE_PATHS = scan_rust_type_paths()


def scan_ffi_function_paths() -> dict[str, str]:
    root = ROOT / "goud_engine" / "src" / "ffi"
    pattern = re.compile(r'\bpub\s+(?:unsafe\s+)?extern\s+"C"\s+fn\s+([A-Za-z_][A-Za-z0-9_]*)\b')
    paths: dict[str, str] = {}
    for path in sorted(root.rglob("*.rs")):
        if is_test_source(path, root):
            continue
        text = path.read_text()
        rel = path.relative_to(root)
        parts = list(rel.parts)
        if parts[-1] == "mod.rs":
            parts = parts[:-1]
        else:
            parts[-1] = parts[-1][:-3]
        module = "crate::ffi"
        if parts:
            module += "::" + "::".join(parts)
        for name in pattern.findall(text):
            record_scanned_path(paths, name, f"{module}::{name}")
    return paths


FFI_FUNCTION_PATHS = scan_ffi_function_paths()
TYPE_ALIASES = {
    "UiManagerHandle": "*mut crate::ui::UiManager",
    "FfiColor": "crate::ffi::FfiColor",
    "FfiRect": "crate::ffi::FfiRect",
    "FfiSprite": "crate::ffi::FfiSprite",
    "FfiSpriteBuilder": "crate::ffi::FfiSpriteBuilder",
    "FfiText": "crate::ffi::FfiText",
    "FfiTransform2D": "crate::ffi::FfiTransform2D",
    "FfiTransform2DBuilder": "crate::ffi::FfiTransform2DBuilder",
    "FfiVec2": "crate::ffi::FfiVec2",
    "FfiMat3x3": "crate::ffi::FfiMat3x3",
    "FfiAnimationClipBuilder": "crate::ffi::FfiAnimationClipBuilder",
    "FfiNetworkStats": "crate::ffi::network::FfiNetworkStats",
    "FfiSpriteAnimator": "crate::ffi::FfiSpriteAnimator",
    "FfiUiEvent": "crate::ffi::ui::FfiUiEvent",
    "FfiUiStyle": "crate::ffi::ui::FfiUiStyle",
    "GoudContact": "crate::core::types::GoudContact",
    "GoudDebuggerConfig": "crate::ffi::context::GoudDebuggerConfig",
    "GoudMemoryCategoryStats": "crate::ffi::debug::GoudMemoryCategoryStats",
    "GoudMemorySummary": "crate::ffi::debug::GoudMemorySummary",
    "GoudRenderStats": "crate::ffi::renderer::GoudRenderStats",
    "FfiAudioCapabilities": "crate::core::providers::types::AudioCapabilities",
    "GoudFpsStats": "crate::sdk::debug_overlay::FpsStats",
    "FfiInputCapabilities": "crate::core::providers::input_types::InputCapabilities",
    "FfiNetworkCapabilities": "crate::core::providers::network_types::NetworkCapabilities",
    "FfiNetworkSimulationConfig": "crate::core::providers::network_types::NetworkSimulationConfig",
    "FfiPhysicsCapabilities": "crate::core::providers::types::PhysicsCapabilities",
    "FfiRenderCapabilities": "crate::core::providers::types::RenderCapabilities",
    "FfiVec3": "crate::core::math::Vec3",
}


def scan_rust_function_paths() -> dict[str, str]:
    root = ROOT / "goud_engine" / "src"
    pattern = re.compile(r"\bpub\s+(?:unsafe\s+)?extern\s+\"C\"\s+fn\s+([A-Za-z_][A-Za-z0-9_]*)\b")
    paths: dict[str, str] = {}
    for path in sorted(root.rglob("*.rs")):
        if path.match("jni/generated.rs") or path.match("jni/generated.g.rs"):
            continue
        if is_test_source(path, root):
            continue
        text = path.read_text()
        rel = path.relative_to(root)
        parts = list(rel.parts)
        if parts[-1] == "mod.rs":
            parts = parts[:-1]
        else:
            parts[-1] = parts[-1][:-3]
        module = "crate"
        if parts and parts != ["lib"]:
            module += "::" + "::".join(parts)
        for name in pattern.findall(text):
            record_scanned_path(paths, name, f"{module}::{name}")
    return paths


FUNCTION_PATHS = scan_rust_function_paths()


def java_identifier(name: str) -> str:
    renamed = JAVA_METHOD_RENAMES.get(name, name)
    if renamed in JAVA_KEYWORDS:
        return f"{renamed}Value"
    return renamed


def base_type(type_name: str) -> str:
    return type_name[:-1] if type_name.endswith("?") else type_name


def is_nullable(type_name: str) -> bool:
    return type_name.endswith("?")


def is_builder_name(type_name: str) -> bool:
    return type_name.endswith("Builder") or type_name == "AnimationClipBuilder"


def is_array_type(type_name: str) -> bool:
    return type_name.endswith("[]")


def array_element_type(type_name: str) -> str:
    return type_name[:-2]


def java_type(type_name: str, *, object_fallback: bool = False) -> str:
    raw = base_type(type_name)
    if raw == "void":
        return "void"
    if raw == "bool":
        return "boolean"
    if raw == "f32":
        return "float"
    if raw == "f64":
        return "double"
    if raw in {"i32", "u8", "u16", "u32"}:
        return "int"
    if raw in {"i64", "u64", "usize", "ptr", "GoudResult", "Entity", "GoudGame", "GoudContextId", "EngineConfigHandle", "UiManagerHandle"}:
        return "long"
    if raw == "string":
        return "String"
    if raw in {"bytes", "u8[]"}:
        return "byte[]"
    if raw == "Entity[]":
        return "long[]"
    if raw == "object":
        return "Object"
    if raw == "f32[9]":
        return "float[]"
    if raw in SCHEMA.get("enums", {}):
        return "int"
    if is_builder_name(raw):
        return "long"
    if raw in SCHEMA.get("types", {}):
        return raw
    return "Object" if object_fallback else raw


def field_sig(type_name: str) -> str:
    raw = base_type(type_name)
    if raw == "bool":
        return "Z"
    if raw == "f32":
        return "F"
    if raw == "f64":
        return "D"
    if raw in {"i32", "u8", "u16", "u32"} or raw in SCHEMA.get("enums", {}):
        return "I"
    if raw in {"i64", "u64", "usize", "ptr", "GoudResult", "Entity", "GoudGame"} or is_builder_name(raw):
        return "J"
    if raw == "string":
        return "Ljava/lang/String;"
    if raw in {"bytes", "u8[]"}:
        return "[B"
    if raw == "Entity[]":
        return "[J"
    if raw == "f32[9]":
        return "[F"
    return f"Lcom/goudengine/internal/{raw};"


def rust_type(raw_type: str) -> str:
    raw = raw_type.strip()
    if raw in {"()", "void"}:
        return "()"
    alias = TYPE_ALIASES.get(raw)
    if alias is not None:
        return rust_type(alias)
    if raw.startswith("*mut ") or raw.startswith("*const "):
        qualifier, inner = raw.split(" ", 1)
        return f"{qualifier} {rust_type(inner)}"
    if raw.startswith("Option<") and raw.endswith(">"):
        inner = raw[len("Option<") : -1]
        return f"Option<{rust_type(inner)}>"
    return TYPE_PATHS.get(raw, raw)


def render_checked_enum_conversion(
    target_name: str,
    enum_name: str,
    raw_expr: str,
    source_name: str,
    *,
    env_expr: str,
) -> list[str]:
    enum_info = SCHEMA["enums"][enum_name]
    underlying = rust_type(enum_info.get("underlying", "i32"))
    allowed_values = sorted(set(enum_info["values"].values()))
    allowed_match = " | ".join(str(value) for value in allowed_values)
    discriminant_name = f"{target_name}_discriminant"
    return [
        f"    let {discriminant_name} = {raw_expr} as {underlying};",
        f"    let {target_name} = match {discriminant_name} {{",
        f"        {allowed_match} => unsafe {{ // SAFETY: the discriminant was validated against the schema enum values above.",
        f"            std::mem::transmute::<{underlying}, _>({discriminant_name})",
        "        },",
        "        invalid => {",
        f'            crate::jni::helpers::throw_illegal_argument({env_expr}, format!("{source_name} has invalid {enum_name} discriminant: {{}}", invalid))?;',
        "            return Err(());",
        "        }",
        "    };",
    ]


def rust_struct_type(schema_type: str) -> str:
    ffi_info = MAPPING["ffi_types"][schema_type]
    ffi_name = ffi_info["ffi_name"]
    if not ffi_name:
        return schema_type
    resolved = rust_type(ffi_name)
    if resolved != ffi_name:
        return resolved
    if schema_type in TYPE_PATHS:
        return rust_type(schema_type)
    return resolved


def ffi_symbol_path(name: str) -> str:
    return FFI_FUNCTION_PATHS.get(name, name)


def rust_value_type(type_name: str) -> str:
    raw = base_type(type_name)
    if raw == "bool":
        return "bool"
    if raw == "f32":
        return "f32"
    if raw == "f64":
        return "f64"
    if raw == "i32":
        return "i32"
    if raw == "i64":
        return "i64"
    if raw == "u8":
        return "u8"
    if raw == "u16":
        return "u16"
    if raw == "u32" or raw in SCHEMA.get("enums", {}):
        return "u32"
    if raw in {"u64", "usize", "Entity"}:
        return "u64"
    if raw == "ptr":
        return "usize"
    if raw in TYPE_PATHS:
        return TYPE_PATHS[raw]
    return rust_struct_type(raw)


def schema_type_closure(initial: Iterable[str]) -> list[str]:
    seen: set[str] = set()
    pending = [base_type(name) for name in initial]
    while pending:
        current = pending.pop()
        if current in seen:
            continue
        if current in PRIMITIVE_TYPES or current in SCHEMA.get("enums", {}) or current in {"GoudGame", "Entity"}:
            continue
        if is_builder_name(current) or is_array_type(current):
            continue
        if current not in SCHEMA.get("types", {}):
            continue
        seen.add(current)
        for field in SCHEMA["types"][current].get("fields", []):
            pending.append(base_type(field["type"]))
    return sorted(seen)


def supports_jni_bridge(method_def: dict[str, object]) -> bool:
    if base_type(method_def.get("returns", "void")) == "ptr":
        return False
    for param in method_def.get("params", []):
        if base_type(param["type"]) == "ptr":
            return False
    return True


def used_carrier_types() -> list[str]:
    used: set[str] = set()
    for tool_name, tool in SCHEMA["tools"].items():
        ffi_tool = SCHEMA["ffi_tools"].get(tool_name, {})
        if tool.get("constructor") and ffi_tool.get("constructor"):
            for param in tool["constructor"].get("params", []):
                used.add(param["type"])
        for method in tool.get("methods", []):
            if method["name"] not in ffi_tool.get("methods", {}):
                continue
            for param in method.get("params", []):
                used.add(param["type"])
            used.add(method.get("returns", "void"))
    for type_name, meta in SCHEMA.get("ffi_type_methods", {}).items():
        used.add(type_name)
        type_def = SCHEMA["types"][type_name]
        for section_name, section in meta.items():
            if section_name == "builder":
                entries = type_def.get("builder", {}).get("methods", [])
            elif section_name == "factories":
                entries = type_def.get("factories", [])
            else:
                entries = type_def.get("methods", [])
            method_names = set(section.keys())
            for entry in entries:
                if entry["name"] not in method_names:
                    continue
                for param in entry.get("params", entry.get("args", [])):
                    used.add(param["type"])
                used.add(entry.get("returns", type_name))
    return schema_type_closure(used)


def rust_handle_type(handle_type: str) -> str:
    return rust_type(handle_type)


def handle_value_expr(handle_type: str, arg_name: str) -> str:
    if handle_type == "GoudContextId":
        return f"goud_context_id_from_jlong({arg_name})"
    if handle_type == "UiManagerHandle":
        return f"{arg_name} as usize as *mut crate::ui::UiManager"
    if handle_type == "EngineConfigHandle":
        return f"{arg_name} as usize as *mut std::ffi::c_void"
    return f"{arg_name} as {rust_handle_type(handle_type)}"


def handle_arg_name(handle_type: str | None) -> str | None:
    if handle_type is None:
        return None
    return HANDLE_ARG_NAME.get(handle_type, "handle")


def component_type_hash(type_name: str) -> str:
    value = 0xCBF29CE484222325
    for byte in type_name.encode("utf-8"):
        value ^= byte
        value = (value * 0x100000001B3) & 0xFFFFFFFFFFFFFFFF
    return f"0x{value:016x}"


def method_ffi_def(mapping: dict) -> dict:
    ffi_name = mapping["ffi"]
    for _module_name, methods in MAPPING["ffi_functions"].items():
        if isinstance(methods, dict) and ffi_name in methods:
            return methods[ffi_name]
    raise KeyError(ffi_name)


def ffi_function_path(ffi_name: str) -> str:
    public_overrides = {
        "goud_component_add": "crate::ffi::component::goud_component_add",
        "goud_component_add_batch": "crate::ffi::component::goud_component_add_batch",
        "goud_component_get": "crate::ffi::component::goud_component_get",
        "goud_component_get_mut": "crate::ffi::component::goud_component_get_mut",
        "goud_component_has": "crate::ffi::component::goud_component_has",
        "goud_component_has_batch": "crate::ffi::component::goud_component_has_batch",
        "goud_component_register_type": "crate::ffi::component::goud_component_register_type",
        "goud_component_remove": "crate::ffi::component::goud_component_remove",
        "goud_component_remove_batch": "crate::ffi::component::goud_component_remove_batch",
        "goud_engine_create": "crate::ffi::engine_config::goud_engine_create",
    }
    public_prefixes = [
        ("goud_context_", "crate::ffi::context"),
        ("goud_network_", "crate::ffi::network"),
        ("goud_debugger_", "crate::ffi::debug"),
        ("goud_debug_", "crate::ffi::debug"),
        ("goud_diagnostic_", "crate::ffi::debug"),
        ("goud_window_", "crate::ffi::window"),
        ("goud_renderer3d_", "crate::ffi::renderer3d"),
        ("goud_renderer_", "crate::ffi::renderer"),
        ("goud_texture_", "crate::ffi::renderer"),
        ("goud_font_", "crate::ffi::renderer"),
        ("goud_input_", "crate::ffi::input"),
        ("goud_engine_config_", "crate::ffi::engine_config"),
    ]
    for prefix, module in public_prefixes:
        if ffi_name.startswith(prefix):
            return f"{module}::{ffi_name}"
    return public_overrides.get(
        ffi_name,
        FFI_FUNCTION_PATHS.get(ffi_name, FUNCTION_PATHS.get(ffi_name, ffi_name)),
    )


def method_required_feature(method: GeneratedMethod) -> str | None:
    ffi_name = method.mapping.get("ffi", "")
    if method.owner_name == "PhysicsWorld3D" or ffi_name.startswith("goud_physics3d_"):
        return "rapier3d"
    if method.owner_name == "PhysicsWorld2D" or ffi_name.startswith("goud_physics_"):
        return "rapier2d"
    return None


def ffi_return_type(ffi_meta: dict) -> str | None:
    return ffi_meta.get("return_type") or ffi_meta.get("returns") or ffi_meta.get("return")


def line_join(lines: list[str], indent: int = 0) -> str:
    pad = " " * indent
    return "\n".join(pad + line if line else "" for line in lines)


def rust_return_type(type_name: str) -> str:
    raw = base_type(type_name)
    if raw == "void":
        return "()"
    if raw == "bool":
        return "jni::sys::jboolean"
    if raw == "f32":
        return "jni::sys::jfloat"
    if raw == "f64":
        return "jni::sys::jdouble"
    if raw in {"i32", "u8", "u16", "u32", "GoudResult"} or raw in SCHEMA.get("enums", {}):
        return "jni::sys::jint"
    if raw in {"i64", "u64", "usize", "ptr", "Entity", "GoudGame", "GoudContextId", "EngineConfigHandle", "UiManagerHandle"} or is_builder_name(raw):
        return "jni::sys::jlong"
    if raw == "string":
        return "jni::sys::jstring"
    if raw in {"bytes", "u8[]"}:
        return "jni::sys::jbyteArray"
    if raw == "Entity[]":
        return "jni::sys::jlongArray"
    return "jni::sys::jobject"


def rust_arg_type(type_name: str) -> str:
    raw = base_type(type_name)
    if raw == "bool":
        return "jni::sys::jboolean"
    if raw == "f32":
        return "jni::sys::jfloat"
    if raw == "f64":
        return "jni::sys::jdouble"
    if raw in {"i32", "u8", "u16", "u32", "GoudResult"} or raw in SCHEMA.get("enums", {}):
        return "jni::sys::jint"
    if raw in {"i64", "u64", "usize", "ptr", "Entity", "GoudGame", "GoudContextId", "EngineConfigHandle", "UiManagerHandle"} or is_builder_name(raw):
        return "jni::sys::jlong"
    if raw == "string":
        return "jni::objects::JString<'local>"
    if raw in {"bytes", "u8[]"}:
        return "jni::objects::JByteArray<'local>"
    if raw == "Entity[]":
        return "jni::objects::JLongArray<'local>"
    return "jni::objects::JObject<'local>"


def rust_default_return(type_name: str) -> str:
    raw = base_type(type_name)
    if raw == "void":
        return ""
    if raw == "bool":
        return "jni::sys::JNI_FALSE"
    if raw in {"f32", "f64"}:
        return "0.0"
    if raw in {"i32", "u8", "u16", "u32", "i64", "u64", "usize", "ptr", "Entity", "GoudGame", "GoudResult", "GoudContextId", "EngineConfigHandle", "UiManagerHandle"} or raw in SCHEMA.get("enums", {}) or is_builder_name(raw):
        return "0"
    if raw == "string":
        return "crate::jni::helpers::null_string()"
    if raw in {"bytes", "u8[]"}:
        return "crate::jni::helpers::null_byte_array()"
    if raw == "Entity[]":
        return "std::ptr::null_mut()"
    return "crate::jni::helpers::null_object()"


def java_field_access_method(schema_type: str) -> str:
    raw = base_type(schema_type)
    if raw == "bool":
        return "z"
    if raw == "f32":
        return "f"
    if raw == "f64":
        return "d"
    if raw in {"i32", "u8", "u16", "u32"} or raw in SCHEMA.get("enums", {}):
        return "i"
    if raw in {"i64", "u64", "usize", "ptr", "Entity"} or is_builder_name(raw):
        return "j"
    return "l"


def rust_zero_value(schema_type: str) -> str:
    raw = base_type(schema_type)
    if raw == "bool":
        return "false"
    if raw in {"f32", "f64"}:
        return "0.0"
    if raw in {"i32", "i64", "u8", "u16", "u32", "u64", "usize"}:
        return "0"
    if raw in SCHEMA.get("enums", {}):
        return "0"
    return f"unsafe {{ // SAFETY: zeroed out-parameter storage is only used before the FFI call writes it.\n        std::mem::zeroed()\n    }}"


def java_carrier_source(type_name: str) -> str:
    type_def = SCHEMA["types"][type_name]
    lines = [
        JAVA_HEADER,
        "package com.goudengine.internal;",
        "",
    ]
    if type_name == "UiStyle":
        lines += [
            "/**",
            " * Internal carrier for UI style data passed through the JNI bridge.",
            " *",
            " * <p>The {@code fontFamilyPtr} and {@code texturePathPtr} fields mirror raw native pointer",
            " * values from {@code FfiUiStyle}. They are only valid during the JNI call frame that",
            " * populated them and must not be cached or reused across calls.",
            " */",
        ]
    lines.append(f"public final class {type_name} {{")
    for field in type_def.get("fields", []):
        java_name = to_camel(field["name"])
        if type_name == "UiStyle" and java_name in {"fontFamilyPtr", "texturePathPtr"}:
            lines += [
                "    /**",
                "     * Raw native pointer borrowed for a single JNI call frame. Do not retain or reuse it.",
                "     */",
            ]
        lines.append(f"    public {java_type(field['type'])} {java_name};")
    lines += [
        "",
        f"    public {type_name}() {{}}",
    ]
    if type_def.get("fields"):
        ctor_params = ", ".join(
            f"{java_type(field['type'])} {to_camel(field['name'])}" for field in type_def["fields"]
        )
        lines += [
            "",
            f"    public {type_name}({ctor_params}) {{",
        ]
        for field in type_def["fields"]:
            name = to_camel(field["name"])
            lines.append(f"        this.{name} = {name};")
        lines.append("    }")
    lines += ["}", ""]
    return "\n".join(lines)


def reorder_jni_params(params: list[dict], mapping: dict) -> list[dict]:
    """Reorder method params so param_order items come first, expand_params last.

    When a mapping specifies both ``param_order`` and ``expand_params``, the
    JNI/Java signature must list the primitive params (from ``param_order``) before
    the carrier-object params (from ``expand_params``) so that the positional
    arguments match the FFI function signature where the expanded fields follow
    the ordered primitives.
    """
    param_order = mapping.get("param_order")
    expand_params = mapping.get("expand_params")
    if not param_order or not expand_params:
        return params

    expand_names = set(expand_params.keys())
    param_by_name = {p["name"]: p for p in params}

    ordered: list[dict] = []
    used: set[str] = set()

    # First: params listed in param_order, in that order
    for name in param_order:
        if name in param_by_name:
            ordered.append(param_by_name[name])
            used.add(name)

    # Second: expand_params (carrier objects like Color)
    for p in params:
        if p["name"] in expand_names and p["name"] not in used:
            ordered.append(p)
            used.add(p["name"])

    # Third: any remaining params not in either category (preserve schema order)
    for p in params:
        if p["name"] not in used:
            ordered.append(p)

    return ordered


def java_native_source(class_name: str, methods: list[GeneratedMethod]) -> str:
    lines = [
        JAVA_HEADER,
        "package com.goudengine.internal;",
        "",
    ]
    lines += [
        f"public final class {class_name} {{",
        f"    private {class_name}() {{}}",
        "",
    ]
    for method in methods:
        # Skip methods with callback parameters — JNI cannot express them.
        if any(p["type"].startswith("callback") for p in method.params):
            continue
        java_params = []
        if method.handle_type is not None and not method.mapping.get("no_context"):
            java_params.append(f"long {handle_arg_name(method.handle_type)}")
        if method.self_param is not None:
            if is_builder_name(method.self_param):
                java_params.append("long selfHandle")
            else:
                java_params.append(f"{base_type(method.self_param)} self")
        for param in reorder_jni_params(method.params, method.mapping):
            java_params.append(f"{java_type(param['type'])} {to_camel(param['name'])}")
        lines.append(
            f"    public static native {java_type(method.returns, object_fallback=True)} {method.java_method_name}({', '.join(java_params)});"
        )
    lines += ["}", ""]
    return "\n".join(lines)


def smoke_java_source() -> str:
    return "\n".join(
        [
            JAVA_HEADER,
            "package com.goudengine.internal;",
            "",
            "public final class JniSmokeMain {",
            "    private JniSmokeMain() {}",
            "",
            "    public static void main(String[] args) {",
            "        if (args.length != 1) throw new IllegalArgumentException(\"Expected library path\");",
            "        System.load(args[0]);",
            "",
            "        long ctx = GoudContextNative.create();",
            "        if (ctx == 0L || !GoudContextNative.isValid(ctx)) throw new AssertionError(\"Context should be valid\");",
            "",
            "        Color color = ColorNative.rgba(0.25f, 0.5f, 0.75f, 1.0f);",
            "        if (Math.abs(color.g - 0.5f) > 0.0001f) throw new AssertionError(\"Color bridge failed\");",
            "",
            "        Transform2D transform = Transform2DNative.create(10.0f, 20.0f, 0.0f, 1.0f, 1.0f);",
            "        Transform2DNative.translate(transform, 5.0f, -3.0f);",
            "        Vec2 position = Transform2DNative.getPosition(transform);",
            "        if (Math.abs(position.x - 15.0f) > 0.0001f || Math.abs(position.y - 17.0f) > 0.0001f) {",
            "            throw new AssertionError(\"Transform mutation bridge failed\");",
            "        }",
            "",
            "        PluginNative.register(ctx, \"jni-smoke\");",
            "        if (!PluginNative.isRegistered(ctx, \"jni-smoke\")) throw new AssertionError(\"Plugin register failed\");",
            "        String list = PluginNative.list(ctx);",
            "        if (!list.contains(\"jni-smoke\")) throw new AssertionError(\"Plugin list missing registration\");",
            "        PluginNative.unregister(ctx, \"jni-smoke\");",
            "",
            "        try {",
            "            PluginNative.register(0L, \"bad\");",
            "            throw new AssertionError(\"Expected IllegalStateException for invalid context\");",
            "        } catch (IllegalStateException expected) {",
            "        }",
            "",
            "        try {",
            "            PluginNative.register(ctx, null);",
            "            throw new AssertionError(\"Expected NullPointerException for null plugin id\");",
            "        } catch (NullPointerException expected) {",
            "        }",
            "",
            "        try {",
            "            NetworkNative.send(ctx, -1L, 1L, new byte[] {1, 2, 3}, 0);",
            "            throw new AssertionError(\"Expected IllegalStateException for invalid network handle\");",
            "        } catch (IllegalStateException expected) {",
            "        }",
            "",
            "        GoudContextNative.destroy(ctx);",
            "    }",
            "}",
            "",
        ]
    )


def collect_generated_methods() -> list[GeneratedMethod]:
    methods: list[GeneratedMethod] = []
    for tool_name, tool in SCHEMA["tools"].items():
        ffi_tool = SCHEMA["ffi_tools"].get(tool_name, {})
        class_name = JAVA_TOOL_NATIVE_NAMES[tool_name]
        if tool.get("constructor") and ffi_tool.get("constructor"):
            methods.append(
                GeneratedMethod(
                    owner_name=tool_name,
                    class_name=class_name,
                    method_name="create",
                    java_method_name="create",
                    params=tool["constructor"].get("params", []),
                    returns=ffi_tool.get("handle", "long"),
                    mapping=ffi_tool["constructor"],
                    kind="tool",
                )
            )
        for method in tool.get("methods", []):
            mapping = ffi_tool.get("methods", {}).get(method["name"])
            if mapping is None:
                continue
            if not supports_jni_bridge(method):
                continue
            methods.append(
                GeneratedMethod(
                    owner_name=tool_name,
                    class_name=class_name,
                    method_name=method["name"],
                    java_method_name=java_identifier(method["name"]),
                    params=method.get("params", []),
                    returns=method.get("returns", "void"),
                    mapping=mapping,
                    handle_type=ffi_tool.get("handle"),
                    kind="tool",
                )
            )
    for type_name, meta in SCHEMA.get("ffi_type_methods", {}).items():
        type_def = SCHEMA["types"][type_name]
        for factory in type_def.get("factories", []):
            mapping = meta.get("factories", {}).get(factory["name"])
            if mapping is None:
                continue
            methods.append(
                GeneratedMethod(
                    owner_name=type_name,
                    class_name=TYPE_NATIVE_NAMES[type_name],
                    method_name=factory["name"],
                    java_method_name=java_identifier(factory["name"]),
                    params=factory.get("args", []),
                    returns=factory.get("returns", type_name) or type_name,
                    mapping=mapping,
                    kind="type",
                    section="factories",
                )
            )
        for method in type_def.get("methods", []):
            mapping = meta.get("methods", {}).get(method["name"])
            if mapping is None:
                continue
            self_param = type_name if "self_param" in mapping else None
            methods.append(
                GeneratedMethod(
                    owner_name=type_name,
                    class_name=TYPE_NATIVE_NAMES[type_name],
                    method_name=method["name"],
                    java_method_name=java_identifier(method["name"]),
                    params=method.get("params", []),
                    returns=method.get("returns", "void"),
                    mapping=mapping,
                    self_param=self_param,
                    kind="type",
                )
            )
        builder_def = type_def.get("builder", {})
        if builder_def:
            for builder_method in builder_def.get("methods", []):
                mapping = meta.get("builder", {}).get(builder_method["name"])
                if mapping is None:
                    continue
                methods.append(
                    GeneratedMethod(
                        owner_name=type_name,
                        class_name=BUILDER_NATIVE_NAMES[type_name],
                        method_name=builder_method["name"],
                        java_method_name=java_identifier(builder_method["name"]),
                        params=builder_method.get("params", []),
                        returns=builder_method.get("returns", "void"),
                        mapping=mapping,
                        self_param=(
                            "AnimationClipBuilder" if type_name == "SpriteAnimator" else f"{type_name}Builder"
                        ) if "self_param" in mapping else None,
                        kind="builder",
                        section="builder",
                    )
                )
    return methods


def build_struct_writer(type_name: str) -> list[str]:
    if type_name in {"DebuggerConfig", "ContextConfig"}:
        return []
    schema_type = SCHEMA["types"][type_name]
    ffi_info = MAPPING["ffi_types"].get(type_name, {})
    if "fields" not in ffi_info:
        return []
    raw_type = rust_struct_type(type_name)
    lines = [
        f"pub(crate) fn set_{type_name}_fields<'local>(env: &mut jni::JNIEnv<'local>, obj: &jni::objects::JObject<'local>, value: {raw_type}) -> crate::jni::helpers::JniCallResult<()> {{",
        "    crate::jni::helpers::ensure_no_pending_exception(env)?;",
    ]
    for schema_field, ffi_field in zip(schema_type.get("fields", []), ffi_info["fields"]):
        field_name = to_camel(schema_field["name"])
        sig = field_sig(schema_field["type"])
        field_type = base_type(schema_field["type"])
        if field_type == "string":
            lines.append(f'    crate::jni::helpers::set_string_field(env, obj, "{field_name}", &value.{ffi_field})?;')
        elif field_type in {"bytes", "u8[]"}:
            lines.append(f'    crate::jni::helpers::set_byte_array_field(env, obj, "{field_name}", &value.{ffi_field})?;')
        elif field_type == "f32[9]":
            lines.append(f'    crate::jni::helpers::set_float_array_field(env, obj, "{field_name}", &value.{ffi_field})?;')
        elif field_type in SCHEMA.get("types", {}):
            nested_value = f"value.{ffi_field}"
            if type_name == "UiStyle" and field_type == "Color":
                nested_value = f"value.{ffi_field}.into()"
            lines += [
                f"    let field_{field_name}_obj = new_{field_type}(env, {nested_value})?;",
                f'    crate::jni::helpers::set_object_field(env, obj, "{field_name}", "{sig}", &field_{field_name}_obj)?;',
            ]
        elif field_type in SCHEMA.get("enums", {}):
            lines.append(f'    crate::jni::helpers::set_int_field(env, obj, "{field_name}", value.{ffi_field} as i32)?;')
        elif field_type == "bool":
            lines.append(f'    crate::jni::helpers::set_boolean_field(env, obj, "{field_name}", value.{ffi_field})?;')
        elif field_type in {"u64", "i64", "usize", "ptr", "Entity"}:
            cast_expr = f"value.{ffi_field} as i64"
            if field_type == "ptr":
                cast_expr = f"value.{ffi_field} as usize as i64"
            lines.append(f'    crate::jni::helpers::set_long_field(env, obj, "{field_name}", {cast_expr})?;')
        elif field_type in {"u32", "u16", "u8", "i32"}:
            lines.append(f'    crate::jni::helpers::set_int_field(env, obj, "{field_name}", value.{ffi_field} as i32)?;')
        elif field_type == "f32":
            lines.append(f'    crate::jni::helpers::set_float_field(env, obj, "{field_name}", value.{ffi_field})?;')
        elif field_type == "f64":
            lines.append(f'    crate::jni::helpers::set_double_field(env, obj, "{field_name}", value.{ffi_field})?;')
    lines += ["    Ok(())", "}", ""]
    lines += [
        f"pub(crate) fn new_{type_name}<'local>(env: &mut jni::JNIEnv<'local>, value: {raw_type}) -> crate::jni::helpers::JniCallResult<jni::objects::JObject<'local>> {{",
        f'    let obj = crate::jni::helpers::new_object(env, "com/goudengine/internal/{type_name}")?;',
        f"    set_{type_name}_fields(env, &obj, value)?;",
        "    Ok(obj)",
        "}",
        "",
    ]
    return lines


def build_struct_reader(type_name: str) -> list[str]:
    if type_name in {"DebuggerConfig", "ContextConfig"}:
        return []
    schema_type = SCHEMA["types"][type_name]
    ffi_info = MAPPING["ffi_types"].get(type_name, {})
    if "fields" not in ffi_info:
        return []
    raw_type = rust_struct_type(type_name)
    lines = [
        f"pub(crate) fn read_{type_name}<'local>(env: &mut jni::JNIEnv<'local>, obj: &jni::objects::JObject<'local>, param_name: &str) -> crate::jni::helpers::JniCallResult<{raw_type}> {{",
        "    if obj.is_null() {",
        '        crate::jni::helpers::throw_null_pointer(env, format!("{param_name} is null"))?;',
        "        return Err(());",
        "    }",
    ]
    for schema_field, ffi_field in zip(schema_type.get("fields", []), ffi_info["fields"]):
        field_name = to_camel(schema_field["name"])
        sig = field_sig(schema_field["type"])
        field_type = base_type(schema_field["type"])
        if field_type == "f32[9]":
            lines.append(f'    let field_{field_name} = crate::jni::helpers::get_float_array_field::<9>(env, obj, "{field_name}")?;')
            continue
        if field_type in SCHEMA.get("types", {}):
            lines += [
                f'    let field_{field_name}_obj = crate::jni::helpers::get_object_field(env, obj, "{field_name}", "{sig}")?;',
                f'    let field_{field_name} = read_{field_type}(env, &field_{field_name}_obj, "{type_name}.{field_name}")?;',
            ]
            continue
        if field_type == "string":
            lines.append(f'    let field_{field_name} = crate::jni::helpers::get_string_field(env, obj, "{field_name}")?;')
            continue
        if field_type in {"bytes", "u8[]"}:
            lines += [
                f'    let field_{field_name}_obj = crate::jni::helpers::get_object_field(env, obj, "{field_name}", "{sig}")?;',
                f'    let field_{field_name} = crate::jni::helpers::require_bytes(env, jni::objects::JByteArray::from(field_{field_name}_obj), "{type_name}.{field_name}")?;',
            ]
            continue
        if field_type == "bool":
            value_call = f'crate::jni::helpers::get_boolean_field(env, obj, "{field_name}")?'
        elif field_type in {"u64", "i64", "usize", "ptr", "Entity"}:
            value_call = f'crate::jni::helpers::get_long_field(env, obj, "{field_name}")?'
        elif field_type in {"u32", "u16", "u8", "i32"} or field_type in SCHEMA.get("enums", {}):
            value_call = f'crate::jni::helpers::get_int_field(env, obj, "{field_name}")?'
        elif field_type == "f32":
            value_call = f'crate::jni::helpers::get_float_field(env, obj, "{field_name}")?'
        elif field_type == "f64":
            value_call = f'crate::jni::helpers::get_double_field(env, obj, "{field_name}")?'
        else:
            raise RuntimeError(f"unhandled JNI reader field type {field_type} for {type_name}.{field_name}")
        if field_type == "bool":
            lines.append(f"    let field_{field_name} = {value_call};")
        elif field_type in {"u64", "i64", "usize", "Entity"}:
            cast_ty = "usize" if field_type == "usize" else "u64" if field_type == "Entity" else field_type
            lines.append(f"    let field_{field_name} = {value_call} as {cast_ty};")
        elif field_type == "ptr":
            lines.append(f"    let field_{field_name} = {value_call} as usize as _;")
        elif field_type in SCHEMA.get("enums", {}):
            lines.extend(
                render_checked_enum_conversion(
                    f"field_{field_name}",
                    field_type,
                    value_call,
                    f"{type_name}.{field_name}",
                    env_expr="env",
                )
            )
        else:
            lines.append(f"    let field_{field_name} = {value_call} as _;")
    lines += [f"    Ok({raw_type} {{"] 
    for schema_field, ffi_field in zip(schema_type.get("fields", []), ffi_info["fields"]):
        field_name = to_camel(schema_field["name"])
        field_expr = f"field_{field_name}"
        if type_name == "UiStyle" and base_type(schema_field["type"]) == "Color":
            field_expr += ".into()"
        lines.append(f"        {ffi_field}: {field_expr},")
    lines += ["    })", "}", ""]
    if type_name not in {"DebuggerConfig", "ContextConfig"}:
        lines += [
            f"pub(crate) fn write_back_{type_name}<'local>(env: &mut jni::JNIEnv<'local>, obj: &jni::objects::JObject<'local>, value: {raw_type}) -> crate::jni::helpers::JniCallResult<()> {{",
            f"    set_{type_name}_fields(env, obj, value)",
            "}",
            "",
        ]
    return lines


def build_local_ffi_structs() -> list[str]:
    lines: list[str] = []
    for type_name in USED_TYPES:
        ffi_info = MAPPING["ffi_types"].get(type_name)
        if ffi_info is None or "fields" not in ffi_info:
            continue
        ffi_name = ffi_info["ffi_name"]
        if not ffi_name or ffi_name in TYPE_PATHS or type_name in TYPE_PATHS:
            continue
        lines += [
            "#[repr(C)]",
            "#[derive(Clone, Copy, Default)]",
            f"struct {ffi_name} {{",
        ]
        for schema_field, ffi_field in zip(SCHEMA["types"][type_name]["fields"], ffi_info["fields"]):
            lines.append(f"    {ffi_field}: {rust_value_type(schema_field['type'])},")
        lines += ["}", ""]
    return lines


def build_type_helpers() -> list[str]:
    lines: list[str] = []
    for type_name in USED_TYPES:
        ffi_info = MAPPING["ffi_types"].get(type_name, {})
        if "fields" in ffi_info:
            lines.extend(build_struct_writer(type_name))
            if type_name not in {"DebuggerConfig", "ContextConfig"}:
                lines.extend(build_struct_reader(type_name))
    lines += [
        "fn ptr_len_to_string(ptr: *const u8, len: usize) -> String {",
        "    if ptr.is_null() || len == 0 {",
        "        return String::new();",
        "    }",
        "    let bytes = unsafe { // SAFETY: the caller guarantees `ptr` is valid for `len` bytes during the FFI borrow.",
        "        std::slice::from_raw_parts(ptr, len)",
        "    };",
        "    String::from_utf8_lossy(bytes).into_owned()",
        "}",
        "",
        "fn goud_context_id_from_jlong(value: jni::sys::jlong) -> crate::ffi::GoudContextId {",
        "    unsafe { // SAFETY: `GoudContextId` is a repr(C) u64 newtype used by the FFI ABI.",
        "        std::mem::transmute::<u64, crate::ffi::GoudContextId>(value as u64)",
        "    }",
        "}",
        "",
        "fn goud_context_id_to_jlong(value: crate::ffi::GoudContextId) -> jni::sys::jlong {",
        "    unsafe { // SAFETY: `GoudContextId` is a repr(C) u64 newtype used by the FFI ABI.",
        "        std::mem::transmute::<crate::ffi::GoudContextId, u64>(value) as i64",
        "    }",
        "}",
        "",
        "fn goud_entity_id_from_jlong(value: jni::sys::jlong) -> crate::ffi::GoudEntityId {",
        "    crate::ffi::GoudEntityId::new(value as u64)",
        "}",
        "",
        "pub(crate) fn parse_json_document(env: &mut jni::JNIEnv<'_>, function_name: &str, json_text: &str) -> crate::jni::helpers::JniCallResult<serde_json::Value> {",
        "    serde_json::from_str(json_text).map_err(|error| {",
        "        let _ = crate::jni::helpers::throw_illegal_argument(env, format!(\"{function_name} returned invalid JSON: {error}\"));",
        "    })",
        "}",
        "",
        "pub(crate) fn parse_json_bytes_field(env: &mut jni::JNIEnv<'_>, function_name: &str, value: &serde_json::Value, field: &str) -> crate::jni::helpers::JniCallResult<Vec<u8>> {",
        "    let Some(array) = value.get(field).and_then(serde_json::Value::as_array) else {",
        "        crate::jni::helpers::throw_illegal_argument(env, format!(\"{function_name} missing byte array field {field}\"))?;",
        "        return Err(());",
        "    };",
        "    let mut bytes = Vec::with_capacity(array.len());",
        "    for (index, item) in array.iter().enumerate() {",
        "        let Some(number) = item.as_u64() else {",
        "            crate::jni::helpers::throw_illegal_argument(env, format!(\"{function_name} field {field}[{index}] must be an unsigned byte\"))?;",
            "            return Err(());",
        "        };",
        "        let Ok(byte) = u8::try_from(number) else {",
        "            crate::jni::helpers::throw_illegal_argument(env, format!(\"{function_name} field {field}[{index}] exceeds byte range: {number}\"))?;",
        "            return Err(());",
        "        };",
        "        bytes.push(byte);",
        "    }",
        "    Ok(bytes)",
        "}",
        "",
        "pub(crate) fn parse_json_string_field(env: &mut jni::JNIEnv<'_>, function_name: &str, value: &serde_json::Value, field: &str) -> crate::jni::helpers::JniCallResult<String> {",
        "    let Some(text) = value.get(field).and_then(serde_json::Value::as_str) else {",
        "        crate::jni::helpers::throw_illegal_argument(env, format!(\"{function_name} missing string field {field}\"))?;",
        "        return Err(());",
        "    };",
        "    Ok(text.to_string())",
        "}",
        "",
        "pub(crate) fn read_buffer_protocol_string<F>(env: &mut jni::JNIEnv<'_>, function_name: &str, mut call: F) -> crate::jni::helpers::JniCallResult<String>",
        "where",
        "    F: FnMut(*mut u8, usize) -> i32,",
        "{",
        "    let required = call(std::ptr::null_mut(), 0);",
        "    if required == -1 {",
        "        if crate::jni::helpers::last_error_code() != 0 {",
        "            let _ = crate::jni::helpers::throw_engine_error(env, function_name, None);",
        "        }",
        "        return Err(());",
        "    }",
        "    if required == 0 {",
        "        return Ok(String::new());",
        "    }",
        "    let mut size = if required < 0 { (-required) as usize } else { required as usize + 1 };",
        "    loop {",
        "        let mut buffer = vec![0u8; size];",
        "        let written = call(buffer.as_mut_ptr(), buffer.len());",
        "        if written == -1 {",
        "            if crate::jni::helpers::last_error_code() != 0 {",
        "                let _ = crate::jni::helpers::throw_engine_error(env, function_name, None);",
        "            }",
        "            return Err(());",
        "        }",
        "        if written < 0 {",
            "            size = (-written) as usize;",
            "            continue;",
        "        }",
        "        let written_len = crate::jni::helpers::checked_output_length(env, function_name, \"buffer\", written as usize, buffer.len())?;",
        "        if crate::jni::helpers::last_error_code() != 0 {",
        "            let _ = crate::jni::helpers::throw_engine_error(env, function_name, Some(written as i64));",
        "            return Err(());",
        "        }",
        "        return Ok(String::from_utf8_lossy(&buffer[..written_len]).into_owned());",
        "    }",
        "}",
        "",
        "pub(crate) fn read_fixed_buffer_string<F>(env: &mut jni::JNIEnv<'_>, function_name: &str, mut call: F, size: usize) -> crate::jni::helpers::JniCallResult<String>",
        "where",
        "    F: FnMut(*mut u8, i32) -> i32,",
        "{",
        "    let mut buffer = vec![0u8; size];",
        "    let written = call(buffer.as_mut_ptr(), buffer.len() as i32);",
        "    if written < 0 {",
        "        if crate::jni::helpers::last_error_code() != 0 {",
        "            let _ = crate::jni::helpers::throw_engine_error(env, function_name, Some(written as i64));",
        "        }",
        "        return Err(());",
        "    }",
        "    let written_len = crate::jni::helpers::checked_output_length(env, function_name, \"buffer\", written as usize, buffer.len())?;",
        "    if crate::jni::helpers::last_error_code() != 0 {",
        "        let _ = crate::jni::helpers::throw_engine_error(env, function_name, Some(written as i64));",
        "        return Err(());",
        "    }",
        "    Ok(String::from_utf8_lossy(&buffer[..written_len]).into_owned())",
        "}",
        "",
        "pub(crate) fn new_NetworkPacket<'local>(",
        "    env: &mut jni::JNIEnv<'local>,",
        "    peer_id: u64,",
        "    data: &[u8],",
        ") -> crate::jni::helpers::JniCallResult<jni::objects::JObject<'local>> {",
        "    let obj = crate::jni::helpers::new_object(env, \"com/goudengine/internal/NetworkPacket\")?;",
        "    crate::jni::helpers::set_long_field(env, &obj, \"peerId\", peer_id as i64)?;",
        "    crate::jni::helpers::set_byte_array_field(env, &obj, \"data\", data)?;",
        "    Ok(obj)",
        "}",
        "",
        "pub(crate) fn new_AnimationEventData<'local>(",
        "    env: &mut jni::JNIEnv<'local>,",
        "    entity: u64,",
        "    name: &str,",
        "    frame_index: u32,",
        "    payload_type: u32,",
        "    payload_int: i32,",
        "    payload_float: f32,",
        "    payload_string: &str,",
        ") -> crate::jni::helpers::JniCallResult<jni::objects::JObject<'local>> {",
        "    let obj = crate::jni::helpers::new_object(env, \"com/goudengine/internal/AnimationEventData\")?;",
        "    crate::jni::helpers::set_long_field(env, &obj, \"entity\", entity as i64)?;",
        "    crate::jni::helpers::set_string_field(env, &obj, \"name\", name)?;",
        "    crate::jni::helpers::set_int_field(env, &obj, \"frameIndex\", frame_index as i32)?;",
        "    crate::jni::helpers::set_int_field(env, &obj, \"payloadType\", payload_type as i32)?;",
        "    crate::jni::helpers::set_int_field(env, &obj, \"payloadInt\", payload_int)?;",
        "    crate::jni::helpers::set_float_field(env, &obj, \"payloadFloat\", payload_float)?;",
        "    crate::jni::helpers::set_string_field(env, &obj, \"payloadString\", payload_string)?;",
        "    Ok(obj)",
        "}",
        "",
        "pub(crate) fn new_DebuggerCapture<'local>(",
        "    env: &mut jni::JNIEnv<'local>,",
        "    function_name: &str,",
        "    json_text: &str,",
        ") -> crate::jni::helpers::JniCallResult<jni::objects::JObject<'local>> {",
        "    let value = parse_json_document(env, function_name, json_text)?;",
        "    let obj = crate::jni::helpers::new_object(env, \"com/goudengine/internal/DebuggerCapture\")?;",
        "    let image_png = parse_json_bytes_field(env, function_name, &value, \"imagePng\")?;",
        "    crate::jni::helpers::set_byte_array_field(env, &obj, \"imagePng\", &image_png)?;",
        "    for field in [\"metadataJson\", \"snapshotJson\", \"metricsTraceJson\"] {",
        "        let field_text = parse_json_string_field(env, function_name, &value, field)?;",
        "        let java_field = match field {",
        "            \"metadataJson\" => \"metadataJson\",",
        "            \"snapshotJson\" => \"snapshotJson\",",
        "            _ => \"metricsTraceJson\",",
        "        };",
        "        crate::jni::helpers::set_string_field(env, &obj, java_field, &field_text)?;",
        "    }",
        "    Ok(obj)",
        "}",
        "",
        "pub(crate) fn new_DebuggerReplayArtifact<'local>(",
        "    env: &mut jni::JNIEnv<'local>,",
        "    function_name: &str,",
        "    json_text: &str,",
        ") -> crate::jni::helpers::JniCallResult<jni::objects::JObject<'local>> {",
        "    let value = parse_json_document(env, function_name, json_text)?;",
        "    let obj = crate::jni::helpers::new_object(env, \"com/goudengine/internal/DebuggerReplayArtifact\")?;",
        "    let manifest_text = parse_json_string_field(env, function_name, &value, \"manifestJson\")?;",
        "    crate::jni::helpers::set_string_field(env, &obj, \"manifestJson\", &manifest_text)?;",
        "    let data = parse_json_bytes_field(env, function_name, &value, \"data\")?;",
        "    crate::jni::helpers::set_byte_array_field(env, &obj, \"data\", &data)?;",
        "    Ok(obj)",
        "}",
        "",
        "pub(crate) fn require_entity_array(env: &mut jni::JNIEnv<'_>, array: jni::objects::JLongArray<'_>, param_name: &str) -> crate::jni::helpers::JniCallResult<Vec<u64>> {",
        "    let values = crate::jni::helpers::require_long_array(env, array.into_raw(), param_name)?;",
        "    Ok(values.into_iter().map(|value| value as u64).collect())",
        "}",
        "",
        "pub(crate) fn new_entity_array(env: &mut jni::JNIEnv<'_>, values: &[u64]) -> crate::jni::helpers::JniCallResult<jni::sys::jlongArray> {",
        "    let longs: Vec<i64> = values.iter().map(|value| *value as i64).collect();",
        "    crate::jni::helpers::new_long_array(env, &longs)",
        "}",
        "",
        "pub(crate) fn marshal_debugger_config<'local>(",
        "    env: &mut jni::JNIEnv<'local>,",
        "    obj: jni::objects::JObject<'local>,",
        ") -> crate::jni::helpers::JniCallResult<(crate::ffi::context::GoudDebuggerConfig, Option<std::ffi::CString>)> {",
        "    let obj = crate::jni::helpers::require_object(env, obj, \"debuggerConfig\")?;",
        "    let enabled = crate::jni::helpers::get_boolean_field(env, &obj, \"enabled\")?;",
        "    let publish = crate::jni::helpers::get_boolean_field(env, &obj, \"publishLocalAttach\")?;",
        "    let route = crate::jni::helpers::get_object_field(env, &obj, \"routeLabel\", \"Ljava/lang/String;\")?;",
        "    let route_cstr = if route.is_null() {",
        "        None",
        "    } else {",
        "        Some(crate::jni::helpers::require_c_string(env, jni::objects::JString::from(route), \"routeLabel\")?)",
        "    };",
        "    let raw = crate::ffi::context::GoudDebuggerConfig {",
        "        enabled,",
        "        publish_local_attach: publish,",
        "        route_label: route_cstr.as_ref().map_or(std::ptr::null(), |value| value.as_ptr()),",
        "    };",
        "    Ok((raw, route_cstr))",
        "}",
        "",
        "pub(crate) fn marshal_context_config<'local>(",
        "    env: &mut jni::JNIEnv<'local>,",
        "    obj: jni::objects::JObject<'local>,",
        ") -> crate::jni::helpers::JniCallResult<(crate::ffi::context::GoudContextConfig, Option<std::ffi::CString>)> {",
        "    let obj = crate::jni::helpers::require_object(env, obj, \"contextConfig\")?;",
        "    let debugger = crate::jni::helpers::get_object_field(env, &obj, \"debugger\", \"Lcom/goudengine/internal/DebuggerConfig;\")?;",
        "    let (debugger, route) = marshal_debugger_config(env, debugger)?;",
        "    Ok((crate::ffi::context::GoudContextConfig { debugger }, route))",
        "}",
        "",
    ]
    return lines


def rust_method_signature(method: GeneratedMethod) -> list[str]:
    args = ["mut env: jni::JNIEnv<'local>", "_class: jni::objects::JClass<'local>"]
    if method.handle_type is not None and not method.mapping.get("no_context"):
        args.append(f"{handle_arg_name(method.handle_type)}: jni::sys::jlong")
    if method.self_param is not None:
        if is_builder_name(method.self_param):
            args.append("selfHandle: jni::sys::jlong")
        else:
            args.append(f"selfObj: {rust_arg_type(method.self_param)}")
    for param in reorder_jni_params(method.params, method.mapping):
        args.append(f"{to_camel(param['name'])}: {rust_arg_type(param['type'])}")
    return args


def render_engine_error_check(ffi_name: str, status_expr: str | None = None) -> list[str]:
    if status_expr is None:
        return [
            "    if crate::jni::helpers::last_error_code() != 0 {",
            f'        let _ = crate::jni::helpers::throw_engine_error(&mut env, "{ffi_name}", None);',
            "        return Err(());",
            "    }",
        ]
    return [
        "    if crate::jni::helpers::last_error_code() != 0 {",
        f'        let _ = crate::jni::helpers::throw_engine_error(&mut env, "{ffi_name}", Some({status_expr} as i64));',
        "        return Err(());",
        "    }",
    ]


def build_ffi_args(method: GeneratedMethod, *, with_out_params: bool = False, skip_special: bool = False) -> tuple[list[str], list[str], int]:
    ffi_meta = method_ffi_def(method.mapping)
    param_defs = ffi_meta.get("params", [])
    consumed_index = 0
    setup: list[str] = ["    crate::jni::helpers::clear_last_error();"]
    args: list[str] = []
    if method.handle_type is not None and not method.mapping.get("no_context"):
        handle_name = handle_arg_name(method.handle_type)
        args.append(handle_value_expr(method.handle_type, handle_name))
        consumed_index += 1
    if method.self_param is not None:
        if is_builder_name(method.self_param):
            args.append("selfHandle as _")
        else:
            setup.append(f'    let mut self_raw = read_{method.self_param}(&mut env, &selfObj, "self")?;')
            self_mode = method.mapping.get("self_param", "")
            if self_mode.startswith("*mut"):
                args.append("&mut self_raw as _")
            elif self_mode.startswith("*const"):
                args.append("&self_raw as _")
            else:
                args.append("self_raw")
        consumed_index += 1
    if skip_special:
        return setup, args, consumed_index
    string_params = set(method.mapping.get("string_params", []))
    entity_params = set(method.mapping.get("entity_params", []))
    struct_params = set(method.mapping.get("struct_params", []))
    expand_params = method.mapping.get("expand_params", {})
    enum_params = set((method.mapping.get("enum_params") or {}).keys())
    for param in reorder_jni_params(method.params, method.mapping):
        name = to_camel(param["name"])
        schema_ty = base_type(param["type"])
        expected = param_defs[consumed_index]["type"] if consumed_index < len(param_defs) else ""
        if schema_ty == "Entity[]":
            setup.append(f'    let {name}_values = require_entity_array(&mut env, {name}, "{name}")?;')
            args.append(f"if {name}_values.is_empty() {{ std::ptr::null() }} else {{ {name}_values.as_ptr() }} as _")
            consumed_index += 1
            if consumed_index < len(param_defs):
                args.append(f"{name}_values.len() as _")
                consumed_index += 1
            continue
        if schema_ty == "u8[]" and method.mapping.get("batch_in"):
            setup.append(f'    let mut {name}_bytes = crate::jni::helpers::require_bytes(&mut env, {name}, "{name}")?;')
            continue
        if schema_ty == "string" and (name in string_params or expected in {"*const c_char", "*const u8", "*const i8"}):
            if expected == "*const c_char":
                setup.append(f'    let {name}_cstr = crate::jni::helpers::require_c_string(&mut env, {name}, "{name}")?;')
                args.append(f"{name}_cstr.as_ptr()")
                consumed_index += 1
            else:
                setup.append(f'    let {name}_bytes = crate::jni::helpers::require_string_bytes(&mut env, {name}, "{name}")?;')
                args.append(f"if {name}_bytes.is_empty() {{ std::ptr::null() }} else {{ {name}_bytes.as_ptr() }} as _")
                consumed_index += 1
                if consumed_index < len(param_defs):
                    args.append(f"{name}_bytes.len() as _")
                    consumed_index += 1
            continue
        if schema_ty in {"bytes", "u8[]"}:
            setup.append(f'    let {name}_bytes = crate::jni::helpers::require_bytes(&mut env, {name}, "{name}")?;')
            args.append(f"if {name}_bytes.is_empty() {{ std::ptr::null() }} else {{ {name}_bytes.as_ptr() }} as _")
            consumed_index += 1
            if consumed_index < len(param_defs):
                args.append(f"{name}_bytes.len() as _")
                consumed_index += 1
            continue
        if name in entity_params or schema_ty == "Entity":
            if expected == "GoudEntityId":
                args.append(f"goud_entity_id_from_jlong({name})")
            else:
                args.append(f"{name} as _")
            consumed_index += 1
            continue
        if name in expand_params:
            setup.append(f'    let {name}_raw = read_{schema_ty}(&mut env, &{name}, "{name}")?;')
            for field_name in expand_params[name]["fields"]:
                args.append(f"{name}_raw.{field_name} as _")
                consumed_index += 1
            continue
        if schema_ty == "ContextConfig":
            setup.append(f"    let ({name}_raw, _{name}_route) = marshal_context_config(&mut env, {name})?;")
            args.append(f"&{name}_raw as _")
            consumed_index += 1
            continue
        if schema_ty == "DebuggerConfig":
            setup.append(f"    let ({name}_raw, _{name}_route) = marshal_debugger_config(&mut env, {name})?;")
            args.append(f"&{name}_raw as _")
            consumed_index += 1
            continue
        if name in struct_params or (schema_ty in SCHEMA.get("types", {}) and schema_ty != "Entity"):
            setup.append(f'    let mut {name}_raw = read_{schema_ty}(&mut env, &{name}, "{name}")?;')
            if expected.startswith("*mut"):
                args.append(f"&mut {name}_raw as _")
            elif expected.startswith("*const"):
                args.append(f"&{name}_raw as _")
            else:
                args.append(f"{name}_raw")
            consumed_index += 1
            continue
        if name in enum_params or schema_ty in SCHEMA.get("enums", {}):
            if expected.startswith("Ffi") or expected.startswith("Goud"):
                setup.extend(
                    render_checked_enum_conversion(
                        f"{name}_raw",
                        schema_ty,
                        name,
                        name,
                        env_expr="&mut env",
                    )
                )
                args.append(f"{name}_raw")
            else:
                args.append(f"{name} as _")
            consumed_index += 1
            continue
        if schema_ty == "bool":
            args.append(f"{name} != 0")
            consumed_index += 1
            continue
        args.append(f"{name} as _")
        consumed_index += 1
    return setup, args, consumed_index


def render_component_strategy(method: GeneratedMethod) -> list[str]:
    strategy = method.mapping["ffi_strategy"]
    component_type = method.mapping.get("component_type", "")
    handle_name = handle_arg_name(method.handle_type)
    component_register = ffi_function_path("goud_component_register_type")
    component_add = ffi_function_path("goud_component_add")
    component_get = ffi_function_path("goud_component_get")
    component_has = ffi_function_path("goud_component_has")
    component_remove = ffi_function_path("goud_component_remove")
    lines = ["    crate::jni::helpers::clear_last_error();"]
    if strategy.startswith("component_"):
        raw_type = rust_struct_type(component_type)
        type_hash = component_type_hash(component_type)
        lines += [
            "    {",
            f'        let name = b"{component_type}";',
            "        let _registered = unsafe { // SAFETY: the component metadata is static and matches the temporary raw value layout used by the bridge.",
            f"            {component_register}(",
            f"                {type_hash},",
            "                name.as_ptr(),",
            "                name.len(),",
            f"                std::mem::size_of::<{raw_type}>(),",
            f"                std::mem::align_of::<{raw_type}>(),",
            "            )",
            "        };",
            "    }",
        ]
    if strategy == "component_add":
        struct_name = to_camel(method.mapping.get("struct_params", [to_camel(component_type)])[0])
        lines += [
            f'    let {struct_name}_raw = read_{component_type}(&mut env, &{struct_name}, "{struct_name}")?;',
            "    let result = unsafe { // SAFETY: the component payload points to validated stack storage for the duration of the FFI call.",
            f"        {component_add}(",
            f"            {handle_value_expr(method.handle_type, handle_name)},",
            "            goud_entity_id_from_jlong(entity),",
            f"            {type_hash},",
            f"            (&{struct_name}_raw as *const {raw_type}).cast::<u8>(),",
            f"            std::mem::size_of::<{raw_type}>(),",
            "        )",
            "    };",
            "    if !result.success {",
            '        let _ = crate::jni::helpers::throw_engine_error(&mut env, "goud_component_add", Some(result.code as i64));',
            "        return Err(());",
            "    }",
            "    Ok(())",
        ]
        return lines
    if strategy == "component_set":
        struct_name = to_camel(method.mapping.get("struct_params", [to_camel(component_type)])[0])
        lines += [
            f'    let {struct_name}_raw = read_{component_type}(&mut env, &{struct_name}, "{struct_name}")?;',
            "    let result = unsafe { // SAFETY: the component payload points to validated stack storage for the duration of the FFI call.",
            f"        {component_add}(",
            f"            {handle_value_expr(method.handle_type, handle_name)},",
            "            goud_entity_id_from_jlong(entity),",
            f"            {type_hash},",
            f"            (&{struct_name}_raw as *const {raw_type}).cast::<u8>(),",
            f"            std::mem::size_of::<{raw_type}>(),",
            "        )",
            "    };",
            "    if !result.success {",
            '        let _ = crate::jni::helpers::throw_engine_error(&mut env, "goud_component_add", Some(result.code as i64));',
            "        return Err(());",
            "    }",
            "    Ok(())",
        ]
        return lines
    if strategy == "component_get":
        lines += [
            f"    let ptr = {component_get}({handle_value_expr(method.handle_type, handle_name)}, goud_entity_id_from_jlong(entity), {type_hash}) as *const {raw_type};",
            "    if ptr.is_null() {",
            "        if crate::jni::helpers::last_error_code() != 0 {",
            '            let _ = crate::jni::helpers::throw_engine_error(&mut env, "goud_component_get", None);',
            "            return Err(());",
            "        }",
            "        return Ok(crate::jni::helpers::null_object());",
            "    }",
            "    let value = unsafe { // SAFETY: the FFI returned a non-null pointer to a component payload with the registered layout.",
            "        *ptr",
            "    };",
            f"    Ok(new_{component_type}(&mut env, value)?.into_raw())",
        ]
        return lines
    if strategy == "component_has":
        lines += [
            f"    let result = {component_has}({handle_value_expr(method.handle_type, handle_name)}, goud_entity_id_from_jlong(entity), {type_hash});",
            "    if !result && crate::jni::helpers::last_error_code() != 0 {",
            '        let _ = crate::jni::helpers::throw_engine_error(&mut env, "goud_component_has", None);',
            "        return Err(());",
            "    }",
            "    Ok(crate::jni::helpers::to_jboolean(result))",
        ]
        return lines
    if strategy == "component_remove":
        lines += [
            f"    let result = {component_remove}({handle_value_expr(method.handle_type, handle_name)}, goud_entity_id_from_jlong(entity), {type_hash});",
            "    if !result.success {",
            '        let _ = crate::jni::helpers::throw_engine_error(&mut env, "goud_component_remove", Some(result.code as i64));',
            "        return Err(());",
            "    }",
            "    Ok(jni::sys::JNI_TRUE)",
        ]
        return lines
    name_type = "crate::ecs::components::Name"
    lines += [
        f"    let context_id = {handle_value_expr(method.handle_type, handle_name)};",
        "    if context_id == crate::ffi::GOUD_INVALID_CONTEXT_ID {",
        "        crate::core::error::set_last_error(crate::core::error::GoudError::InvalidContext);",
        '        let _ = crate::jni::helpers::throw_engine_error(&mut env, "jni_name_strategy", None);',
        "        return Err(());",
        "    }",
        "    let entity_id = crate::ecs::Entity::from_bits(entity as u64);",
    ]
    if strategy == "name_add":
        lines += [
            '    let name_cstr = crate::jni::helpers::require_string_bytes(&mut env, name, "name")?;',
            "    let name_string = String::from_utf8(name_cstr).map_err(|_| ())?;",
            "    let mut registry = crate::ffi::context::get_context_registry().lock().map_err(|_| ())?;",
            "    let context = registry.get_mut(context_id).ok_or(())?;",
            "    context.world_mut().insert(entity_id, " + name_type + "::new(name_string));",
            "    Ok(())",
        ]
    elif strategy == "name_get":
        lines += [
            "    let registry = crate::ffi::context::get_context_registry().lock().map_err(|_| ())?;",
            "    let context = registry.get(context_id).ok_or(())?;",
            f"    let Some(name) = context.world().get::<{name_type}>(entity_id) else {{",
            "        return Ok(crate::jni::helpers::null_string());",
            "    };",
            "    crate::jni::helpers::new_java_string(&mut env, name.as_str())",
        ]
    elif strategy == "name_has":
        lines += [
            "    let registry = crate::ffi::context::get_context_registry().lock().map_err(|_| ())?;",
            "    let context = registry.get(context_id).ok_or(())?;",
            f"    Ok(crate::jni::helpers::to_jboolean(context.world().has::<{name_type}>(entity_id)))",
        ]
    elif strategy == "name_remove":
        lines += [
            "    let mut registry = crate::ffi::context::get_context_registry().lock().map_err(|_| ())?;",
            "    let context = registry.get_mut(context_id).ok_or(())?;",
            f"    Ok(crate::jni::helpers::to_jboolean(context.world_mut().remove::<{name_type}>(entity_id).is_some()))",
        ]
    return lines


def render_returns_entity(method: GeneratedMethod) -> list[str]:
    setup, args, _ = build_ffi_args(method)
    ffi_name = method.mapping["ffi"]
    ffi_path = ffi_function_path(ffi_name)
    ffi_meta = method_ffi_def(method.mapping)
    lines = setup
    call_expr = f"{ffi_path}({', '.join(args)})"
    if ffi_meta.get("unsafe"):
        lines.append(f"    let result = unsafe {{ // SAFETY: JNI inputs are validated and temporary buffers stay alive across the FFI call.\n        {call_expr}\n    }};")
    else:
        lines.append(f"    let result = {call_expr};")
    lines.extend(render_engine_error_check(ffi_name))
    lines.append("    Ok(result as i64)")
    return lines


def render_batch_out(method: GeneratedMethod) -> list[str]:
    handle_name = handle_arg_name(method.handle_type)
    ffi_name = method.mapping["ffi"]
    ffi_path = ffi_function_path(ffi_name)
    lines = [
        "    crate::jni::helpers::clear_last_error();",
        "    let count_value = count.max(0) as usize;",
        "    let mut entities = vec![0u64; count_value];",
        f"    let written = unsafe {{ // SAFETY: the output buffer is sized for `count` entities and remains valid for the duration of the FFI call.\n        {ffi_path}({handle_value_expr(method.handle_type, handle_name)}, count as u32, entities.as_mut_ptr())\n    }};",
    ]
    lines.extend(render_engine_error_check(ffi_name, "written"))
    lines += [
        f'    let written_len = crate::jni::helpers::checked_output_length(&mut env, "{ffi_name}", "entities", written as usize, entities.len())?;',
        "    entities.truncate(written_len);",
        "    new_entity_array(&mut env, &entities)",
    ]
    return lines


def render_batch_in(method: GeneratedMethod) -> list[str]:
    handle_name = handle_arg_name(method.handle_type)
    ffi_name = method.mapping["ffi"]
    ffi_path = ffi_function_path(ffi_name)
    lines = [
        "    crate::jni::helpers::clear_last_error();",
        '    let entities_values = require_entity_array(&mut env, entities, "entities")?;',
    ]
    args = [
        handle_value_expr(method.handle_type, handle_name),
        "if entities_values.is_empty() { std::ptr::null() } else { entities_values.as_ptr() } as _",
        "entities_values.len() as u32",
    ]
    if method.method_name in {"componentAddBatch", "componentRemoveBatch", "componentHasBatch"}:
        args.append("typeIdHash as u64")
    if method.method_name == "componentAddBatch":
        args.extend(["dataPtr as usize as *const u8", "componentSize as usize"])
    if method.mapping.get("batch_out_results"):
        lines += [
            "    if outResults.is_null() {",
            '        crate::jni::helpers::throw_null_pointer(&mut env, "outResults is null")?;',
            "        return Err(());",
            "    }",
            "    let results_len = crate::jni::helpers::byte_array_length(&mut env, &outResults)?;",
            "    if results_len < entities_values.len() {",
            '        crate::jni::helpers::throw_illegal_argument(&mut env, "outResults is smaller than entities")?;',
            "        return Err(());",
            "    }",
            "    let mut result_bytes = vec![0u8; results_len];",
        ]
        args.append("result_bytes.as_mut_ptr()")
        lines.append(f"    let written = unsafe {{ // SAFETY: the entity and output buffers remain valid for the duration of the FFI call.\n        {ffi_path}({', '.join(args)})\n    }};")
        lines.extend(render_engine_error_check(ffi_name, "written"))
        lines.append(
            f'    let written_len = crate::jni::helpers::checked_output_length(&mut env, "{ffi_name}", "outResults", written as usize, result_bytes.len())?;'
        )
        lines.append("    crate::jni::helpers::write_byte_array(&mut env, &outResults, &result_bytes[..written_len])?;")
        lines.append("    Ok(written_len as i32)")
        return lines
    lines.append(f"    let written = unsafe {{ // SAFETY: the entity buffer remains valid for the duration of the FFI call.\n        {ffi_path}({', '.join(args)})\n    }};")
    lines.extend(render_engine_error_check(ffi_name, "written"))
    lines.append("    Ok(written as i32)")
    return lines


def render_out_params(method: GeneratedMethod) -> list[str]:
    ffi_name = method.mapping["ffi"]
    ffi_path = ffi_function_path(ffi_name)
    setup, args, consumed_index = build_ffi_args(method, skip_special=False)
    ffi_meta = method_ffi_def(method.mapping)
    ffi_ret = ffi_return_type(ffi_meta)
    out_params = method.mapping["out_params"]
    lines = setup
    out_locals: list[str] = []
    for op in out_params:
        local = f"out_{to_camel(op['name'])}"
        out_locals.append(local)
        out_type = op["type"]
        if out_type in SCHEMA.get("types", {}):
            lines.append(f"    let mut {local} = {rust_zero_value(out_type)};")
        elif out_type in TYPE_PATHS or out_type.startswith("Ffi") or out_type.startswith("Goud"):
            lines.append(f"    let mut {local}: {rust_type(out_type)} = {rust_zero_value(out_type)};")
        else:
            rust_out_type = rust_type(out_type)
            if rust_out_type == "u64" and out_type == "Entity":
                rust_out_type = "u64"
            lines.append(f"    let mut {local}: {rust_out_type} = {rust_zero_value(out_type)};")
    for local in out_locals:
        args.append(f"&mut {local} as _")
        consumed_index += 1
    call_expr = f"{ffi_path}({', '.join(args)})"
    if ffi_meta.get("unsafe"):
        lines.append(f"    let status = unsafe {{ // SAFETY: out-parameter storage and marshaled inputs remain valid for the duration of the FFI call.\n        {call_expr}\n    }};")
    else:
        lines.append(f"    let status = {call_expr};")
    if method.mapping.get("returns_nullable_struct"):
        if ffi_ret == "bool":
            lines += [
                "    if !status {",
                "        if crate::jni::helpers::last_error_code() != 0 {",
                f'            let _ = crate::jni::helpers::throw_engine_error(&mut env, "{ffi_name}", None);',
                "            return Err(());",
                "        }",
                "        return Ok(crate::jni::helpers::null_object());",
                "    }",
            ]
        else:
            lines += [
                "    if status == 0 {",
                "        return Ok(crate::jni::helpers::null_object());",
                "    }",
            ]
        lines.extend(render_engine_error_check(ffi_name))
        struct_name = method.mapping["returns_nullable_struct"]
        if len(out_params) == 1:
            lines.append(f"    Ok(new_{struct_name}(&mut env, out_{to_camel(out_params[0]['name'])})?.into_raw())")
        else:
            raise ValueError(f"Unexpected nullable-struct shape for {method.owner_name}.{method.method_name}")
        return lines
    if method.mapping.get("status_nullable_struct") or method.mapping.get("status_struct"):
        lines += [
            "    if status < 0 {",
            f'        let _ = crate::jni::helpers::throw_engine_error(&mut env, "{ffi_name}", Some(status as i64));',
            "        return Err(());",
            "    }",
        ]
        if method.mapping.get("status_nullable_struct"):
            lines += [
                "    if status == 0 {",
                "        return Ok(crate::jni::helpers::null_object());",
                "    }",
            ]
    else:
        lines.extend(render_engine_error_check(ffi_name))
    if method.mapping.get("returns_scalar"):
        local = out_locals[0]
        scalar_type = base_type(method.mapping["returns_scalar"])
        if scalar_type == "f32":
            lines.append(f"    Ok({local})")
        elif scalar_type == "bool":
            lines.append(f"    Ok(crate::jni::helpers::to_jboolean({local}))")
        else:
            lines.append(f"    Ok({local} as _)")
        return lines
    struct_name = method.mapping.get("returns_struct")
    if struct_name is None:
        actual_return = base_type(method.returns)
        if actual_return == "void":
            lines.append("    Ok(())")
        elif len(out_locals) == 1 and actual_return in {"f32", "f64"}:
            lines.append(f"    Ok({out_locals[0]})")
        elif len(out_locals) == 1 and actual_return == "bool":
            lines.append(f"    Ok(crate::jni::helpers::to_jboolean({out_locals[0]} != 0))")
        elif len(out_locals) == 1 and actual_return in {"i32", "u8", "u16", "u32", "i64", "u64", "usize", "ptr", "Entity", "GoudGame", "GoudResult"}:
            lines.append(f"    Ok({out_locals[0]} as _)")
        elif actual_return == "object":
            lines.append('    let map = crate::jni::helpers::new_hash_map(&mut env)?;')
            for field, local in zip(out_params, out_locals):
                java_name = to_camel(field["name"])
                field_type = base_type(field["type"])
                if field_type in {"u64", "usize", "Entity", "i64"}:
                    lines.append(f'    let value_{java_name} = crate::jni::helpers::new_boxed_long(&mut env, {local} as i64)?;')
                elif field_type == "f32":
                    lines.append(f'    let value_{java_name} = crate::jni::helpers::new_boxed_float(&mut env, {local})?;')
                elif field_type == "f64":
                    lines.append(f'    let value_{java_name} = crate::jni::helpers::new_boxed_double(&mut env, {local})?;')
                else:
                    lines.append(f'    let value_{java_name} = crate::jni::helpers::new_boxed_int(&mut env, {local} as i32)?;')
                lines.append(f'    crate::jni::helpers::put_hash_map_value(&mut env, &map, "{java_name}", &value_{java_name})?;')
            lines.append("    Ok(map.into_raw())")
        elif ffi_ret == "i32":
            lines.append("    Ok(status as i32)")
        else:
            lines.append("    Ok(status as _)")
        return lines
    ffi_type = MAPPING["ffi_types"].get(struct_name)
    if ffi_type is None:
        lines.append(f'    let obj = crate::jni::helpers::new_object(&mut env, "com/goudengine/internal/{struct_name}")?;')
        for field, local in zip(SCHEMA["types"][struct_name]["fields"], out_locals):
            java_name = to_camel(field["name"])
            sig = field_sig(field["type"])
            field_type = base_type(field["type"])
            if field_type == "bool":
                lines.append(f'    crate::jni::helpers::set_boolean_field(&mut env, &obj, "{java_name}", {local})?;')
            elif field_type in {"f32"}:
                lines.append(f'    crate::jni::helpers::set_float_field(&mut env, &obj, "{java_name}", {local})?;')
            elif field_type in {"f64"}:
                lines.append(f'    crate::jni::helpers::set_double_field(&mut env, &obj, "{java_name}", {local})?;')
            elif field_type in {"i64", "u64", "usize", "ptr", "Entity"}:
                lines.append(f'    crate::jni::helpers::set_long_field(&mut env, &obj, "{java_name}", {local} as i64)?;')
            else:
                lines.append(f'    crate::jni::helpers::set_int_field(&mut env, &obj, "{java_name}", {local} as i32)?;')
        lines.append("    Ok(obj.into_raw())")
        return lines
    if len(out_params) == 1 and ffi_type and out_params[0]["type"] in {ffi_type["ffi_name"], struct_name}:
        lines.append(f"    Ok(new_{struct_name}(&mut env, {out_locals[0]})?.into_raw())")
        return lines
    raw_type = rust_struct_type(struct_name)
    lines.append(f"    let value = {raw_type} {{")
    for ffi_field, local in zip(ffi_type["fields"], out_locals):
        lines.append(f"        {ffi_field}: {local},")
    lines.append("    };")
    lines.append(f"    Ok(new_{struct_name}(&mut env, value)?.into_raw())")
    return lines


def render_out_buffer(method: GeneratedMethod) -> list[str]:
    handle_name = handle_arg_name(method.handle_type)
    ffi_name = method.mapping["ffi"]
    ffi_path = ffi_function_path(ffi_name)
    lines = ["    crate::jni::helpers::clear_last_error();"]
    if method.mapping.get("no_context"):
        lines.append("    let buffer_size = 65536usize;")
    else:
        caps_type = rust_struct_type("NetworkCapabilities")
        lines += [
            f"    let mut caps: {caps_type} = unsafe {{ // SAFETY: zeroed provider capability storage is immediately filled by the FFI call.\n        std::mem::zeroed()\n    }};",
            f"    unsafe {{ // SAFETY: the provider capability out-parameter points to local stack storage for the duration of the FFI call.\n        {ffi_function_path('goud_provider_network_capabilities')}({handle_value_expr(method.handle_type, handle_name)}, &mut caps as _);\n    }}",
            "    let buffer_size = if caps.max_message_size == 0 { 65536usize } else { caps.max_message_size as usize };",
        ]
    lines += [
        "    let mut buffer = vec![0u8; buffer_size];",
        "    let mut peer_id: u64 = 0;",
        f"    let written = unsafe {{ // SAFETY: the receive buffer and peer id out-parameter remain valid for the duration of the FFI call.\n        {ffi_path}({handle_value_expr(method.handle_type, handle_name)}, handle as _, buffer.as_mut_ptr(), buffer.len() as i32, &mut peer_id as _)\n    }};",
        "    if written < 0 {",
        f'        let _ = crate::jni::helpers::throw_engine_error(&mut env, "{ffi_name}", Some(written as i64));',
        "        return Err(());",
        "    }",
    ]
    lines.extend(render_engine_error_check(ffi_name, "written"))
    lines += [
        f'    let written_len = crate::jni::helpers::checked_output_length(&mut env, "{ffi_name}", "buffer", written as usize, buffer.len())?;',
    ]
    if method.mapping.get("returns_struct") and method.mapping.get("status_nullable_struct"):
        lines += [
            "    if written == 0 {",
            "        return Ok(crate::jni::helpers::null_object());",
            "    }",
            "    buffer.truncate(written_len);",
            "    Ok(new_NetworkPacket(&mut env, peer_id, &buffer)?.into_raw())",
        ]
    elif method.mapping.get("returns_struct"):
        lines += [
            "    if written == 0 {",
            "        return Ok(crate::jni::helpers::null_object());",
            "    }",
            "    buffer.truncate(written_len);",
            "    Ok(new_NetworkPacket(&mut env, peer_id, &buffer)?.into_raw())",
        ]
    else:
        lines += [
            "    if written == 0 {",
            "        return Ok(crate::jni::helpers::null_byte_array());",
            "    }",
            "    crate::jni::helpers::new_byte_array(&mut env, &buffer[..written_len])",
        ]
    return lines


def render_out_string(method: GeneratedMethod) -> list[str]:
    ffi_name = method.mapping["ffi"]
    ffi_path = ffi_function_path(ffi_name)
    if method.owner_name == "AnimationEvents" and method.method_name == "read":
        handle_name = handle_arg_name(method.handle_type)
        return [
            "    crate::jni::helpers::clear_last_error();",
            "    let mut out_entity: u64 = 0;",
            "    let mut out_name_ptr: *const u8 = std::ptr::null();",
            "    let mut out_name_len: u32 = 0;",
            "    let mut out_frame: u32 = 0;",
            "    let mut out_payload_type: u32 = 0;",
            "    let mut out_payload_int: i32 = 0;",
            "    let mut out_payload_float: f32 = 0.0;",
            "    let mut out_payload_str_ptr: *const u8 = std::ptr::null();",
            "    let mut out_payload_str_len: u32 = 0;",
            f"    let status = unsafe {{ // SAFETY: all out-parameters point to local stack storage for the duration of the FFI call.\n        {ffi_path}({handle_value_expr(method.handle_type, handle_name)}, index as u32, &mut out_entity as _, &mut out_name_ptr as _, &mut out_name_len as _, &mut out_frame as _, &mut out_payload_type as _, &mut out_payload_int as _, &mut out_payload_float as _, &mut out_payload_str_ptr as _, &mut out_payload_str_len as _)\n    }};",
            "    if status < 0 {",
            f'        let _ = crate::jni::helpers::throw_engine_error(&mut env, "{ffi_name}", Some(status as i64));',
            "        return Err(());",
            "    }",
            "    let event_name = ptr_len_to_string(out_name_ptr, out_name_len as usize);",
            "    let payload_string = ptr_len_to_string(out_payload_str_ptr, out_payload_str_len as usize);",
            "    Ok(new_AnimationEventData(&mut env, out_entity, &event_name, out_frame, out_payload_type, out_payload_int, out_payload_float, &payload_string)?.into_raw())",
        ]
    if method.mapping.get("out_params"):
        return render_out_params(method)
    setup, args, _ = build_ffi_args(method)
    lines = setup
    ffi_meta = method_ffi_def(method.mapping)
    ffi_ret = ffi_return_type(ffi_meta)
    lines += [
        f'    let result = read_fixed_buffer_string(&mut env, "{ffi_name}", |buf, len| unsafe {{ // SAFETY: the temporary output buffer is valid for the duration of the FFI call.\n        {ffi_path}({", ".join(args)}, buf, {"len as u32" if ffi_ret == "i32" and ffi_name == "goud_plugin_list" else "len"})\n    }}, 65536)?;',
        "    crate::jni::helpers::new_java_string(&mut env, &result)",
    ]
    return lines


def render_buffer_protocol(method: GeneratedMethod) -> list[str]:
    setup, args, _ = build_ffi_args(method)
    ffi_name = method.mapping["ffi"]
    ffi_path = ffi_function_path(ffi_name)
    lines = setup
    call_args = ", ".join(args)
    lines += [
        f'    let value = read_buffer_protocol_string(&mut env, "{ffi_name}", |buf, len| unsafe {{ // SAFETY: the caller-provided buffer is valid for the duration of each FFI call.\n        {ffi_path}({call_args}{", " if call_args else ""}buf, len)\n    }})?;',
        "    crate::jni::helpers::new_java_string(&mut env, &value)",
    ]
    return lines


def render_json_buffer_struct(method: GeneratedMethod) -> list[str]:
    setup, args, _ = build_ffi_args(method)
    ffi_name = method.mapping["ffi"]
    ffi_path = ffi_function_path(ffi_name)
    struct_name = method.mapping["json_buffer_struct"]
    call_args = ", ".join(args)
    lines = setup
    constructor_args = '&mut env, &value'
    if struct_name in {"DebuggerCapture", "DebuggerReplayArtifact"}:
        constructor_args = f'&mut env, "{ffi_name}", &value'
    lines += [
        f'    let value = read_buffer_protocol_string(&mut env, "{ffi_name}", |buf, len| unsafe {{ // SAFETY: the caller-provided buffer is valid for the duration of each FFI call.\n        {ffi_path}({call_args}{", " if call_args else ""}buf, len)\n    }})?;',
        f"    Ok(new_{struct_name}({constructor_args})?.into_raw())",
    ]
    return lines


def render_direct_call(method: GeneratedMethod) -> list[str]:
    ffi_meta = method_ffi_def(method.mapping)
    ffi_ret = ffi_return_type(ffi_meta)
    ffi_name = method.mapping["ffi"]
    ffi_path = ffi_function_path(ffi_name)
    setup, call_args, _ = build_ffi_args(method)
    for extra in method.mapping.get("append_args", []):
        call_args.append("true" if extra is True else "false" if extra is False else repr(extra))
    returns = base_type(method.returns)
    lines = setup
    call_expr = f"{ffi_path}({', '.join(call_args)})"
    if (
        method.self_param is not None
        and not is_builder_name(method.self_param)
        and method.mapping.get("self_param", "").startswith("*mut")
        and returns == "void"
    ):
        lines += [
            f"    unsafe {{ // SAFETY: JNI inputs are validated and the raw carrier stays alive for the duration of the FFI call.\n        {call_expr};\n    }}",
            f"    write_back_{method.self_param}(&mut env, &selfObj, self_raw)?;",
            "    Ok(())",
        ]
        return lines
    if ffi_meta.get("unsafe"):
        safety_comment = "JNI inputs are validated and temporary buffers stay alive across the FFI call."
        if ffi_name == "goud_ui_set_style":
            safety_comment = (
                "JNI inputs are validated, and any raw pointer fields borrowed through `style_raw` "
                "remain valid only for this call frame. The FFI consumes them synchronously and does "
                "not retain those pointers after returning."
            )
        lines.append(f"    let result = unsafe {{ // SAFETY: {safety_comment}\n        {call_expr}\n    }};")
    else:
        lines.append(f"    let result = {call_expr};")
    if method.mapping.get("returns_bool_from_i32"):
        lines += [
            "    if result < 0 {",
            f'        let _ = crate::jni::helpers::throw_engine_error(&mut env, "{ffi_name}", Some(result as i64));',
            "        return Err(());",
            "    }",
            "    Ok(crate::jni::helpers::to_jboolean(result != 0))",
        ]
        return lines
    if returns == "void":
        lines.extend(render_engine_error_check(ffi_name))
        lines.append("    Ok(())")
        return lines
    if returns == "bool":
        if ffi_ret == "i32":
            lines += [
                "    if result < 0 {",
                f'        let _ = crate::jni::helpers::throw_engine_error(&mut env, "{ffi_name}", Some(result as i64));',
                "        return Err(());",
                "    }",
                "    Ok(crate::jni::helpers::to_jboolean(result != 0))",
            ]
            return lines
        if ffi_ret == "GoudResult":
            lines += [
                "    if !result.success {",
                f'        let _ = crate::jni::helpers::throw_engine_error(&mut env, "{ffi_name}", Some(result.code as i64));',
                "        return Err(());",
                "    }",
                "    Ok(jni::sys::JNI_TRUE)",
            ]
            return lines
        lines += [
            "    if !result && crate::jni::helpers::last_error_code() != 0 {",
            f'        let _ = crate::jni::helpers::throw_engine_error(&mut env, "{ffi_name}", None);',
            "        return Err(());",
            "    }",
            "    Ok(crate::jni::helpers::to_jboolean(result))",
        ]
        return lines
    if method.mapping.get("returns_context") or returns == "GoudContextId":
        lines.extend(render_engine_error_check(ffi_name))
        lines.append("    Ok(goud_context_id_to_jlong(result))")
        return lines
    if returns in {"EngineConfigHandle", "UiManagerHandle"}:
        lines.extend(render_engine_error_check(ffi_name))
        lines.append("    Ok(result as usize as i64)")
        return lines
    if returns in {"i32", "u8", "u16", "u32", "GoudResult"} or returns in SCHEMA.get("enums", {}):
        if returns == "GoudResult":
            lines += [
                "    if !result.success {",
                f'        let _ = crate::jni::helpers::throw_engine_error(&mut env, "{ffi_name}", Some(result.code as i64));',
                "        return Err(());",
                "    }",
                "    Ok(result.code as i32)",
            ]
        else:
            lines.extend(render_engine_error_check(ffi_name, "result"))
            lines.append("    Ok(result as i32)")
        return lines
    if returns in {"i64", "u64", "usize", "ptr", "Entity"} or returns == "GoudGame" or is_builder_name(returns):
        lines.extend(render_engine_error_check(ffi_name))
        lines.append("    Ok(result as i64)")
        return lines
    if returns == "f32" or returns == "f64":
        lines.extend(render_engine_error_check(ffi_name))
        lines.append("    Ok(result)")
        return lines
    if returns == "string":
        lines.extend(render_engine_error_check(ffi_name))
        lines.append("    crate::jni::helpers::new_java_string(&mut env, &result)")
        return lines
    if returns in {"bytes", "u8[]"}:
        lines.extend(render_engine_error_check(ffi_name))
        lines.append("    crate::jni::helpers::new_byte_array(&mut env, &result)")
        return lines
    lines.extend(render_engine_error_check(ffi_name))
    lines.append(f"    Ok(new_{returns}(&mut env, result)?.into_raw())")
    return lines


def render_method_body(method: GeneratedMethod) -> list[str]:
    if method.mapping.get("ffi_strategy"):
        return render_component_strategy(method)
    if method.mapping.get("returns_entity"):
        return render_returns_entity(method)
    if method.mapping.get("batch_out"):
        return render_batch_out(method)
    if method.mapping.get("batch_in"):
        return render_batch_in(method)
    if method.mapping.get("out_buffer"):
        return render_out_buffer(method)
    if method.mapping.get("json_buffer_struct"):
        return render_json_buffer_struct(method)
    if method.mapping.get("buffer_protocol"):
        return render_buffer_protocol(method)
    if method.mapping.get("out_string"):
        return render_out_string(method)
    if method.mapping.get("out_params"):
        return render_out_params(method)
    if "ffi" not in method.mapping:
        return ['    crate::jni::helpers::throw_illegal_state(&mut env, "wrapper-only method is not exposed in the internal JNI bridge")?;', "    Ok(())"]
    return render_direct_call(method)


def rust_export_name(method: GeneratedMethod) -> str:
    return f"Java_com_goudengine_internal_{method.class_name}_{method.java_method_name}"


def rust_method_source(method: GeneratedMethod) -> list[str]:
    feature = method_required_feature(method)
    lines: list[str] = []
    if feature is not None:
        lines.append(f'#[cfg(feature = "{feature}")]')
    lines += [
        "#[allow(non_snake_case)]",
        "#[no_mangle]",
        f"pub extern \"system\" fn {rust_export_name(method)}<'local>(",
        line_join([f"{arg}," for arg in rust_method_signature(method)], 4),
        f") -> {rust_return_type(method.returns)} {{",
    ]
    body = ["    crate::jni::helpers::prepare_call(&mut env)?;"]
    body.extend(render_method_body(method))
    export_body = [line.replace("&mut env", "env") for line in body]
    if base_type(method.returns) == "void":
        lines += [
            f'    crate::jni::helpers::catch_jni_panic(&mut env, "{rust_export_name(method)}", (), |env| -> crate::jni::helpers::JniCallResult<()> {{',
            line_join(export_body, 8),
            "    });",
        ]
    else:
        lines += [
            f'    crate::jni::helpers::catch_jni_panic(&mut env, "{rust_export_name(method)}", {rust_default_return(method.returns)}, |env| -> crate::jni::helpers::JniCallResult<{rust_return_type(method.returns)}> {{',
            line_join(export_body, 8),
            "    })",
        ]
    lines += ["}", ""]
    if feature is not None:
        lines += [
            f'#[cfg(not(feature = "{feature}"))]',
            "#[allow(non_snake_case)]",
            "#[no_mangle]",
            f"pub extern \"system\" fn {rust_export_name(method)}<'local>(",
            line_join([f"{arg}," for arg in rust_method_signature(method)], 4),
            f") -> {rust_return_type(method.returns)} {{",
            f'    let _ = env.throw_new("java/lang/IllegalStateException", "This JNI export requires the `{feature}` feature.");',
        ]
        if base_type(method.returns) != "void":
            lines.append(f"    {rust_default_return(method.returns)}")
        lines += ["}", ""]
    return lines


def rust_source(methods: list[GeneratedMethod]) -> str:
    lines = [
        f"// {HEADER_COMMENT}",
        "",
        "#![allow(clippy::all)]",
        "#![allow(clippy::too_many_lines)]",
        "#![allow(missing_docs)]",
        "#![allow(non_snake_case)]",
        "#![allow(dead_code)]",
        "#![allow(private_interfaces)]",
        "#![allow(unused_imports)]",
        "#![allow(unused_mut)]",
        "#![allow(unused_variables)]",
        "",
        "use crate::ffi::*;",
        "use jni::objects::{JByteArray, JClass, JLongArray, JObject, JString};",
        "use jni::sys::{jboolean, jbyteArray, jdouble, jfloat, jint, jlong, jlongArray, jobject, jstring};",
        "",
    ]
    lines.extend(build_local_ffi_structs())
    lines.extend(build_type_helpers())
    for method in methods:
        # Skip methods with callback parameters — JNI cannot express them.
        if any(p["type"].startswith("callback") for p in method.params):
            continue
        lines.extend(rust_method_source(method))
    return "\n".join(lines)


def write_java_sources(methods: list[GeneratedMethod]) -> None:
    JAVA_DIR.mkdir(parents=True, exist_ok=True)
    for path in JAVA_DIR.glob("*.java"):
        path.unlink()
    for type_name in USED_TYPES:
        write_generated(JAVA_DIR / f"{type_name}.java", java_carrier_source(type_name))
    grouped: dict[str, list[GeneratedMethod]] = {}
    for method in methods:
        grouped.setdefault(method.class_name, []).append(method)
    for class_name, class_methods in grouped.items():
        write_generated(JAVA_DIR / f"{class_name}.java", java_native_source(class_name, class_methods))
    # SpriteCmd is not in schema types but is used as an array parameter in drawSpriteBatch.
    # Generate a minimal carrier so the test fixtures compile.
    if "SpriteCmd" not in USED_TYPES:
        _SPRITE_CMD_FIELDS = [
            ("long", "texture"), ("float", "x"), ("float", "y"),
            ("float", "width"), ("float", "height"), ("float", "rotation"),
            ("float", "srcX"), ("float", "srcY"), ("float", "srcW"), ("float", "srcH"),
            ("float", "r"), ("float", "g"), ("float", "b"), ("float", "a"),
            ("int", "zLayer"),
        ]
        sc_lines = [JAVA_HEADER, "package com.goudengine.internal;", ""]
        sc_lines.append("public final class SpriteCmd {")
        for jtype, jname in _SPRITE_CMD_FIELDS:
            sc_lines.append(f"    public {jtype} {jname};")
        sc_lines += ["", "    public SpriteCmd() {}", "}", ""]
        write_generated(JAVA_DIR / "SpriteCmd.java", "\n".join(sc_lines))
    write_generated(JAVA_DIR / "JniSmokeMain.java", smoke_java_source())


def main() -> int:
    methods = collect_generated_methods()
    write_generated(JNI_RS, rust_source(methods))
    write_java_sources(methods)
    print(f"Generated {JNI_RS}")
    print(f"Generated Java fixtures in {JAVA_DIR}")
    return 0


SCHEMA = load_schema()
MAPPING = load_ffi_mapping(SCHEMA)
USED_TYPES = used_carrier_types()


if __name__ == "__main__":
    raise SystemExit(main())
