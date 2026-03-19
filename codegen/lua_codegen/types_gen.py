"""Generates types.g.rs -- Lua UserData wrappers for FFI struct types."""

import sys
import os

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
import sdk_common

from .context import (
    HEADER,
    IMPORTABLE_FFI_TYPES,
    SKIP_TYPE_WRAPPERS,
)


def _field_rust_type(ffi_field_name: str, schema_field_type_map: dict) -> str:
    """Determine the Rust type for an FFI struct field."""
    if ffi_field_name in schema_field_type_map:
        return schema_field_type_map[ffi_field_name]
    if ffi_field_name.startswith("has_") or ffi_field_name.startswith("flip_"):
        return "bool"
    if ffi_field_name.endswith("_handle"):
        return "u64"
    if ffi_field_name == "z_layer":
        return "i32"
    if ffi_field_name in ("alignment",):
        return "u8"
    if ffi_field_name in ("direction",):
        return "i32"
    return "f32"


def _zero_value(rust_type: str) -> str:
    """Return the Rust zero literal for a type."""
    if rust_type == "bool":
        return "false"
    if rust_type in ("f32", "f64"):
        return "0.0"
    return "0"


def _lua_getter_expr(field_name: str, rust_type: str) -> str:
    if rust_type == "bool":
        return f"this.0.{field_name}"
    if rust_type in ("f32", "f64"):
        return f"this.0.{field_name} as f64"
    return f"this.0.{field_name} as i64"


def _lua_setter_stmt(field_name: str, rust_type: str) -> str:
    if rust_type == "bool":
        return f"this.0.{field_name} = val;"
    if rust_type == "f32":
        return f"this.0.{field_name} = val as f32;"
    if rust_type == "f64":
        return f"this.0.{field_name} = val;"
    return f"this.0.{field_name} = val as {rust_type};"


def _setter_lua_type(rust_type: str) -> str:
    if rust_type == "bool":
        return "bool"
    if rust_type in ("f32", "f64"):
        return "f64"
    return "i64"


def _ctor_extract(field_name: str, rust_type: str) -> str:
    """Generate the table-get line for a constructor."""
    if rust_type == "bool":
        return f'        if let Ok(val) = tbl.get::<bool>("{field_name}") {{ v.{field_name} = val; }}'
    if rust_type == "f32":
        return f'        if let Ok(val) = tbl.get::<f64>("{field_name}") {{ v.{field_name} = val as f32; }}'
    if rust_type == "f64":
        return f'        if let Ok(val) = tbl.get::<f64>("{field_name}") {{ v.{field_name} = val; }}'
    if rust_type == "u8":
        return f'        if let Ok(val) = tbl.get::<i64>("{field_name}") {{ v.{field_name} = val as u8; }}'
    return f'        if let Ok(val) = tbl.get::<i64>("{field_name}") {{ v.{field_name} = val as {rust_type}; }}'


def generate(schema: dict, ffi_mapping: dict) -> str:
    lines = [HEADER]
    lines.append("use mlua::prelude::*;")
    lines.append("use crate::core::types::*;")
    lines.append("use crate::core::context_id::GoudContextId;")
    lines.append("")

    ffi_types = ffi_mapping.get("ffi_types", {})
    schema_types = schema.get("types", {})

    gen_types = []
    for type_name, type_def in schema_types.items():
        if type_name in SKIP_TYPE_WRAPPERS:
            continue
        ffi_info = ffi_types.get(type_name)
        if ffi_info is None:
            continue
        ffi_name = ffi_info.get("ffi_name", "")
        if ffi_name not in IMPORTABLE_FFI_TYPES:
            continue
        fields = ffi_info.get("fields")
        if fields is None:
            continue
        gen_types.append((type_name, type_def, ffi_info))

    # Build schema field type map per type
    def _schema_ftm(type_def):
        m = {}
        for sf in type_def.get("fields", []):
            t = sf.get("type", "f32")
            if t in ("f32", "f64", "u8", "u16", "u32", "u64", "i8", "i16", "i32", "i64", "bool"):
                m[sf["name"]] = t
        return m

    for type_name, type_def, ffi_info in gen_types:
        ffi_name = ffi_info["ffi_name"]
        ffi_fields = ffi_info["fields"]
        ftm = _schema_ftm(type_def)

        lines.append(f"/// Lua wrapper for `{ffi_name}`.")
        lines.append(f"#[derive(Clone)]")
        lines.append(f"pub(crate) struct Lua{type_name}(pub(crate) {ffi_name});")
        lines.append("")
        lines.append(f"impl LuaUserData for Lua{type_name} {{")
        lines.append(f"    fn add_fields<F: LuaUserDataFields<Self>>(fields: &mut F) {{")
        for fname in ffi_fields:
            rt = _field_rust_type(fname, ftm)
            getter = _lua_getter_expr(fname, rt)
            setter = _lua_setter_stmt(fname, rt)
            stype = _setter_lua_type(rt)
            lines.append(f'        fields.add_field_method_get("{fname}", |_, this| Ok({getter}));')
            lines.append(f'        fields.add_field_method_set("{fname}", |_, this, val: {stype}| {{ {setter} Ok(()) }});')
        lines.append(f"    }}")
        lines.append(f"}}")
        lines.append("")

    # Factory registration
    lines.append("pub(crate) fn register_type_factories(lua: &Lua) -> LuaResult<()> {")
    lines.append("    let globals = lua.globals();")
    for type_name, type_def, ffi_info in gen_types:
        ffi_name = ffi_info["ffi_name"]
        ffi_fields = ffi_info["fields"]
        ftm = _schema_ftm(type_def)

        lines.append(f"    // {type_name} constructor")
        lines.append(f"    let ctor = lua.create_function(|_, tbl: LuaTable| {{")
        # Use zeroed initialization via unsafe to avoid needing Default
        lines.append(f"        // SAFETY: {ffi_name} is repr(C) with only primitive fields;")
        lines.append(f"        // zeroed memory is valid for all its field types.")
        lines.append(f"        let mut v: {ffi_name} = unsafe {{ std::mem::zeroed() }};")
        for fname in ffi_fields:
            rt = _field_rust_type(fname, ftm)
            lines.append(_ctor_extract(fname, rt))
        lines.append(f"        Ok(Lua{type_name}(v))")
        lines.append(f"    }})?;")
        lines.append(f'    globals.set("{type_name}", ctor)?;')
    lines.append("    Ok(())")
    lines.append("}")
    lines.append("")

    return "\n".join(lines)
