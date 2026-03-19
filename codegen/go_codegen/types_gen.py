"""Generator for `goud/types.go` -- value types (Color, Vec2, Rect, Vec3, etc.)."""

from .context import GO_HEADER, GO_TYPES, GO_ZERO, OUT, schema, to_go_field, write_generated


def _go_field_type(field: dict) -> str:
    ft = field.get("type", "f32")
    # Handle fixed-size array types like "f32[9]"
    import re
    arr_match = re.match(r"^(\w+)\[(\d+)]$", ft)
    if arr_match:
        inner = arr_match.group(1)
        count = arr_match.group(2)
        inner_type = GO_TYPES.get(inner, inner)
        return f"[{count}]{inner_type}"
    if ft in GO_TYPES:
        return GO_TYPES[ft]
    if ft in schema.get("types", {}):
        return ft
    return "float32"


def _go_zero(field: dict) -> str:
    ft = field.get("type", "f32")
    import re
    arr_match = re.match(r"^(\w+)\[(\d+)]$", ft)
    if arr_match:
        inner = arr_match.group(1)
        count = arr_match.group(2)
        inner_type = GO_TYPES.get(inner, inner)
        return f"[{count}]{inner_type}{{}}"
    return GO_ZERO.get(ft, "0")


def _gen_value_type(type_name: str, type_def: dict, lines: list[str]) -> None:
    fields = type_def.get("fields", [])
    doc = type_def.get("doc", f"{type_name} value type.")

    lines.append(f"// {type_name} {doc}")
    lines.append(f"type {type_name} struct {{")
    for f in fields:
        fname = to_go_field(f["name"])
        ftype = _go_field_type(f)
        lines.append(f"\t{fname} {ftype}")
    lines.append("}")
    lines.append("")

    # Constructors: New<Type> with all fields
    if fields:
        params = ", ".join(
            f"{f['name']} {_go_field_type(f)}" for f in fields
        )
        assigns = "\n".join(
            f"\t\t{to_go_field(f['name'])}: {f['name']}," for f in fields
        )
        lines.append(f"// New{type_name} creates a new {type_name}.")
        lines.append(f"func New{type_name}({params}) {type_name} {{")
        lines.append(f"\treturn {type_name}{{")
        for f in fields:
            lines.append(f"\t\t{to_go_field(f['name'])}: {f['name']},")
        lines.append("\t}")
        lines.append("}")
        lines.append("")

    # Factories
    for factory in type_def.get("factories", []):
        fname = to_go_field(factory["name"])
        fargs = factory.get("args", [])
        val = factory.get("value")

        if val and not fargs:
            lines.append(f"// {type_name}{fname} returns a predefined {type_name}.")
            lines.append(f"func {type_name}{fname}() {type_name} {{")
            field_vals = ", ".join(str(v) for v in val)
            lines.append(f"\treturn New{type_name}({field_vals})")
            lines.append("}")
            lines.append("")
        elif fargs:
            go_params = ", ".join(
                f"{a['name']} {GO_TYPES.get(a.get('type', 'f32'), 'float32')}"
                for a in fargs
            )
            lines.append(f"// {type_name}{fname} creates a {type_name} from the given arguments.")
            lines.append(f"func {type_name}{fname}({go_params}) {type_name} {{")
            if fname == "FromHex":
                lines.append("\treturn NewColor(")
                lines.append("\t\tfloat32((hex>>16)&0xFF)/255.0,")
                lines.append("\t\tfloat32((hex>>8)&0xFF)/255.0,")
                lines.append("\t\tfloat32(hex&0xFF)/255.0,")
                lines.append("\t\t1.0,")
                lines.append("\t)")
            elif fname == "FromU8":
                lines.append("\treturn NewColor(float32(r)/255.0, float32(g)/255.0, float32(b)/255.0, float32(a)/255.0)")
            elif fname == "Rgba":
                lines.append(f"\treturn New{type_name}(r, g, b, a)")
            elif fname == "Rgb":
                lines.append(f"\treturn New{type_name}(r, g, b, 1.0)")
            else:
                # Generic: try to match args to fields
                field_vals = []
                for f in fields:
                    matched = False
                    for a in fargs:
                        if a["name"] == f["name"]:
                            field_vals.append(a["name"])
                            matched = True
                            break
                    if not matched:
                        field_vals.append(_go_zero(f))
                lines.append(f"\treturn New{type_name}({', '.join(field_vals)})")
            lines.append("}")
            lines.append("")

    # Methods
    for meth in type_def.get("methods", []):
        mname = to_go_field(meth["name"])
        ret = meth.get("returns", "void")
        ret_type = GO_TYPES.get(ret, ret)
        mparams = meth.get("params", [])

        if mname == "Add" and type_name == "Vec2":
            lines.append(f"// Add returns the sum of two Vec2 values.")
            lines.append(f"func (v {type_name}) Add(other {type_name}) {type_name} {{")
            lines.append(f"\treturn New{type_name}(v.X+other.X, v.Y+other.Y)")
            lines.append("}")
        elif mname == "Sub" and type_name == "Vec2":
            lines.append(f"// Sub returns the difference of two Vec2 values.")
            lines.append(f"func (v {type_name}) Sub(other {type_name}) {type_name} {{")
            lines.append(f"\treturn New{type_name}(v.X-other.X, v.Y-other.Y)")
            lines.append("}")
        elif mname == "Scale" and type_name == "Vec2":
            lines.append(f"// Scale returns the Vec2 scaled by s.")
            lines.append(f"func (v {type_name}) Scale(s float32) {type_name} {{")
            lines.append(f"\treturn New{type_name}(v.X*s, v.Y*s)")
            lines.append("}")
        elif mname == "Length" and type_name == "Vec2":
            lines.append(f"// Length returns the magnitude of the Vec2.")
            lines.append(f"func (v {type_name}) Length() float32 {{")
            lines.append(f"\treturn float32(math.Sqrt(float64(v.X*v.X + v.Y*v.Y)))")
            lines.append("}")
        elif mname == "Normalize" and type_name == "Vec2":
            lines.append(f"// Normalize returns a unit-length Vec2.")
            lines.append(f"func (v {type_name}) Normalize() {type_name} {{")
            lines.append(f"\tl := v.Length()")
            lines.append(f"\tif l == 0 {{ return Vec2Zero() }}")
            lines.append(f"\treturn New{type_name}(v.X/l, v.Y/l)")
            lines.append("}")
        elif mname == "Dot" and type_name == "Vec2":
            lines.append(f"// Dot returns the dot product of two Vec2 values.")
            lines.append(f"func (v {type_name}) Dot(other {type_name}) float32 {{")
            lines.append(f"\treturn v.X*other.X + v.Y*other.Y")
            lines.append("}")
        elif mname == "Distance" and type_name == "Vec2":
            lines.append(f"// Distance returns the distance between two Vec2 values.")
            lines.append(f"func (v {type_name}) Distance(other {type_name}) float32 {{")
            lines.append(f"\treturn v.Sub(other).Length()")
            lines.append("}")
        elif mname == "Lerp" and type_name == "Vec2":
            lines.append(f"// Lerp linearly interpolates between two Vec2 values.")
            lines.append(f"func (v {type_name}) Lerp(other {type_name}, t float32) {type_name} {{")
            lines.append(f"\treturn New{type_name}(v.X+(other.X-v.X)*t, v.Y+(other.Y-v.Y)*t)")
            lines.append("}")
        elif mname == "WithAlpha" and type_name == "Color":
            lines.append(f"// WithAlpha returns a new Color with the given alpha.")
            lines.append(f"func (c Color) WithAlpha(a float32) Color {{")
            lines.append(f"\treturn NewColor(c.R, c.G, c.B, a)")
            lines.append("}")
        elif mname == "Lerp" and type_name == "Color":
            lines.append(f"// Lerp linearly interpolates between two Color values.")
            lines.append(f"func (c Color) Lerp(other Color, t float32) Color {{")
            lines.append(f"\treturn NewColor(")
            lines.append(f"\t\tc.R+(other.R-c.R)*t,")
            lines.append(f"\t\tc.G+(other.G-c.G)*t,")
            lines.append(f"\t\tc.B+(other.B-c.B)*t,")
            lines.append(f"\t\tc.A+(other.A-c.A)*t,")
            lines.append(f"\t)")
            lines.append("}")
        elif mname == "Contains" and type_name == "Rect":
            lines.append(f"// Contains returns true if the point is inside the Rect.")
            lines.append(f"func (r Rect) Contains(point Vec2) bool {{")
            lines.append(f"\treturn point.X >= r.X && point.X <= r.X+r.Width &&")
            lines.append(f"\t\tpoint.Y >= r.Y && point.Y <= r.Y+r.Height")
            lines.append("}")
        elif mname == "Intersects" and type_name == "Rect":
            lines.append(f"// Intersects returns true if two Rects overlap.")
            lines.append(f"func (r Rect) Intersects(other Rect) bool {{")
            lines.append(f"\treturn r.X < other.X+other.Width && r.X+r.Width > other.X &&")
            lines.append(f"\t\tr.Y < other.Y+other.Height && r.Y+r.Height > other.Y")
            lines.append("}")
        else:
            # Skip unknown methods
            continue
        lines.append("")


def gen_types() -> None:
    lines = [
        GO_HEADER,
        "",
        "package goud",
        "",
        'import "math"',
        "",
    ]

    for type_name, type_def in schema["types"].items():
        kind = type_def.get("kind")
        if kind == "value":
            _gen_value_type(type_name, type_def, lines)

    write_generated(OUT / "types.go", "\n".join(lines))
