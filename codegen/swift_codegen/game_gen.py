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
    safe_swift_name,
)


# ── Swift type for FFI out-param scalars ──────────────────────────

_OUT_PARAM_SWIFT_TYPE: dict[str, str] = {
    "f32": "Float",
    "f64": "Double",
    "u8": "UInt8",
    "u16": "UInt16",
    "u32": "UInt32",
    "u64": "UInt64",
    "i8": "Int8",
    "i16": "Int16",
    "i32": "Int32",
    "i64": "Int64",
    "bool": "Bool",
}


def _sc(name: str) -> str:
    """to_camel + keyword escape for Swift parameter/variable names."""
    return safe_swift_name(to_camel(name))


def _swift_enum_default(enum_type: str, default_val) -> str:
    """Convert a schema enum default value to a Swift enum case literal."""
    enum_def = schema.get("enums", {}).get(enum_type, {})
    values = enum_def.get("values", {})
    for vname, vval in values.items():
        if vval == default_val:
            return f".{to_screaming_snake(vname)}"
    # Fallback: use init(rawValue:)
    return f"{enum_type}(rawValue: {default_val})!"


def _get_ffi_meta(tool_name: str, mname: str) -> dict:
    """Get the ffi_tools metadata for a method."""
    return (
        schema.get("ffi_tools", {})
        .get(tool_name, {})
        .get("methods", {})
        .get(mname, {})
    )


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
    lines.append("    private var _deltaTime: Float = 0")
    lines.append("")

    # Public init
    ctor_swift_params = []
    ctor_ffi_args = []
    for p in ctor_params:
        pname = _sc(p["name"])
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
        # Use a local var to avoid capturing self before all members are initialized
        indent = "        "
        lines.append(f"{indent}var _ctxLocal: GoudContextId!")
        for sp in string_params:
            spname = _sc(sp["name"])
            lines.append(f"{indent}{spname}.withCString {{ {spname}Ptr in")
            indent += "    "
        lines.append(f"{indent}_ctxLocal = {create_ffi}({', '.join(ctor_ffi_args)})")
        for _ in string_params:
            indent = indent[:-4]
            lines.append(f"{indent}}}")
        lines.append("        self._ctx = _ctxLocal")
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

    # Lifecycle: deltaTime property
    lines.append("    /// The delta time (in seconds) returned by the last call to `beginFrame`.")
    lines.append("    public var deltaTime: Float { _deltaTime }")
    lines.append("")

    # Lifecycle: beginFrame / endFrame
    ffi_tool_def = schema.get("ffi_tools", {}).get(tool_name, {})
    lifecycle = ffi_tool_def.get("lifecycle", {})
    if "beginFrame" in lifecycle:
        lines.append("    /// Polls events, clears the screen, begins the renderer, and enables blending.")
        lines.append("    public func beginFrame(r: Float = 0, g: Float = 0, b: Float = 0, a: Float = 1) {")
        lines.append("        _deltaTime = goud_window_poll_events(_ctx)")
        lines.append("        goud_window_clear(_ctx, r, g, b, a)")
        lines.append("        let _ = goud_renderer_begin(_ctx)")
        lines.append("        goud_renderer_enable_blending(_ctx)")
        lines.append("    }")
        lines.append("")

    if "endFrame" in lifecycle:
        lines.append("    /// Ends the renderer and swaps the window buffers.")
        lines.append("    public func endFrame() {")
        lines.append("        let _ = goud_renderer_end(_ctx)")
        lines.append("        goud_window_swap_buffers(_ctx)")
        lines.append("    }")
        lines.append("")

    # Methods known to have FFI signature mismatches that need manual bridging.
    # These will be implemented in a follow-up once the codegen handles their
    # specific patterns (GoudResult returns, pointer-to-pointer params, etc.).
    _SKIP_METHODS = {
        "despawn", "cloneEntity", "cloneEntityRecursive",
        "setState", "setParameterBool", "setParameterFloat",
        "createSkinnedMesh", "setSkinnedMeshBones",
        "stepDebugger",
        "physicsSetCollisionCallback",
        "componentRegisterType", "componentAdd", "componentRemove",
        "componentHas", "componentGet", "componentGetMut",
        "componentAddBatch", "componentRemoveBatch", "componentHasBatch",
        "networkConnect", "networkConnectWithPeer", "networkSend",
        "networkReceive", "networkReceivePacket",
        "physicsCollisionEventsCount", "physicsCollisionEventsRead",
    }

    for m in methods:
        mname = m["name"]
        if mname == "destroy":
            continue
        if mname in _SKIP_METHODS:
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
    """Generate a single method for a tool class.

    Consults ``ffi_tools`` metadata to decide how to marshal the call.
    """
    mname = m["name"]
    params = m.get("params", [])
    ret_type = m.get("returns", "void")
    doc = m.get("doc", "")

    meta = _get_ffi_meta(tool_name, mname)

    # Skip strategies we cannot auto-generate yet
    if meta.get("ffi_strategy") or meta.get("batch_in") is not None or meta.get("batch_out") is not None:
        return []
    if meta.get("buffer_protocol") or meta.get("json_buffer_struct") or meta.get("out_buffer"):
        return []
    # Skip methods with out-params that need complex return marshaling
    if meta.get("out_params") or meta.get("returns_struct") or meta.get("returns_nullable_struct"):
        return []

    out_params = meta.get("out_params")
    returns_struct = meta.get("returns_struct")
    status_struct = meta.get("status_struct")
    status_nullable = meta.get("status_nullable_struct")
    returns_nullable = meta.get("returns_nullable_struct")
    expand_params_meta = meta.get("expand_params", {})
    no_context = meta.get("no_context")
    entity_params_set = set(meta.get("entity_params", []))
    enum_params_map = meta.get("enum_params", {})
    string_params_set = set(meta.get("string_params", []))
    returns_bool_from_i32 = meta.get("returns_bool_from_i32")
    append_args = meta.get("append_args", [])

    swift_name = to_camel(mname)
    if mname == "close":
        swift_name = "requestClose"

    # ── Build Swift param list ──
    param_parts = []
    for p in params:
        pname = _sc(p["name"])
        ptype = p["type"]
        # Override type to Entity for params marked as entity params
        if p["name"] in entity_params_set:
            st = "Entity"
        else:
            st = swift_type(ptype)
        default = p.get("default")
        if default is not None:
            if ptype == "string":
                param_parts.append(f'{pname}: {st} = "{default}"')
            elif is_enum(ptype.rstrip("?")):
                param_parts.append(f"{pname}: {st} = {_swift_enum_default(ptype.rstrip('?'), default)}")
            elif is_value_type(ptype.rstrip("?")):
                param_parts.append(f"{pname}: {st} = {default}")
            else:
                param_parts.append(f"{pname}: {st} = {swift_literal(default, ptype)}")
        else:
            param_parts.append(f"{pname}: {st}")
    param_str = ", ".join(param_parts)

    # ── Determine Swift return type ──
    target_struct = returns_struct or returns_nullable
    if out_params and target_struct:
        if returns_nullable or status_nullable:
            swift_ret = f"{swift_type(target_struct)}?"
        else:
            swift_ret = swift_type(target_struct)
    elif returns_bool_from_i32:
        swift_ret = "Bool"
    elif ret_type != "void":
        swift_ret = swift_type(ret_type)
    else:
        swift_ret = None

    # ── Build FFI call arguments ──
    call_args: list[str] = []
    if not no_context:
        call_args.append(handle_var)

    # Collect string params that need withCString wrapping
    string_wrap_params: list[dict] = []

    # Determine param order
    param_order = meta.get("param_order")
    if param_order:
        ordered_params = []
        param_by_name = {p["name"]: p for p in params}
        for pn in param_order:
            if pn in param_by_name:
                ordered_params.append(param_by_name[pn])
    else:
        ordered_params = list(params)

    for p in ordered_params:
        pname_raw = p["name"]
        pname = _sc(pname_raw)
        ptype = p["type"]

        # Check if this param should be expanded (e.g., Color -> r, g, b, a)
        if pname_raw in expand_params_meta:
            ep_info = expand_params_meta[pname_raw]
            for field in ep_info["fields"]:
                call_args.append(f"{pname}.{to_camel(field)}")
            continue

        # Entity params
        if pname_raw in entity_params_set:
            call_args.append(f"{pname}.bits")
            continue

        # Enum params (cast to FFI integer)
        if pname_raw in enum_params_map:
            cast_type = enum_params_map[pname_raw]
            swift_cast = _OUT_PARAM_SWIFT_TYPE.get(cast_type, "Int32")
            call_args.append(f"{swift_cast}({pname}.rawValue)")
            continue

        # String params
        if pname_raw in string_params_set or ptype == "string":
            string_wrap_params.append(p)
            call_args.append(f"{pname}Ptr")
            continue

        # Value type params
        if is_value_type(ptype.rstrip("?")):
            ffi_info = mapping.get("ffi_types", {}).get(ptype.rstrip("?"), {})
            if ffi_info.get("ffi_name"):
                call_args.append(f"{pname}.toFFI()")
                continue

        # bytes/array params
        if ptype in ("bytes", "u8[]", "Data"):
            call_args.append(f"{pname}BasePtr")
            call_args.append(f"{pname}.count")
            continue
        if ptype.endswith("[]"):
            call_args.append(f"{pname}BasePtr")
            call_args.append(f"UInt32({pname}.count)")
            continue

        call_args.append(convert_param_to_ffi(pname, ptype))

    # Expand remaining params from expand_params that are not in param_order
    if param_order:
        for ep_name, ep_info in expand_params_meta.items():
            if ep_name not in param_order:
                pname = _sc(ep_name)
                for field in ep_info["fields"]:
                    call_args.append(f"{pname}.{to_camel(field)}")

    # Append literal args (e.g., `true` for close)
    for a in append_args:
        call_args.append(swift_literal(a))

    # ── Handle out_params (append out-pointer args) ──
    out_var_decls: list[str] = []
    if out_params and target_struct:
        ffi_info = mapping.get("ffi_types", {}).get(target_struct, {})
        ffi_name_for_struct = ffi_info.get("ffi_name") if ffi_info else None

        if ffi_name_for_struct:
            # Single struct out-pointer (e.g., GoudRenderStats, FpsStats)
            out_var_decls.append(f"var _out = {ffi_name_for_struct}()")
            call_args.append("&_out")
        else:
            # Individual scalar out-pointers (e.g., float *out_x, float *out_y)
            for op in out_params:
                op_name = to_camel(op["name"])
                op_swift_type = _OUT_PARAM_SWIFT_TYPE.get(op["type"], "Float")
                out_var_decls.append(f"var _out_{op_name}: {op_swift_type} = 0")
                call_args.append(f"&_out_{op_name}")

    call_str = ", ".join(call_args)

    # ── Data/array params needing unsafe wrapping ──
    data_params = [p for p in params if p["type"] in ("bytes", "u8[]", "Data")]
    array_params = [
        p for p in params
        if p["type"].endswith("[]") and p["type"] not in ("u8[]",) and p not in data_params
    ]

    # ── Generate the method ──
    lines: list[str] = []
    if doc:
        lines.append(f"    /// {doc}")

    if swift_ret:
        lines.append(f"    public func {swift_name}({param_str}) -> {swift_ret} {{")
    else:
        lines.append(f"    public func {swift_name}({param_str}) {{")

    # Determine body indent
    indent = "        "

    # String wrapping
    for sp in string_wrap_params:
        spname = _sc(sp["name"])
        lines.append(f"{indent}{spname}.withCString {{ {spname}Ptr in")
        indent += "    "

    # Data wrapping
    for dp in data_params:
        dpname = _sc(dp["name"])
        lines.append(f"{indent}{dpname}.withUnsafeBytes {{ {dpname}RawBuf in")
        lines.append(
            f"{indent}    let {dpname}BasePtr = {dpname}RawBuf.baseAddress!"
            f".assumingMemoryBound(to: UInt8.self)"
        )
        indent += "    "

    # Array wrapping
    for ap in array_params:
        apname = _sc(ap["name"])
        elem = ap["type"][:-2]
        if elem == "Entity":
            lines.append(f"{indent}let {apname}Bits = {apname}.map {{ $0.bits }}")
            lines.append(f"{indent}{apname}Bits.withUnsafeBufferPointer {{ {apname}Buf in")
            lines.append(f"{indent}    let {apname}BasePtr = {apname}Buf.baseAddress!")
        else:
            lines.append(f"{indent}{apname}.withUnsafeBufferPointer {{ {apname}Buf in")
            lines.append(f"{indent}    let {apname}BasePtr = {apname}Buf.baseAddress!")
        indent += "    "

    # Out-param variable declarations
    for decl in out_var_decls:
        lines.append(f"{indent}{decl}")

    # The FFI call
    raw_call = f"{ffi_name}({call_str})"

    if out_params and target_struct:
        ffi_info = mapping.get("ffi_types", {}).get(target_struct, {})
        ffi_name_for_struct = ffi_info.get("ffi_name") if ffi_info else None

        if status_nullable or returns_nullable:
            lines.append(f"{indent}let _ok = {raw_call}")
            if returns_nullable:
                lines.append(f"{indent}guard _ok else {{ return nil }}")
            else:
                lines.append(f"{indent}guard _ok == 0 else {{ return nil }}")
            if ffi_name_for_struct:
                lines.append(f"{indent}return {target_struct}(ffi: _out)")
            else:
                _emit_struct_from_out_params(lines, indent, target_struct, out_params)
        elif status_struct:
            lines.append(f"{indent}let _ = {raw_call}")
            if ffi_name_for_struct:
                lines.append(f"{indent}return {target_struct}(ffi: _out)")
            else:
                _emit_struct_from_out_params(lines, indent, target_struct, out_params)
        else:
            lines.append(f"{indent}let _ = {raw_call}")
            if ffi_name_for_struct:
                lines.append(f"{indent}return {target_struct}(ffi: _out)")
            else:
                _emit_struct_from_out_params(lines, indent, target_struct, out_params)
    elif returns_bool_from_i32:
        lines.append(f"{indent}return {raw_call} != 0")
    elif ret_type == "void":
        lines.append(f"{indent}let _ = {raw_call}")
    elif ret_type != "void":
        result_expr = convert_return_from_ffi(raw_call, ret_type)
        lines.append(f"{indent}return {result_expr}")

    # Close wrappers
    for _ in array_params:
        indent = indent[:-4]
        lines.append(f"{indent}}}")
    for _ in data_params:
        indent = indent[:-4]
        lines.append(f"{indent}}}")
    for _ in string_wrap_params:
        indent = indent[:-4]
        lines.append(f"{indent}}}")

    lines.append("    }")
    lines.append("")
    return lines


def _emit_struct_from_out_params(
    lines: list[str], indent: str, struct_name: str, out_params: list[dict]
) -> None:
    """Build a Swift struct from individual out-pointer variables."""
    field_parts = []
    for op in out_params:
        op_name = to_camel(op["name"])
        field_parts.append(f"{op_name}: _out_{op_name}")
    fields_str = ", ".join(field_parts)
    lines.append(f"{indent}return {struct_name}({fields_str})")


def _gen_method_simple(
    tool_name: str, m: dict, ffi_name: str, handle_var: str, meta: dict
) -> list[str]:
    """Fallback: skip methods with complex FFI strategies we cannot auto-generate yet.

    Batch operations, buffer protocols, component strategies, etc. require
    manual FFI bridging code that the codegen does not yet produce.
    """
    # These methods need hand-written FFI bridges; skip to keep the SDK compilable.
    return []

    # --- dead code below preserved for future use ---
    mname = m["name"]
    params = m.get("params", [])
    ret_type = m.get("returns", "void")
    doc = m.get("doc", "")

    swift_name = to_camel(mname)

    no_context = meta.get("no_context")
    entity_params_set = set(meta.get("entity_params", []))
    enum_params_map = meta.get("enum_params", {})
    string_params_set = set(meta.get("string_params", []))

    param_parts = []
    for p in params:
        pname = _sc(p["name"])
        ptype = p["type"]
        st = swift_type(ptype)
        default = p.get("default")
        if default is not None:
            if ptype == "string":
                param_parts.append(f'{pname}: {st} = "{default}"')
            elif is_enum(ptype.rstrip("?")):
                param_parts.append(
                    f"{pname}: {st} = {_swift_enum_default(ptype.rstrip('?'), default)}"
                )
            elif is_value_type(ptype.rstrip("?")):
                param_parts.append(f"{pname}: {st} = {default}")
            else:
                param_parts.append(f"{pname}: {st} = {swift_literal(default, ptype)}")
        else:
            param_parts.append(f"{pname}: {st}")
    param_str = ", ".join(param_parts)

    swift_ret = swift_type(ret_type) if ret_type != "void" else None

    call_args: list[str] = []
    if not no_context:
        call_args.append(handle_var)

    string_wrap_params: list[dict] = []
    data_params: list[dict] = []
    array_params: list[dict] = []

    for p in params:
        pname_raw = p["name"]
        pname = _sc(pname_raw)
        ptype = p["type"]

        if pname_raw in entity_params_set:
            call_args.append(f"{pname}.bits")
        elif pname_raw in enum_params_map:
            cast_type = enum_params_map[pname_raw]
            swift_cast = _OUT_PARAM_SWIFT_TYPE.get(cast_type, "Int32")
            call_args.append(f"{swift_cast}({pname}.rawValue)")
        elif pname_raw in string_params_set or ptype == "string":
            string_wrap_params.append(p)
            call_args.append(f"{pname}Ptr")
        elif ptype in ("bytes", "u8[]", "Data"):
            data_params.append(p)
            call_args.append(f"{pname}BasePtr")
            call_args.append(f"{pname}.count")
        elif ptype.endswith("[]"):
            array_params.append(p)
            call_args.append(f"{pname}BasePtr")
            call_args.append(f"UInt32({pname}.count)")
        else:
            call_args.append(convert_param_to_ffi(pname, ptype))

    call_str = ", ".join(call_args)

    lines: list[str] = []
    if doc:
        lines.append(f"    /// {doc}")

    if swift_ret:
        lines.append(f"    public func {swift_name}({param_str}) -> {swift_ret} {{")
    else:
        lines.append(f"    public func {swift_name}({param_str}) {{")

    indent = "        "
    for sp in string_wrap_params:
        spname = _sc(sp["name"])
        lines.append(f"{indent}{spname}.withCString {{ {spname}Ptr in")
        indent += "    "
    for dp in data_params:
        dpname = _sc(dp["name"])
        lines.append(f"{indent}{dpname}.withUnsafeBytes {{ {dpname}RawBuf in")
        lines.append(
            f"{indent}    let {dpname}BasePtr = {dpname}RawBuf.baseAddress!"
            f".assumingMemoryBound(to: UInt8.self)"
        )
        indent += "    "
    for ap in array_params:
        apname = _sc(ap["name"])
        elem = ap["type"][:-2]
        if elem == "Entity":
            lines.append(f"{indent}let {apname}Bits = {apname}.map {{ $0.bits }}")
            lines.append(f"{indent}{apname}Bits.withUnsafeBufferPointer {{ {apname}Buf in")
            lines.append(f"{indent}    let {apname}BasePtr = {apname}Buf.baseAddress!")
        else:
            lines.append(f"{indent}{apname}.withUnsafeBufferPointer {{ {apname}Buf in")
            lines.append(f"{indent}    let {apname}BasePtr = {apname}Buf.baseAddress!")
        indent += "    "

    raw_call = f"{ffi_name}({call_str})"
    if ret_type == "void":
        lines.append(f"{indent}let _ = {raw_call}")
    else:
        result_expr = convert_return_from_ffi(raw_call, ret_type)
        lines.append(f"{indent}return {result_expr}")

    for _ in array_params:
        indent = indent[:-4]
        lines.append(f"{indent}}}")
    for _ in data_params:
        indent = indent[:-4]
        lines.append(f"{indent}}}")
    for _ in string_wrap_params:
        indent = indent[:-4]
        lines.append(f"{indent}}}")

    lines.append("    }")
    lines.append("")
    return lines


def _emit_unsafe_wrapping(
    lines: list[str],
    string_params: list[dict],
    data_params: list[dict],
    array_params: list[dict],
    inner_line: str,
    base_indent: str,
) -> None:
    """Wrap string, Data, and array params with appropriate unsafe closures."""
    indent = base_indent
    for sp in string_params:
        spname = _sc(sp["name"])
        lines.append(f"{indent}{spname}.withCString {{ {spname}Ptr in")
        indent += "    "
    for dp in data_params:
        dpname = _sc(dp["name"])
        lines.append(f"{indent}{dpname}.withUnsafeBytes {{ {dpname}RawBuf in")
        lines.append(
            f"{indent}    let {dpname}BasePtr = {dpname}RawBuf.baseAddress!"
            f".assumingMemoryBound(to: UInt8.self)"
        )
        indent += "    "
    for ap in array_params:
        apname = _sc(ap["name"])
        elem = ap["type"][:-2]
        if elem == "Entity":
            lines.append(f"{indent}let {apname}Bits = {apname}.map {{ $0.bits }}")
            lines.append(f"{indent}{apname}Bits.withUnsafeBufferPointer {{ {apname}Buf in")
            lines.append(f"{indent}    let {apname}BasePtr = {apname}Buf.baseAddress!")
        else:
            lines.append(f"{indent}{apname}.withUnsafeBufferPointer {{ {apname}Buf in")
            lines.append(f"{indent}    let {apname}BasePtr = {apname}Buf.baseAddress!")
        indent += "    "
    lines.append(f"{indent}{inner_line.strip()}")
    for _ in array_params:
        indent = indent[:-4]
        lines.append(f"{indent}}}")
    for _ in data_params:
        indent = indent[:-4]
        lines.append(f"{indent}}}")
    for _ in string_params:
        indent = indent[:-4]
        lines.append(f"{indent}}}")
