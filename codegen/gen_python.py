#!/usr/bin/env python3
"""Generates the complete Python SDK from the universal schema.

Produces:
  sdks/python/goud_engine/__init__.py
  sdks/python/goud_engine/_ffi.g.py
  sdks/python/goud_engine/_types.g.py
  sdks/python/goud_engine/_keys.g.py
  sdks/python/goud_engine/_game.g.py
"""

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))
from sdk_common import (
    HEADER_COMMENT, SDKS_DIR, load_schema, load_ffi_mapping, load_errors,
    to_snake, to_screaming_snake, write_generated, CTYPES_MAP, PYTHON_TYPES,
)

OUT = SDKS_DIR / "python" / "goud_engine" / "generated"
schema = load_schema()
mapping = load_ffi_mapping()


def _resolve_ffi_return(ret: str) -> str:
    """Map an FFI return type string to its ctypes restype."""
    if ret == "void":
        return "None"
    ct = CTYPES_MAP.get(ret)
    if ct:
        return ct
    return "ctypes.c_uint64"


def _resolve_ffi_param(ptype: str) -> str:
    """Map an FFI param type string to its ctypes argtype."""
    ct = CTYPES_MAP.get(ptype)
    if ct:
        return ct
    return "ctypes.c_uint64"


# ── _ffi.g.py ───────────────────────────────────────────────────────

def gen_ffi():
    lines = [
        f'"""{HEADER_COMMENT}"""',
        "", "import ctypes", "import platform", "import os", "from pathlib import Path", "",
        "# ── Library loading ──",
        "",
        "def _load_library():",
        '    """Load the GoudEngine shared library."""',
        "    system = platform.system()",
        '    if system == "Darwin":',
        '        ext, prefix = ".dylib", "lib"',
        '    elif system == "Linux":',
        '        ext, prefix = ".so", "lib"',
        '    elif system == "Windows":',
        '        ext, prefix = ".dll", ""',
        "    else:",
        f'        raise OSError(f"Unsupported platform: {{system}}")',
        "",
        '    name = f"{prefix}goud_engine{ext}"',
        "    search = [",
        '        Path(__file__).parent / name,',
        '        Path(__file__).parent.parent / name,',
        '        Path(__file__).parent.parent.parent.parent.parent / "target" / "release" / name,',
        '        Path(__file__).parent.parent.parent.parent.parent / "target" / "debug" / name,',
        '        Path(os.environ.get("GOUD_ENGINE_LIB", "")) / name,',
        "    ]",
        "    for p in search:",
        "        if p.exists():",
        "            return ctypes.cdll.LoadLibrary(str(p))",
        f'    raise OSError(f"Could not find {{name}}. Set GOUD_ENGINE_LIB env var.")',
        "",
        "_lib = _load_library()",
        "",
    ]

    lines.append("# ── FFI struct types ──")
    lines.append("")
    lines.append("class GoudContextId(ctypes.Structure):")
    lines.append('    _fields_ = [("_bits", ctypes.c_uint64)]')
    lines.append("")
    lines.append("class GoudResult(ctypes.Structure):")
    lines.append('    _fields_ = [("code", ctypes.c_int32), ("success", ctypes.c_bool)]')
    lines.append("")

    # Map of ffi_type field names -> ctypes types for struct generation
    _FIELD_CTYPES = {
        "f32": "ctypes.c_float",
        "u32": "ctypes.c_uint32",
        "u64": "ctypes.c_uint64",
        "bool": "ctypes.c_bool",
        "i32": "ctypes.c_int32",
    }

    for type_name, type_def in mapping["ffi_types"].items():
        ffi_name = type_def["ffi_name"]
        if ffi_name == "u64":
            continue
        sdk_type = schema["types"].get(type_name)
        if not sdk_type or "fields" not in sdk_type:
            continue
        lines.append(f"class {ffi_name}(ctypes.Structure):")
        fields_list = []
        for f in sdk_type["fields"]:
            fn = to_snake(f["name"])
            ft = f.get("type", "f32")
            # Handle array types like f32[9]
            if "[" in ft:
                base = ft.split("[")[0]
                count = int(ft.split("[")[1].rstrip("]"))
                ct = _FIELD_CTYPES.get(base, "ctypes.c_float")
                fields_list.append(f'        ("{fn}", {ct} * {count})')
            else:
                ct = _FIELD_CTYPES.get(ft, "ctypes.c_float")
                fields_list.append(f'        ("{fn}", {ct})')
        lines.append("    _fields_ = [")
        lines.append(",\n".join(fields_list))
        lines.append("    ]")
        lines.append("")

    lines.append("# ── Function signatures ──")
    lines.append("")
    lines.append("def _setup():")

    for module, funcs in mapping["ffi_functions"].items():
        if not isinstance(funcs, dict):
            continue
        optional = funcs.get("_feature") == "optional"
        lines.append(f"    # {module}")
        if optional:
            lines.append("    try:")
        indent = "        " if optional else "    "
        for fname, fdef in funcs.items():
            if fname.startswith("_"):
                continue
            # Skip alias entries
            if fdef.get("alias_of"):
                # Still declare it since it exists as a symbol
                alias_fdef = funcs.get(fdef["alias_of"], fdef)
                argtypes = [_resolve_ffi_param(p["type"]) for p in alias_fdef.get("params", fdef.get("params", []))]
                ret = alias_fdef.get("returns", fdef.get("returns", "void"))
                restype = _resolve_ffi_return(ret)
                at_str = ", ".join(argtypes) if argtypes else ""
                lines.append(f"{indent}_lib.{fname}.argtypes = [{at_str}]")
                lines.append(f"{indent}_lib.{fname}.restype = {restype}")
                continue

            argtypes = [_resolve_ffi_param(p["type"]) for p in fdef["params"]]
            restype = _resolve_ffi_return(fdef["returns"])
            at_str = ", ".join(argtypes) if argtypes else ""
            lines.append(f"{indent}_lib.{fname}.argtypes = [{at_str}]")
            lines.append(f"{indent}_lib.{fname}.restype = {restype}")
        if optional:
            lines.append("    except AttributeError:")
            lines.append("        pass  # feature not compiled in")
        lines.append("")

    lines.append("_setup()")
    lines.append("")
    lines.append("def get_lib():")
    lines.append("    return _lib")
    lines.append("")

    write_generated(OUT / "_ffi.py", "\n".join(lines))


# ── _keys.g.py ──────────────────────────────────────────────────────

def gen_keys():
    lines = [f'"""{HEADER_COMMENT}"""', ""]

    for enum_name, enum_def in schema["enums"].items():
        class_name = enum_name
        lines.append(f"class {class_name}:")
        if enum_def.get("doc"):
            lines.append(f'    """{enum_def["doc"]}"""')
        for vname, vval in enum_def["values"].items():
            lines.append(f"    {to_screaming_snake(vname)} = {vval}")
        lines.append("")

    write_generated(OUT / "_keys.py", "\n".join(lines))


# ── _types.g.py ─────────────────────────────────────────────────────

def _py_field_default(field: dict) -> str:
    """Return the Python type annotation and default value for a schema field."""
    t = field.get("type", "f32")
    if t == "bool":
        return "bool = False"
    elif t in ("u32", "i32", "u64", "i64"):
        return "int = 0"
    else:
        return "float = 0.0"


def _get_ffi_func_def(ffi_name: str) -> dict | None:
    """Look up an FFI function definition from the mapping by name."""
    for _module, funcs in mapping["ffi_functions"].items():
        if not isinstance(funcs, dict):
            continue
        if ffi_name in funcs:
            return funcs[ffi_name]
    return None


def _ffi_uses_ptr_len(ffi_name: str) -> bool:
    """Check if the FFI function uses *const u8 ptr+len for string params."""
    fdef = _get_ffi_func_def(ffi_name)
    if not fdef:
        return False
    param_types = [p.get("type", "") for p in fdef.get("params", [])]
    return "*const u8" in param_types


def _ffi_to_sdk_return(ffi_returns: str, type_name: str) -> str:
    """Map an FFI return type to the SDK type string for annotations.

    Component types (Transform2D, Sprite) are returned as forward-ref strings
    so they work inside class bodies where the name is not yet defined.
    """
    if ffi_returns == "void":
        return "None"
    if ffi_returns == "f32":
        return "float"
    if ffi_returns == "u32" or ffi_returns == "u64":
        return "int"
    if ffi_returns == "bool":
        return "bool"
    if ffi_returns == "FfiVec2":
        return "Vec2"
    if ffi_returns == "FfiColor":
        return "Color"
    if ffi_returns == "FfiRect":
        return "Rect"
    if ffi_returns == "FfiMat3x3":
        return "Mat3x3"
    if ffi_returns == "FfiTransform2D":
        return "'Transform2D'"
    if ffi_returns == "FfiSprite":
        return "'Sprite'"
    return f"'{type_name}'"


def _gen_ffi_method_body(
    type_name: str, mname: str, ffi_name: str, fdef: dict,
    method_mapping: dict, schema_method: dict | None,
) -> list[str]:
    """Generate the method body lines for an FFI-backed type method."""
    body = []
    params = fdef["params"]
    ret = fdef["returns"]
    self_param = method_mapping.get("self_param", "")
    is_static = method_mapping.get("static", False)

    # Determine non-self params (skip the first param if it's the self pointer)
    if self_param and params:
        extra_params = params[1:]
    else:
        extra_params = params

    # Build the FFI call arguments
    ffi_args = []
    needs_sync_before = False
    if self_param:
        if "*mut" in self_param or "*const" in self_param:
            ffi_args.append("ctypes.byref(self._ffi)")
            needs_sync_before = True
        else:
            # By-value: pass the struct directly
            ffi_args.append("self._ffi")

    for p in extra_params:
        ffi_args.append(to_snake(p["name"]))

    args_str = ", ".join(ffi_args)
    call = f"_lib.{ffi_name}({args_str})"

    # Ensure _ffi struct exists and is synced before pointer-based calls
    if needs_sync_before:
        body.append("        self._sync_to_ffi()")

    mutates = schema_method.get("mutates", False) if schema_method else False

    if ret == "void":
        body.append(f"        {call}")
        if mutates and self_param and ("*mut" in self_param):
            body.append("        self._sync_from_ffi()")
    elif ret == "FfiTransform2D" and not is_static:
        body.append(f"        ffi = {call}")
        body.append(f"        return Transform2D._from_ffi(ffi)")
    elif ret == "FfiSprite" and not is_static:
        body.append(f"        ffi = {call}")
        body.append(f"        return Sprite._from_ffi(ffi)")
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


def _gen_component_type(type_name: str, type_def: dict, lines: list):
    """Generate a component wrapper class with FFI-backed methods."""
    fields = type_def.get("fields", [])
    field_names = [to_snake(f["name"]) for f in fields]
    type_methods = mapping.get("type_methods", {}).get(type_name, {})
    ffi_type_info = mapping.get("ffi_types", {}).get(type_name, {})
    ffi_struct_name = ffi_type_info.get("ffi_name", f"Ffi{type_name}")

    lines.append(f"class {type_name}:")
    if type_def.get("doc"):
        lines.append(f'    """{type_def["doc"]}"""')

    # __init__ takes Python values
    params = ", ".join(
        f"{to_snake(f['name'])}: {_py_field_default(f)}" for f in fields
    )
    lines.append(f"    def __init__(self, {params}):")
    for fn in field_names:
        lines.append(f"        self.{fn} = {fn}")
    lines.append("        self._ffi = None")
    lines.append("")

    # _from_ffi classmethod
    lines.append("    @classmethod")
    lines.append(f"    def _from_ffi(cls, ffi) -> '{type_name}':")
    lines.append(f"        obj = cls.__new__(cls)")
    for fn in field_names:
        lines.append(f"        obj.{fn} = ffi.{fn}")
    lines.append(f"        obj._ffi = ffi")
    lines.append("        return obj")
    lines.append("")

    # _sync_to_ffi: copy Python fields -> _ffi struct, creating it if needed
    lines.append("    def _sync_to_ffi(self):")
    lines.append("        _ensure_ffi()")
    lines.append(f"        if self._ffi is None:")
    lines.append(f"            self._ffi = _ffi_module.{ffi_struct_name}()")
    for fn in field_names:
        lines.append(f"        self._ffi.{fn} = self.{fn}")
    lines.append("")

    # _sync_from_ffi: copy _ffi struct -> Python fields
    lines.append("    def _sync_from_ffi(self):")
    for fn in field_names:
        lines.append(f"        self.{fn} = self._ffi.{fn}")
    lines.append("")

    # Factory methods from type_methods
    factories = type_methods.get("factories", {})
    schema_factories = {to_snake(f["name"]): f for f in type_def.get("factories", [])}

    for fact_name, fact_map in factories.items():
        py_name = to_snake(fact_name)
        ffi_fn = fact_map["ffi"]
        fdef = _get_ffi_func_def(ffi_fn)
        if not fdef:
            continue

        ffi_params = fdef["params"]
        if ffi_params:
            # Get schema factory for arg naming
            sfact = schema_factories.get(py_name, {})
            sfact_args = sfact.get("args", [])
            if sfact_args:
                arg_str = ", ".join(
                    f"{to_snake(a['name'])}: "
                    f"{PYTHON_TYPES.get(a.get('type', 'f32'), 'float')}"
                    for a in sfact_args
                )
                ffi_call_args = ", ".join(to_snake(a["name"]) for a in sfact_args)
            else:
                # Fall back to FFI param names
                arg_str = ", ".join(
                    f"{to_snake(p['name'])}: "
                    f"{PYTHON_TYPES.get(p['type'], 'float')}"
                    for p in ffi_params
                )
                ffi_call_args = ", ".join(
                    to_snake(p["name"]) for p in ffi_params
                )
            lines.append("    @staticmethod")
            lines.append(
                f"    def {py_name}({arg_str}) -> '{type_name}':"
            )
            lines.append("        _ensure_ffi()")
            lines.append(
                f"        ffi = _lib.{ffi_fn}({ffi_call_args})"
            )
        else:
            lines.append("    @staticmethod")
            lines.append(f"    def {py_name}() -> '{type_name}':")
            lines.append("        _ensure_ffi()")
            lines.append(f"        ffi = _lib.{ffi_fn}()")

        lines.append(f"        return {type_name}._from_ffi(ffi)")
        lines.append("")

    # Instance/static methods from type_methods
    methods = type_methods.get("methods", {})
    schema_methods = {
        to_snake(m["name"]): m for m in type_def.get("methods", [])
    }

    for meth_name, meth_map in methods.items():
        py_name = to_snake(meth_name)
        ffi_fn = meth_map["ffi"]
        fdef = _get_ffi_func_def(ffi_fn)
        if not fdef:
            continue

        is_static = meth_map.get("static", False)
        self_param = meth_map.get("self_param", "")
        schema_meth = schema_methods.get(py_name)

        # Build Python param list from extra FFI params
        ffi_params = fdef["params"]
        if self_param and ffi_params:
            extra = ffi_params[1:]
        else:
            extra = ffi_params

        param_parts = []
        if schema_meth and schema_meth.get("params"):
            for sp in schema_meth["params"]:
                pn = to_snake(sp["name"])
                pt = sp.get("type", "f32")
                if pt in schema["types"]:
                    param_parts.append(f"{pn}: '{pt}'")
                else:
                    param_parts.append(
                        f"{pn}: {PYTHON_TYPES.get(pt, 'float')}"
                    )
        else:
            for p in extra:
                pn = to_snake(p["name"])
                param_parts.append(
                    f"{pn}: {PYTHON_TYPES.get(p['type'], 'float')}"
                )

        ret_type = _ffi_to_sdk_return(fdef["returns"], type_name)

        if is_static:
            lines.append("    @staticmethod")
            sig = ", ".join(param_parts)
            lines.append(f"    def {py_name}({sig}) -> {ret_type}:")
        else:
            sig = ", ".join(["self"] + param_parts)
            lines.append(f"    def {py_name}({sig}) -> {ret_type}:")

        if schema_meth and schema_meth.get("doc"):
            lines.append(f'        """{schema_meth["doc"]}"""')
        lines.append("        _ensure_ffi()")

        # For methods that take self by value (non-pointer), we need to
        # sync to FFI first, pass by value, and construct result
        if self_param and "*" not in self_param:
            # By-value self: sync, call, return new instance
            lines.append("        self._sync_to_ffi()")
            ffi_args = ["self._ffi"]
            for p in extra:
                pname = to_snake(p["name"])
                # If the param is another component by value, pass its _ffi
                if p["type"] in ("FfiTransform2D", "FfiSprite", "FfiColor"):
                    ffi_args.append(f"{pname}._ffi")
                else:
                    ffi_args.append(pname)
            args_str = ", ".join(ffi_args)
            call = f"_lib.{ffi_fn}({args_str})"

            if fdef["returns"].startswith("Ffi"):
                lines.append(f"        ffi = {call}")
                lines.append(
                    f"        return {type_name}._from_ffi(ffi)"
                )
            elif fdef["returns"] in ("f32", "u32", "u64", "bool"):
                lines.append(f"        return {call}")
            else:
                lines.append(f"        return {call}")
        else:
            body = _gen_ffi_method_body(
                type_name, py_name, ffi_fn, fdef, meth_map, schema_meth,
            )
            lines.extend(body)

        lines.append("")

    # __repr__
    lines.append("    def __repr__(self):")
    vals = ", ".join(f"{fn}={{self.{fn}}}" for fn in field_names)
    lines.append(f'        return f"{type_name}({vals})"')
    lines.append("")

    # Builder class
    builder_defs = type_methods.get("builder")
    schema_builder = type_def.get("builder")
    if builder_defs and schema_builder:
        _gen_builder_class(type_name, builder_defs, schema_builder, lines)


def _gen_builder_class(
    type_name: str, builder_defs: dict, schema_builder: dict, lines: list,
):
    """Generate a builder class for a component type."""
    builder_class = f"{type_name}Builder"
    lines.append(f"class {builder_class}:")
    if schema_builder.get("doc"):
        lines.append(f'    """{schema_builder["doc"]}"""')
    lines.append("")

    # Schema builder methods indexed by snake name
    schema_methods = {
        to_snake(m["name"]): m for m in schema_builder.get("methods", [])
    }

    for bm_name, bm_map in builder_defs.items():
        py_name = to_snake(bm_name)
        ffi_fn = bm_map["ffi"]
        fdef = _get_ffi_func_def(ffi_fn)
        if not fdef:
            continue

        self_param = bm_map.get("self_param", "")
        consumes = bm_map.get("consumes", False)
        schema_meth = schema_methods.get(py_name)

        ffi_params = fdef["params"]
        if self_param and ffi_params:
            extra = ffi_params[1:]
        else:
            extra = ffi_params

        # Build param list from schema
        param_parts = []
        if schema_meth and schema_meth.get("params"):
            for sp in schema_meth["params"]:
                pn = to_snake(sp["name"])
                pt = sp.get("type", "f32")
                param_parts.append(
                    f"{pn}: {PYTHON_TYPES.get(pt, 'float')}"
                )
        else:
            for p in extra:
                pn = to_snake(p["name"])
                param_parts.append(
                    f"{pn}: {PYTHON_TYPES.get(p['type'], 'float')}"
                )

        if py_name == "free":
            # __del__ handles free; also expose explicit free()
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
            lines.append(
                "            raise RuntimeError("
                f"'{builder_class} already consumed')"
            )
            lines.append("        _ensure_ffi()")
            lines.append(f"        ffi = _lib.{ffi_fn}(self._ptr)")
            lines.append("        self._ptr = None")
            lines.append(f"        return {type_name}._from_ffi(ffi)")
            lines.append("")
            continue

        # Constructor methods (no self_param)
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

        # Chained builder methods (self_param is pointer)
        sig = ", ".join(["self"] + param_parts)
        lines.append(f"    def {py_name}({sig}) -> '{builder_class}':")
        if schema_meth and schema_meth.get("doc"):
            lines.append(f'        """{schema_meth["doc"]}"""')
        lines.append("        _ensure_ffi()")
        ffi_call_args = ["self._ptr"] + [
            to_snake(p["name"]) for p in extra
        ]
        args_str = ", ".join(ffi_call_args)
        lines.append(f"        self._ptr = _lib.{ffi_fn}({args_str})")
        lines.append("        return self")
        lines.append("")

    # __del__
    free_ffi = builder_defs.get("free", {}).get("ffi")
    if free_ffi:
        lines.append("    def __del__(self):")
        lines.append("        if hasattr(self, '_ptr') and self._ptr:")
        lines.append("            _ensure_ffi()")
        lines.append(f"            _lib.{free_ffi}(self._ptr)")
        lines.append("            self._ptr = None")
        lines.append("")

    # __repr__
    lines.append("    def __repr__(self):")
    lines.append(
        f'        return f"{builder_class}(ptr={{self._ptr}})"'
    )
    lines.append("")


def gen_types():
    lines = [
        f'"""{HEADER_COMMENT}"""',
        "",
        "import ctypes",
        "import math",
        "",
        "# Lazy FFI loading -- allows pure-Python types (Color, Vec2, etc.) to work",
        "# without the native library. FFI is only loaded on first use by component",
        "# types (Transform2D, Sprite) and their builders.",
        "_ffi_module = None",
        "_lib = None",
        "",
        "",
        "def _ensure_ffi():",
        '    """Load the FFI module and native library on first use."""',
        "    global _ffi_module, _lib",
        "    if _lib is not None:",
        "        return",
        "    from . import _ffi as ffi_mod",
        "    _ffi_module = ffi_mod",
        "    _lib = ffi_mod.get_lib()",
        "",
    ]

    for type_name, type_def in schema["types"].items():
        kind = type_def.get("kind")

        if kind == "handle":
            lines.append(f"class {type_name}:")
            if type_def.get("doc"):
                lines.append(f'    """{type_def["doc"]}"""')
            lines.append("    def __init__(self, bits: int):")
            lines.append("        self._bits = bits")
            lines.append("")
            for prop in type_def.get("properties", []):
                pname = to_snake(prop["name"])
                if pname == "index":
                    lines.append("    @property")
                    lines.append(f"    def {pname}(self) -> int:")
                    lines.append("        return self._bits & 0xFFFFFFFF")
                elif pname == "generation":
                    lines.append("    @property")
                    lines.append(f"    def {pname}(self) -> int:")
                    lines.append("        return self._bits >> 32")
                elif pname == "is_placeholder":
                    lines.append("    @property")
                    lines.append(f"    def {pname}(self) -> bool:")
                    lines.append(
                        "        return self._bits == 0xFFFFFFFFFFFFFFFF"
                    )
            for meth in type_def.get("methods", []):
                mname = to_snake(meth["name"])
                if mname == "to_bits":
                    lines.append(f"    def {mname}(self) -> int:")
                    lines.append("        return self._bits")
            lines.append("")
            lines.append("    def __repr__(self):")
            lines.append(
                '        return f"Entity({self.index}v{self.generation})"'
            )
            lines.append("")
            continue

        if kind == "component":
            _gen_component_type(type_name, type_def, lines)
            continue

        if kind != "value":
            continue

        # Value types: keep the existing pure-Python logic
        fields = type_def.get("fields", [])
        field_names = [to_snake(f["name"]) for f in fields]

        lines.append(f"class {type_name}:")
        if type_def.get("doc"):
            lines.append(f'    """{type_def["doc"]}"""')

        params = ", ".join(
            f"{to_snake(f['name'])}: {_py_field_default(f)}" for f in fields
        )
        lines.append(f"    def __init__(self, {params}):")
        for fn in field_names:
            lines.append(f"        self.{fn} = {fn}")
        lines.append("")

        for factory in type_def.get("factories", []):
            fname = to_snake(factory["name"])
            fargs = factory.get("args", [])
            val = factory.get("value")
            if val and not fargs:
                val_str = ", ".join(str(v) for v in val)
                lines.append("    @staticmethod")
                lines.append(f"    def {fname}() -> '{type_name}':")
                lines.append(f"        return {type_name}({val_str})")
            elif fargs:
                arg_str = ", ".join(
                    f"{to_snake(a['name'])}: "
                    f"{PYTHON_TYPES.get(a.get('type', 'f32'), 'float')}"
                    for a in fargs
                )
                lines.append("    @staticmethod")
                lines.append(f"    def {fname}({arg_str}) -> '{type_name}':")
                if fname == "from_hex":
                    lines.append(
                        f"        return {type_name}("
                        "((hex >> 16) & 0xFF) / 255.0, "
                        "((hex >> 8) & 0xFF) / 255.0, "
                        "(hex & 0xFF) / 255.0, 1.0)"
                    )
                elif fname == "from_u8":
                    lines.append(
                        f"        return {type_name}("
                        "r / 255.0, g / 255.0, b / 255.0, a / 255.0)"
                    )
                else:
                    field_vals = {}
                    for f in fields:
                        fn = to_snake(f["name"])
                        ft = f.get("type", "f32")
                        if fn in ("scale_x", "scale_y"):
                            field_vals[fn] = "1.0"
                        elif ft == "bool":
                            field_vals[fn] = "False"
                        elif ft in ("u32", "i32", "u64"):
                            field_vals[fn] = "0"
                        else:
                            field_vals[fn] = "0.0"
                    for dk, dv in factory.get("defaults", {}).items():
                        field_vals[to_snake(dk)] = repr(dv)
                    factory_prefix = (
                        fname[len("from_"):]
                        if fname.startswith("from_")
                        else ""
                    )
                    assigned_fields = set()
                    for a in fargs:
                        aname = to_snake(a["name"])
                        matched = False
                        if (
                            aname in field_vals
                            and aname not in assigned_fields
                        ):
                            field_vals[aname] = aname
                            assigned_fields.add(aname)
                            matched = True
                        if (
                            not matched
                            and factory_prefix
                            and factory_prefix in field_vals
                            and factory_prefix not in assigned_fields
                        ):
                            field_vals[factory_prefix] = aname
                            assigned_fields.add(factory_prefix)
                            matched = True
                        if not matched:
                            prefixed = (
                                (factory_prefix + "_" + aname)
                                if factory_prefix
                                else ""
                            )
                            if (
                                prefixed
                                and prefixed in field_vals
                                and prefixed not in assigned_fields
                            ):
                                field_vals[prefixed] = aname
                                assigned_fields.add(prefixed)
                                matched = True
                        if not matched:
                            candidates = (
                                [
                                    fn
                                    for fn in field_vals
                                    if fn.startswith(factory_prefix + "_")
                                    and fn not in assigned_fields
                                ]
                                if factory_prefix
                                else []
                            )
                            for fn in candidates:
                                if fn.endswith("_" + aname) or fn == aname:
                                    field_vals[fn] = aname
                                    assigned_fields.add(fn)
                                    matched = True
                                    break
                        if not matched:
                            for fn in field_vals:
                                if fn not in assigned_fields and (
                                    fn.endswith("_" + aname) or fn == aname
                                ):
                                    field_vals[fn] = aname
                                    assigned_fields.add(fn)
                                    break
                    vals = ", ".join(
                        field_vals[to_snake(f["name"])] for f in fields
                    )
                    lines.append(f"        return {type_name}({vals})")
            lines.append("")

        for meth in type_def.get("methods", []):
            mname = to_snake(meth["name"])
            ret = meth["returns"]
            if mname == "add" and ret == "Vec2":
                lines.append(
                    f"    def {mname}(self, other: '{type_name}')"
                    f" -> '{type_name}':"
                )
                lines.append(
                    f"        return {type_name}"
                    "(self.x + other.x, self.y + other.y)"
                )
            elif mname == "sub" and ret == "Vec2":
                lines.append(
                    f"    def {mname}(self, other: '{type_name}')"
                    f" -> '{type_name}':"
                )
                lines.append(
                    f"        return {type_name}"
                    "(self.x - other.x, self.y - other.y)"
                )
            elif mname == "scale":
                lines.append(
                    f"    def {mname}(self, s: float)"
                    f" -> '{type_name}':"
                )
                lines.append(
                    f"        return {type_name}(self.x * s, self.y * s)"
                )
            elif mname == "length":
                lines.append(f"    def {mname}(self) -> float:")
                lines.append(
                    "        return math.sqrt("
                    "self.x * self.x + self.y * self.y)"
                )
            elif mname == "normalize":
                lines.append(
                    f"    def {mname}(self) -> '{type_name}':"
                )
                lines.append("        l = self.length()")
                lines.append("        if l == 0: return Vec2.zero()")
                lines.append(
                    f"        return {type_name}(self.x / l, self.y / l)"
                )
            elif mname == "dot":
                lines.append(
                    f"    def {mname}(self, other: '{type_name}')"
                    " -> float:"
                )
                lines.append(
                    "        return self.x * other.x + self.y * other.y"
                )
            elif mname == "distance":
                lines.append(
                    f"    def {mname}(self, other: '{type_name}')"
                    " -> float:"
                )
                lines.append("        return self.sub(other).length()")
            elif mname == "lerp" and type_name == "Color":
                lines.append(
                    f"    def {mname}(self, other: '{type_name}',"
                    f" t: float) -> '{type_name}':"
                )
                lines.append(
                    f"        return {type_name}("
                    "self.r + (other.r - self.r) * t, "
                    "self.g + (other.g - self.g) * t, "
                    "self.b + (other.b - self.b) * t, "
                    "self.a + (other.a - self.a) * t)"
                )
            elif mname == "lerp":
                lines.append(
                    f"    def {mname}(self, other: '{type_name}',"
                    f" t: float) -> '{type_name}':"
                )
                lines.append(
                    f"        return {type_name}("
                    "self.x + (other.x - self.x) * t, "
                    "self.y + (other.y - self.y) * t)"
                )
            elif mname == "with_alpha":
                lines.append(
                    f"    def {mname}(self, a: float) -> '{type_name}':"
                )
                lines.append(
                    f"        return {type_name}(self.r, self.g, self.b, a)"
                )
            elif mname == "contains":
                lines.append(f"    def {mname}(self, point) -> bool:")
                lines.append(
                    "        return ("
                    "self.x <= point.x <= self.x + self.width and"
                )
                lines.append(
                    "                "
                    "self.y <= point.y <= self.y + self.height)"
                )
            elif mname == "intersects":
                lines.append(f"    def {mname}(self, other) -> bool:")
                lines.append(
                    "        return ("
                    "self.x < other.x + other.width and "
                    "self.x + self.width > other.x and"
                )
                lines.append(
                    "                "
                    "self.y < other.y + other.height and "
                    "self.y + self.height > other.y)"
                )
            lines.append("")

        if type_name == "Vec2":
            lines.append(
                f"    def __add__(self, other: '{type_name}')"
                f" -> '{type_name}':"
            )
            lines.append(
                f"        return {type_name}"
                "(self.x + other.x, self.y + other.y)"
            )
            lines.append(
                f"    def __sub__(self, other: '{type_name}')"
                f" -> '{type_name}':"
            )
            lines.append(
                f"        return {type_name}"
                "(self.x - other.x, self.y - other.y)"
            )
            lines.append(
                f"    def __mul__(self, s: float) -> '{type_name}':"
            )
            lines.append(
                f"        return {type_name}(self.x * s, self.y * s)"
            )
            lines.append(
                f"    def __truediv__(self, s: float) -> '{type_name}':"
            )
            lines.append(
                f"        return {type_name}(self.x / s, self.y / s)"
            )
            lines.append(
                f"    def __neg__(self) -> '{type_name}':"
            )
            lines.append(
                f"        return {type_name}(-self.x, -self.y)"
            )
            lines.append("")

        lines.append("    def __repr__(self):")
        vals = ", ".join(f"{fn}={{self.{fn}}}" for fn in field_names)
        lines.append(f'        return f"{type_name}({vals})"')
        lines.append("")

    write_generated(OUT / "_types.py", "\n".join(lines))


# ── _game.g.py ──────────────────────────────────────────────────────

def _gen_component_strategy(
    strategy: str, comp_type: str, mmap: dict, lines: list,
):
    """Generate real FFI calls for component_add/get/set/has/remove."""
    ffi_type_info = mapping.get("ffi_types", {}).get(comp_type, {})
    ffi_struct = ffi_type_info.get("ffi_name", f"Ffi{comp_type}")

    # Determine the struct parameter name from the mapping's struct_params
    struct_params = mmap.get("struct_params", [])
    struct_var = struct_params[0] if struct_params else to_snake(comp_type)

    if strategy == "component_add":
        lines.append(f"        {struct_var}._sync_to_ffi()")
        lines.append(
            f"        self._lib.goud_component_add("
            f"self._ctx, entity._bits, "
            f"_TYPEID_{comp_type.upper()}, "
            f"ctypes.cast(ctypes.pointer({struct_var}._ffi), "
            f"ctypes.POINTER(ctypes.c_uint8)), "
            f"ctypes.sizeof({ffi_struct}))"
        )
    elif strategy == "component_get":
        lines.append(
            f"        ptr = self._lib.goud_component_get("
            f"self._ctx, entity._bits, "
            f"_TYPEID_{comp_type.upper()})"
        )
        lines.append("        if not ptr:")
        lines.append("            return None")
        lines.append(
            f"        ffi = ctypes.cast(ptr, "
            f"ctypes.POINTER({ffi_struct})).contents"
        )
        lines.append(f"        return {comp_type}._from_ffi(ffi)")
    elif strategy == "component_set":
        lines.append(f"        {struct_var}._sync_to_ffi()")
        lines.append(
            f"        ptr = self._lib.goud_component_get_mut("
            f"self._ctx, entity._bits, "
            f"_TYPEID_{comp_type.upper()})"
        )
        lines.append("        if ptr:")
        lines.append(
            f"            ctypes.memmove(ptr, "
            f"ctypes.addressof({struct_var}._ffi), "
            f"ctypes.sizeof({ffi_struct}))"
        )
    elif strategy == "component_has":
        lines.append(
            f"        return self._lib.goud_component_has("
            f"self._ctx, entity._bits, "
            f"_TYPEID_{comp_type.upper()})"
        )
    elif strategy == "component_remove":
        lines.append(
            f"        result = self._lib.goud_component_remove("
            f"self._ctx, entity._bits, "
            f"_TYPEID_{comp_type.upper()})"
        )
        lines.append("        return result.success")


def _gen_tool_class(tool_name: str, lines: list):
    """Generate a tool class (GoudGame or GoudContext)."""
    tool = schema["tools"][tool_name]
    tool_mapping = mapping["tools"][tool_name]

    is_game = tool_name == "GoudGame"
    class_name = tool_name

    lines.append(f"class {class_name}:")
    lines.append(f'    """{tool["doc"]}"""')
    lines.append("")

    # Constructor
    if is_game:
        lines.append(
            "    def __init__(self, width: int = 800, "
            "height: int = 600, title: str = 'GoudEngine'):"
        )
        lines.append("        lib = get_lib()")
        lines.append("        self._lib = lib")
        lines.append(
            "        self._ctx = lib.goud_window_create("
            "width, height, title.encode('utf-8'))"
        )
        lines.append("        self._delta_time = 0.0")
        lines.append("        self._title = title")
        lines.append("        self._frame_count = 0")
        lines.append("        self._total_time = 0.0")
    else:
        lines.append("    def __init__(self):")
        lines.append("        lib = get_lib()")
        lines.append("        self._lib = lib")
        lines.append("        self._ctx = lib.goud_context_create()")
    lines.append("")

    lines.append("    def __del__(self):")
    lines.append("        self.destroy()")
    lines.append("")

    # Properties (only GoudGame has them)
    for prop in tool.get("properties", []):
        pname = to_snake(prop["name"])
        py_type = PYTHON_TYPES.get(prop.get("type", "f32"), "float")
        lines.append("    @property")
        lines.append(f"    def {pname}(self) -> {py_type}:")
        prop_map = tool_mapping.get("properties", {}).get(
            prop["name"], {}
        )
        src = prop_map.get("source")
        if src == "cached":
            field_name = prop_map.get("field", "_delta_time")
            lines.append(f"        return self.{field_name}")
        elif src == "computed":
            lines.append(
                "        return 1.0 / self._delta_time "
                "if self._delta_time > 0 else 0.0"
            )
        elif "ffi" in prop_map:
            ffi_fn = prop_map["ffi"]
            if "out_index" in prop_map:
                idx = prop_map["out_index"]
                lines.append("        w = ctypes.c_uint32(0)")
                lines.append("        h = ctypes.c_uint32(0)")
                lines.append(
                    f"        self._lib.{ffi_fn}("
                    "self._ctx, ctypes.byref(w), ctypes.byref(h))"
                )
                lines.append(
                    f"        return {'w' if idx == 0 else 'h'}.value"
                )
            else:
                lines.append(
                    f"        return self._lib.{ffi_fn}(self._ctx)"
                )
        lines.append("")

    # Methods
    tool_methods = tool.get("methods", [])
    for method in tool_methods:
        mname = to_snake(method["name"])
        mmap = tool_mapping["methods"].get(method["name"], {})
        params = method.get("params", [])
        ret = method.get("returns", "void")

        # Default handling
        has_default = [
            p.get("default") is not None
            or p["type"] in schema["types"] and p.get("default")
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
                if default and allow_default[i]:
                    param_strs.append(f"{pn} = None")
                else:
                    param_strs.append(f"{pn}")
            elif pt in schema["enums"]:
                if default is not None and allow_default[i]:
                    param_strs.append(f"{pn} = {default}")
                else:
                    param_strs.append(f"{pn}")
            else:
                if default is not None and allow_default[i]:
                    param_strs.append(f"{pn} = {default}")
                else:
                    param_strs.append(f"{pn}")

        sig = ", ".join(["self"] + param_strs)
        lines.append(f"    def {mname}({sig}):")
        if method.get("doc"):
            lines.append(f'        """{method["doc"]}"""')

        if mname == "destroy":
            if is_game:
                lines.append("        if hasattr(self, '_ctx'):")
                lines.append(
                    "            self._lib.goud_window_destroy(self._ctx)"
                )
                lines.append("            del self._ctx")
            else:
                lines.append("        if hasattr(self, '_ctx'):")
                lines.append(
                    "            self._lib.goud_context_destroy(self._ctx)"
                )
                lines.append("            del self._ctx")
        elif mname == "begin_frame":
            lines.append(
                "        self._delta_time = "
                "self._lib.goud_window_poll_events(self._ctx)"
            )
            lines.append(
                "        self._lib.goud_window_clear("
                "self._ctx, r, g, b, a)"
            )
            lines.append(
                "        self._lib.goud_renderer_begin(self._ctx)"
            )
            lines.append(
                "        self._lib.goud_renderer_enable_blending(self._ctx)"
            )
        elif mname == "end_frame":
            lines.append(
                "        self._lib.goud_renderer_end(self._ctx)"
            )
            lines.append(
                "        self._lib.goud_window_swap_buffers(self._ctx)"
            )
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
            lines.append(
                "        if color is None: color = Color.white()"
            )
            lines.append(
                "        self._lib.goud_renderer_draw_sprite("
                "self._ctx, texture, x, y, width, height, rotation, "
                "color.r, color.g, color.b, color.a)"
            )
        elif mname == "draw_quad":
            lines.append(
                "        if color is None: color = Color.white()"
            )
            lines.append(
                "        self._lib.goud_renderer_draw_quad("
                "self._ctx, x, y, width, height, "
                "color.r, color.g, color.b, color.a)"
            )
        elif mname == "load_texture":
            lines.append(
                "        return self._lib.goud_texture_load("
                "self._ctx, path.encode('utf-8'))"
            )
        elif mname == "destroy_texture":
            lines.append(
                "        self._lib.goud_texture_destroy("
                "self._ctx, handle)"
            )
        elif "ffi_strategy" in mmap:
            strategy = mmap["ffi_strategy"]
            comp_type = mmap.get("component_type", "")
            if strategy.startswith("component_"):
                _gen_component_strategy(
                    strategy, comp_type, mmap, lines,
                )
            elif strategy == "name_add":
                lines.append(
                    "        self._lib.goud_name_add("
                    "self._ctx, entity._bits, name.encode('utf-8'))"
                )
            elif strategy == "name_get":
                lines.append(
                    "        # TODO: wire to goud_name_get FFI"
                )
                lines.append("        return None")
            elif strategy == "name_has":
                lines.append(
                    "        # TODO: wire to goud_name_has FFI"
                )
                lines.append("        return False")
            elif strategy == "name_remove":
                lines.append(
                    "        # TODO: wire to goud_name_remove FFI"
                )
                lines.append("        return False")
            else:
                lines.append(
                    f"        pass  # Unknown strategy: {strategy}"
                )
        elif "returns_entity" in mmap:
            if "entity_params" in mmap:
                # Convert entity parameters to bits
                entity_args = ", ".join(f"{p}._bits" for p in mmap["entity_params"])
                lines.append(
                    f"        bits = self._lib.{mmap['ffi']}(self._ctx, {entity_args})"
                )
            else:
                lines.append(
                    f"        bits = self._lib.{mmap['ffi']}(self._ctx)"
                )
            lines.append("        return Entity(bits)")
        elif "entity_params" in mmap and "ffi" in mmap:
            lines.append(
                f"        return self._lib.{mmap['ffi']}("
                "self._ctx, entity._bits)"
            )
        elif "out_params" in mmap and "returns_struct" in mmap and any(op["type"] != "f32" for op in mmap["out_params"]):
            struct_name = mmap["returns_struct"]
            ffi_type_name = mapping["ffi_types"][struct_name]["ffi_name"]
            rs_fields = schema["types"][struct_name]["fields"]
            field_args = ", ".join(
                f"_stats.{to_snake(f['name'])}" for f in rs_fields
            )
            lines.append(f"        _stats = {ffi_type_name}()")
            lines.append(
                f"        self._lib.{mmap['ffi']}("
                "self._ctx, ctypes.byref(_stats))"
            )
            lines.append(f"        return {struct_name}({field_args})")
        elif "out_params" in mmap:
            for op in mmap["out_params"]:
                lines.append(
                    f"        _{op['name']} = ctypes.c_float(0.0)"
                )
            out_refs = ", ".join(
                f"ctypes.byref(_{op['name']})"
                for op in mmap["out_params"]
            )
            lines.append(
                f"        self._lib.{mmap['ffi']}("
                f"self._ctx, {out_refs})"
            )
            out_vals = ", ".join(
                f"_{op['name']}.value" for op in mmap["out_params"]
            )
            lines.append(f"        return Vec2({out_vals})")
        elif "enum_params" in mmap:
            enum_arg = list(mmap["enum_params"].keys())[0]
            lines.append(
                f"        return self._lib.{mmap['ffi']}("
                f"self._ctx, int({to_snake(enum_arg)}))"
            )
        elif "ffi" in mmap:
            ffi_fn = mmap["ffi"]
            if "append_args" in mmap:
                extra = ", ".join(str(a) for a in mmap["append_args"])
                lines.append(
                    f"        self._lib.{ffi_fn}(self._ctx, {extra})"
                )
            elif any(p["type"] == "string" for p in params) and _ffi_uses_ptr_len(ffi_fn):
                # String params need encoding to bytes ptr + len (only for *const u8 FFI)
                for p in params:
                    if p["type"] == "string":
                        sn = to_snake(p["name"])
                        lines.append(
                            f"        _{sn}_bytes = {sn}.encode('utf-8')"
                        )
                ffi_parts = ["self._ctx"]
                for p in params:
                    sn = to_snake(p["name"])
                    if p["type"] == "string":
                        ffi_parts.append(
                            f"ctypes.cast(ctypes.create_string_buffer(_{sn}_bytes, len(_{sn}_bytes)), ctypes.POINTER(ctypes.c_uint8))"
                        )
                        ffi_parts.append(f"len(_{sn}_bytes)")
                    else:
                        ffi_parts.append(sn)
                args_str = ", ".join(ffi_parts)
                if ret == "void":
                    lines.append(
                        f"        self._lib.{ffi_fn}({args_str})"
                    )
                else:
                    lines.append(
                        f"        return self._lib.{ffi_fn}({args_str})"
                    )
            else:
                ffi_args = [to_snake(p["name"]) for p in params]
                args_str = ", ".join(["self._ctx"] + ffi_args)
                if ret == "void":
                    lines.append(
                        f"        self._lib.{ffi_fn}({args_str})"
                    )
                else:
                    lines.append(
                        f"        return self._lib.{ffi_fn}({args_str})"
                    )
        lines.append("")


def _gen_engine_config(lines: list):
    """Generate EngineConfig builder class for Python."""
    tool = schema["tools"]["EngineConfig"]
    tm = mapping["tools"]["EngineConfig"]

    lines.append("class EngineConfig:")
    lines.append(f'    """{tool["doc"]}"""')
    lines.append("")
    lines.append("    def __init__(self):")
    lines.append("        lib = get_lib()")
    lines.append("        self._lib = lib")
    lines.append(f"        self._handle = lib.{tm['constructor']['ffi']}()")
    lines.append("")
    lines.append("    def __del__(self):")
    lines.append("        self.destroy()")
    lines.append("")

    for method in tool.get("methods", []):
        mn = method["name"]
        mm = tm.get("methods", {}).get(mn, {})
        ffi_fn = mm.get("ffi", "")
        params = method.get("params", [])
        mname = to_snake(mn)

        if mname == "destroy":
            lines.append("    def destroy(self):")
            if method.get("doc"):
                lines.append(f'        """{method["doc"]}"""')
            lines.append("        if hasattr(self, '_handle') and self._handle:")
            lines.append(f"            self._lib.{ffi_fn}(self._handle)")
            lines.append("            self._handle = None")
            lines.append("")
        elif mname == "build":
            lines.append("    def build(self):")
            if method.get("doc"):
                lines.append(f'        """{method["doc"]}"""')
            lines.append("        if not self._handle:")
            lines.append("            raise RuntimeError('EngineConfig already consumed')")
            lines.append(f"        ctx = self._lib.{ffi_fn}(self._handle)")
            lines.append("        self._handle = None")
            lines.append("        if ctx._bits == 0xFFFFFFFFFFFFFFFF:")
            lines.append("            raise RuntimeError('Failed to create engine context from EngineConfig')")
            lines.append("        game = GoudGame.__new__(GoudGame)")
            lines.append("        game._lib = self._lib")
            lines.append("        game._ctx = ctx")
            lines.append("        game._delta_time = 0.0")
            lines.append("        game._title = ''")
            lines.append("        game._frame_count = 0")
            lines.append("        game._total_time = 0.0")
            lines.append("        return game")
            lines.append("")
        elif mname == "set_title":
            lines.append(f"    def {mname}(self, title):")
            if method.get("doc"):
                lines.append(f'        """{method["doc"]}"""')
            lines.append("        if not self._handle:")
            lines.append("            raise RuntimeError('EngineConfig already consumed or destroyed')")
            lines.append(f"        self._lib.{ffi_fn}(self._handle, title.encode('utf-8'))")
            lines.append("        return self")
            lines.append("")
        else:
            param_strs = ["self"] + [to_snake(p["name"]) for p in params]
            sig = ", ".join(param_strs)
            lines.append(f"    def {mname}({sig}):")
            if method.get("doc"):
                lines.append(f'        """{method["doc"]}"""')
            lines.append("        if not self._handle:")
            lines.append("            raise RuntimeError('EngineConfig already consumed or destroyed')")
            ffi_args = ", ".join(["self._handle"] + [to_snake(p["name"]) for p in params])
            lines.append(f"        self._lib.{ffi_fn}({ffi_args})")
            lines.append("        return self")
            lines.append("")


def gen_game():
    lines = [
        f'"""{HEADER_COMMENT}"""',
        "",
        "import ctypes",
        "from ._ffi import (get_lib, GoudContextId, FfiVec2, "
        "FfiTransform2D, FfiSprite,",
        "    GoudRenderStats, GoudContact)",
        "from ._types import Entity, Vec2, Color, Transform2D, Sprite, RenderStats",
        "from ._keys import Key, MouseButton",
        "",
        "# Type IDs for built-in component types (hash of type name)",
        "_TYPEID_TRANSFORM2D = hash('Transform2D') & 0xFFFFFFFFFFFFFFFF",
        "_TYPEID_SPRITE = hash('Sprite') & 0xFFFFFFFFFFFFFFFF",
        "",
    ]

    _gen_tool_class("GoudGame", lines)

    # Generate GoudContext if present in both schema and mapping
    if "GoudContext" in schema.get("tools", {}) and "GoudContext" in mapping.get("tools", {}):
        lines.append("")
        _gen_tool_class("GoudContext", lines)

    # Generate EngineConfig if present in both schema and mapping
    if "EngineConfig" in schema.get("tools", {}) and "EngineConfig" in mapping.get("tools", {}):
        lines.append("")
        _gen_engine_config(lines)

    write_generated(OUT / "_game.py", "\n".join(lines))


# ── __init__.py ─────────────────────────────────────────────────────

def gen_init():
    has_context = "GoudContext" in schema.get("tools", {}) and "GoudContext" in mapping.get("tools", {})
    has_engine_config = "EngineConfig" in schema.get("tools", {}) and "EngineConfig" in mapping.get("tools", {})

    type_imports = "Color, Vec2, Rect, Transform2D, Sprite, Entity"
    builder_imports = []
    for tn in ("Transform2D", "Sprite"):
        td = schema["types"].get(tn, {})
        if td.get("builder"):
            builder_imports.append(f"{tn}Builder")
    if builder_imports:
        type_imports += ", " + ", ".join(builder_imports)

    game_imports = ["GoudGame"]
    if has_context:
        game_imports.append("GoudContext")
    if has_engine_config:
        game_imports.append("EngineConfig")

    lines = [
        f'"""{HEADER_COMMENT}"""',
        "",
        f"from ._types import {type_imports}",
        "from ._keys import Key, MouseButton",
        f"from ._game import {', '.join(game_imports)}",
        "",
        "__all__ = [",
    ]
    for gi in game_imports:
        lines.append(f'    "{gi}",')
    lines.append('    "Entity",')
    lines.append('    "Color", "Vec2", "Rect", "Transform2D", "Sprite",')
    for bi in builder_imports:
        lines.append(f'    "{bi}",')
    lines.append('    "Key", "MouseButton",')
    lines.append("]")
    lines.append("")
    write_generated(OUT / "__init__.py", "\n".join(lines))

    # Package root __init__.py delegates to generated/
    root_init = [
        f'"""{HEADER_COMMENT}"""',
        "",
        "from .generated import *  # noqa: F401,F403",
        "",
    ]

    # Include error types if the errors section exists in schema
    if "errors" in schema:
        root_init.append("from .generated._errors import (  # noqa: F401")
        root_init.append("    GoudError,")
        for cat in schema["errors"].get("categories", []):
            cls = cat["base_class"]
            root_init.append(f"    {cls},")
        root_init.append("    RecoveryClass,")
        root_init.append(")")
        root_init.append("")
    root = OUT.parent / "__init__.py"
    write_generated(root, "\n".join(root_init))


def gen_errors():
    categories, codes = load_errors(schema)
    if not categories:
        return

    lines = [
        f'"""{HEADER_COMMENT}',
        "",
        "Typed error classes for GoudEngine Python SDK.",
        "",
        "Maps FFI error codes to language-idiomatic exceptions with code,",
        "message, context, and recovery information. All recovery logic",
        'lives in Rust; these classes only marshal the data.',
        '"""',
        "",
        "import ctypes",
        "",
        "",
        "class RecoveryClass:",
        '    """Recovery classification matching Rust RecoveryClass enum."""',
        "    RECOVERABLE = 0",
        "    FATAL = 1",
        "    DEGRADED = 2",
        "",
        '    _NAMES = {0: "recoverable", 1: "fatal", 2: "degraded"}',
        "",
        "    @classmethod",
        "    def name(cls, value):",
        "        return cls._NAMES.get(value, \"unknown\")",
        "",
        "",
        "class GoudError(Exception):",
        '    """Base exception for all GoudEngine errors."""',
        "",
        "    def __init__(self, error_code, message, category, subsystem,",
        "                 operation, recovery, recovery_hint):",
        "        super().__init__(message)",
        "        self.error_code = error_code",
        "        self.category = category",
        "        self.subsystem = subsystem",
        "        self.operation = operation",
        "        self.recovery = recovery",
        "        self.recovery_hint = recovery_hint",
        "",
        "    def __repr__(self):",
        "        return (",
        '            f"{type(self).__name__}(code={self.error_code}, "',
        '            f"category={self.category!r}, "',
        '            f"recovery={RecoveryClass.name(self.recovery)})"',
        "        )",
        "",
        "    @classmethod",
        "    def from_last_error(cls, lib):",
        '        """Query FFI error state and build the correct typed exception.',
        "",
        '        Returns None if no error is set (code == 0).',
        '        """',
        "        code = lib.goud_last_error_code()",
        "        if code == 0:",
        "            return None",
        "",
        "        message = _read_string(lib.goud_last_error_message)",
        "        subsystem = _read_string(lib.goud_last_error_subsystem)",
        "        operation = _read_string(lib.goud_last_error_operation)",
        "",
        "        recovery = lib.goud_error_recovery_class(code)",
        "        hint = _read_hint(lib, code)",
        "",
        "        category = _category_from_code(code)",
        "        subclass = _CATEGORY_CLASS_MAP.get(category, GoudError)",
        "",
        "        return subclass(",
        "            error_code=code,",
        "            message=message,",
        "            category=category,",
        "            subsystem=subsystem,",
        "            operation=operation,",
        "            recovery=recovery,",
        "            recovery_hint=hint,",
        "        )",
        "",
        "",
    ]

    # Generate category subclasses
    for cat in categories:
        cls = cat["base_class"]
        doc = f'{cat["name"]} errors (codes {cat["range_start"]}-{cat["range_end"]}).'
        lines += [
            f"class {cls}(GoudError):",
            f'    """{doc}"""',
            "    pass",
            "",
            "",
        ]

    # _CATEGORY_CLASS_MAP
    lines.append("_CATEGORY_CLASS_MAP = {")
    for cat in categories:
        lines.append(f'    "{cat["name"]}": {cat["base_class"]},')
    lines += ["}", "", ""]

    # _category_from_code
    lines.append("def _category_from_code(code):")
    sorted_cats = sorted(categories, key=lambda c: c["range_start"], reverse=True)
    for cat in sorted_cats:
        lines.append(f'    if code >= {cat["range_start"]}:')
        lines.append(f'        return "{cat["name"]}"')
    lines += ['    return "Unknown"', "", ""]

    # _read_string helper
    lines += [
        "def _read_string(ffi_fn):",
        '    """Call a buffer-writing FFI function and return the string."""',
        "    buf = (ctypes.c_uint8 * 256)()",
        "    written = ffi_fn(buf, 256)",
        "    if written <= 0:",
        '        return ""',
        '    return bytes(buf[:written]).decode("utf-8", errors="replace")',
        "",
        "",
    ]

    # _read_hint helper
    lines += [
        "def _read_hint(lib, code):",
        '    """Call goud_error_recovery_hint and return the string."""',
        "    buf = (ctypes.c_uint8 * 256)()",
        "    written = lib.goud_error_recovery_hint(code, buf, 256)",
        "    if written <= 0:",
        '        return ""',
        '    return bytes(buf[:written]).decode("utf-8", errors="replace")',
        "",
    ]

    write_generated(OUT / "_errors.py", "\n".join(lines))


if __name__ == "__main__":
    print("Generating Python SDK...")
    gen_ffi()
    gen_keys()
    gen_types()
    gen_game()
    gen_errors()
    gen_init()
    print("Python SDK generation complete.")
