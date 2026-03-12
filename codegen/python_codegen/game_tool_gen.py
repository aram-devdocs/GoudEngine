"""Tool class generation for `_game.py`."""

from .context import PYTHON_TYPES, mapping, schema, to_snake
from .game_tool_helpers import emit_tool_method_body


def _emit_tool_constructor(
    lines: list[str],
    *,
    is_game: bool,
    is_physics_world_2d: bool,
    is_physics_world_3d: bool,
) -> None:
    if is_game:
        lines.append("    def __init__(self, width: int = 800, height: int = 600, title: str = 'GoudEngine'):")
        lines.append("        lib = get_lib()")
        lines.append("        self._lib = lib")
        lines.append("        self._ctx = lib.goud_window_create(width, height, title.encode('utf-8'))")
        lines.append("        self._delta_time = 0.0")
        lines.append("        self._title = title")
        lines.append("        self._frame_count = 0")
        lines.append("        self._total_time = 0.0")
    elif is_physics_world_2d:
        lines.append("    def __init__(self, gravity_x: float, gravity_y: float, backend=PhysicsBackend2D.DEFAULT):")
        lines.append("        lib = get_lib()")
        lines.append("        self._lib = lib")
        lines.append("        self._ctx = lib.goud_context_create()")
        lines.append("        if self._ctx._bits == 0xFFFFFFFFFFFFFFFF:")
        lines.append("            raise RuntimeError('Failed to create headless context')")
        lines.append("        status = lib.goud_physics_create_with_backend(self._ctx, gravity_x, gravity_y, int(backend))")
        lines.append("        if status != 0:")
        lines.append("            lib.goud_context_destroy(self._ctx)")
        lines.append("            raise RuntimeError(f'Failed to create PhysicsWorld2D (status {status})')")
    elif is_physics_world_3d:
        lines.append("    def __init__(self, gravity_x: float, gravity_y: float, gravity_z: float):")
        lines.append("        lib = get_lib()")
        lines.append("        self._lib = lib")
        lines.append("        self._ctx = lib.goud_context_create()")
        lines.append("        if self._ctx._bits == 0xFFFFFFFFFFFFFFFF:")
        lines.append("            raise RuntimeError('Failed to create headless context')")
        lines.append("        status = lib.goud_physics3d_create(self._ctx, gravity_x, gravity_y, gravity_z)")
        lines.append("        if status != 0:")
        lines.append("            lib.goud_context_destroy(self._ctx)")
        lines.append("            raise RuntimeError(f'Failed to create PhysicsWorld3D (status {status})')")
    else:
        lines.append("    def __init__(self):")
        lines.append("        lib = get_lib()")
        lines.append("        self._lib = lib")
        lines.append("        self._ctx = lib.goud_context_create()")
    lines.append("")


def _emit_tool_properties(tool: dict, tool_mapping: dict, lines: list[str]) -> None:
    for prop in tool.get("properties", []):
        pname = to_snake(prop["name"])
        py_type = PYTHON_TYPES.get(prop.get("type", "f32"), "float")
        lines.append("    @property")
        lines.append(f"    def {pname}(self) -> {py_type}:")
        prop_map = tool_mapping.get("properties", {}).get(prop["name"], {})
        src = prop_map.get("source")
        if src == "cached":
            field_name = prop_map.get("field", "_delta_time")
            lines.append(f"        return self.{field_name}")
        elif src == "computed":
            lines.append("        return 1.0 / self._delta_time if self._delta_time > 0 else 0.0")
        elif "ffi" in prop_map:
            ffi_fn = prop_map["ffi"]
            if "out_index" in prop_map:
                idx = prop_map["out_index"]
                lines.append("        w = ctypes.c_uint32(0)")
                lines.append("        h = ctypes.c_uint32(0)")
                lines.append(f"        self._lib.{ffi_fn}(self._ctx, ctypes.byref(w), ctypes.byref(h))")
                lines.append(f"        return {'w' if idx == 0 else 'h'}.value")
            else:
                lines.append(f"        return self._lib.{ffi_fn}(self._ctx)")
        lines.append("")


def gen_tool_class(tool_name: str, lines: list[str]) -> None:
    """Generate a tool class (GoudGame or GoudContext)."""
    tool = schema["tools"][tool_name]
    tool_mapping = mapping["tools"][tool_name]

    is_game = tool_name == "GoudGame"
    is_physics_world_2d = tool_name == "PhysicsWorld2D"
    is_physics_world_3d = tool_name == "PhysicsWorld3D"
    uses_network_status_errors = any(
        tool_mapping["methods"].get(method["name"], {}).get("ffi", "").startswith("goud_network_")
        and (
            tool_mapping["methods"].get(method["name"], {}).get("out_buffer")
            or tool_mapping["methods"].get(method["name"], {}).get("status_struct")
            or tool_mapping["methods"].get(method["name"], {}).get("status_nullable_struct")
        )
        for method in tool.get("methods", [])
    )

    lines.append(f"class {tool_name}:")
    lines.append(f'    """{tool["doc"]}"""')
    lines.append("")

    if uses_network_status_errors:
        lines.append("    def _raise_network_error_or_runtime(self, message):")
        lines.append("        error = GoudError.from_last_error(self._lib)")
        lines.append("        if error is not None:")
        lines.append("            raise error")
        lines.append("        raise RuntimeError(message)")
        lines.append("")

    _emit_tool_constructor(
        lines,
        is_game=is_game,
        is_physics_world_2d=is_physics_world_2d,
        is_physics_world_3d=is_physics_world_3d,
    )

    lines.append("    def __del__(self):")
    lines.append("        self.destroy()")
    lines.append("")

    _emit_tool_properties(tool, tool_mapping, lines)

    for method in tool.get("methods", []):
        mname = to_snake(method["name"])
        mmap = tool_mapping["methods"].get(method["name"], {})
        params = method.get("params", [])
        ret = method.get("returns", "void")

        has_default = [
            p.get("default") is not None or p["type"] in schema["types"] and p.get("default")
            for p in params
        ]
        allow_default = [False] * len(params)
        tail_ok = True
        for i in range(len(params) - 1, -1, -1):
            if not has_default[i]:
                tail_ok = False
            allow_default[i] = tail_ok

        param_strs = []
        for i, p in enumerate(params):
            pn = to_snake(p["name"])
            pt = p["type"]
            default = p.get("default")
            if pt == "callback(f32)":
                param_strs.append(f"{pn}")
            elif pt in schema["types"]:
                param_strs.append(f"{pn} = None" if default and allow_default[i] else f"{pn}")
            elif pt in schema["enums"]:
                param_strs.append(f"{pn} = {default}" if default is not None and allow_default[i] else f"{pn}")
            else:
                param_strs.append(f"{pn} = {default}" if default is not None and allow_default[i] else f"{pn}")

        lines.append(f"    def {mname}({', '.join(['self'] + param_strs)}):")
        if method.get("doc"):
            lines.append(f'        """{method["doc"]}"""')

        emit_tool_method_body(
            mname,
            method,
            mmap,
            params,
            ret,
            lines,
            is_game=is_game,
            is_physics_world_2d=is_physics_world_2d,
            is_physics_world_3d=is_physics_world_3d,
            uses_network_status_errors=uses_network_status_errors,
        )
        lines.append("")
