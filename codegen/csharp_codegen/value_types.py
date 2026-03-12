"""Value type generation."""

from sdk_common import to_pascal, write_generated
from .context import HEADER_COMMENT, NS, OUT, schema, mapping, _FFI_TO_SDK_FIELDS, _FFI_TO_SDK_RETURN
from .helpers import cs_type, _type_hash

# ── Value types (Vec2, Color, Rect -- NOT components) ────────────────

_VEC2_METHODS = {
    "Add":       "public Vec2 Add(Vec2 other) => new Vec2(X + other.X, Y + other.Y);",
    "Sub":       "public Vec2 Sub(Vec2 other) => new Vec2(X - other.X, Y - other.Y);",
    "Scale":     "public Vec2 Scale(float s) => new Vec2(X * s, Y * s);",
    "Length":    "public float Length() => MathF.Sqrt(X * X + Y * Y);",
    "Normalize": "public Vec2 Normalize() { var l = Length(); return l == 0 ? Zero() : new Vec2(X / l, Y / l); }",
    "Dot":       "public float Dot(Vec2 other) => X * other.X + Y * other.Y;",
    "Distance":  "public float Distance(Vec2 other) => Sub(other).Length();",
    "Lerp":      "public Vec2 Lerp(Vec2 other, float t) => new Vec2(X + (other.X - X) * t, Y + (other.Y - Y) * t);",
}
_OTHER_METHODS = {
    ("Color", "WithAlpha"):    "public Color WithAlpha(float a) => new Color(R, G, B, a);",
    ("Color", "Lerp"):         "public Color Lerp(Color other, float t) => new Color(R + (other.R - R) * t, G + (other.G - G) * t, B + (other.B - B) * t, A + (other.A - A) * t);",
    ("Rect", "Contains"):      "public bool Contains(Vec2 p) => p.X >= X && p.X <= X + Width && p.Y >= Y && p.Y <= Y + Height;",
    ("Rect", "Intersects"):    "public bool Intersects(Rect o) => X < o.X + o.Width && X + Width > o.X && Y < o.Y + o.Height && Y + Height > o.Y;",
}

_FACTORY_OVERRIDES: dict = {
    ("Color", "Rgb"):     "public static Color Rgb(float r, float g, float b) => new Color(r, g, b, 1f);",
    ("Color", "FromHex"): "public static Color FromHex(uint hex) => new Color(((hex >> 16) & 0xFF) / 255f, ((hex >> 8) & 0xFF) / 255f, (hex & 0xFF) / 255f, 1f);",
    ("Color", "FromU8"):  "public static Color FromU8(byte r, byte g, byte b, byte a) => new Color(r / 255f, g / 255f, b / 255f, a / 255f);",
}

def _factory_line(type_name: str, fname: str, fargs: list, fields: list) -> str:
    override = _FACTORY_OVERRIDES.get((type_name, fname))
    if override:
        return f"        {override}"
    arg_str = ", ".join(f"{cs_type(a['type'])} {a['name']}" for a in fargs)
    pass_str = ", ".join(a["name"] for a in fargs)
    return f"        public static {type_name} {fname}({arg_str}) => new {type_name}({pass_str});"


def _gen_mat3x3():
    """Generate Mat3x3 as a type alias for FfiMat3x3 (unsafe fixed float[9])."""
    lines = [
        f"// {HEADER_COMMENT}",
        "using System;",
        "using System.Runtime.InteropServices;",
        "",
        f"namespace {NS}",
        "{",
        "    /// <summary>3x3 matrix in column-major order for 2D transforms</summary>",
        "    [StructLayout(LayoutKind.Sequential)]",
        "    public struct Mat3x3",
        "    {",
        "        /// <summary>Matrix elements in column-major order</summary>",
        "        public FfiMat3x3 Inner;",
        "",
        "        /// <summary>Gets element at index (0-8)</summary>",
        "        public unsafe float this[int i]",
        "        {",
        "            get { fixed (float* p = Inner.M) { return p[i]; } }",
        "            set { fixed (float* p = Inner.M) { p[i] = value; } }",
        "        }",
        "",
        '        public override string ToString() => "Mat3x3(...)";',
        "    }",
        "}",
        "",
    ]
    write_generated(OUT / "Math" / "Mat3x3.g.cs", "\n".join(lines))


def gen_value_types():
    """Generate value-type structs (Vec2, Color, Rect, etc.) -- NOT component types."""
    for type_name, type_def in schema["types"].items():
        kind = type_def.get("kind")
        if kind == "component":
            continue  # handled by gen_component_wrappers
        if kind != "value":
            continue
        if type_name == "Mat3x3":
            _gen_mat3x3()
            continue
        if type_name == "UiStyle":
            lines = [
                f"// {HEADER_COMMENT}",
                "using System;",
                "",
                f"namespace {NS}",
                "{",
            ]
            if type_def.get("doc"):
                lines.append(f"    /// <summary>{type_def['doc']}</summary>")
            lines += [
                "    public struct UiStyle",
                "    {",
                "        public bool HasBackgroundColor;",
                "        public Color BackgroundColor;",
                "        public bool HasForegroundColor;",
                "        public Color ForegroundColor;",
                "        public bool HasBorderColor;",
                "        public Color BorderColor;",
                "        public bool HasBorderWidth;",
                "        public float BorderWidth;",
                "        public bool HasFontFamily;",
                "        public string? FontFamily;",
                "        public bool HasFontSize;",
                "        public float FontSize;",
                "        public bool HasTexturePath;",
                "        public string? TexturePath;",
                "        public bool HasWidgetSpacing;",
                "        public float WidgetSpacing;",
                "",
                "        public UiStyle(bool hasBackgroundColor, Color backgroundColor, bool hasForegroundColor, Color foregroundColor, bool hasBorderColor, Color borderColor, bool hasBorderWidth, float borderWidth, bool hasFontFamily, string? fontFamily, bool hasFontSize, float fontSize, bool hasTexturePath, string? texturePath, bool hasWidgetSpacing, float widgetSpacing)",
                "        {",
                "            HasBackgroundColor = hasBackgroundColor;",
                "            BackgroundColor = backgroundColor;",
                "            HasForegroundColor = hasForegroundColor;",
                "            ForegroundColor = foregroundColor;",
                "            HasBorderColor = hasBorderColor;",
                "            BorderColor = borderColor;",
                "            HasBorderWidth = hasBorderWidth;",
                "            BorderWidth = borderWidth;",
                "            HasFontFamily = hasFontFamily;",
                "            FontFamily = fontFamily;",
                "            HasFontSize = hasFontSize;",
                "            FontSize = fontSize;",
                "            HasTexturePath = hasTexturePath;",
                "            TexturePath = texturePath;",
                "            HasWidgetSpacing = hasWidgetSpacing;",
                "            WidgetSpacing = widgetSpacing;",
                "        }",
                "",
                '        public override string ToString() => $"UiStyle({HasBackgroundColor}, {BackgroundColor}, {HasForegroundColor}, {ForegroundColor}, {HasBorderColor}, {BorderColor}, {HasBorderWidth}, {BorderWidth}, {HasFontFamily}, {FontFamily}, {HasFontSize}, {FontSize}, {HasTexturePath}, {TexturePath}, {HasWidgetSpacing}, {WidgetSpacing})";',
                "    }",
                "}",
                "",
            ]
            write_generated(OUT / "Math" / "UiStyle.g.cs", "\n".join(lines))
            continue
        fields = type_def.get("fields", [])
        lines = [f"// {HEADER_COMMENT}", "using System;", "", f"namespace {NS}", "{"]
        if type_def.get("doc"):
            lines.append(f"    /// <summary>{type_def['doc']}</summary>")
        lines += [f"    public struct {type_name}", "    {"]
        for f in fields:
            lines.append(f"        public {cs_type(f['type'])} {to_pascal(f['name'])};")
        lines.append("")
        ctor_params = ", ".join(
            f"{cs_type(f['type'])} {to_pascal(f['name']).lower()}" for f in fields
        )
        lines += [f"        public {type_name}({ctor_params})", "        {"]
        for f in fields:
            pn = to_pascal(f["name"])
            lines.append(f"            {pn} = {pn.lower()};")
        lines += ["        }", ""]
        for factory in type_def.get("factories", []):
            fname = to_pascal(factory["name"])
            fargs = factory.get("args", [])
            val = factory.get("value")
            if val and not fargs:
                val_str = ", ".join(f"{v}f" if isinstance(v, (int, float)) else str(v) for v in val)
                lines.append(f"        public static {type_name} {fname}() => new {type_name}({val_str});")
            elif fargs:
                lines.append(_factory_line(type_name, fname, fargs, fields))
        lines.append("")
        for meth in type_def.get("methods", []):
            mn = to_pascal(meth["name"])
            impl = _VEC2_METHODS.get(mn) if type_name == "Vec2" else _OTHER_METHODS.get((type_name, mn))
            if impl:
                lines.append(f"        {impl}")
        lines.append("")
        field_interp = ", ".join(f"{{{to_pascal(f['name'])}}}" for f in fields)
        lines += [f'        public override string ToString() => $"{type_name}({field_interp})";',
                  "    }", "}", ""]
        write_generated(OUT / "Math" / f"{type_name}.g.cs", "\n".join(lines))


# ── Component wrapper generation (Transform2D, Sprite) ───────────────
