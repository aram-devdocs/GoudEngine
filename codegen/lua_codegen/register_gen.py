"""Generates register.g.rs -- top-level Lua binding registration."""

import sys
import os

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
import sdk_common

from .context import HEADER


def generate(schema: dict, has_tools: bool, tool_names_with_features: list) -> str:
    """Generate register.g.rs.

    *tool_names_with_features* is a list of ``(tool_name, feature_or_none)``
    tuples returned by ``tools_gen.generated_tool_names()``.
    """
    lines = [HEADER]
    lines.append("use mlua::prelude::*;")
    lines.append("")
    lines.append("/// Registers all Lua bindings (types, enums, tools) on the given Lua state.")
    ctx_param = "ctx_id" if (has_tools and tool_names_with_features) else "_ctx_id"
    lines.append(f"pub(crate) fn register_lua_bindings(lua: &Lua, {ctx_param}: u64) -> LuaResult<()> {{")
    lines.append("    super::types::register_type_factories(lua)?;")
    lines.append("    super::enums::register_enum_constants(lua)?;")

    if has_tools and tool_names_with_features:
        lines.append("    #[cfg(feature = \"native\")]")
        lines.append("    {")
        for tool_name, feature in tool_names_with_features:
            snake = sdk_common.to_snake(tool_name)
            if feature:
                lines.append(f"        #[cfg(feature = \"{feature}\")]")
            lines.append(f"        super::tools::register_{snake}_tools(lua, ctx_id)?;")
        lines.append("    }")

    # Suppress unused variable warning when native feature is off
    if has_tools and tool_names_with_features:
        lines.append("    #[cfg(not(feature = \"native\"))]")
        lines.append("    let _ = ctx_id;")

    lines.append("    Ok(())")
    lines.append("}")
    lines.append("")
    return "\n".join(lines)
