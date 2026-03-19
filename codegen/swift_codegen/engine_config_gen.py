"""Generator for EngineConfig.g.swift."""

from .context import OUT, schema, mapping, write_generated
from .shared_helpers import (
    swift_file_header,
    swift_type,
    swift_literal,
    to_camel,
    is_enum,
    is_value_type,
    method_exists_in_ffi,
    ffi_func_name,
    convert_param_to_ffi,
)
from .game_gen import _emit_unsafe_wrapping


def gen_engine_config() -> None:
    tool_name = "EngineConfig"
    tool_def = schema["tools"][tool_name]
    doc = tool_def.get("doc", "")
    methods = tool_def.get("methods", [])

    lines = [swift_file_header(), "import Foundation", "import CGoudEngine", ""]

    if doc:
        lines.append(f"/// {doc}")
    lines.append("public final class EngineConfig {")
    lines.append("    private var _handle: UnsafeMutableRawPointer?")
    lines.append("    private var _alive: Bool = true")
    lines.append("")

    create_ffi = ffi_func_name(tool_name, "new") or "goud_engine_config_create"
    lines.append("    public init() {")
    lines.append(f"        self._handle = {create_ffi}()")
    lines.append("    }")
    lines.append("")

    for m in methods:
        mname = m["name"]
        if mname == "destroy":
            continue
        if not method_exists_in_ffi(tool_name, mname):
            continue
        ffi_name = ffi_func_name(tool_name, mname)
        if not ffi_name:
            continue

        params = m.get("params", [])
        ret_type = m.get("returns", "void")
        mdoc = m.get("doc", "")

        if mdoc:
            lines.append(f"    /// {mdoc}")

        if mname == "build":
            lines.append("    public func build() -> GoudGame {")
            lines.append(f"        let ctx = {ffi_name}(_handle!)")
            lines.append("        _handle = nil")
            lines.append("        _alive = false")
            lines.append("        return GoudGame(ctx: ctx)")
            lines.append("    }")
            lines.append("")
            continue

        # Chaining methods return self
        param_parts = []
        call_parts = ["_handle!"]
        string_params = []
        value_type_params = []
        for p in params:
            pname = to_camel(p["name"])
            ptype = p["type"]
            st = swift_type(ptype)
            default = p.get("default")
            if default is not None:
                if ptype == "string":
                    param_parts.append(f'{pname}: {st} = "{default}"')
                else:
                    param_parts.append(f"{pname}: {st} = {swift_literal(default, ptype)}")
            else:
                param_parts.append(f"{pname}: {st}")

            if is_value_type(ptype.rstrip("?")):
                ffi_info = mapping.get("ffi_types", {}).get(ptype.rstrip("?"), {})
                if ffi_info.get("ffi_name"):
                    # Pass value types via pointer
                    value_type_params.append((pname, ffi_info["ffi_name"]))
                    call_parts.append(f"&_{pname}Ffi")
                else:
                    call_parts.append(convert_param_to_ffi(pname, ptype))
            else:
                call_parts.append(convert_param_to_ffi(pname, ptype))
            if ptype == "string":
                string_params.append(p)
        param_str = ", ".join(param_parts)
        call_str = ", ".join(call_parts)

        lines.append(f"    @discardableResult")
        lines.append(f"    public func {to_camel(mname)}({param_str}) -> EngineConfig {{")
        for vname, vffi_name in value_type_params:
            lines.append(f"        var _{vname}Ffi = {vname}.toFFI()")
        if string_params:
            _emit_unsafe_wrapping(lines, string_params, [], [], f"        {ffi_name}({call_str})", "        ")
        else:
            lines.append(f"        {ffi_name}({call_str})")
        lines.append("        return self")
        lines.append("    }")
        lines.append("")

    destroy_ffi = ffi_func_name(tool_name, "destroy") or "goud_engine_config_destroy"
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

    write_generated(OUT / "EngineConfig.g.swift", "\n".join(lines))
