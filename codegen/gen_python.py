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
    HEADER_COMMENT, SDKS_DIR, load_schema, load_ffi_mapping,
    to_snake, to_screaming_snake, write_generated, CTYPES_MAP, PYTHON_TYPES,
)

OUT = SDKS_DIR / "python" / "goud_engine" / "generated"
schema = load_schema()
mapping = load_ffi_mapping()


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

    for type_name, type_def in mapping["ffi_types"].items():
        ffi_name = type_def["ffi_name"]
        if ffi_name in ("u64", "FfiSprite"):
            continue
        sdk_type = schema["types"].get(type_name)
        if not sdk_type or "fields" not in sdk_type:
            continue
        lines.append(f"class {ffi_name}(ctypes.Structure):")
        fields = []
        for f in sdk_type["fields"]:
            ct = CTYPES_MAP.get(f["type"], "ctypes.c_float")
            fields.append(f'        ("{to_snake(f["name"])}", {ct})')
        lines.append("    _fields_ = [")
        lines.append(",\n".join(fields))
        lines.append("    ]")
        lines.append("")

    lines.append("# ── Function signatures ──")
    lines.append("")
    lines.append("def _setup():")

    for module, funcs in mapping["ffi_functions"].items():
        if not isinstance(funcs, dict):
            continue
        lines.append(f"    # {module}")
        for fname, fdef in funcs.items():
            argtypes = []
            for p in fdef["params"]:
                ct = CTYPES_MAP.get(p["type"], "ctypes.c_uint64")
                argtypes.append(ct)
            ret = fdef["returns"]
            if ret == "void":
                restype = "None"
            elif ret == "bool":
                restype = "ctypes.c_bool"
            elif ret == "f32":
                restype = "ctypes.c_float"
            elif ret == "u32":
                restype = "ctypes.c_uint32"
            elif ret == "u64":
                restype = "ctypes.c_uint64"
            elif ret == "GoudContextId":
                restype = "GoudContextId"
            elif ret == "GoudResult":
                restype = "GoudResult"
            else:
                restype = "ctypes.c_uint64"
            at_str = ", ".join(argtypes) if argtypes else ""
            lines.append(f"    _lib.{fname}.argtypes = [{at_str}]")
            lines.append(f"    _lib.{fname}.restype = {restype}")
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


def gen_types():
    lines = [f'"""{HEADER_COMMENT}"""', "", "import math", ""]

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
                    lines.append("        return self._bits == 0xFFFFFFFFFFFFFFFF")
            for meth in type_def.get("methods", []):
                mname = to_snake(meth["name"])
                if mname == "to_bits":
                    lines.append(f"    def {mname}(self) -> int:")
                    lines.append("        return self._bits")
            lines.append("")
            lines.append("    def __repr__(self):")
            lines.append('        return f"Entity({self.index}v{self.generation})"')
            lines.append("")
            continue

        if kind not in ("value", "component"):
            continue

        fields = type_def.get("fields", [])
        field_names = [to_snake(f["name"]) for f in fields]

        lines.append(f"class {type_name}:")
        if type_def.get("doc"):
            lines.append(f'    """{type_def["doc"]}"""')

        params = ", ".join(f"{to_snake(f['name'])}: {_py_field_default(f)}" for f in fields)
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
                arg_str = ", ".join(f"{to_snake(a['name'])}: {PYTHON_TYPES.get(a.get('type', 'f32'), 'float')}" for a in fargs)
                lines.append("    @staticmethod")
                lines.append(f"    def {fname}({arg_str}) -> '{type_name}':")
                if fname == "from_hex":
                    lines.append(f"        return {type_name}(((hex >> 16) & 0xFF) / 255.0, ((hex >> 8) & 0xFF) / 255.0, (hex & 0xFF) / 255.0, 1.0)")
                elif fname == "from_u8":
                    lines.append(f"        return {type_name}(r / 255.0, g / 255.0, b / 255.0, a / 255.0)")
                else:
                    # Build field values: start with type-aware defaults, then override with factory args
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
                    # Apply schema defaults
                    for dk, dv in factory.get("defaults", {}).items():
                        field_vals[to_snake(dk)] = repr(dv)
                    # Map factory args to field names.
                    # Strategy: exact match, then factory-prefix-based match, then
                    # suffix match. For single-arg factories like fromRotation(radians),
                    # if the factory prefix itself is a field name, assign the arg there.
                    factory_prefix = fname[len("from_"):] if fname.startswith("from_") else ""
                    # Track which fields have already been assigned by this loop
                    assigned_fields = set()
                    for a in fargs:
                        aname = to_snake(a["name"])
                        matched = False
                        # 1. Exact match: arg name is a field name
                        if aname in field_vals and aname not in assigned_fields:
                            field_vals[aname] = aname
                            assigned_fields.add(aname)
                            matched = True
                        # 2. Factory prefix IS the field name (e.g. from_rotation -> rotation field)
                        if not matched and factory_prefix and factory_prefix in field_vals and factory_prefix not in assigned_fields:
                            field_vals[factory_prefix] = aname
                            assigned_fields.add(factory_prefix)
                            matched = True
                        # 3. Prefixed: factory_prefix + "_" + arg (e.g. scale + x -> scale_x)
                        if not matched:
                            prefixed = (factory_prefix + "_" + aname) if factory_prefix else ""
                            if prefixed and prefixed in field_vals and prefixed not in assigned_fields:
                                field_vals[prefixed] = aname
                                assigned_fields.add(prefixed)
                                matched = True
                        # 4. Suffix match among fields with factory prefix, then globally
                        if not matched:
                            candidates = [fn for fn in field_vals if fn.startswith(factory_prefix + "_") and fn not in assigned_fields] if factory_prefix else []
                            for fn in candidates:
                                if fn.endswith("_" + aname) or fn == aname:
                                    field_vals[fn] = aname
                                    assigned_fields.add(fn)
                                    matched = True
                                    break
                        if not matched:
                            for fn in field_vals:
                                if fn not in assigned_fields and (fn.endswith("_" + aname) or fn == aname):
                                    field_vals[fn] = aname
                                    assigned_fields.add(fn)
                                    break
                    vals = ", ".join(field_vals[to_snake(f["name"])] for f in fields)
                    lines.append(f"        return {type_name}({vals})")
            lines.append("")

        for meth in type_def.get("methods", []):
            mname = to_snake(meth["name"])
            ret = meth["returns"]
            if mname == "add" and ret == "Vec2":
                lines.append(f"    def {mname}(self, other: '{type_name}') -> '{type_name}':")
                lines.append(f"        return {type_name}(self.x + other.x, self.y + other.y)")
            elif mname == "sub" and ret == "Vec2":
                lines.append(f"    def {mname}(self, other: '{type_name}') -> '{type_name}':")
                lines.append(f"        return {type_name}(self.x - other.x, self.y - other.y)")
            elif mname == "scale":
                lines.append(f"    def {mname}(self, s: float) -> '{type_name}':")
                lines.append(f"        return {type_name}(self.x * s, self.y * s)")
            elif mname == "length":
                lines.append(f"    def {mname}(self) -> float:")
                lines.append("        return math.sqrt(self.x * self.x + self.y * self.y)")
            elif mname == "normalize":
                lines.append(f"    def {mname}(self) -> '{type_name}':")
                lines.append("        l = self.length()")
                lines.append("        if l == 0: return Vec2.zero()")
                lines.append(f"        return {type_name}(self.x / l, self.y / l)")
            elif mname == "dot":
                lines.append(f"    def {mname}(self, other: '{type_name}') -> float:")
                lines.append("        return self.x * other.x + self.y * other.y")
            elif mname == "distance":
                lines.append(f"    def {mname}(self, other: '{type_name}') -> float:")
                lines.append("        return self.sub(other).length()")
            elif mname == "lerp" and type_name == "Color":
                lines.append(f"    def {mname}(self, other: '{type_name}', t: float) -> '{type_name}':")
                lines.append(f"        return {type_name}(self.r + (other.r - self.r) * t, self.g + (other.g - self.g) * t, self.b + (other.b - self.b) * t, self.a + (other.a - self.a) * t)")
            elif mname == "lerp":
                lines.append(f"    def {mname}(self, other: '{type_name}', t: float) -> '{type_name}':")
                lines.append(f"        return {type_name}(self.x + (other.x - self.x) * t, self.y + (other.y - self.y) * t)")
            elif mname == "with_alpha":
                lines.append(f"    def {mname}(self, a: float) -> '{type_name}':")
                lines.append(f"        return {type_name}(self.r, self.g, self.b, a)")
            elif mname == "contains":
                lines.append(f"    def {mname}(self, point) -> bool:")
                lines.append("        return (self.x <= point.x <= self.x + self.width and")
                lines.append("                self.y <= point.y <= self.y + self.height)")
            elif mname == "intersects":
                lines.append(f"    def {mname}(self, other) -> bool:")
                lines.append("        return (self.x < other.x + other.width and self.x + self.width > other.x and")
                lines.append("                self.y < other.y + other.height and self.y + self.height > other.y)")
            lines.append("")

        if type_name == "Vec2":
            lines.append(f"    def __add__(self, other: '{type_name}') -> '{type_name}':")
            lines.append(f"        return {type_name}(self.x + other.x, self.y + other.y)")
            lines.append(f"    def __sub__(self, other: '{type_name}') -> '{type_name}':")
            lines.append(f"        return {type_name}(self.x - other.x, self.y - other.y)")
            lines.append(f"    def __mul__(self, s: float) -> '{type_name}':")
            lines.append(f"        return {type_name}(self.x * s, self.y * s)")
            lines.append(f"    def __truediv__(self, s: float) -> '{type_name}':")
            lines.append(f"        return {type_name}(self.x / s, self.y / s)")
            lines.append(f"    def __neg__(self) -> '{type_name}':")
            lines.append(f"        return {type_name}(-self.x, -self.y)")
            lines.append("")

        lines.append("    def __repr__(self):")
        vals = ", ".join(f"{fn}={{self.{fn}}}" for fn in field_names)
        lines.append(f'        return f"{type_name}({vals})"')
        lines.append("")

    write_generated(OUT / "_types.py", "\n".join(lines))


# ── _game.g.py ──────────────────────────────────────────────────────

def gen_game():
    tool = schema["tools"]["GoudGame"]
    tool_mapping = mapping["tools"]["GoudGame"]

    lines = [
        f'"""{HEADER_COMMENT}"""',
        "",
        "import ctypes",
        "from ._ffi import get_lib, GoudContextId, FfiVec2, GoudRenderStats, GoudContact",
        "from ._types import Entity, Vec2, Color, Transform2D",
        "from ._keys import Key, MouseButton",
        "",
        "",
        "class GoudGame:",
        f'    """{tool["doc"]}"""',
        "",
    ]

    ctor = tool["constructor"]
    lines.append("    def __init__(self, width: int = 800, height: int = 600, title: str = 'GoudEngine'):")
    lines.append("        lib = get_lib()")
    lines.append("        self._lib = lib")
    lines.append("        self._ctx = lib.goud_window_create(width, height, title.encode('utf-8'))")
    lines.append("        self._delta_time = 0.0")
    lines.append("")

    lines.append("    def __del__(self):")
    lines.append("        self.destroy()")
    lines.append("")

    for prop in tool["properties"]:
        pname = to_snake(prop["name"])
        lines.append("    @property")
        lines.append(f"    def {pname}(self) -> float:")
        prop_map = tool_mapping["properties"].get(prop["name"], {})
        src = prop_map.get("source")
        if src == "cached":
            lines.append(f"        return self._delta_time")
        elif src == "computed":
            lines.append("        return 1.0 / self._delta_time if self._delta_time > 0 else 0.0")
        elif "ffi" in prop_map:
            ffi_fn = prop_map["ffi"]
            if "out_index" in prop_map:
                idx = prop_map["out_index"]
                lines.append(f"        w = ctypes.c_uint32(0)")
                lines.append(f"        h = ctypes.c_uint32(0)")
                lines.append(f"        self._lib.{ffi_fn}(self._ctx, ctypes.byref(w), ctypes.byref(h))")
                lines.append(f"        return {'w' if idx == 0 else 'h'}.value")
            else:
                lines.append(f"        return self._lib.{ffi_fn}(self._ctx)")
        lines.append("")

    for method in tool["methods"]:
        mname = to_snake(method["name"])
        mmap = tool_mapping["methods"].get(method["name"], {})
        params = method.get("params", [])
        ret = method.get("returns", "void")

        # Determine which params can have defaults (Python requires all
        # non-default params before any default params).
        has_default = [p.get("default") is not None or p["type"] in schema["types"] and p.get("default") for p in params]
        # Scan backwards: once we hit a param without a default, no earlier param can have one
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
            lines.append("        if hasattr(self, '_ctx'):")
            lines.append("            self._lib.goud_window_destroy(self._ctx)")
            lines.append("            del self._ctx")
        elif mname == "begin_frame":
            lines.append("        self._delta_time = self._lib.goud_window_poll_events(self._ctx)")
            lines.append("        self._lib.goud_window_clear(self._ctx, r, g, b, a)")
            lines.append("        self._lib.goud_renderer_begin(self._ctx)")
            lines.append("        self._lib.goud_renderer_enable_blending(self._ctx)")
        elif mname == "end_frame":
            lines.append("        self._lib.goud_renderer_end(self._ctx)")
            lines.append("        self._lib.goud_window_swap_buffers(self._ctx)")
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
        elif "ffi_strategy" in mmap:
            strategy = mmap["ffi_strategy"]
            comp_type = mmap.get("component_type", "")
            comp_snake = to_snake(comp_type).lower() if comp_type else ""
            if strategy == "component_add":
                lines.append(f"        # TODO: wire to goud_{comp_snake}_add FFI when available")
                lines.append("        pass")
            elif strategy == "component_get":
                lines.append(f"        # TODO: wire to goud_{comp_snake}_get FFI when available")
                lines.append("        return None")
            elif strategy == "component_set":
                lines.append(f"        # TODO: wire to goud_{comp_snake}_set FFI when available")
                lines.append("        pass")
            elif strategy == "component_has":
                lines.append(f"        # TODO: wire to goud_{comp_snake}_has FFI when available")
                lines.append("        return False")
            elif strategy == "component_remove":
                lines.append(f"        # TODO: wire to goud_{comp_snake}_remove FFI when available")
                lines.append("        return False")
            elif strategy == "name_add":
                lines.append("        self._lib.goud_name_add(self._ctx, entity._bits, name.encode('utf-8'))")
            elif strategy == "name_get":
                lines.append("        # TODO: wire to goud_name_get FFI when available")
                lines.append("        return None")
            elif strategy == "name_has":
                lines.append("        # TODO: wire to goud_name_has FFI when available")
                lines.append("        return False")
            elif strategy == "name_remove":
                lines.append("        # TODO: wire to goud_name_remove FFI when available")
                lines.append("        return False")
            else:
                lines.append(f"        pass  # Unknown strategy: {strategy}")
        elif "returns_entity" in mmap:
            lines.append(f"        bits = self._lib.{mmap['ffi']}(self._ctx)")
            lines.append("        return Entity(bits)")
        elif "entity_params" in mmap and "ffi" in mmap:
            lines.append(f"        return self._lib.{mmap['ffi']}(self._ctx, entity._bits)")
        elif "out_params" in mmap:
            for op in mmap["out_params"]:
                lines.append(f"        _{op['name']} = ctypes.c_float(0.0)")
            out_refs = ", ".join(f"ctypes.byref(_{op['name']})" for op in mmap["out_params"])
            lines.append(f"        self._lib.{mmap['ffi']}(self._ctx, {out_refs})")
            out_vals = ", ".join(f"_{op['name']}.value" for op in mmap["out_params"])
            lines.append(f"        return Vec2({out_vals})")
        elif "enum_params" in mmap:
            enum_arg = list(mmap["enum_params"].keys())[0]
            lines.append(f"        return self._lib.{mmap['ffi']}(self._ctx, int({to_snake(enum_arg)}))")
        elif "ffi" in mmap:
            ffi_fn = mmap["ffi"]
            if "append_args" in mmap:
                extra = ", ".join(str(a) for a in mmap["append_args"])
                lines.append(f"        self._lib.{ffi_fn}(self._ctx, {extra})")
            else:
                ffi_args = [to_snake(p["name"]) for p in params]
                args_str = ", ".join(["self._ctx"] + ffi_args)
                if ret == "void":
                    lines.append(f"        self._lib.{ffi_fn}({args_str})")
                else:
                    lines.append(f"        return self._lib.{ffi_fn}({args_str})")
        lines.append("")

    write_generated(OUT / "_game.py", "\n".join(lines))


# ── __init__.py ─────────────────────────────────────────────────────

def gen_init():
    # generated/__init__.py re-exports everything from sibling modules
    lines = [
        f'"""{HEADER_COMMENT}"""',
        "",
        "from ._types import Color, Vec2, Rect, Transform2D, Sprite, Entity",
        "from ._keys import Key, MouseButton",
        "from ._game import GoudGame",
        "",
        "__all__ = [",
        '    "GoudGame",',
        '    "Entity",',
        '    "Color", "Vec2", "Rect", "Transform2D", "Sprite",',
        '    "Key", "MouseButton",',
        "]",
        "",
    ]
    write_generated(OUT / "__init__.py", "\n".join(lines))

    # Package root __init__.py delegates to generated/
    root_init = [
        f'"""{HEADER_COMMENT}"""',
        "",
        "from .generated import *  # noqa: F401,F403",
        "",
    ]
    root = OUT.parent / "__init__.py"
    write_generated(root, "\n".join(root_init))


if __name__ == "__main__":
    print("Generating Python SDK...")
    gen_ffi()
    gen_keys()
    gen_types()
    gen_game()
    gen_init()
    print("Python SDK generation complete.")
