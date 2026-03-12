"""Shared helper functions for Python code generation modules."""

from __future__ import annotations

from .context import CTYPES_MAP, PYTHON_TYPES, mapping, resolve_ctypes_type, schema, to_snake


def resolve_ffi_return(ret: str) -> str:
    """Map an FFI return type string to its ctypes restype."""
    if ret.startswith("*const ") or ret.startswith("*mut "):
        return resolve_ffi_pointer(ret)
    return resolve_ctypes_type(ret, enums=schema.get("enums", {}), default="ctypes.c_uint64")


def resolve_ffi_pointer(pointer_type: str) -> str:
    """Resolve Rust pointer types for Python ctypes signatures."""
    direct = CTYPES_MAP.get(pointer_type)
    if direct:
        return direct

    qualifier, inner = pointer_type.split(" ", 1)
    inner = inner.strip()

    if inner == "c_void":
        return "ctypes.c_void_p"
    if inner == "c_char" and qualifier == "*const":
        return "ctypes.c_char_p"

    ffi_info = mapping.get("ffi_types", {}).get(inner, {})
    ffi_name = ffi_info.get("ffi_name")
    if ffi_name:
        return f"ctypes.POINTER({ffi_name})"

    if inner in schema.get("enums", {}):
        underlying = schema["enums"][inner].get("underlying", "i32")
        underlying_ctypes = CTYPES_MAP.get(underlying, "ctypes.c_int32")
        return f"ctypes.POINTER({underlying_ctypes})"

    inner_direct = CTYPES_MAP.get(inner)
    if inner_direct:
        return f"ctypes.POINTER({inner_direct})"

    if inner.startswith("Ffi") or inner.startswith("Goud"):
        return f"ctypes.POINTER({inner})"

    # Opaque handle pointers should stay untyped in Python.
    return "ctypes.c_void_p"


def resolve_ffi_param(ptype: str) -> str:
    """Map an FFI param type string to its ctypes argtype."""
    if ptype in CTYPES_MAP:
        return CTYPES_MAP[ptype]

    # Handle ref types (out-pointers): "ref FfiRenderCapabilities" -> POINTER(FfiRenderCapabilities)
    if ptype.startswith("ref "):
        inner = ptype[4:].strip()
        ffi_info = mapping.get("ffi_types", {}).get(inner, {})
        ffi_name = ffi_info.get("ffi_name", inner)
        if ffi_name.startswith("Ffi") or ffi_name.startswith("Goud"):
            return f"ctypes.POINTER({ffi_name})"
        return "ctypes.c_void_p"

    if ptype.startswith("*const ") or ptype.startswith("*mut "):
        return resolve_ffi_pointer(ptype)

    resolved = resolve_ctypes_type(ptype, enums=schema.get("enums", {}), default="ctypes.c_uint64")
    if resolved != "ctypes.c_uint64":
        return resolved
    if ptype.startswith("Ffi") and ptype[3:] in schema.get("types", {}):
        return ptype
    return resolved


def py_value_param_ffi_setup(param: dict) -> tuple[list[str], str] | None:
    """Marshal a schema value type into its ctypes struct form for tool calls."""
    raw_type = param["type"]
    type_def = schema.get("types", {}).get(raw_type, {})
    ffi_info = mapping.get("ffi_types", {}).get(raw_type, {})
    ffi_name = ffi_info.get("ffi_name")
    if type_def.get("kind") != "value" or not ffi_name:
        return None

    param_name = to_snake(param["name"])
    local_name = f"_{param_name}_ffi"
    lines = [f"        {local_name} = _ffi_module.{ffi_name}()"]

    def emit_value_assignments(target_expr: str, value_expr: str, schema_type: str) -> None:
        nested_def = schema.get("types", {}).get(schema_type, {})
        for field in nested_def.get("fields", []):
            field_name = to_snake(field["name"])
            field_type = field.get("type", "f32")
            source_expr = f"{value_expr}.{field_name}"
            target_field = f"{target_expr}.{field_name}"
            if field_type == "string":
                lines.append(f"        {target_field} = {source_expr}.encode('utf-8')")
            elif (
                field_type in schema.get("types", {})
                and schema["types"][field_type].get("kind") == "value"
                and field_type in mapping.get("ffi_types", {})
            ):
                emit_value_assignments(target_field, source_expr, field_type)
            else:
                lines.append(f"        {target_field} = {source_expr}")

    emit_value_assignments(local_name, param_name, raw_type)
    return lines, local_name


def py_field_default(field: dict) -> str:
    """Return the Python type annotation and default value for a schema field."""
    return f"{py_field_type(field)} = {py_field_default_expr(field)}"


def py_field_type(field: dict) -> str:
    """Return the Python annotation type for a schema field."""
    t = field.get("type", "f32")
    if t == "bool":
        return "bool"
    if t == "string":
        return "str"
    if t in ("bytes", "u8[]"):
        return "bytes"
    if t in ("u8", "u16", "u32", "i8", "i16", "i32", "u64", "i64", "usize", "ptr"):
        return "int"
    if t in schema.get("types", {}):
        return f"'{t}'"
    return "float"


def py_field_default_expr(field: dict) -> str:
    """Return a raw Python default expression for a schema field."""
    t = field.get("type", "f32")
    if t == "bool":
        return "False"
    if t == "string":
        return '""'
    if t in ("bytes", "u8[]"):
        return "b''"
    if t in ("u8", "u16", "u32", "i8", "i16", "i32", "u64", "i64", "usize", "ptr"):
        return "0"
    if t in schema.get("types", {}):
        return "None"
    return "0.0"


def get_ffi_func_def(ffi_name: str) -> dict | None:
    """Look up an FFI function definition from the mapping by name."""
    for _module, funcs in mapping["ffi_functions"].items():
        if not isinstance(funcs, dict):
            continue
        if ffi_name in funcs:
            return funcs[ffi_name]
    return None


def ffi_uses_ptr_len(ffi_name: str) -> bool:
    """Check if the FFI function uses *const u8 ptr+len for string params."""
    fdef = get_ffi_func_def(ffi_name)
    if not fdef:
        return False
    param_types = [p.get("type", "") for p in fdef.get("params", [])]
    return "*const u8" in param_types


def py_out_var_ctype(raw_type: str) -> str:
    """Resolve an out-param type to the ctypes declaration type."""
    if raw_type in schema.get("enums", {}):
        underlying = schema["enums"][raw_type].get("underlying", "i32")
        return CTYPES_MAP.get(underlying, "ctypes.c_int32")
    if raw_type in schema.get("types", {}):
        ffi_info = mapping.get("ffi_types", {}).get(raw_type, {})
        ffi_name = ffi_info.get("ffi_name")
        if ffi_name:
            return ffi_name
    if raw_type.startswith("Ffi") or raw_type.startswith("Goud"):
        return raw_type
    return CTYPES_MAP.get(raw_type, "ctypes.c_float")


def ffi_to_sdk_return(ffi_returns: str, type_name: str) -> str:
    """Map an FFI return type to the SDK type string for annotations."""
    if ffi_returns == "void":
        return "None"
    if ffi_returns == "f32":
        return "float"
    if ffi_returns == "u32" or ffi_returns == "u64":
        return "int"
    if ffi_returns == "bool":
        return "bool"
    if ffi_returns == "FfiVec2":
        return "Vec2"
    if ffi_returns == "FfiColor":
        return "Color"
    if ffi_returns == "FfiRect":
        return "Rect"
    if ffi_returns == "FfiMat3x3":
        return "Mat3x3"
    if ffi_returns == "FfiTransform2D":
        return "'Transform2D'"
    if ffi_returns == "FfiSprite":
        return "'Sprite'"
    return f"'{type_name}'"


def py_schema_return_type(ret_type: str, self_type: str) -> str:
    """Map schema return type to Python annotation string."""
    if not ret_type:
        return "None"
    if ret_type.endswith("?"):
        ret_type = ret_type[:-1]
    if ret_type == "void":
        return "None"
    if ret_type in ("f32", "f64"):
        return "float"
    if ret_type in ("u8", "u16", "u32", "i8", "i16", "i32", "u64", "i64", "usize", "ptr"):
        return "int"
    if ret_type == "bool":
        return "bool"
    if ret_type == "string":
        return "str"
    if ret_type in schema.get("types", {}):
        return f"'{ret_type}'" if ret_type == self_type else ret_type
    return PYTHON_TYPES.get(ret_type, ret_type)
