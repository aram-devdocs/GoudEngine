"""Shared helper functions for C# code generation."""

from sdk_common import CSHARP_TYPES, CSHARP_FFI_TYPES, to_pascal, to_snake
from .context import schema, mapping, _CSHARP_KEYWORDS, _CSHARP_FFI_ALIASES

def cs_type(t: str) -> str:
    if t == "ptr":
        return "IntPtr"
    if t == "usize":
        return "nuint"
    return CSHARP_TYPES.get(t, to_pascal(t))


def ffi_type(t: str) -> str:
    return CSHARP_FFI_TYPES.get(t, t)


def _to_cs_field(snake: str) -> str:
    """Convert _snake_case to _camelCase for C# private fields.

    Examples: _delta_time -> _deltaTime, _title -> _title,
              _total_time -> _totalTime, _frame_count -> _frameCount
    """
    parts = snake.lstrip('_').split('_')
    return '_' + parts[0] + ''.join(p.capitalize() for p in parts[1:])

def _cs_identifier(name: str) -> str:
    if name in _CSHARP_KEYWORDS:
        return f"@{name}"
    return name


def _cs_default_value(cs_ty: str) -> str:
    """Return the C# default literal for a given C# type."""
    if cs_ty == "string":
        return '""'
    if cs_ty == "double":
        return "0.0"
    if cs_ty == "float":
        return "0.0f"
    if cs_ty in ("uint", "int", "ulong", "long", "byte", "ushort", "short", "sbyte"):
        return "0"
    if cs_ty == "bool":
        return "false"
    return "default"


def _cs_out_var_type(raw_type: str) -> str:
    """Resolve an out-param type to the C# declaration type."""
    if raw_type in CSHARP_TYPES or raw_type in ("ptr", "usize"):
        return cs_type(raw_type)
    if raw_type in schema.get("enums", {}):
        return cs_type(raw_type)
    ffi_info = mapping.get("ffi_types", {}).get(raw_type)
    if isinstance(ffi_info, dict) and ffi_info.get("ffi_name"):
        return ffi_info["ffi_name"]
    return ffi_type(raw_type)


def _cs_value_param_ffi_setup(raw_type: str, param_name: str) -> tuple[list[str], str] | None:
    """Marshal a schema value type into its FFI struct form for tool calls."""
    type_def = schema.get("types", {}).get(raw_type, {})
    ffi_info = mapping.get("ffi_types", {}).get(raw_type, {})
    ffi_name = ffi_info.get("ffi_name")
    if type_def.get("kind") != "value" or not ffi_name:
        return None

    local_name = f"_{param_name}Ffi"
    lines = [f"            var {local_name} = new {ffi_name}", "            {"]
    fields = type_def.get("fields", [])
    for idx, field in enumerate(fields):
        field_name = to_pascal(field["name"])
        comma = "," if idx < len(fields) - 1 else ""
        lines.append(
            f"                {field_name} = {param_name}.{field_name}{comma}"
        )
    lines.append("            };")
    return lines, local_name


def _normalize_manifest_ffi_type(raw_type: str) -> str:
    t = raw_type.strip()
    if t == "()":
        return "void"
    if t.startswith("Option<") and t.endswith(">"):
        inner = t[len("Option<"):-1]
        return f"Option<{_normalize_manifest_ffi_type(inner)}>"
    if t.startswith("*mut ") or t.startswith("*const "):
        qualifier, inner = t.split(" ", 1)
        return f"{qualifier} {_normalize_manifest_ffi_type(inner)}"
    if "::" in t:
        t = t.split("::")[-1]
    if t in _CSHARP_FFI_ALIASES:
        return _normalize_manifest_ffi_type(_CSHARP_FFI_ALIASES[t])
    return t


def _cs_ffi_param_type(raw: str) -> str:
    """Convert an ffi_mapping param type to valid C# for NativeMethods."""
    def _ffi_struct_name(type_name: str) -> str:
        ffi_info = mapping.get("ffi_types", {}).get(type_name, {})
        return ffi_info.get("ffi_name", type_name)

    raw = _normalize_manifest_ffi_type(raw)
    ptr_map = {
        "*mut f32": "ref float",
        "*mut i64": "ref long",
        "*mut i32": "ref int",
        "*mut u32": "ref uint",
        "*mut u64": "ref ulong",
        "*const u64": "IntPtr",
        "*mut FfiTransform2D": "ref FfiTransform2D",
        "*const FfiTransform2D": "ref FfiTransform2D",
        "*mut FfiSprite": "ref FfiSprite",
        "*const FfiSprite": "ref FfiSprite",
        "*mut FfiTransform2DBuilder": "IntPtr",
        "*mut FfiSpriteBuilder": "IntPtr",
        "*mut FfiAnimationClipBuilder": "IntPtr",
        "*const FfiSpriteAnimator": "ref FfiSpriteAnimator",
        "*mut FfiText": "ref FfiText",
        "*const FfiText": "ref FfiText",
        "FfiPlaybackMode": "PlaybackMode",
        "FfiTransitionType": "TransitionType",
        "*const FfiUiStyle": "ref FfiUiStyle",
        "*mut FfiUiEvent": "ref FfiUiEvent",
        "*const u8": "IntPtr",
        "*mut u8": "IntPtr",
        "*mut *const u8": "ref IntPtr",
        "ptr": "IntPtr",
        "*mut c_void": "IntPtr",
        "*const c_char": "string",
        "usize": "nuint",
        "u8": "byte",
    }
    if raw in ptr_map:
        return ptr_map[raw]
    if raw.startswith("Goud") and raw[4:] in schema.get("enums", {}):
        return to_pascal(raw[4:])
    if raw.startswith("Option<") and raw.endswith(">"):
        inner = raw[len("Option<"):-1]
        if "Callback" in inner:
            return "IntPtr"
    if raw.startswith("*mut ") or raw.startswith("*const "):
        qualifier, inner = raw.split(" ", 1)
        ffi_inner = _ffi_struct_name(inner.strip())
        if ffi_inner.startswith("Ffi"):
            return f"ref {ffi_inner}"
        if ffi_inner.startswith("Goud") and ffi_inner not in ("GoudTextureHandle", "GoudFontHandle"):
            return f"ref {ffi_inner}"
        if ffi_inner == "c_char" and qualifier == "*const":
            return "string"
        if ffi_inner in ("UiManager",):
            return "IntPtr"
        return "IntPtr"
    ffi_info = mapping.get("ffi_types", {}).get(raw, {})
    if ffi_info.get("ffi_name"):
        return ffi_info["ffi_name"]
    return ffi_type(raw)


def _cs_ffi_ret_type(raw: str) -> str:
    """Convert a return type to valid C#."""
    raw = _normalize_manifest_ffi_type(raw)
    ret_map = {
        "*mut FfiTransform2DBuilder": "IntPtr",
        "*mut FfiSpriteBuilder": "IntPtr",
        "*mut FfiAnimationClipBuilder": "IntPtr",
        "*const u8": "IntPtr",
        "*mut u8": "IntPtr",
        "*mut c_void": "IntPtr",
        "usize": "nuint",
    }
    if raw in ret_map:
        return ret_map[raw]
    if raw.startswith("Goud") and raw[4:] in schema.get("enums", {}):
        return to_pascal(raw[4:])
    if raw.startswith("*mut ") or raw.startswith("*const "):
        return "IntPtr"
    if raw.startswith("Option<") and raw.endswith(">") and "Callback" in raw:
        return "IntPtr"
    ffi_info = mapping.get("ffi_types", {}).get(raw, {})
    if ffi_info.get("ffi_name"):
        return ffi_info["ffi_name"]
    return ffi_type(raw)


def _ffi_return_type(ffi_fn_name: str) -> str:
    """Return the FFI return type string for a named function."""
    for _mod, funcs in mapping["ffi_functions"].items():
        if isinstance(funcs, dict) and ffi_fn_name in funcs:
            return _normalize_manifest_ffi_type(funcs[ffi_fn_name].get("returns", "void"))
    return "void"


def _ffi_fn_def(ffi_fn_name: str) -> dict:
    """Look up the full FFI function definition by name."""
    for _mod, funcs in mapping["ffi_functions"].items():
        if isinstance(funcs, dict) and ffi_fn_name in funcs:
            return funcs[ffi_fn_name]
    return {}


def _ffi_uses_ptr_len(ffi_fn_name: str) -> bool:
    """Check if the FFI function uses *const u8 ptr+len for string params."""
    fdef = _ffi_fn_def(ffi_fn_name)
    param_types = [p.get("type", "") for p in fdef.get("params", [])]
    return "*const u8" in param_types


def _ffi_param_type_at(ffi_fn_name: str, index: int) -> str:
    """Return the raw ffi_mapping type for a function parameter index."""
    fdef = _ffi_fn_def(ffi_fn_name)
    params = fdef.get("params", [])
    if 0 <= index < len(params):
        return params[index].get("type", "")
    return ""


def _cs_len_cast_expr(ffi_param_type: str, expr: str) -> str:
    """Cast a managed byte-length expression to the expected FFI integer type."""
    cast_map = {
        "i8": "sbyte",
        "u8": "byte",
        "i16": "short",
        "u16": "ushort",
        "i32": "int",
        "u32": "uint",
        "i64": "long",
        "u64": "ulong",
        "usize": "nuint",
    }
    cast = cast_map.get(ffi_param_type, "uint")
    return f"({cast}){expr}"


def _enum_cs_name(key: str) -> str:
    if key == "Key":
        return "Keys"
    if key == "MouseButton":
        return "MouseButtons"
    return to_pascal(key)


def _type_hash(type_name: str) -> str:
    """FNV-1a 64-bit hash for component type discrimination."""
    h = 0xcbf29ce484222325
    for b in type_name.encode("utf-8"):
        h ^= b
        h = (h * 0x100000001b3) & 0xFFFFFFFFFFFFFFFF
    return f"0x{h:016x}UL"


# ── NativeMethods.g.cs ──────────────────────────────────────────────
