"""Component and builder generation helpers for `_types.py`."""

from .context import PYTHON_TYPES, mapping, schema, to_snake
from .shared_helpers import (
    ffi_to_sdk_return,
    get_ffi_func_def,
    py_field_default,
    py_out_var_ctype,
    py_schema_return_type,
)


def gen_ffi_method_body(
    type_name: str,
    ffi_name: str,
    fdef: dict,
    method_mapping: dict,
    schema_method: dict | None,
) -> list[str]:
    """Generate the method body lines for an FFI-backed type method."""
    body: list[str] = []
    params = fdef["params"]
    ret = fdef["returns"]
    self_param = method_mapping.get("self_param", "")
    is_static = method_mapping.get("static", False)

    extra_params = params[1:] if self_param and params else params

    ffi_args = []
    needs_sync_before = False
    if self_param:
        if "*mut" in self_param or "*const" in self_param:
            ffi_args.append("ctypes.byref(self._ffi)")
            needs_sync_before = True
        else:
            ffi_args.append("self._ffi")

    out_params = method_mapping.get("out_params")
    out_param_names = {to_snake(op["name"]) for op in (out_params or [])}

    for p in extra_params:
        pname = to_snake(p["name"])
        if pname in out_param_names:
            continue
        ffi_args.append(pname)

    if needs_sync_before:
        body.append("        self._sync_to_ffi()")

    call = f"_lib.{ffi_name}({', '.join(ffi_args)})"
    returns_struct = method_mapping.get("returns_struct")

    if out_params and returns_struct:
        for op in out_params:
            ctype = py_out_var_ctype(op["type"])
            body.append(f"        _{to_snake(op['name'])} = {ctype}()")

        for op in out_params:
            ffi_args.append(f"ctypes.byref(_{to_snake(op['name'])})")

        body.append(f"        _lib.{ffi_name}({', '.join(ffi_args)})")

        rs_fields = schema["types"][returns_struct]["fields"]
        if len(out_params) == 1 and (
            out_params[0]["type"] in schema.get("types", {})
            or out_params[0]["type"].startswith("Ffi")
            or out_params[0]["type"].startswith("Goud")
        ):
            src = f"_{to_snake(out_params[0]['name'])}"
            field_args = ", ".join(f"{src}.{to_snake(f['name'])}" for f in rs_fields)
        else:
            field_args = ", ".join(f"_{to_snake(op['name'])}.value" for op in out_params)
        body.append(f"        return {returns_struct}({field_args})")
        return body

    mutates = schema_method.get("mutates", False) if schema_method else False

    if ret == "void":
        body.append(f"        {call}")
        if mutates and self_param and ("*mut" in self_param):
            body.append("        self._sync_from_ffi()")
    elif ret == "FfiTransform2D" and not is_static:
        body.append(f"        ffi = {call}")
        body.append("        return Transform2D._from_ffi(ffi)")
    elif ret == "FfiSprite" and not is_static:
        body.append(f"        ffi = {call}")
        body.append("        return Sprite._from_ffi(ffi)")
    elif ret == "FfiVec2":
        body.append(f"        ffi = {call}")
        body.append("        return Vec2(ffi.x, ffi.y)")
    elif ret == "FfiColor":
        body.append(f"        ffi = {call}")
        body.append("        return Color(ffi.r, ffi.g, ffi.b, ffi.a)")
    elif ret == "FfiRect":
        body.append(f"        ffi = {call}")
        body.append("        return Rect(ffi.x, ffi.y, ffi.width, ffi.height)")
    elif ret == "FfiMat3x3":
        body.append(f"        ffi = {call}")
        body.append("        return Mat3x3(list(ffi.m))")
    elif ret in ("f32", "u32", "u64", "bool"):
        body.append(f"        return {call}")
    else:
        body.append(f"        return {call}")

    return body


def gen_builder_class(type_name: str, builder_defs: dict, schema_builder: dict, lines: list[str]) -> None:
    """Generate a builder class for a component type."""
    builder_class = f"{type_name}Builder"
    lines.append(f"class {builder_class}:")
    if schema_builder.get("doc"):
        lines.append(f'    """{schema_builder["doc"]}"""')
    lines.append("")

    schema_methods = {to_snake(m["name"]): m for m in schema_builder.get("methods", [])}

    for bm_name, bm_map in builder_defs.items():
        py_name = to_snake(bm_name)
        ffi_fn = bm_map["ffi"]
        fdef = get_ffi_func_def(ffi_fn)
        if not fdef:
            continue

        self_param = bm_map.get("self_param", "")
        schema_meth = schema_methods.get(py_name)

        ffi_params = fdef["params"]
        extra = ffi_params[1:] if self_param and ffi_params else ffi_params

        param_parts = []
        if schema_meth and schema_meth.get("params"):
            for sp in schema_meth["params"]:
                pn = to_snake(sp["name"])
                pt = sp.get("type", "f32")
                param_parts.append(f"{pn}: {PYTHON_TYPES.get(pt, 'float')}")
        else:
            for p in extra:
                pn = to_snake(p["name"])
                param_parts.append(f"{pn}: {PYTHON_TYPES.get(p['type'], 'float')}")

        if py_name == "free":
            lines.append(f"    def {py_name}(self):")
            if schema_meth and schema_meth.get("doc"):
                lines.append(f'        """{schema_meth["doc"]}"""')
            lines.append("        if self._ptr:")
            lines.append("            _ensure_ffi()")
            lines.append(f"            _lib.{ffi_fn}(self._ptr)")
            lines.append("            self._ptr = None")
            lines.append("")
            continue

        if py_name == "build":
            lines.append(f"    def {py_name}(self) -> '{type_name}':")
            if schema_meth and schema_meth.get("doc"):
                lines.append(f'        """{schema_meth["doc"]}"""')
            lines.append("        if not self._ptr:")
            lines.append(f"            raise RuntimeError('{builder_class} already consumed')")
            lines.append("        _ensure_ffi()")
            lines.append(f"        ffi = _lib.{ffi_fn}(self._ptr)")
            lines.append("        self._ptr = None")
            lines.append(f"        return {type_name}._from_ffi(ffi)")
            lines.append("")
            continue

        if not self_param:
            sig = ", ".join(param_parts)
            lines.append("    @staticmethod")
            lines.append(f"    def {py_name}({sig}) -> '{builder_class}':")
            if schema_meth and schema_meth.get("doc"):
                lines.append(f'        """{schema_meth["doc"]}"""')
            lines.append("        _ensure_ffi()")
            ffi_args = ", ".join(to_snake(p["name"]) for p in extra)
            call = f"_lib.{ffi_fn}({ffi_args})" if ffi_args else f"_lib.{ffi_fn}()"
            lines.append(f"        obj = {builder_class}.__new__({builder_class})")
            lines.append(f"        obj._ptr = {call}")
            lines.append("        return obj")
            lines.append("")
            continue

        sig = ", ".join(["self"] + param_parts)
        lines.append(f"    def {py_name}({sig}) -> '{builder_class}':")
        if schema_meth and schema_meth.get("doc"):
            lines.append(f'        """{schema_meth["doc"]}"""')
        lines.append("        _ensure_ffi()")
        ffi_call_args = ["self._ptr"] + [to_snake(p["name"]) for p in extra]
        lines.append(f"        self._ptr = _lib.{ffi_fn}({', '.join(ffi_call_args)})")
        lines.append("        return self")
        lines.append("")

    free_ffi = builder_defs.get("free", {}).get("ffi")
    if free_ffi:
        lines.append("    def __del__(self):")
        lines.append("        if hasattr(self, '_ptr') and self._ptr:")
        lines.append("            _ensure_ffi()")
        lines.append(f"            _lib.{free_ffi}(self._ptr)")
        lines.append("            self._ptr = None")
        lines.append("")

    lines.append("    def __repr__(self):")
    lines.append(f'        return f"{builder_class}(ptr={{self._ptr}})"')
    lines.append("")


def gen_component_type(type_name: str, type_def: dict, lines: list[str]) -> None:
    """Generate a component wrapper class with FFI-backed methods."""
    fields = type_def.get("fields", [])
    field_names = [to_snake(f["name"]) for f in fields]
    type_methods = mapping.get("type_methods", {}).get(type_name, {})
    ffi_type_info = mapping.get("ffi_types", {}).get(type_name, {})
    ffi_struct_name = ffi_type_info.get("ffi_name", f"Ffi{type_name}")

    lines.append(f"class {type_name}:")
    if type_def.get("doc"):
        lines.append(f'    """{type_def["doc"]}"""')

    params = ", ".join(f"{to_snake(f['name'])}: {py_field_default(f)}" for f in fields)
    lines.append(f"    def __init__(self, {params}):")
    for field in fields:
        fn = to_snake(field["name"])
        ft = field.get("type", "f32")
        if ft in schema.get("types", {}) and schema["types"][ft].get("kind") == "value":
            lines.append(f"        self.{fn} = {fn} if {fn} is not None else {ft}()")
        else:
            lines.append(f"        self.{fn} = {fn}")
    lines.append("        self._ffi = None")
    lines.append("")

    lines.append("    @classmethod")
    lines.append(f"    def _from_ffi(cls, ffi) -> '{type_name}':")
    lines.append("        obj = cls.__new__(cls)")
    for fn in field_names:
        lines.append(f"        obj.{fn} = ffi.{fn}")
    lines.append("        obj._ffi = ffi")
    lines.append("        return obj")
    lines.append("")

    lines.append("    def _sync_to_ffi(self):")
    lines.append("        _ensure_ffi()")
    lines.append("        if self._ffi is None:")
    lines.append(f"            self._ffi = _ffi_module.{ffi_struct_name}()")
    for fn in field_names:
        lines.append(f"        self._ffi.{fn} = self.{fn}")
    lines.append("")

    lines.append("    def _sync_from_ffi(self):")
    for fn in field_names:
        lines.append(f"        self.{fn} = self._ffi.{fn}")
    lines.append("")

    factories = type_methods.get("factories", {})
    schema_factories = {to_snake(f["name"]): f for f in type_def.get("factories", [])}

    for fact_name, fact_map in factories.items():
        py_name = to_snake(fact_name)
        ffi_fn = fact_map["ffi"]
        fdef = get_ffi_func_def(ffi_fn)
        if not fdef:
            continue

        ffi_params = fdef["params"]
        if ffi_params:
            sfact = schema_factories.get(py_name, {})
            sfact_args = sfact.get("args", [])
            if sfact_args:
                arg_str = ", ".join(
                    f"{to_snake(a['name'])}: {PYTHON_TYPES.get(a.get('type', 'f32'), 'float')}"
                    for a in sfact_args
                )
                ffi_call_args = ", ".join(to_snake(a["name"]) for a in sfact_args)
            else:
                arg_str = ", ".join(
                    f"{to_snake(p['name'])}: {PYTHON_TYPES.get(p['type'], 'float')}"
                    for p in ffi_params
                )
                ffi_call_args = ", ".join(to_snake(p["name"]) for p in ffi_params)
            lines.append("    @staticmethod")
            lines.append(f"    def {py_name}({arg_str}) -> '{type_name}':")
            lines.append("        _ensure_ffi()")
            lines.append(f"        ffi = _lib.{ffi_fn}({ffi_call_args})")
        else:
            lines.append("    @staticmethod")
            lines.append(f"    def {py_name}() -> '{type_name}':")
            lines.append("        _ensure_ffi()")
            lines.append(f"        ffi = _lib.{ffi_fn}()")

        lines.append(f"        return {type_name}._from_ffi(ffi)")
        lines.append("")

    methods = type_methods.get("methods", {})
    schema_methods = {to_snake(m["name"]): m for m in type_def.get("methods", [])}

    for meth_name, meth_map in methods.items():
        py_name = to_snake(meth_name)
        ffi_fn = meth_map["ffi"]
        fdef = get_ffi_func_def(ffi_fn)
        if not fdef:
            continue

        is_static = meth_map.get("static", False)
        self_param = meth_map.get("self_param", "")
        schema_meth = schema_methods.get(py_name)

        ffi_params = fdef["params"]
        extra = ffi_params[1:] if self_param and ffi_params else ffi_params

        param_parts = []
        if schema_meth and "params" in schema_meth:
            for sp in schema_meth["params"]:
                pn = to_snake(sp["name"])
                pt = sp.get("type", "f32")
                if pt in schema["types"]:
                    param_parts.append(f"{pn}: '{pt}'")
                else:
                    param_parts.append(f"{pn}: {PYTHON_TYPES.get(pt, 'float')}")
        else:
            for p in extra:
                pn = to_snake(p["name"])
                param_parts.append(f"{pn}: {PYTHON_TYPES.get(p['type'], 'float')}")

        ret_type = (
            py_schema_return_type(schema_meth["returns"], type_name)
            if schema_meth and schema_meth.get("returns")
            else ffi_to_sdk_return(fdef["returns"], type_name)
        )

        if is_static:
            lines.append("    @staticmethod")
            lines.append(f"    def {py_name}({', '.join(param_parts)}) -> {ret_type}:")
        else:
            lines.append(f"    def {py_name}({', '.join(['self'] + param_parts)}) -> {ret_type}:")

        if schema_meth and schema_meth.get("doc"):
            lines.append(f'        """{schema_meth["doc"]}"""')
        lines.append("        _ensure_ffi()")

        if self_param and "*" not in self_param:
            lines.append("        self._sync_to_ffi()")
            ffi_args = ["self._ffi"]
            for p in extra:
                pname = to_snake(p["name"])
                if p["type"] in ("FfiTransform2D", "FfiSprite", "FfiColor"):
                    ffi_args.append(f"{pname}._ffi")
                else:
                    ffi_args.append(pname)
            call = f"_lib.{ffi_fn}({', '.join(ffi_args)})"

            if fdef["returns"].startswith("Ffi"):
                lines.append(f"        ffi = {call}")
                lines.append(f"        return {type_name}._from_ffi(ffi)")
            else:
                lines.append(f"        return {call}")
        else:
            lines.extend(gen_ffi_method_body(type_name, ffi_fn, fdef, meth_map, schema_meth))

        lines.append("")

    lines.append("    def __repr__(self):")
    vals = ", ".join(f"{fn}={{self.{fn}}}" for fn in field_names)
    lines.append(f'        return f"{type_name}({vals})"')
    lines.append("")

    builder_defs = type_methods.get("builder")
    schema_builder = type_def.get("builder")
    if builder_defs and schema_builder:
        gen_builder_class(type_name, builder_defs, schema_builder, lines)
