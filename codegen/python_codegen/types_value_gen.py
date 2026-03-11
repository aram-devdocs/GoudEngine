"""Value-type generation helper for `_types.py`."""

from .context import PYTHON_TYPES, schema, to_snake
from .shared_helpers import py_field_default


def gen_ui_style(type_name: str, type_def: dict, lines: list[str]) -> None:
    lines.append(f"class {type_name}:")
    if type_def.get("doc"):
        lines.append(f'    """{type_def["doc"]}"""')
    lines.append(
        "    def __init__(self, has_background_color: bool = False, background_color: 'Color' = None, "
        "has_foreground_color: bool = False, foreground_color: 'Color' = None, "
        "has_border_color: bool = False, border_color: 'Color' = None, "
        "has_border_width: bool = False, border_width: float = 0.0, "
        "has_font_family: bool = False, font_family: str = '', "
        "has_font_size: bool = False, font_size: float = 0.0, "
        "has_texture_path: bool = False, texture_path: str = '', "
        "has_widget_spacing: bool = False, widget_spacing: float = 0.0):"
    )
    lines.append("        self.has_background_color = has_background_color")
    lines.append("        self.background_color = background_color if background_color is not None else Color()")
    lines.append("        self.has_foreground_color = has_foreground_color")
    lines.append("        self.foreground_color = foreground_color if foreground_color is not None else Color()")
    lines.append("        self.has_border_color = has_border_color")
    lines.append("        self.border_color = border_color if border_color is not None else Color()")
    lines.append("        self.has_border_width = has_border_width")
    lines.append("        self.border_width = border_width")
    lines.append("        self.has_font_family = has_font_family")
    lines.append("        self.font_family = '' if font_family is None else font_family")
    lines.append("        self.has_font_size = has_font_size")
    lines.append("        self.font_size = font_size")
    lines.append("        self.has_texture_path = has_texture_path")
    lines.append("        self.texture_path = '' if texture_path is None else texture_path")
    lines.append("        self.has_widget_spacing = has_widget_spacing")
    lines.append("        self.widget_spacing = widget_spacing")
    lines.append("")
    lines.append("    def __repr__(self):")
    lines.append(
        '        return f"UiStyle(has_background_color={self.has_background_color}, background_color={self.background_color}, '
        'has_foreground_color={self.has_foreground_color}, foreground_color={self.foreground_color}, '
        'has_border_color={self.has_border_color}, border_color={self.border_color}, '
        'has_border_width={self.has_border_width}, border_width={self.border_width}, '
        'has_font_family={self.has_font_family}, font_family={self.font_family}, '
        'has_font_size={self.has_font_size}, font_size={self.font_size}, '
        'has_texture_path={self.has_texture_path}, texture_path={self.texture_path}, '
        'has_widget_spacing={self.has_widget_spacing}, widget_spacing={self.widget_spacing})"'
    )
    lines.append("")


def gen_value_type(type_name: str, type_def: dict, lines: list[str]) -> None:
    fields = type_def.get("fields", [])
    field_names = [to_snake(f["name"]) for f in fields]

    if type_name == "UiStyle":
        gen_ui_style(type_name, type_def, lines)
        return

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
                f"{to_snake(a['name'])}: {PYTHON_TYPES.get(a.get('type', 'f32'), 'float')}"
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
                lines.append(f"        return {type_name}(r / 255.0, g / 255.0, b / 255.0, a / 255.0)")
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
                factory_prefix = fname[len("from_"):] if fname.startswith("from_") else ""
                assigned_fields = set()
                for a in fargs:
                    aname = to_snake(a["name"])
                    matched = False
                    if aname in field_vals and aname not in assigned_fields:
                        field_vals[aname] = aname
                        assigned_fields.add(aname)
                        matched = True
                    if not matched and factory_prefix and factory_prefix in field_vals and factory_prefix not in assigned_fields:
                        field_vals[factory_prefix] = aname
                        assigned_fields.add(factory_prefix)
                        matched = True
                    if not matched:
                        prefixed = (factory_prefix + "_" + aname) if factory_prefix else ""
                        if prefixed and prefixed in field_vals and prefixed not in assigned_fields:
                            field_vals[prefixed] = aname
                            assigned_fields.add(prefixed)
                            matched = True
                    if not matched:
                        candidates = [
                            fn
                            for fn in field_vals
                            if fn.startswith(factory_prefix + "_") and fn not in assigned_fields
                        ] if factory_prefix else []
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
            lines.append(
                f"        return {type_name}("
                "self.r + (other.r - self.r) * t, "
                "self.g + (other.g - self.g) * t, "
                "self.b + (other.b - self.b) * t, "
                "self.a + (other.a - self.a) * t)"
            )
        elif mname == "lerp":
            lines.append(f"    def {mname}(self, other: '{type_name}', t: float) -> '{type_name}':")
            lines.append(
                f"        return {type_name}("
                "self.x + (other.x - self.x) * t, "
                "self.y + (other.y - self.y) * t)"
            )
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
