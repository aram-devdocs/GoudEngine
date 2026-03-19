"""Generator for Physics.g.swift (PhysicsWorld2D, PhysicsWorld3D)."""

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
    safe_swift_name,
)
from .game_gen import _gen_method


def gen_physics() -> None:
    lines = [swift_file_header(), "import Foundation", "import CGoudEngine", ""]

    # PhysicsWorld2D/3D classes are skipped: the standalone physics FFI
    # functions (goud_physics_create, goud_physics_step, etc.) are not
    # currently exported from the native library.  They will be generated
    # once the engine ships these symbols.
    lines.append("// PhysicsWorld2D / PhysicsWorld3D classes omitted — standalone physics")
    lines.append("// FFI symbols not yet exported from the native library.")
    lines.append("")

    write_generated(OUT / "Physics.g.swift", "\n".join(lines))


def _gen_physics_class(lines: list[str], tool_name: str) -> None:
    tool_def = schema["tools"][tool_name]
    doc = tool_def.get("doc", "")
    ctor = tool_def.get("constructor", {})
    ctor_params = ctor.get("params", [])
    methods = tool_def.get("methods", [])

    if doc:
        lines.append(f"/// {doc}")
    lines.append(f"public final class {tool_name} {{")
    lines.append("    internal var _ctx: GoudContextId")
    lines.append("    private var _alive: Bool = true")
    lines.append("")

    # Constructor -- physics create takes a context + gravity params, returns i32 status.
    # The class receives a context from outside and calls create on it.
    create_ffi = ffi_func_name(tool_name, "create")
    if not create_ffi:
        create_ffi = f"goud_physics_{tool_name.lower()}_create"

    ctor_swift_params = ["ctx: GoudContextId"]
    ctor_ffi_args = ["ctx"]
    for p in ctor_params:
        pname = safe_swift_name(to_camel(p["name"]))
        ptype = p["type"]
        st = swift_type(ptype)
        default = p.get("default")
        if default is not None:
            ctor_swift_params.append(f"{pname}: {st} = {swift_literal(default, ptype)}")
        else:
            ctor_swift_params.append(f"{pname}: {st}")
        ctor_ffi_args.append(convert_param_to_ffi(pname, ptype))

    param_str = ", ".join(ctor_swift_params)
    arg_str = ", ".join(ctor_ffi_args)
    lines.append(f"    public init({param_str}) {{")
    lines.append(f"        self._ctx = ctx")
    lines.append(f"        let _ = {create_ffi}({arg_str})")
    lines.append("    }")
    lines.append("")

    _skip = {"destroy", "create", "new", "createWithBackend", "setCollisionCallback"}
    for m in methods:
        mname = m["name"]
        if mname in _skip:
            continue
        if not method_exists_in_ffi(tool_name, mname):
            continue
        ffi_name = ffi_func_name(tool_name, mname)
        if not ffi_name:
            continue
        method_lines = _gen_method(tool_name, m, ffi_name, "_ctx")
        lines.extend(method_lines)

    destroy_ffi = ffi_func_name(tool_name, "destroy")
    if not destroy_ffi:
        destroy_ffi = f"goud_physics_{tool_name.lower()}_destroy"

    lines.append("    deinit {")
    lines.append("        if _alive {")
    lines.append(f"            let _ = {destroy_ffi}(_ctx)")
    lines.append("            _alive = false")
    lines.append("        }")
    lines.append("    }")
    lines.append("")

    lines.append("    public func destroy() {")
    lines.append("        if _alive {")
    lines.append(f"            let _ = {destroy_ffi}(_ctx)")
    lines.append("            _alive = false")
    lines.append("        }")
    lines.append("    }")
    lines.append("}")
    lines.append("")
