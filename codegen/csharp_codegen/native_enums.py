"""NativeMethods and enum generation."""

from sdk_common import CSHARP_TYPES, to_pascal, write_generated
from .context import HEADER_COMMENT, NS, OUT, schema, mapping
from .helpers import (
    cs_type,
    ffi_type,
    _ffi_return_type,
    _cs_ffi_param_type,
    _cs_ffi_ret_type,
    _enum_cs_name,
    _cs_identifier,
)

def gen_native_methods():
    lines = [
        f"// {HEADER_COMMENT}",
        "using System;", "using System.Runtime.InteropServices;", "",
        f"namespace {NS}", "{",
        "    [StructLayout(LayoutKind.Sequential)]",
        "    public struct GoudContextId", "    {",
        "        public ulong _bits;",
        "        public static readonly GoudContextId Invalid = new GoudContextId { _bits = ulong.MaxValue };",
        "        public bool IsValid => _bits != ulong.MaxValue;",
        "    }", "",
        "    [StructLayout(LayoutKind.Sequential)]",
        "    public struct GoudResult", "    {",
        "        public int Code;",
        "        [MarshalAs(UnmanagedType.U1)] public bool Success;",
        "    }", "",
    ]
    # Emit FFI struct declarations for all types (including FfiSprite)
    for type_name, type_def in mapping["ffi_types"].items():
        ffi_name = type_def["ffi_name"]
        if ffi_name in ("u64",):
            continue
        sdk_type = schema["types"].get(type_name)
        if not sdk_type or "fields" not in sdk_type:
            continue
        # Skip FfiMat3x3 -- handled separately with fixed array
        if ffi_name == "FfiMat3x3":
            continue
        lines += ["    [StructLayout(LayoutKind.Sequential)]", f"    public struct {ffi_name}", "    {"]
        for f in sdk_type["fields"]:
            ft = f["type"]
            cs = CSHARP_TYPES.get(ft)
            if cs is None:
                if ft in ("ptr", "usize"):
                    cs = cs_type(ft)
                # Check if it's a schema enum type
                elif ft in schema.get("enums", {}):
                    cs = to_pascal(ft)
                elif ft in mapping.get("ffi_types", {}):
                    cs = mapping["ffi_types"][ft].get("ffi_name", "float")
                else:
                    cs = "float"
            if ft == "bool":
                lines.append("        [MarshalAs(UnmanagedType.U1)]")
            elif ft == "string":
                lines.append("        [MarshalAs(UnmanagedType.LPUTF8Str)]")
            lines.append(f"        public {cs} {to_pascal(f['name'])};")
        lines += ["    }", ""]

    # Emit FfiMat3x3 with fixed array
    lines += [
        "    [StructLayout(LayoutKind.Sequential)]",
        "    public unsafe struct FfiMat3x3", "    {",
        "        public fixed float M[9];",
        "    }", "",
    ]

    # P/Invoke declarations
    lines += ["    public static class NativeMethods", "    {",
              '        private const string DllName = "libgoud_engine";', ""]
    for module, funcs in mapping["ffi_functions"].items():
        if not isinstance(funcs, dict):
            continue
        lines.append(f"        // {module}")
        for fname, fdef in funcs.items():
            if fname.startswith("_") or not isinstance(fdef, dict):
                continue
            params = [f"{_cs_ffi_param_type(p['type'])} {_cs_identifier(p['name'])}" for p in fdef["params"]]
            raw_ret = fdef["returns"]
            ret = _cs_ffi_ret_type(raw_ret)
            if raw_ret == "bool":
                lines += [
                    "        [DllImport(DllName, CallingConvention = CallingConvention.Cdecl)]",
                    "        [return: MarshalAs(UnmanagedType.U1)]",
                    f"        public static extern bool {fname}({', '.join(params)});",
                    "",
                ]
            else:
                lines += [
                    "        [DllImport(DllName, CallingConvention = CallingConvention.Cdecl)]",
                    f"        public static extern {ret} {fname}({', '.join(params)});",
                    "",
                ]
    lines += ["    }", "}", ""]
    write_generated(OUT / "NativeMethods.g.cs", "\n".join(lines))


# ── Enums ────────────────────────────────────────────────────────────

def gen_enums():
    for enum_name, enum_def in schema["enums"].items():
        pascal_name = _enum_cs_name(enum_name)
        underlying = enum_def.get("underlying")
        cs_underlying = CSHARP_TYPES.get(underlying, "") if underlying else ""
        # Omit suffix for int (C# enum default) to avoid unnecessary churn
        suffix = f" : {cs_underlying}" if cs_underlying and cs_underlying != "int" else ""
        lines = [f"// {HEADER_COMMENT}", f"namespace {NS}", "{",
                 f"    public enum {pascal_name}{suffix}", "    {"]
        for vname, vval in enum_def["values"].items():
            lines.append(f"        {vname} = {vval},")
        lines += ["    }", "}", ""]
        write_generated(OUT / "Input" / f"{pascal_name}.g.cs", "\n".join(lines))


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
