"""Generates enums.g.rs -- Lua enum constant tables."""

import sys
import os

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
import sdk_common

from .context import HEADER


def generate(schema: dict) -> str:
    lines = [HEADER]
    lines.append("use mlua::prelude::*;\n")
    lines.append("pub(crate) fn register_enum_constants(lua: &Lua) {")
    lines.append("    let globals = lua.globals();")

    enums = schema.get("enums", {})
    for enum_name, enum_def in enums.items():
        snake = sdk_common.to_snake(enum_name)
        lines.append(f"    // {enum_name}")
        lines.append(f"    let tbl = lua.create_table().unwrap();")
        values = enum_def.get("values", {})
        if isinstance(values, dict):
            for vname, vval in values.items():
                lines.append(f'    tbl.set("{sdk_common.to_snake(vname)}", {vval}_i64).unwrap();')
        lines.append(f'    globals.set("{snake}", tbl).unwrap();')

    lines.append("}")
    lines.append("")
    return "\n".join(lines)
