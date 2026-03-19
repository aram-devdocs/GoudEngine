"""Generator for UiManager.g.swift."""

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


def gen_ui() -> None:
    tool_name = "UiManager"
    if tool_name not in schema.get("tools", {}):
        return

    tool_def = schema["tools"][tool_name]
    doc = tool_def.get("doc", "")
    methods = tool_def.get("methods", [])

    lines = [swift_file_header(), "import Foundation", "import CGoudEngine", ""]

    if doc:
        lines.append(f"/// {doc}")
    lines.append("public final class UiManager {")
    # goud_ui_manager_create returns OpaquePointer (struct UiManager *)
    lines.append("    private var _handle: OpaquePointer?")
    lines.append("    private var _alive: Bool = true")
    lines.append("")

    create_ffi = ffi_func_name(tool_name, "new") or "goud_ui_manager_create"
    lines.append("    public init() {")
    lines.append(f"        self._handle = {create_ffi}()")
    lines.append("    }")
    lines.append("")

    # Skip methods with UiStyle (type was skipped) or arg count mismatches
    _SKIP = {"setStyle", "setLabelText", "setSlider", "setImageTexturePath"}
    for m in methods:
        mname = m["name"]
        if mname == "destroy" or mname in _SKIP:
            continue
        if not method_exists_in_ffi(tool_name, mname):
            continue
        ffi_name = ffi_func_name(tool_name, mname)
        if not ffi_name:
            continue
        method_lines = _gen_method(tool_name, m, ffi_name, "_handle!")
        lines.extend(method_lines)

    destroy_ffi = ffi_func_name(tool_name, "destroy") or "goud_ui_manager_destroy"
    lines.append("    deinit {")
    lines.append("        if _alive, let h = _handle {")
    lines.append(f"            {destroy_ffi}(h)")
    lines.append("        }")
    lines.append("    }")
    lines.append("")

    lines.append("    public func destroy() {")
    lines.append("        if _alive, let h = _handle {")
    lines.append(f"            {destroy_ffi}(h)")
    lines.append("            _handle = nil")
    lines.append("            _alive = false")
    lines.append("        }")
    lines.append("    }")
    lines.append("}")
    lines.append("")

    write_generated(OUT / "UiManager.g.swift", "\n".join(lines))
