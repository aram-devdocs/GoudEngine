"""Generator for GoudContext.g.swift."""

from .context import OUT, schema, write_generated
from .shared_helpers import (
    swift_file_header,
    swift_type,
    swift_literal,
    to_camel,
    is_enum,
    method_exists_in_ffi,
    ffi_func_name,
    convert_param_to_ffi,
    convert_return_from_ffi,
)
from .game_gen import _gen_method


def gen_context() -> None:
    tool_name = "GoudContext"
    tool_def = schema["tools"][tool_name]
    doc = tool_def.get("doc", "")
    methods = tool_def.get("methods", [])

    lines = [swift_file_header(), "import Foundation", "import CGoudEngine", ""]

    if doc:
        lines.append(f"/// {doc}")
    lines.append("public final class GoudContext {")
    lines.append("    internal var _ctx: GoudContextId")
    lines.append("    private var _alive: Bool = true")
    lines.append("")

    create_ffi = ffi_func_name(tool_name, "new") or "goud_context_create"
    lines.append("    public init() {")
    lines.append(f"        self._ctx = {create_ffi}()")
    lines.append("    }")
    lines.append("")

    lines.append("    internal init(ctx: GoudContextId) {")
    lines.append("        self._ctx = ctx")
    lines.append("    }")
    lines.append("")

    # Skip methods with known FFI signature mismatches
    _SKIP = {
        "despawn", "cloneEntity", "cloneEntityRecursive",
        "componentRegisterType", "componentAdd", "componentRemove",
        "componentHas", "componentGet", "componentGetMut",
        "componentAddBatch", "componentRemoveBatch", "componentHasBatch",
        "componentGetEntities", "componentGetAll",
        "networkConnect", "networkConnectWithPeer", "networkSend",
        "networkReceive", "networkReceivePacket",
        "stepDebugger",
        "sceneCreate", "sceneGetByName", "loadScene", "unloadScene",
    }
    for m in methods:
        mname = m["name"]
        if mname == "destroy" or mname in _SKIP:
            continue
        if not method_exists_in_ffi(tool_name, mname):
            continue
        ffi_name = ffi_func_name(tool_name, mname)
        if not ffi_name:
            continue
        method_lines = _gen_method(tool_name, m, ffi_name, "_ctx")
        lines.extend(method_lines)

    destroy_ffi = ffi_func_name(tool_name, "destroy") or "goud_context_destroy"
    lines.append("    deinit {")
    lines.append("        if _alive {")
    lines.append(f"            {destroy_ffi}(_ctx)")
    lines.append("            _alive = false")
    lines.append("        }")
    lines.append("    }")
    lines.append("")

    lines.append("    public func destroy() {")
    lines.append("        if _alive {")
    lines.append(f"            {destroy_ffi}(_ctx)")
    lines.append("            _alive = false")
    lines.append("        }")
    lines.append("    }")
    lines.append("}")
    lines.append("")

    write_generated(OUT / "GoudContext.g.swift", "\n".join(lines))
