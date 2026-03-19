"""Generator for Network.g.swift and other sub-tool classes."""

from .context import OUT, schema, write_generated
from .shared_helpers import (
    swift_file_header,
    swift_type,
    swift_literal,
    to_camel,
    method_exists_in_ffi,
    ffi_func_name,
    convert_param_to_ffi,
    convert_return_from_ffi,
)
from .game_gen import _gen_method


_SUB_TOOLS = [
    "AnimationController",
    "Tween",
    "Skeleton",
    "AnimationEvents",
    "AnimationLayerStack",
    "Network",
    "Plugin",
    "Audio",
]


def gen_network() -> None:
    lines = [swift_file_header(), "import Foundation", "import CGoudEngine", ""]

    for tool_name in _SUB_TOOLS:
        if tool_name not in schema.get("tools", {}):
            continue
        _gen_sub_tool(lines, tool_name)

    write_generated(OUT / "SubTools.g.swift", "\n".join(lines))


def _gen_sub_tool(lines: list[str], tool_name: str) -> None:
    tool_def = schema["tools"][tool_name]
    doc = tool_def.get("doc", "")
    methods = tool_def.get("methods", [])

    if doc:
        lines.append(f"/// {doc}")
    lines.append(f"public final class {tool_name} {{")
    lines.append("    internal let _ctx: GoudContextId")
    lines.append("")
    lines.append(f"    public init(ctx: GoudContextId) {{")
    lines.append("        self._ctx = ctx")
    lines.append("    }")
    lines.append("")

    # TODO: Sub-tool methods need codegen improvements for expand_params,
    # entity_params, and correct argument marshaling. Skipped for now;
    # will be enabled in a follow-up when the codegen handles them.
    lines.append("    // Methods will be generated once codegen handles sub-tool FFI patterns.")

    lines.append("}")
    lines.append("")
