"""Helpers for generating methods in `_game.py` tool classes."""

from .context import mapping, schema, to_snake
from .shared_helpers import ffi_uses_ptr_len, py_out_var_ctype, py_value_param_ffi_setup


def gen_component_strategy(strategy: str, comp_type: str, mmap: dict, lines: list[str]) -> None:
    """Generate real FFI calls for component_add/get/set/has/remove."""
    ffi_type_info = mapping.get("ffi_types", {}).get(comp_type, {})
    ffi_struct = ffi_type_info.get("ffi_name", f"Ffi{comp_type}")

    struct_params = mmap.get("struct_params", [])
    struct_var = struct_params[0] if struct_params else to_snake(comp_type)

    if strategy == "component_add":
        lines.append(f"        {struct_var}._sync_to_ffi()")
        lines.append(
            f"        self._lib.goud_component_add(self._ctx, entity._bits, _TYPEID_{comp_type.upper()}, "
            f"ctypes.cast(ctypes.pointer({struct_var}._ffi), ctypes.POINTER(ctypes.c_uint8)), "
            f"ctypes.sizeof({ffi_struct}))"
        )
    elif strategy == "component_get":
        lines.append(f"        ptr = self._lib.goud_component_get(self._ctx, entity._bits, _TYPEID_{comp_type.upper()})")
        lines.append("        if not ptr:")
        lines.append("            return None")
        lines.append(f"        ffi = ctypes.cast(ptr, ctypes.POINTER({ffi_struct})).contents")
        lines.append(f"        return {comp_type}._from_ffi(ffi)")
    elif strategy == "component_set":
        lines.append(f"        {struct_var}._sync_to_ffi()")
        lines.append(f"        ptr = self._lib.goud_component_get_mut(self._ctx, entity._bits, _TYPEID_{comp_type.upper()})")
        lines.append("        if ptr:")
        lines.append(
            f"            ctypes.memmove(ptr, ctypes.addressof({struct_var}._ffi), ctypes.sizeof({ffi_struct}))"
        )
    elif strategy == "component_has":
        lines.append(f"        return self._lib.goud_component_has(self._ctx, entity._bits, _TYPEID_{comp_type.upper()})")
    elif strategy == "component_remove":
        lines.append(f"        result = self._lib.goud_component_remove(self._ctx, entity._bits, _TYPEID_{comp_type.upper()})")
        lines.append("        return result.success")


def emit_tool_method_body(
    mname: str,
    method: dict,
    mmap: dict,
    params: list[dict],
    ret: str,
    lines: list[str],
    *,
    is_game: bool,
    is_physics_world_2d: bool,
    is_physics_world_3d: bool,
    uses_network_status_errors: bool,
) -> None:
    """Emit method body for a generated tool method."""
    if mname == "destroy":
        if is_game:
            lines.append("        if hasattr(self, '_ctx'):")
            lines.append("            self._lib.goud_window_destroy(self._ctx)")
            lines.append("            del self._ctx")
        elif is_physics_world_2d:
            lines.append("        if hasattr(self, '_ctx'):")
            lines.append("            self._lib.goud_physics_destroy(self._ctx)")
            lines.append("            self._lib.goud_context_destroy(self._ctx)")
            lines.append("            del self._ctx")
        elif is_physics_world_3d:
            lines.append("        if hasattr(self, '_ctx'):")
            lines.append("            self._lib.goud_physics3d_destroy(self._ctx)")
            lines.append("            self._lib.goud_context_destroy(self._ctx)")
            lines.append("            del self._ctx")
        else:
            lines.append("        if hasattr(self, '_ctx'):")
            lines.append("            self._lib.goud_context_destroy(self._ctx)")
            lines.append("            del self._ctx")
    elif mname == "begin_frame":
        lines.append("        self._delta_time = self._lib.goud_window_poll_events(self._ctx)")
        lines.append("        self._lib.goud_window_clear(self._ctx, r, g, b, a)")
        lines.append("        self._lib.goud_renderer_begin(self._ctx)")
        lines.append("        self._lib.goud_renderer_enable_blending(self._ctx)")
    elif mname == "end_frame":
        lines.append("        self._lib.goud_renderer_end(self._ctx)")
        lines.append("        self._lib.goud_window_swap_buffers(self._ctx)")
    elif mname == "update_frame":
        lines.append("        self._delta_time = dt")
        lines.append("        self._frame_count += 1")
        lines.append("        self._total_time += dt")
    elif mname == "run":
        lines.append("        while not self.should_close():")
        lines.append("            self.begin_frame()")
        lines.append("            update(self._delta_time)")
        lines.append("            self.end_frame()")
    elif mname == "draw_sprite":
        lines.append("        if color is None: color = Color.white()")
        lines.append("        self._lib.goud_renderer_draw_sprite(self._ctx, texture, x, y, width, height, rotation, color.r, color.g, color.b, color.a)")
    elif mname == "draw_quad":
        lines.append("        if color is None: color = Color.white()")
        lines.append("        self._lib.goud_renderer_draw_quad(self._ctx, x, y, width, height, color.r, color.g, color.b, color.a)")
    elif mname == "load_texture":
        lines.append("        return self._lib.goud_texture_load(self._ctx, path.encode('utf-8'))")
    elif mname == "destroy_texture":
        lines.append("        self._lib.goud_texture_destroy(self._ctx, handle)")
    elif mname == "load_font":
        lines.append("        return self._lib.goud_font_load(self._ctx, path.encode('utf-8'))")
    elif mname == "destroy_font":
        lines.append("        return self._lib.goud_font_destroy(self._ctx, handle)")
    elif mname == "draw_text":
        lines.append("        if color is None: color = Color.white()")
        lines.append("        return self._lib.goud_renderer_draw_text(self._ctx, font_handle, text.encode('utf-8'), x, y, font_size, int(alignment), max_width, line_spacing, int(direction), color.r, color.g, color.b, color.a)")
    elif mname == "physics_set_collision_callback":
        lines.append("        if callback_ptr not in (0, None) or user_data not in (0, None):")
        lines.append("            raise RuntimeError('Python cannot safely pass raw function pointers here; pass 0 to clear callback')")
        lines.append("        return self._lib.goud_physics_set_collision_callback(self._ctx, 0, 0)")
    elif "ffi_strategy" in mmap:
        strategy = mmap["ffi_strategy"]
        comp_type = mmap.get("component_type", "")
        if strategy.startswith("component_"):
            gen_component_strategy(strategy, comp_type, mmap, lines)
        elif strategy == "name_add":
            lines.append("        self._lib.goud_name_add(self._ctx, entity._bits, name.encode('utf-8'))")
        elif strategy == "name_get":
            lines.append("        # TODO: wire to goud_name_get FFI")
            lines.append("        return None")
        elif strategy == "name_has":
            lines.append("        # TODO: wire to goud_name_has FFI")
            lines.append("        return False")
        elif strategy == "name_remove":
            lines.append("        # TODO: wire to goud_name_remove FFI")
            lines.append("        return False")
        else:
            lines.append(f"        pass  # Unknown strategy: {strategy}")
    elif "returns_entity" in mmap:
        if "entity_params" in mmap:
            entity_args = ", ".join(f"{p}._bits" for p in mmap["entity_params"])
            lines.append(f"        bits = self._lib.{mmap['ffi']}(self._ctx, {entity_args})")
        else:
            lines.append(f"        bits = self._lib.{mmap['ffi']}(self._ctx)")
        lines.append("        return Entity(bits)")
    elif "entity_params" in mmap and "ffi" in mmap:
        ffi_fn = mmap["ffi"]
        no_ctx = mmap.get("no_context", False)
        entity_set = set(mmap["entity_params"])
        string_set = set(mmap.get("string_params", []))
        uses_ptr_len = ffi_uses_ptr_len(ffi_fn)

        ffi_parts = [] if no_ctx else ["self._ctx"]
        for p in params:
            pn = p["name"]
            sn = to_snake(pn)
            if pn in entity_set:
                ffi_parts.append(f"{sn}._bits")
            elif p["type"] == "string" and pn in string_set and uses_ptr_len:
                lines.append(f"        _{sn}_bytes = {sn}.encode('utf-8')")
                lines.append(f"        _{sn}_buf = ctypes.create_string_buffer(_{sn}_bytes, len(_{sn}_bytes))")
                ffi_parts.append(f"ctypes.cast(_{sn}_buf, ctypes.POINTER(ctypes.c_uint8))")
                ffi_parts.append(f"len(_{sn}_bytes)")
            elif p["type"] in schema.get("enums", {}):
                ffi_parts.append(f"int({sn})")
            else:
                value_setup = py_value_param_ffi_setup(p)
                if value_setup:
                    value_lines, value_arg = value_setup
                    lines.extend(value_lines)
                    ffi_parts.append(value_arg)
                else:
                    ffi_parts.append(sn)

        args_str = ", ".join(ffi_parts)
        if ret == "void":
            lines.append(f"        self._lib.{ffi_fn}({args_str})")
        else:
            lines.append(f"        return self._lib.{ffi_fn}({args_str})")
    elif "out_params" in mmap and "returns_struct" in mmap:
        ffi_fn = mmap["ffi"]
        no_ctx = mmap.get("no_context", False)
        status_nullable_struct = bool(mmap.get("status_nullable_struct"))
        status_struct = bool(mmap.get("status_struct"))
        entity_set = set(mmap.get("entity_params", []))
        enum_set = set((mmap.get("enum_params") or {}).keys())
        string_set = set(mmap.get("string_params", []))
        uses_ptr_len = ffi_uses_ptr_len(ffi_fn)
        out_params = mmap["out_params"]

        for op in out_params:
            ctype = py_out_var_ctype(op["type"])
            lines.append(f"        _{to_snake(op['name'])} = {ctype}()")

        ffi_parts = [] if no_ctx else ["self._ctx"]
        for p in params:
            pn = p["name"]
            sn = to_snake(pn)
            if pn in entity_set:
                ffi_parts.append(f"{sn}._bits")
            elif p["type"] == "string" and pn in string_set and uses_ptr_len:
                lines.append(f"        _{sn}_bytes = {sn}.encode('utf-8')")
                lines.append(f"        _{sn}_buf = ctypes.create_string_buffer(_{sn}_bytes, len(_{sn}_bytes))")
                ffi_parts.append(f"ctypes.cast(_{sn}_buf, ctypes.POINTER(ctypes.c_uint8))")
                ffi_parts.append(f"len(_{sn}_bytes)")
            elif pn in enum_set or p["type"] in schema.get("enums", {}):
                ffi_parts.append(f"int({sn})")
            else:
                ffi_parts.append(sn)
        ffi_parts.extend(f"ctypes.byref(_{to_snake(op['name'])})" for op in out_params)

        if status_nullable_struct or status_struct:
            lines.append(f"        _status = self._lib.{ffi_fn}({', '.join(ffi_parts)})")
            lines.append("        if _status < 0:")
            if uses_network_status_errors and ffi_fn.startswith("goud_network_"):
                lines.append(f"            self._raise_network_error_or_runtime(f'{ffi_fn} failed with status {{_status}}')")
            else:
                lines.append(f"            raise RuntimeError(f'{ffi_fn} failed with status {{_status}}')")
            if status_nullable_struct:
                lines.append("        if _status == 0:")
                lines.append("            return None")
        else:
            lines.append(f"        self._lib.{ffi_fn}({', '.join(ffi_parts)})")

        struct_name = mmap["returns_struct"]
        rs_fields = schema["types"][struct_name]["fields"]
        op0_type = out_params[0]["type"]
        if len(out_params) == 1 and (op0_type in schema.get("types", {}) or op0_type.startswith("Ffi") or op0_type.startswith("Goud")):
            src = f"_{to_snake(out_params[0]['name'])}"
            field_args = ", ".join(f"{src}.{to_snake(f['name'])}" for f in rs_fields)
        else:
            field_args = ", ".join(f"_{to_snake(op['name'])}.value" for op in out_params)
        lines.append(f"        return {struct_name}({field_args})")
    else:
        _emit_tool_method_body_tail(mmap, params, ret, lines, uses_network_status_errors)


def _emit_tool_method_body_tail(
    mmap: dict,
    params: list[dict],
    ret: str,
    lines: list[str],
    uses_network_status_errors: bool,
) -> None:
    if "out_params" in mmap and "returns_scalar" in mmap:
        ffi_fn = mmap["ffi"]
        no_ctx = mmap.get("no_context", False)
        entity_set = set(mmap.get("entity_params", []))
        enum_set = set((mmap.get("enum_params") or {}).keys())
        out = mmap["out_params"][0]
        ctype = py_out_var_ctype(out["type"])
        lines.append(f"        _{out['name']} = {ctype}()")

        ffi_parts = [] if no_ctx else ["self._ctx"]
        for p in params:
            pn = p["name"]
            sn = to_snake(pn)
            if pn in entity_set:
                ffi_parts.append(f"{sn}._bits")
            elif pn in enum_set or p["type"] in schema.get("enums", {}):
                ffi_parts.append(f"int({sn})")
            else:
                ffi_parts.append(sn)
        ffi_parts.append(f"ctypes.byref(_{out['name']})")
        lines.append(f"        self._lib.{ffi_fn}({', '.join(ffi_parts)})")
        lines.append(f"        return _{out['name']}.value")
    elif "out_params" in mmap:
        for op in mmap["out_params"]:
            lines.append(f"        _{op['name']} = ctypes.c_float(0.0)")
        out_refs = ", ".join(f"ctypes.byref(_{op['name']})" for op in mmap["out_params"])
        lines.append(f"        self._lib.{mmap['ffi']}(self._ctx, {out_refs})")
        out_vals = ", ".join(f"_{op['name']}.value" for op in mmap["out_params"])
        lines.append(f"        return Vec2({out_vals})")
    elif mmap.get("out_buffer"):
        _emit_out_buffer_method(mmap, params, lines, uses_network_status_errors)
    elif "enum_params" in mmap:
        enum_arg = list(mmap["enum_params"].keys())[0]
        lines.append(f"        return self._lib.{mmap['ffi']}(self._ctx, int({to_snake(enum_arg)}))")
    elif "ffi" in mmap:
        _emit_plain_ffi_method(mmap, params, ret, lines)


def _emit_out_buffer_method(mmap: dict, params: list[dict], lines: list[str], uses_network_status_errors: bool) -> None:
    ffi_fn = mmap["ffi"]
    no_ctx = mmap.get("no_context", False)
    entity_set = set(mmap.get("entity_params", []))
    enum_set = set((mmap.get("enum_params") or {}).keys())
    returns_struct = mmap.get("returns_struct")
    status_nullable_struct = bool(mmap.get("status_nullable_struct"))

    if not no_ctx:
        lines.append("        _caps = _ffi_module.FfiNetworkCapabilities()")
        lines.append("        self._lib.goud_provider_network_capabilities(self._ctx, ctypes.byref(_caps))")
        lines.append("        _buf_len = int(_caps.max_message_size) if _caps.max_message_size else 65536")
    else:
        lines.append("        _buf_len = 65536")
    lines.append("        _out_buf = (ctypes.c_uint8 * _buf_len)()")
    lines.append("        _out_peer_id = ctypes.c_uint64()")

    ffi_parts = [] if no_ctx else ["self._ctx"]
    for p in params:
        pn = p["name"]
        sn = to_snake(pn)
        if pn in entity_set:
            ffi_parts.append(f"{sn}._bits")
        elif pn in enum_set or p["type"] in schema.get("enums", {}):
            ffi_parts.append(f"int({sn})")
        else:
            ffi_parts.append(sn)
    ffi_parts.extend([
        "ctypes.cast(_out_buf, ctypes.POINTER(ctypes.c_uint8))",
        "_buf_len",
        "ctypes.byref(_out_peer_id)",
    ])
    lines.append(f"        _status = self._lib.{ffi_fn}({', '.join(ffi_parts)})")
    lines.append("        if _status < 0:")
    if uses_network_status_errors and ffi_fn.startswith("goud_network_"):
        lines.append(f"            self._raise_network_error_or_runtime(f'{ffi_fn} failed with status {{_status}}')")
    else:
        lines.append(f"            raise RuntimeError(f'{ffi_fn} failed with status {{_status}}')")
    if returns_struct and status_nullable_struct:
        lines.append("        if _status == 0:")
        lines.append("            return None")
    else:
        lines.append("        if _status == 0:")
        lines.append("            return b''")
    if returns_struct:
        rs_fields = schema["types"][returns_struct]["fields"]
        field_args = []
        for field in rs_fields:
            if field["type"] in ("bytes", "u8[]"):
                field_args.append("bytes(_out_buf[:_status])")
            elif field["name"] == "peerId":
                field_args.append("_out_peer_id.value")
            else:
                field_args.append("0" if field["type"] in ("u64", "u32", "i32") else "0.0")
        lines.append(f"        return {returns_struct}({', '.join(field_args)})")
    else:
        lines.append("        return bytes(_out_buf[:_status])")


def _emit_plain_ffi_method(mmap: dict, params: list[dict], ret: str, lines: list[str]) -> None:
    ffi_fn = mmap["ffi"]
    no_ctx = mmap.get("no_context", False)
    if "append_args" in mmap:
        extra = ", ".join(str(a) for a in mmap["append_args"])
        lines.append(f"        self._lib.{ffi_fn}(self._ctx, {extra})")
        return

    if ffi_uses_ptr_len(ffi_fn) and any(p["type"] in ("string", "bytes") for p in params):
        for p in params:
            sn = to_snake(p["name"])
            if p["type"] == "string":
                lines.append(f"        _{sn}_bytes = {sn}.encode('utf-8')")
                lines.append(f"        _{sn}_buf = ctypes.create_string_buffer(_{sn}_bytes, len(_{sn}_bytes))")
            elif p["type"] == "bytes":
                lines.append(f"        _{sn}_buf = (ctypes.c_uint8 * len({sn})).from_buffer_copy({sn})")

        ffi_parts = ["self._ctx"]
        for p in params:
            sn = to_snake(p["name"])
            if p["type"] == "string":
                ffi_parts.append(f"ctypes.cast(_{sn}_buf, ctypes.POINTER(ctypes.c_uint8))")
                ffi_parts.append(f"len(_{sn}_bytes)")
            elif p["type"] == "bytes":
                ffi_parts.append(f"ctypes.cast(_{sn}_buf, ctypes.POINTER(ctypes.c_uint8))")
                ffi_parts.append(f"len({sn})")
            elif p["type"] in schema.get("enums", {}):
                ffi_parts.append(f"int({sn})")
            else:
                ffi_parts.append(sn)

        args_str = ", ".join(ffi_parts)
        if ret == "void":
            lines.append(f"        self._lib.{ffi_fn}({args_str})")
        elif mmap.get("returns_bool_from_i32"):
            lines.append(f"        return self._lib.{ffi_fn}({args_str}) != 0")
        else:
            lines.append(f"        return self._lib.{ffi_fn}({args_str})")
        return

    ffi_args = []
    setup_lines = []
    for p in params:
        sn = to_snake(p["name"])
        if p["type"] in schema.get("enums", {}):
            ffi_args.append(f"int({sn})")
            continue
        value_setup = py_value_param_ffi_setup(p)
        if value_setup:
            value_lines, value_arg = value_setup
            setup_lines.extend(value_lines)
            ffi_args.append(value_arg)
            continue
        ffi_args.append(sn)
    lines.extend(setup_lines)
    args_str = ", ".join(([] if no_ctx else ["self._ctx"]) + ffi_args)
    if ret == "void":
        lines.append(f"        self._lib.{ffi_fn}({args_str})")
    elif mmap.get("returns_bool_from_i32"):
        lines.append(f"        return self._lib.{ffi_fn}({args_str}) != 0")
    else:
        lines.append(f"        return self._lib.{ffi_fn}({args_str})")
