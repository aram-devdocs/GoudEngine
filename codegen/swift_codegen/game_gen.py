"""Generator for GoudGame.g.swift."""

from .context import HEADER_COMMENT, OUT, schema, mapping, write_generated
from .shared_helpers import (
    swift_file_header,
    swift_type,
    swift_default,
    swift_literal,
    to_camel,
    to_screaming_snake,
    is_enum,
    is_value_type,
    method_exists_in_ffi,
    ffi_func_name,
    convert_param_to_ffi,
    convert_return_from_ffi,
)


def _swift_enum_default(enum_type: str, default_val) -> str:
    """Convert a schema enum default value to a Swift enum case literal."""
    enum_def = schema.get("enums", {}).get(enum_type, {})
    values = enum_def.get("values", {})
    for vname, vval in values.items():
        if vval == default_val:
            return f".{to_screaming_snake(vname)}"
    # Fallback: use init(rawValue:)
    return f"{enum_type}(rawValue: {default_val})!"


def gen_game() -> None:
    tool_name = "GoudGame"
    tool_def = schema["tools"][tool_name]
    doc = tool_def.get("doc", "")
    ctor = tool_def.get("constructor", {})
    ctor_params = ctor.get("params", [])
    methods = tool_def.get("methods", [])

    lines = [swift_file_header(), "import Foundation", "import CGoudEngine", ""]

    if doc:
        lines.append(f"/// {doc}")
    lines.append("public final class GoudGame {")
    lines.append("    internal var _ctx: GoudContextId")
    lines.append("    private var _alive: Bool = true")
    lines.append("")

    # Public init
    ctor_swift_params = []
    ctor_ffi_args = []
    for p in ctor_params:
        pname = to_camel(p["name"])
        ptype = p["type"]
        st = swift_type(ptype)
        default = p.get("default")
        if default is not None:
            if ptype == "string":
                ctor_swift_params.append(f'{pname}: {st} = "{default}"')
            else:
                ctor_swift_params.append(f"{pname}: {st} = {swift_literal(default, ptype)}")
        else:
            ctor_swift_params.append(f"{pname}: {st}")
        if ptype == "string":
            ctor_ffi_args.append(f"{pname}Ptr")
        else:
            ctor_ffi_args.append(pname)

    # Determine the create FFI function
    create_ffi = ffi_func_name(tool_name, "new") or "goud_window_create"

    has_string_params = any(p["type"] == "string" for p in ctor_params)
    param_str = ", ".join(ctor_swift_params)
    lines.append(f"    public init({param_str}) {{")

    if has_string_params:
        string_params = [p for p in ctor_params if p["type"] == "string"]
        non_string_params = [p for p in ctor_params if p["type"] != "string"]
        # Build nested withCString calls
        indent = "        "
        for sp in string_params:
            spname = to_camel(sp["name"])
            lines.append(f"{indent}{spname}.withCString {{ {spname}Ptr in")
            indent += "    "
        lines.append(f"{indent}self._ctx = {create_ffi}({', '.join(ctor_ffi_args)})")
        for _ in string_params:
            indent = indent[:-4]
            lines.append(f"{indent}}}")
    else:
        arg_str = ", ".join(ctor_ffi_args)
        lines.append(f"        self._ctx = {create_ffi}({arg_str})")

    lines.append("    }")
    lines.append("")

    # Internal init for EngineConfig.build()
    lines.append("    internal init(ctx: GoudContextId) {")
    lines.append("        self._ctx = ctx")
    lines.append("    }")
    lines.append("")

    # Methods
    for m in methods:
        mname = m["name"]
        if mname == "destroy":
            continue
        if not method_exists_in_ffi(tool_name, mname):
            continue
        ffi_name = ffi_func_name(tool_name, mname)
        if not ffi_name:
            continue
        method_lines = _gen_method(tool_name, m, ffi_name, "_ctx")
        lines.extend(method_lines)

    # deinit
    destroy_ffi = ffi_func_name(tool_name, "destroy") or "goud_window_destroy"
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

    write_generated(OUT / "GoudGame.g.swift", "\n".join(lines))


def _gen_method(tool_name: str, m: dict, ffi_name: str, handle_var: str) -> list[str]:
    """Generate a single method for a tool class."""
    mname = m["name"]
    params = m.get("params", [])
    ret_type = m.get("returns", "void")
    doc = m.get("doc", "")

    swift_name = to_camel(mname)
    if mname == "close":
        swift_name = "requestClose"

    # Build param string
    param_parts = []
    for p in params:
        pname = to_camel(p["name"])
        ptype = p["type"]
        st = swift_type(ptype)
        default = p.get("default")
        if default is not None:
            if ptype == "string":
                param_parts.append(f'{pname}: {st} = "{default}"')
            elif is_enum(ptype.rstrip("?")):
                param_parts.append(f"{pname}: {st} = {_swift_enum_default(ptype.rstrip('?'), default)}")
            elif is_value_type(ptype.rstrip("?")):
                # Schema stores defaults like "Color.white()" as strings
                param_parts.append(f"{pname}: {st} = {default}")
            else:
                param_parts.append(f"{pname}: {st} = {swift_literal(default, ptype)}")
        else:
            param_parts.append(f"{pname}: {st}")
    param_str = ", ".join(param_parts)

    # Determine return type
    base_ret = ret_type.rstrip("?")
    swift_ret = swift_type(ret_type) if ret_type != "void" else None

    # Check for string params
    string_params = [p for p in params if p["type"] == "string"]
    has_strings = len(string_params) > 0

    # Build call args
    call_args = [handle_var]
    for p in params:
        pname = to_camel(p["name"])
        ptype = p["type"]
        call_args.append(convert_param_to_ffi(pname, ptype))
    call_str = ", ".join(call_args)

    lines: list[str] = []
    if doc:
        lines.append(f"    /// {doc}")

    if ret_type == "void":
        lines.append(f"    public func {swift_name}({param_str}) {{")
        if has_strings:
            _emit_string_wrapping(lines, string_params, f"        {ffi_name}({call_str})", "        ")
        else:
            lines.append(f"        {ffi_name}({call_str})")
        lines.append("    }")
    else:
        lines.append(f"    public func {swift_name}({param_str}) -> {swift_ret} {{")
        raw_call = f"{ffi_name}({call_str})"
        result_expr = convert_return_from_ffi(raw_call, ret_type) if base_ret != "void" else raw_call
        if has_strings:
            _emit_string_wrapping(lines, string_params, f"        return {result_expr}", "        ")
        else:
            lines.append(f"        return {result_expr}")
        lines.append("    }")

    lines.append("")
    return lines


def _emit_string_wrapping(lines: list[str], string_params: list[dict], inner_line: str, base_indent: str) -> None:
    """Wrap string params with withCString closures."""
    indent = base_indent
    for sp in string_params:
        spname = to_camel(sp["name"])
        lines.append(f"{indent}{spname}.withCString {{ {spname}Ptr in")
        indent += "    "
    lines.append(f"{indent}{inner_line.strip()}")
    for _ in string_params:
        indent = indent[:-4]
        lines.append(f"{indent}}}")
