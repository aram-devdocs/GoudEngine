"""Generates tools.g.rs -- Lua tool method closures backed by extern C FFI."""

import sys
import os

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
import sdk_common

from .context import HEADER, FFI_EXTRACT

# Maximum number of user-facing parameters (mlua tuple limit)
MAX_TUPLE_PARAMS = 12


def _parse_manifest_param(raw):
    """Parse 'name: Type' -> (name, type_str)."""
    if isinstance(raw, dict):
        return raw["name"], raw.get("type", "")
    name, _, ty = raw.partition(":")
    return name.strip(), ty.strip()


def _should_skip_method(method_def, manifest_entry):
    """Return True if we should skip code generation for this method."""
    if method_def.get("string_params"):
        return True
    if method_def.get("expand_params"):
        return True
    if method_def.get("out_params"):
        return True
    if method_def.get("batch_op"):
        return True
    if method_def.get("ffi_strategy"):
        return True
    if method_def.get("append_args"):
        return True

    if manifest_entry is None:
        return True

    user_params = []
    for p in manifest_entry.get("params", []):
        pname, ptype = _parse_manifest_param(p)
        ptype = sdk_common.normalize_ffi_type(ptype)
        if ptype == "GoudContextId" or pname == "context_id":
            continue
        user_params.append((pname, ptype))

    if len(user_params) > MAX_TUPLE_PARAMS:
        return True

    for pname, ptype in user_params:
        if ptype.startswith("*"):
            return True
        if ptype.startswith("Option<"):
            return True
        if ptype in ("void",):
            return True
        if "Callback" in ptype:
            return True
        if ptype not in (
            "f32", "f64",
            "u8", "u16", "u32", "u64",
            "i8", "i16", "i32", "i64",
            "bool",
        ):
            return True

    ret = sdk_common.normalize_ffi_type(manifest_entry.get("return_type", "void"))
    if ret.startswith("*"):
        return True
    allowed_returns = {
        "void", "()", "bool",
        "f32", "f64",
        "u8", "u16", "u32", "u64",
        "i8", "i16", "i32", "i64",
        "GoudResult", "GoudTextureHandle",
    }
    if ret not in allowed_returns:
        return True

    return False


def _rust_type_for_param(ptype):
    """Map a manifest param type to the Rust type used in extern C decl."""
    ptype = sdk_common.normalize_ffi_type(ptype)
    if ptype == "GoudContextId":
        return "u64"
    return ptype


def _lua_arg_type(ptype):
    """Map an FFI param type to the mlua extraction type."""
    ptype = sdk_common.normalize_ffi_type(ptype)
    if ptype == "bool":
        return "bool"
    if ptype in ("f32", "f64"):
        return "f64"
    return "i64"


def generated_tool_names(schema, ffi_manifest):
    """Return (has_tools, tool_names) for the tools that would be generated."""
    ffi_tools = schema.get("ffi_tools", {})
    tools_methods = {}
    for tool_name, tool_def in ffi_tools.items():
        methods = tool_def.get("methods", {})
        for method_name, method_def in methods.items():
            ffi_fn = method_def.get("ffi")
            if not ffi_fn:
                continue
            manifest_entry = ffi_manifest.get(ffi_fn)
            if _should_skip_method(method_def, manifest_entry):
                continue
            tools_methods.setdefault(tool_name, []).append(method_name)
    tool_names = list(tools_methods.keys())
    return (bool(tool_names), tool_names)


def generate(schema, ffi_manifest):
    lines = [HEADER]
    lines.append("use mlua::prelude::*;")
    lines.append("use crate::core::types::GoudResult;")
    lines.append("")

    ffi_tools = schema.get("ffi_tools", {})

    all_externs = {}
    all_methods = []

    for tool_name, tool_def in ffi_tools.items():
        methods = tool_def.get("methods", {})
        for method_name, method_def in methods.items():
            ffi_fn = method_def.get("ffi")
            if not ffi_fn:
                continue
            manifest_entry = ffi_manifest.get(ffi_fn)
            if _should_skip_method(method_def, manifest_entry):
                continue
            all_externs[ffi_fn] = manifest_entry
            all_methods.append((tool_name, method_name, ffi_fn, manifest_entry, method_def))

    if not all_methods:
        lines.append("// No tool methods generated (all skipped).")
        lines.append("")
        return "\n".join(lines)

    # Generate extern C block
    lines.append("extern \"C\" {")
    for ffi_fn, entry in sorted(all_externs.items()):
        params = entry.get("params", [])
        ret = sdk_common.normalize_ffi_type(entry.get("return_type", "void"))

        param_strs = []
        for p in params:
            pname, ptype = _parse_manifest_param(p)
            ptype = sdk_common.normalize_ffi_type(ptype)
            rust_type = _rust_type_for_param(ptype)
            param_strs.append(f"{pname}: {rust_type}")

        ret_str = ""
        if ret not in ("void", "()"):
            ret_str = f" -> {ret}"

        lines.append(f"    fn {ffi_fn}({', '.join(param_strs)}){ret_str};")
    lines.append("}")
    lines.append("")

    # Group methods by tool
    tools_methods = {}
    for tool_name, method_name, ffi_fn, manifest_entry, method_def in all_methods:
        tools_methods.setdefault(tool_name, []).append(
            (method_name, ffi_fn, manifest_entry, method_def)
        )

    for tool_name, methods in tools_methods.items():
        snake_tool = sdk_common.to_snake(tool_name)
        lines.append(f"pub(crate) fn register_{snake_tool}_tools(lua: &Lua, ctx_id: u64) -> LuaResult<()> {{")
        lines.append(f"    let globals = lua.globals();")
        lines.append(f'    let tbl = globals.get::<LuaTable>("{snake_tool}").or_else(|_| lua.create_table())?;')

        for method_name, ffi_fn, manifest_entry, method_def in methods:
            snake_method = sdk_common.to_snake(method_name)
            params = manifest_entry.get("params", [])
            ret = sdk_common.normalize_ffi_type(manifest_entry.get("return_type", "void"))

            user_params = []
            for p in params:
                pname, ptype = _parse_manifest_param(p)
                ptype = sdk_common.normalize_ffi_type(ptype)
                if ptype == "GoudContextId" or pname == "context_id":
                    continue
                user_params.append((pname, ptype))

            if len(user_params) == 0:
                args_type = "()"
                args_pat = "_"
            elif len(user_params) == 1:
                _, pt = user_params[0]
                lua_t = _lua_arg_type(pt)
                args_type = lua_t
                args_pat = "arg0"
            else:
                lua_types = [_lua_arg_type(pt) for _, pt in user_params]
                args_type = f"({', '.join(lua_types)})"
                args_pat = f"({', '.join(f'arg{i}' for i in range(len(user_params)))})"

            lines.append(f"    // {tool_name}.{method_name}")
            lines.append(f"    let f_{snake_method} = lua.create_function(move |_, {args_pat}: {args_type}| {{")

            # Build call args
            call_args = []
            user_idx = 0
            for p in params:
                pname, ptype = _parse_manifest_param(p)
                ptype = sdk_common.normalize_ffi_type(ptype)
                if ptype == "GoudContextId" or pname == "context_id":
                    call_args.append("ctx_id")
                    continue
                rust_type = _rust_type_for_param(ptype)
                var = f"arg{user_idx}"
                lua_type = _lua_arg_type(ptype)
                if rust_type != lua_type:
                    call_args.append(f"{var} as {rust_type}")
                else:
                    call_args.append(var)
                user_idx += 1

            call = f"{ffi_fn}({', '.join(call_args)})"
            # All extern C calls require unsafe
            lines.append(f"        // SAFETY: FFI call with validated primitive parameters from Lua.")

            if ret in ("void", "()"):
                lines.append(f"        unsafe {{ {call} }};")
                lines.append(f"        Ok(())")
            elif ret == "bool":
                lines.append(f"        Ok(unsafe {{ {call} }})")
            elif ret in ("f32", "f64"):
                lines.append(f"        Ok(unsafe {{ {call} }} as f64)")
            elif ret == "GoudResult":
                lines.append(f"        let r = unsafe {{ {call} }};")
                lines.append(f"        Ok(r.code as i64)")
            else:
                lines.append(f"        Ok(unsafe {{ {call} }} as i64)")

            lines.append(f"    }})?;")
            lines.append(f'    tbl.set("{snake_method}", f_{snake_method})?;')

        lines.append(f'    globals.set("{snake_tool}", tbl)?;')
        lines.append(f"    Ok(())")
        lines.append(f"}}")
        lines.append("")

    return "\n".join(lines)
