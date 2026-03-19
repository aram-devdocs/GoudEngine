"""Generates tools.g.rs -- Lua tool method closures backed by crate-internal FFI calls."""

import sys
import os

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
import sdk_common

from .context import HEADER, FFI_EXTRACT

# Maximum number of user-facing parameters (mlua tuple limit)
MAX_TUPLE_PARAMS = 12

# FFI submodules that do NOT re-export their children's functions at the
# parent level.  For these, the codegen must use the full submodule path
# (e.g. `crate::ffi::audio::controls::goud_audio_stop`).  All other
# directory-based FFI modules (animation, renderer, entity, …) re-export
# via `pub use` in their mod.rs, so functions are reachable from the
# parent path (e.g. `crate::ffi::renderer::goud_renderer_begin`).
_NO_REEXPORT_MODULES = {"audio", "ui"}

# Mapping from FFI module prefix to Cargo feature gate.  Imports from
# these modules are wrapped in ``#[cfg(feature = "...")]``.  Modules not
# listed here are assumed to be available unconditionally (or gated by
# the outer ``#[cfg(feature = "native")]`` that wraps the entire tools
# module).
_MODULE_FEATURE_GATES = {
    "physics::physics2d": "rapier2d",
    "physics::physics3d": "rapier3d",
    "physics::physics2d_events": "rapier2d",
    "physics::physics2d_ex": "rapier2d",
    "physics::physics2d_material": "rapier2d",
    "physics::physics3d_material": "rapier3d",
}


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
    """Map a manifest param type to the Rust type used in the call."""
    ptype = sdk_common.normalize_ffi_type(ptype)
    return ptype


def _lua_arg_type(ptype):
    """Map an FFI param type to the mlua extraction type."""
    ptype = sdk_common.normalize_ffi_type(ptype)
    if ptype == "bool":
        return "bool"
    if ptype in ("f32", "f64"):
        return "f64"
    return "i64"


def _crate_path_for_fn(fn_name, manifest_entry):
    """Derive the crate-internal Rust path for an FFI function.

    Returns ``(use_path, feature_gate)`` where *use_path* is the ``use``
    import string and *feature_gate* is an optional Cargo feature name
    (or ``None``).
    """
    source = manifest_entry.get("source_file", "")
    if not source:
        return (f"crate::ffi::{fn_name}", None)

    # Strip .rs suffix and split into parts: ['ffi', 'renderer', 'lifecycle']
    parts = source.replace(".rs", "").split("/")
    if parts and parts[0] == "ffi":
        parts = parts[1:]

    if not parts:
        return (f"crate::ffi::{fn_name}", None)

    # Determine module path
    if len(parts) == 1:
        mod_path = parts[0]
    else:
        top_module = parts[0]
        if top_module in _NO_REEXPORT_MODULES:
            sub_parts = [p for p in parts if p != "mod"]
            mod_path = "::".join(sub_parts)
        else:
            mod_path = top_module

    # Determine feature gate
    feature = None
    # Check progressively shorter prefixes against the gate map
    for depth in range(len(parts), 0, -1):
        key = "::".join(parts[:depth])
        if key in _MODULE_FEATURE_GATES:
            feature = _MODULE_FEATURE_GATES[key]
            break

    return (f"crate::ffi::{mod_path}::{fn_name}", feature)


def generated_tool_names(schema, ffi_manifest):
    """Return (has_tools, [(tool_name, feature_or_none), ...])."""
    ffi_tools = schema.get("ffi_tools", {})
    tools_features = {}  # tool_name -> set of features
    for tool_name, tool_def in ffi_tools.items():
        methods = tool_def.get("methods", {})
        for method_name, method_def in methods.items():
            ffi_fn = method_def.get("ffi")
            if not ffi_fn:
                continue
            manifest_entry = ffi_manifest.get(ffi_fn)
            if _should_skip_method(method_def, manifest_entry):
                continue
            _, feature = _crate_path_for_fn(ffi_fn, manifest_entry)
            tools_features.setdefault(tool_name, set()).add(feature)

    result = []
    for tool_name, features in tools_features.items():
        # If all methods share the same non-None feature, the tool gets that gate
        if len(features) == 1:
            tool_feature = features.pop()
        else:
            tool_feature = None
        result.append((tool_name, tool_feature))

    return (bool(result), result)


def generate(schema, ffi_manifest):
    lines = [HEADER]
    lines.append("use mlua::prelude::*;")
    lines.append("use crate::core::types::GoudResult;")
    lines.append("")

    ffi_tools = schema.get("ffi_tools", {})

    all_fns = {}  # ffi_fn -> manifest_entry
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
            all_fns[ffi_fn] = manifest_entry
            all_methods.append((tool_name, method_name, ffi_fn, manifest_entry, method_def))

    if not all_methods:
        lines.append("// No tool methods generated (all skipped).")
        lines.append("")
        return "\n".join(lines)

    # Import GoudContextId for wrapping the raw u64 context handle.
    lines.append("use crate::core::context_id::GoudContextId;")
    lines.append("")

    # Generate `use` imports instead of extern "C" block.
    # Track feature gates per function for conditional registration.
    fn_features = {}  # ffi_fn -> feature (or None)
    for ffi_fn, entry in sorted(all_fns.items()):
        use_path, feature = _crate_path_for_fn(ffi_fn, entry)
        fn_features[ffi_fn] = feature
        last_sep = use_path.rfind("::")
        mod_path = use_path[:last_sep]
        if feature:
            lines.append(f'#[cfg(feature = "{feature}")]')
        lines.append(f"use {mod_path}::{ffi_fn};")

    lines.append("")

    # Group methods by tool
    tools_methods = {}
    for tool_name, method_name, ffi_fn, manifest_entry, method_def in all_methods:
        tools_methods.setdefault(tool_name, []).append(
            (method_name, ffi_fn, manifest_entry, method_def)
        )

    for tool_name, methods in tools_methods.items():
        snake_tool = sdk_common.to_snake(tool_name)

        # Determine if the entire registration function needs a feature gate.
        method_features = {fn_features.get(ffi_fn) for _, ffi_fn, _, _ in methods}
        # If all methods share the same non-None feature, gate the whole fn.
        tool_feature = None
        if len(method_features) == 1:
            tool_feature = method_features.pop()

        if tool_feature:
            lines.append(f'#[cfg(feature = "{tool_feature}")]')
        lines.append(f"pub(crate) fn register_{snake_tool}_tools(lua: &Lua, ctx_id: u64) -> LuaResult<()> {{")
        lines.append(f"    let ctx = GoudContextId::from_raw(ctx_id);")
        lines.append(f"    let globals = lua.globals();")
        lines.append(f'    let tbl = globals.get::<LuaTable>("{snake_tool}").or_else(|_| lua.create_table())?;')

        for method_name, ffi_fn, manifest_entry, method_def in methods:
            snake_method = sdk_common.to_snake(method_name)
            params = manifest_entry.get("params", [])
            ret = sdk_common.normalize_ffi_type(manifest_entry.get("return_type", "void"))

            # Gate individual methods if their feature differs from the tool's.
            method_feature = fn_features.get(ffi_fn)
            needs_individual_gate = (not tool_feature) and method_feature

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

            indent = "    "
            if needs_individual_gate:
                lines.append(f'    #[cfg(feature = "{method_feature}")]')
                lines.append(f"    {{")
                indent = "        "

            lines.append(f"{indent}// {tool_name}.{method_name}")
            lines.append(f"{indent}let f_{snake_method} = lua.create_function(move |_, {args_pat}: {args_type}| {{")

            # Build call args — pass `ctx` (GoudContextId) for context_id params
            call_args = []
            user_idx = 0
            for p in params:
                pname, ptype = _parse_manifest_param(p)
                ptype = sdk_common.normalize_ffi_type(ptype)
                if ptype == "GoudContextId" or pname == "context_id":
                    call_args.append("ctx")
                    continue
                rust_type = _rust_type_for_param(ptype)
                var = f"arg{user_idx}"
                lua_type = _lua_arg_type(ptype)
                if rust_type != lua_type:
                    call_args.append(f"{var} as {rust_type}")
                else:
                    call_args.append(var)
                user_idx += 1

            is_unsafe = manifest_entry.get("is_unsafe", False)
            call = f"{ffi_fn}({', '.join(call_args)})"

            if is_unsafe:
                lines.append(f"{indent}    // SAFETY: FFI call with validated primitive parameters from Lua.")

            if ret in ("void", "()"):
                if is_unsafe:
                    lines.append(f"{indent}    unsafe {{ {call} }};")
                else:
                    lines.append(f"{indent}    {call};")
                lines.append(f"{indent}    Ok(())")
            elif ret == "bool":
                if is_unsafe:
                    lines.append(f"{indent}    Ok(unsafe {{ {call} }})")
                else:
                    lines.append(f"{indent}    Ok({call})")
            elif ret in ("f32", "f64"):
                if is_unsafe:
                    lines.append(f"{indent}    Ok(unsafe {{ {call} }} as f64)")
                else:
                    lines.append(f"{indent}    Ok({call} as f64)")
            elif ret == "GoudResult":
                if is_unsafe:
                    lines.append(f"{indent}    let r = unsafe {{ {call} }};")
                else:
                    lines.append(f"{indent}    let r = {call};")
                lines.append(f"{indent}    Ok(r.code as i64)")
            else:
                if is_unsafe:
                    lines.append(f"{indent}    Ok(unsafe {{ {call} }} as i64)")
                else:
                    lines.append(f"{indent}    Ok({call} as i64)")

            lines.append(f"{indent}}})?;")
            lines.append(f'{indent}tbl.set("{snake_method}", f_{snake_method})?;')

            if needs_individual_gate:
                lines.append(f"    }}")

        lines.append(f'    globals.set("{snake_tool}", tbl)?;')
        lines.append(f"    Ok(())")
        lines.append(f"}}")
        lines.append("")

    return "\n".join(lines)
