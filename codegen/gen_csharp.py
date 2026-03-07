#!/usr/bin/env python3
"""Generates the complete C# SDK from the universal schema."""

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))
from sdk_common import (
    HEADER_COMMENT, SDKS_DIR, load_schema, load_ffi_mapping,
    to_pascal, to_snake, write_generated, CSHARP_TYPES, CSHARP_FFI_TYPES,
)

OUT = SDKS_DIR / "csharp" / "generated"
schema = load_schema()
mapping = load_ffi_mapping()
NS = "GoudEngine"

# FFI struct field names (PascalCase as declared in codegen NativeMethods)
_FFI_TO_SDK_FIELDS = {
    "FfiColor": ["R", "G", "B", "A"],
    "FfiVec2": ["X", "Y"],
    "FfiVec3": ["X", "Y", "Z"],
    "FfiRect": ["X", "Y", "Width", "Height"],
    "FfiTransform2D": ["PositionX", "PositionY", "Rotation", "ScaleX", "ScaleY"],
}
_FFI_TO_SDK_RETURN = {
    "FfiTransform2D": "Transform2D",
    "FfiSprite": "Sprite",
    "FfiColor": "Color",
    "FfiVec2": "Vec2",
    "FfiVec3": "Vec3",
    "FfiRect": "Rect",
    "FfiMat3x3": "Mat3x3",
}


def cs_type(t: str) -> str:
    if t == "ptr":
        return "IntPtr"
    if t == "usize":
        return "nuint"
    return CSHARP_TYPES.get(t, to_pascal(t))


def ffi_type(t: str) -> str:
    return CSHARP_FFI_TYPES.get(t, t)


def _to_cs_field(snake: str) -> str:
    """Convert _snake_case to _camelCase for C# private fields.

    Examples: _delta_time -> _deltaTime, _title -> _title,
              _total_time -> _totalTime, _frame_count -> _frameCount
    """
    parts = snake.lstrip('_').split('_')
    return '_' + parts[0] + ''.join(p.capitalize() for p in parts[1:])


def _cs_default_value(cs_ty: str) -> str:
    """Return the C# default literal for a given C# type."""
    if cs_ty == "string":
        return '""'
    if cs_ty == "double":
        return "0.0"
    if cs_ty == "float":
        return "0.0f"
    if cs_ty in ("uint", "int", "ulong", "long", "byte", "ushort", "short", "sbyte"):
        return "0"
    if cs_ty == "bool":
        return "false"
    return "default"


def _cs_ffi_param_type(raw: str) -> str:
    """Convert an ffi_mapping param type to valid C# for NativeMethods."""
    ptr_map = {
        "*mut FfiTransform2D": "ref FfiTransform2D",
        "*const FfiTransform2D": "ref FfiTransform2D",
        "*mut FfiSprite": "ref FfiSprite",
        "*const FfiSprite": "ref FfiSprite",
        "*mut FfiTransform2DBuilder": "IntPtr",
        "*mut FfiSpriteBuilder": "IntPtr",
        "*mut FfiAnimationClipBuilder": "IntPtr",
        "*const FfiSpriteAnimator": "ref FfiSpriteAnimator",
        "FfiPlaybackMode": "PlaybackMode",
        "*const u8": "IntPtr",
        "*mut u8": "IntPtr",
        "usize": "nuint",
        "u8": "byte",
    }
    if raw in ptr_map:
        return ptr_map[raw]
    return ffi_type(raw)


def _cs_ffi_ret_type(raw: str) -> str:
    """Convert a return type to valid C#."""
    ret_map = {
        "*mut FfiTransform2DBuilder": "IntPtr",
        "*mut FfiSpriteBuilder": "IntPtr",
        "*mut FfiAnimationClipBuilder": "IntPtr",
        "*const u8": "IntPtr",
        "*mut u8": "IntPtr",
        "usize": "nuint",
    }
    if raw in ret_map:
        return ret_map[raw]
    return ffi_type(raw)


def _ffi_return_type(ffi_fn_name: str) -> str:
    """Return the FFI return type string for a named function."""
    for _mod, funcs in mapping["ffi_functions"].items():
        if isinstance(funcs, dict) and ffi_fn_name in funcs:
            return funcs[ffi_fn_name].get("returns", "void")
    return "void"


def _ffi_fn_def(ffi_fn_name: str) -> dict:
    """Look up the full FFI function definition by name."""
    for _mod, funcs in mapping["ffi_functions"].items():
        if isinstance(funcs, dict) and ffi_fn_name in funcs:
            return funcs[ffi_fn_name]
    return {}


def _ffi_uses_ptr_len(ffi_fn_name: str) -> bool:
    """Check if the FFI function uses *const u8 ptr+len for string params."""
    fdef = _ffi_fn_def(ffi_fn_name)
    param_types = [p.get("type", "") for p in fdef.get("params", [])]
    return "*const u8" in param_types


def _enum_cs_name(key: str) -> str:
    if key == "Key":
        return "Keys"
    if key == "MouseButton":
        return "MouseButtons"
    return to_pascal(key)


def _type_hash(type_name: str) -> str:
    """FNV-1a 64-bit hash for component type discrimination."""
    h = 0xcbf29ce484222325
    for b in type_name.encode("utf-8"):
        h ^= b
        h = (h * 0x100000001b3) & 0xFFFFFFFFFFFFFFFF
    return f"0x{h:016x}UL"


# ── NativeMethods.g.cs ──────────────────────────────────────────────

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
                # Check if it's a schema enum type
                if ft in schema.get("enums", {}):
                    cs = to_pascal(ft)
                else:
                    cs = "float"
            if ft == "bool":
                lines.append("        [MarshalAs(UnmanagedType.U1)]")
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
            params = [f"{_cs_ffi_param_type(p['type'])} {p['name']}" for p in fdef["params"]]
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
        lines = [f"// {HEADER_COMMENT}", f"namespace {NS}", "{",
                 f"    public enum {pascal_name}", "    {"]
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

def _gen_comp_factory(type_name, factory_name, ffi_info, schema_factory, lines):
    """Generate a static factory method for a component wrapper struct."""
    ffi_fn = ffi_info["ffi"]
    ffi_def = _ffi_fn_def(ffi_fn)
    ffi_ret = ffi_def.get("returns", "void")
    ffi_name = mapping["ffi_types"].get(type_name, {}).get("ffi_name", "")
    pascal_name = to_pascal(factory_name)
    args = schema_factory.get("args", []) if schema_factory else []
    cs_params = ", ".join(f"{cs_type(a['type'])} {a['name']}" for a in args)
    ffi_args = ", ".join(a["name"] for a in args)
    fields = _FFI_TO_SDK_FIELDS.get(ffi_name, [])

    if schema_factory and schema_factory.get("doc"):
        lines.append(f"        /// <summary>{schema_factory['doc']}</summary>")

    if fields:
        field_refs = ", ".join(f"__r.{f}" for f in fields)
        body = f"var __r = NativeMethods.{ffi_fn}({ffi_args}); return new {type_name}({field_refs});"
    else:
        # No field decomposition -- use internal constructor (e.g., FfiSprite)
        body = f"return new {type_name}(NativeMethods.{ffi_fn}({ffi_args}));"

    lines.append(f"        public static {type_name} {pascal_name}({cs_params})")
    lines.append(f"        {{ {body} }}")
    lines.append("")


def _get_method_names(type_def: dict) -> set:
    """Collect all PascalCase method names for a component type."""
    names = set()
    for m in type_def.get("methods", []):
        names.add(to_pascal(m["name"]))
    return names


def _gen_comp_method(type_name, method_name, ffi_info, schema_method, lines):
    """Generate an instance or static method for a component wrapper struct."""
    ffi_fn = ffi_info["ffi"]
    ffi_def = _ffi_fn_def(ffi_fn)
    ffi_ret = ffi_def.get("returns", "void")
    self_param = ffi_info.get("self_param", "")
    is_static = ffi_info.get("static", False)
    ffi_name = mapping["ffi_types"].get(type_name, {}).get("ffi_name", "")
    pascal_name = to_pascal(method_name)
    params = schema_method.get("params", []) if schema_method else []
    schema_ret = schema_method.get("returns", "void") if schema_method else "void"
    cs_params = ", ".join(f"{cs_type(p['type'])} {p['name']}" for p in params)

    # Determine C# return type
    cs_ret = cs_type(schema_ret) if schema_ret != "void" else "void"
    if schema_ret in ("Transform2D", "Sprite"):
        cs_ret = schema_ret

    if schema_method and schema_method.get("doc"):
        lines.append(f"        /// <summary>{schema_method['doc']}</summary>")

    is_mut_ptr = "*mut" in self_param
    is_const_ptr = "*const" in self_param
    is_by_value = self_param and not is_mut_ptr and not is_const_ptr

    if is_static:
        # Static method (e.g., normalizeAngle)
        ffi_args = ", ".join(p["name"] for p in params)
        lines.append(f"        public static {cs_ret} {pascal_name}({cs_params})")
        if ffi_ret == "void":
            lines.append(f"        {{ NativeMethods.{ffi_fn}({ffi_args}); }}")
        else:
            lines.append(f"        {{ return NativeMethods.{ffi_fn}({ffi_args}); }}")
        lines.append("")

    elif is_mut_ptr and ffi_ret == "void":
        # Mutation method: pass ref _inner
        extra_args = ", ".join(p["name"] for p in params)
        all_args = f"ref _inner, {extra_args}" if extra_args else "ref _inner"
        lines.append(f"        public void {pascal_name}({cs_params})")
        lines.append(f"        {{ NativeMethods.{ffi_fn}({all_args}); }}")
        lines.append("")

    elif (is_const_ptr or is_mut_ptr) and ffi_ret != "void":
        # Query method: pass ref _inner, return converted result
        extra_args = ", ".join(p["name"] for p in params)
        all_args = f"ref _inner, {extra_args}" if extra_args else "ref _inner"
        sdk_ret = _FFI_TO_SDK_RETURN.get(ffi_ret)
        if sdk_ret and ffi_ret in _FFI_TO_SDK_FIELDS:
            fields = _FFI_TO_SDK_FIELDS[ffi_ret]
            field_refs = ", ".join(f"__r.{f}" for f in fields)
            lines.append(f"        public {cs_ret} {pascal_name}({cs_params})")
            lines.append(f"        {{ var __r = NativeMethods.{ffi_fn}({all_args}); return new {sdk_ret}({field_refs}); }}")
        elif sdk_ret:
            # Return type needs wrapping (e.g., FfiMat3x3 -> Mat3x3)
            lines.append(f"        public {cs_ret} {pascal_name}({cs_params})")
            lines.append(f"        {{ var __r = NativeMethods.{ffi_fn}({all_args}); var __w = new {sdk_ret}(); __w.Inner = __r; return __w; }}")
        else:
            lines.append(f"        public {cs_ret} {pascal_name}({cs_params})")
            lines.append(f"        {{ return NativeMethods.{ffi_fn}({all_args}); }}")
        lines.append("")

    elif is_by_value:
        # By-value self: struct passed by value, returns new struct
        ffi_all_params = ffi_def.get("params", [])
        ffi_arg_parts = ["_inner"]
        sdk_param_idx = 0
        for i, fp in enumerate(ffi_all_params):
            if i == 0:
                continue  # skip self
            if sdk_param_idx < len(params):
                p = params[sdk_param_idx]
                if p["type"] in mapping["ffi_types"]:
                    ffi_arg_parts.append(f"{p['name']}._inner")
                else:
                    ffi_arg_parts.append(p["name"])
                sdk_param_idx += 1
            else:
                ffi_arg_parts.append(fp["name"])
        ffi_args = ", ".join(ffi_arg_parts)

        sdk_ret = _FFI_TO_SDK_RETURN.get(ffi_ret)
        if sdk_ret and ffi_ret in _FFI_TO_SDK_FIELDS:
            fields = _FFI_TO_SDK_FIELDS[ffi_ret]
            field_refs = ", ".join(f"__r.{f}" for f in fields)
            lines.append(f"        public {cs_ret} {pascal_name}({cs_params})")
            lines.append(f"        {{ var __r = NativeMethods.{ffi_fn}({ffi_args}); return new {sdk_ret}({field_refs}); }}")
        elif sdk_ret:
            # Return type is a component (e.g., FfiSprite -> Sprite) but no field decomposition
            lines.append(f"        public {cs_ret} {pascal_name}({cs_params})")
            lines.append(f"        {{ return new {sdk_ret}(NativeMethods.{ffi_fn}({ffi_args})); }}")
        elif ffi_ret == "void":
            lines.append(f"        public void {pascal_name}({cs_params})")
            lines.append(f"        {{ NativeMethods.{ffi_fn}({ffi_args}); }}")
        else:
            lines.append(f"        public {cs_ret} {pascal_name}({cs_params})")
            lines.append(f"        {{ return NativeMethods.{ffi_fn}({ffi_args}); }}")
        lines.append("")

    elif is_const_ptr:
        # Const ptr, void return
        extra_args = ", ".join(p["name"] for p in params)
        all_args = f"ref _inner, {extra_args}" if extra_args else "ref _inner"
        lines.append(f"        public void {pascal_name}({cs_params})")
        lines.append(f"        {{ NativeMethods.{ffi_fn}({all_args}); }}")
        lines.append("")

    else:
        lines.append(f"        public {cs_ret} {pascal_name}({cs_params})")
        lines.append('        { throw new System.NotImplementedException(); }')
        lines.append("")


def gen_component_wrappers():
    """Generate component wrapper structs (Transform2D, Sprite) with FFI calls."""
    type_methods = mapping.get("type_methods", {})

    for type_name, type_def in schema["types"].items():
        if type_def.get("kind") != "component":
            continue
        ffi_name = mapping["ffi_types"].get(type_name, {}).get("ffi_name")
        if not ffi_name:
            continue

        tm = type_methods.get(type_name, {})
        fields = type_def.get("fields", [])
        method_names = _get_method_names(type_def)

        lines = [
            f"// {HEADER_COMMENT}",
            "using System;",
            "using System.Runtime.InteropServices;",
            "",
            f"namespace {NS}",
            "{",
        ]
        if type_def.get("doc"):
            lines.append(f"    /// <summary>{type_def['doc']}</summary>")
        lines += [f"    public struct {type_name}", "    {",
                  f"        internal {ffi_name} _inner;", ""]

        # Public properties wrapping inner fields
        # Skip properties whose PascalCase name collides with a method name
        for f in fields:
            pn = to_pascal(f["name"])
            ct = cs_type(f["type"])
            if pn in method_names:
                continue  # Skip: would collide with method of same name
            lines += [
                f"        public {ct} {pn}",
                "        {",
                f"            get => _inner.{pn};",
                f"            set => _inner.{pn} = value;",
                "        }",
            ]
        lines.append("")

        # Constructor from individual fields
        ctor_params = ", ".join(
            f"{cs_type(f['type'])} {to_pascal(f['name']).lower()}" for f in fields
        )
        lines += [f"        public {type_name}({ctor_params})", "        {",
                  f"            _inner = new {ffi_name}();"]
        for f in fields:
            pn = to_pascal(f["name"])
            lines.append(f"            _inner.{pn} = {pn.lower()};")
        lines += ["        }", ""]

        # Internal constructor from FFI struct
        lines += [
            f"        internal {type_name}({ffi_name} inner)",
            "        {",
            "            _inner = inner;",
            "        }", "",
        ]

        # Factory methods
        factories_map = tm.get("factories", {})
        schema_factories = {f["name"]: f for f in type_def.get("factories", [])}
        for fname, ffi_info in factories_map.items():
            _gen_comp_factory(type_name, fname, ffi_info, schema_factories.get(fname), lines)

        # Instance/static methods
        methods_map = tm.get("methods", {})
        schema_methods = {m["name"]: m for m in type_def.get("methods", [])}
        for mname, ffi_info in methods_map.items():
            _gen_comp_method(type_name, mname, ffi_info, schema_methods.get(mname), lines)

        # ToString - use _inner.X for fields whose properties were skipped
        field_parts = []
        for f in fields:
            pn = to_pascal(f["name"])
            if pn in method_names:
                field_parts.append(f"{{_inner.{pn}}}")
            else:
                field_parts.append(f"{{{pn}}}")
        field_interp = ", ".join(field_parts)
        lines.append(f'        public override string ToString() => $"{type_name}({field_interp})";')
        lines += ["    }", ""]

        # Builder class
        builder_def = type_def.get("builder")
        builder_map = tm.get("builder", {})
        if builder_def and builder_map:
            builder_name = f"{type_name}Builder"
            lines += [
                f"    /// <summary>{builder_def.get('doc', '')}</summary>",
                f"    public class {builder_name} : IDisposable",
                "    {",
                "        private IntPtr _ptr;", "",
            ]
            schema_builder_methods = {m["name"]: m for m in builder_def.get("methods", [])}

            for bm_name, bm_info in builder_map.items():
                bm_ffi = bm_info["ffi"]
                bm_schema = schema_builder_methods.get(bm_name, {})
                bm_params = bm_schema.get("params", [])
                bm_pascal = to_pascal(bm_name)
                cs_bm_params = ", ".join(f"{cs_type(p['type'])} {p['name']}" for p in bm_params)

                if bm_schema.get("doc"):
                    lines.append(f"        /// <summary>{bm_schema['doc']}</summary>")

                if bm_name in ("new", "default", "atPosition"):
                    # Static factory for builder
                    ffi_args = ", ".join(p["name"] for p in bm_params)
                    lines.append(f"        public static {builder_name} {bm_pascal}({cs_bm_params})")
                    lines.append(f"        {{ return new {builder_name}(NativeMethods.{bm_ffi}({ffi_args})); }}")
                    lines.append("")
                elif bm_name == "build":
                    # Consumes pointer, returns component
                    ffi_fields = _FFI_TO_SDK_FIELDS.get(ffi_name, [])
                    lines.append(f"        public {type_name} Build()")
                    if ffi_fields:
                        fr = ", ".join(f"__r.{f}" for f in ffi_fields)
                        lines.append(f"        {{ var __r = NativeMethods.{bm_ffi}(_ptr); _ptr = IntPtr.Zero; return new {type_name}({fr}); }}")
                    else:
                        lines.append(f"        {{ var __r = NativeMethods.{bm_ffi}(_ptr); _ptr = IntPtr.Zero; return new {type_name}(__r); }}")
                    lines.append("")
                elif bm_name == "free":
                    lines.append("        public void Free()")
                    lines.append(f"        {{ if (_ptr != IntPtr.Zero) {{ NativeMethods.{bm_ffi}(_ptr); _ptr = IntPtr.Zero; }} }}")
                    lines.append("")
                else:
                    # Chaining method
                    extra = ", ".join(p["name"] for p in bm_params)
                    args = f"_ptr, {extra}" if extra else "_ptr"
                    lines.append(f"        public {builder_name} {bm_pascal}({cs_bm_params})")
                    lines.append(f"        {{ _ptr = NativeMethods.{bm_ffi}({args}); return this; }}")
                    lines.append("")

            lines += [
                f"        private {builder_name}(IntPtr ptr) {{ _ptr = ptr; }}", "",
                "        public void Dispose() => Free();",
            ]
            lines += ["    }", ""]

        lines += ["}", ""]
        write_generated(OUT / "Components" / f"{type_name}.g.cs", "\n".join(lines))


# ── Entity.g.cs ─────────────────────────────────────────────────────

def gen_entity():
    lines = [
        f"// {HEADER_COMMENT}", f"namespace {NS}", "{",
        "    public struct Entity", "    {",
        "        private readonly ulong _bits;", "",
        "        public Entity(ulong bits) { _bits = bits; }", "",
        "        public uint Index => (uint)(_bits & 0xFFFFFFFF);",
        "        public uint Generation => (uint)(_bits >> 32);",
        "        public bool IsPlaceholder => _bits == ulong.MaxValue;",
        "        public ulong ToBits() => _bits;", "",
        "        public static readonly Entity Placeholder = new Entity(ulong.MaxValue);", "",
        '        public override string ToString() => $"Entity({Index}v{Generation})";',
        "    }", "}", "",
    ]
    write_generated(OUT / "Core" / "Entity.g.cs", "\n".join(lines))


# ── Component body generation for tool classes ───────────────────────

def _gen_component_body(mm: dict, ret: str, L: list):
    """Generate method body for component FFI strategies in GoudGame/GoudContext."""
    strategy = mm["ffi_strategy"]
    comp_type = mm.get("component_type", "")
    ffi_name = mapping["ffi_types"].get(comp_type, {}).get("ffi_name", "")
    # Get the actual parameter name from struct_params (e.g., "transform", "sprite")
    struct_params = mm.get("struct_params", [])
    param_name = struct_params[0] if struct_params else to_snake(comp_type)

    if strategy == "component_add":
        L.append("            unsafe")
        L.append("            {")
        L.append(f"                var ffi = {param_name}._inner;")
        L.append("                byte* ptr = (byte*)&ffi;")
        L.append("                NativeMethods.goud_component_add(")
        L.append("                    _ctx, entity.ToBits(),")
        L.append(f"                    {_type_hash(comp_type)},")
        L.append(f"                    (IntPtr)ptr, (nuint)sizeof({ffi_name}));")
        L.append("            }")
    elif strategy == "component_set":
        L.append("            unsafe")
        L.append("            {")
        L.append(f"                var ffi = {param_name}._inner;")
        L.append("                byte* ptr = (byte*)&ffi;")
        L.append("                NativeMethods.goud_component_add(")
        L.append("                    _ctx, entity.ToBits(),")
        L.append(f"                    {_type_hash(comp_type)},")
        L.append(f"                    (IntPtr)ptr, (nuint)sizeof({ffi_name}));")
        L.append("            }")
    elif strategy == "component_get":
        fields = _FFI_TO_SDK_FIELDS.get(ffi_name, [])
        L.append("            unsafe")
        L.append("            {")
        L.append("                IntPtr ptr = NativeMethods.goud_component_get(")
        L.append("                    _ctx, entity.ToBits(),")
        L.append(f"                    {_type_hash(comp_type)});")
        L.append("                if (ptr == IntPtr.Zero) return null;")
        L.append(f"                var ffi = *({ffi_name}*)ptr;")
        if fields:
            field_refs = ", ".join(f"ffi.{f}" for f in fields)
            L.append(f"                return new {comp_type}({field_refs});")
        else:
            # Use internal constructor (e.g., Sprite)
            L.append(f"                return new {comp_type}(ffi);")
        L.append("            }")
    elif strategy == "component_has":
        L.append("            return NativeMethods.goud_component_has(")
        L.append("                _ctx, entity.ToBits(),")
        L.append(f"                {_type_hash(comp_type)});")
    elif strategy == "component_remove":
        L.append("            return NativeMethods.goud_component_remove(")
        L.append("                _ctx, entity.ToBits(),")
        L.append(f"                {_type_hash(comp_type)}).Success;")
    elif strategy == "name_add":
        L.append('            unsafe')
        L.append('            {')
        L.append('                var bytes = System.Text.Encoding.UTF8.GetBytes(name);')
        L.append('                fixed (byte* ptr = bytes)')
        L.append('                {')
        L.append('                    NativeMethods.goud_component_add(')
        L.append('                        _ctx, entity.ToBits(),')
        L.append(f'                        {_type_hash("Name")},')
        L.append('                        (IntPtr)ptr, (nuint)bytes.Length);')
        L.append('                }')
        L.append('            }')
    elif strategy == "name_get":
        L.append('            unsafe')
        L.append('            {')
        L.append('                IntPtr ptr = NativeMethods.goud_component_get(')
        L.append('                    _ctx, entity.ToBits(),')
        L.append(f'                    {_type_hash("Name")});')
        L.append('                if (ptr == IntPtr.Zero) return null;')
        L.append('                return System.Runtime.InteropServices.Marshal.PtrToStringUTF8(ptr);')
        L.append('            }')
    elif strategy == "name_has":
        L.append("            return NativeMethods.goud_component_has(")
        L.append("                _ctx, entity.ToBits(),")
        L.append(f"                {_type_hash('Name')});")
    elif strategy == "name_remove":
        L.append("            return NativeMethods.goud_component_remove(")
        L.append("                _ctx, entity.ToBits(),")
        L.append(f"                {_type_hash('Name')}).Success;")
    else:
        L.append(f'            throw new System.NotImplementedException("Unknown strategy: {strategy}");')


_I = "            "  # 12-space indent shorthand for body lines
_WINDOWED_BODIES: dict = {
    "BeginFrame": [
        f"{_I}_deltaTime = NativeMethods.goud_window_poll_events(_ctx);",
        f"{_I}NativeMethods.goud_window_clear(_ctx, r, g, b, a);",
        f"{_I}NativeMethods.goud_renderer_begin(_ctx);",
        f"{_I}NativeMethods.goud_renderer_enable_blending(_ctx);",
    ],
    "EndFrame": [f"{_I}NativeMethods.goud_renderer_end(_ctx);", f"{_I}NativeMethods.goud_window_swap_buffers(_ctx);"],
    "Run":      [f"{_I}while (!ShouldClose())", f"{_I}{{", f"{_I}    BeginFrame();",
                 f"{_I}    update(_deltaTime);", f"{_I}    EndFrame();", f"{_I}}}"],
    "DrawSprite":     [f"{_I}var c = color ?? Color.White();",
                       f"{_I}NativeMethods.goud_renderer_draw_sprite(_ctx, texture, x, y, width, height, rotation, c.R, c.G, c.B, c.A);"],
    "DrawSpriteRect": [f"{_I}var c = color ?? Color.White();",
                       f"{_I}return NativeMethods.goud_renderer_draw_sprite_rect(_ctx, texture, x, y, width, height, rotation, srcX, srcY, srcW, srcH, c.R, c.G, c.B, c.A);"],
    "DrawQuad":       [f"{_I}var c = color ?? Color.White();",
                       f"{_I}NativeMethods.goud_renderer_draw_quad(_ctx, x, y, width, height, c.R, c.G, c.B, c.A);"],
    "LoadTexture":    [f"{_I}return NativeMethods.goud_texture_load(_ctx, path);"],
    "DestroyTexture": [f"{_I}NativeMethods.goud_texture_destroy(_ctx, handle);"],
    "Close":          [f"{_I}NativeMethods.goud_window_set_should_close(_ctx, true);"],
    "ShouldClose":    [f"{_I}return NativeMethods.goud_window_should_close(_ctx);"],
    "UpdateFrame":    [f"{_I}_deltaTime = (float)dt;",
                       f"{_I}_frameCount++;",
                       f"{_I}_totalTime += dt;"],
}


def _gen_method_body(mn: str, mm: dict, params: list, ret: str, L: list, is_windowed: bool):
    """Emit the body statements for one method into list L."""
    if mn == "Destroy":
        fn = "goud_window_destroy" if is_windowed else "goud_context_destroy"
        L.append(f"            if (!_disposed) {{ NativeMethods.{fn}(_ctx); _disposed = true; }}")
        if ret == "bool": L.append("            return true;")
        return
    if mn == "IsValid":
        L.append("            return NativeMethods.goud_context_is_valid(_ctx);"); return
    if mn in _WINDOWED_BODIES:
        L.extend(_WINDOWED_BODIES[mn])
        return
    if "ffi_strategy" in mm:
        _gen_component_body(mm, ret, L)
        return
    if mm.get("returns_nullable_struct"):
        struct_name = mm["returns_nullable_struct"]
        ffi_fn = mm["ffi"]
        ctx_prefix = "" if mm.get("no_context") else "_ctx, "
        ffi_args = ", ".join(p["name"] for p in params)
        L += ["            var contact = new GoudContact();",
              f"            if (!NativeMethods.{ffi_fn}({ctx_prefix}{ffi_args}, ref contact)) return null;",
              f"            return new {struct_name}(contact.PointX, contact.PointY, contact.NormalX, contact.NormalY, contact.Penetration);"]
        return
    if mm.get("batch_out"):
        ffi_fn = mm["ffi"]
        L += ["            var buf = new ulong[count];",
              "            uint filled;",
              "            unsafe { fixed (ulong* p = buf) { filled = NativeMethods." + ffi_fn + "(_ctx, count, ref buf[0]); } }",
              "            var result = new Entity[filled];",
              "            for (uint i = 0; i < filled; i++) result[i] = new Entity(buf[i]);",
              "            return result;"]
        return
    if mm.get("batch_in"):
        ffi_fn = mm["ffi"]
        # Collect extra params beyond the entities array
        extra_params = [p for p in params if p["type"] != "Entity[]" and p["name"] != "entities"]
        # Build extra args, converting u8[] to pinned IntPtr
        has_byte_array = any(p["type"] == "u8[]" for p in extra_params)
        non_byte_extra = [p for p in extra_params if p["type"] != "u8[]"]
        byte_array_param = next((p for p in extra_params if p["type"] == "u8[]"), None)
        L += ["            var buf = new ulong[entities.Length];",
              "            for (int i = 0; i < entities.Length; i++) buf[i] = entities[i].ToBits();"]
        if byte_array_param:
            ba_name = byte_array_param["name"]
            non_byte_args = ", ".join(p["name"] for p in non_byte_extra)
            non_byte_suffix = f", {non_byte_args}" if non_byte_args else ""
            L += ["            unsafe",
                  "            {",
                  "                fixed (ulong* p = buf)",
                  f"                fixed (byte* outp = {ba_name})",
                  "                {",
                  f"                    return NativeMethods.{ffi_fn}(_ctx, (System.IntPtr)p, (uint)entities.Length{non_byte_suffix}, (System.IntPtr)outp);",
                  "                }",
                  "            }"]
        else:
            extra_args_str = ", ".join(p["name"] for p in extra_params)
            extra_suffix = f", {extra_args_str}" if extra_args_str else ""
            L += ["            unsafe",
                  "            {",
                  "                fixed (ulong* p = buf)",
                  "                {",
                  f"                    return NativeMethods.{ffi_fn}(_ctx, (System.IntPtr)p, (uint)entities.Length{extra_suffix});",
                  "                }",
                  "            }"]
        return
    if mm.get("returns_entity"):
        if "entity_params" in mm:
            # Convert entity parameters to bits
            entity_args = ", ".join(f"{p}.ToBits()" for p in mm["entity_params"])
            L.append(f"            return new Entity(NativeMethods.{mm['ffi']}(_ctx, {entity_args}));")
        else:
            L.append(f"            return new Entity(NativeMethods.{mm['ffi']}(_ctx));")
        return
    if "entity_params" in mm and "ffi" in mm:
        ffi_fn = mm["ffi"]
        ffi_ret = _ffi_return_type(ffi_fn)
        suffix = ".Success" if ret == "bool" and ffi_ret == "GoudResult" else ""
        # Collect non-entity params
        entity_set = set(mm["entity_params"])
        extra_params = [p for p in params if p["name"] not in entity_set]
        if extra_params:
            extra_args = ", ".join(p["name"] for p in extra_params)
            L.append(f"            return NativeMethods.{ffi_fn}(_ctx, entity.ToBits(), {extra_args}){suffix};")
        else:
            L.append(f"            return NativeMethods.{ffi_fn}(_ctx, entity.ToBits()){suffix};")
        return
    if "enum_params" in mm and mm.get("ffi"):
        ek = list(mm["enum_params"].keys())[0]
        ffi_fn = mm["ffi"]
        prefix = "return " if ret != "void" else ""
        if mm.get("string_params"):
            sp = mm["string_params"][0]
            L.append(f"            {prefix}NativeMethods.{ffi_fn}(_ctx, {sp}, (int){ek});")
        else:
            L.append(f"            {prefix}NativeMethods.{ffi_fn}(_ctx, (int){ek});")
        return
    if "out_params" in mm and "returns_struct" in mm and any(op["type"] != "f32" for op in mm["out_params"]):
        struct_name = mm["returns_struct"]
        ffi_fn = mm["ffi"]
        ffi_type_name = mapping["ffi_types"][struct_name]["ffi_name"]
        rs_fields = schema["types"][struct_name]["fields"]
        field_args = ", ".join(f"stats.{to_pascal(f['name'])}" for f in rs_fields)
        L += [f"            var stats = new {ffi_type_name}();",
              f"            NativeMethods.{ffi_fn}(_ctx, ref stats);",
              f"            return new {struct_name}({field_args});"]
        return
    if "out_params" in mm:
        for op in mm["out_params"]:
            L.append(f"            float {op['name']} = 0;")
        refs = ", ".join(f"ref {op['name']}" for op in mm["out_params"])
        vals = ", ".join(op["name"] for op in mm["out_params"])
        L += [f"            NativeMethods.{mm['ffi']}(_ctx, {refs});",
              f"            return new Vec2({vals});"]
        return
    if "append_args" in mm:
        extra = ", ".join(str(a).lower() for a in mm["append_args"])
        L.append(f"            NativeMethods.{mm['ffi']}(_ctx, {extra});")
        return
    if "ffi" in mm:
        no_ctx = mm.get("no_context", False)
        ffi_fn = mm["ffi"]
        # Special case: componentRegisterType needs string->IntPtr marshalling
        if ffi_fn == "goud_component_register_type":
            L += ["            unsafe",
                  "            {",
                  "                var nameBytes = System.Text.Encoding.UTF8.GetBytes(name);",
                  "                fixed (byte* np = nameBytes)",
                  "                {",
                  f"                    return NativeMethods.{ffi_fn}(typeIdHash, (IntPtr)np, (nuint)nameBytes.Length, size, align);",
                  "                }",
                  "            }"]
            return
        # Generic string param marshalling: string -> UTF8 ptr + len
        # Only applies when the FFI function uses *const u8 (ptr+len), not *const c_char
        string_params = [p for p in params if p["type"] == "string"]
        if string_params and _ffi_uses_ptr_len(ffi_fn):
            non_string = [p for p in params if p["type"] != "string"]
            L.append("            unsafe")
            L.append("            {")
            byte_vars = []
            fixed_lines = []
            ffi_arg_parts = [] if no_ctx else ["_ctx"]
            for p in params:
                if p["type"] == "string":
                    bvar = f"{p['name']}Bytes"
                    pvar = f"{p['name']}Ptr"
                    L.append(f"                var {bvar} = System.Text.Encoding.UTF8.GetBytes({p['name']});")
                    fixed_lines.append(f"byte* {pvar} = {bvar}")
                    ffi_arg_parts.append(f"(IntPtr){pvar}")
                    ffi_arg_parts.append(f"(uint){bvar}.Length")
                else:
                    ffi_arg_parts.append(p["name"])
            fixed_expr = "\n                ".join(f"fixed ({fl})" for fl in fixed_lines)
            L.append(f"                {fixed_expr}")
            L.append("                {")
            call = f"NativeMethods.{ffi_fn}({', '.join(ffi_arg_parts)});"
            L.append(f"                    {'return ' if ret != 'void' else ''}{call}")
            L.append("                }")
            L.append("            }")
            return
        ffi_args = ", ".join(p["name"] for p in params)
        all_args = ffi_args if no_ctx else (f"_ctx, {ffi_args}" if ffi_args else "_ctx")
        stmt = f"NativeMethods.{ffi_fn}({all_args});"
        L.append(f"            {'return ' if ret != 'void' else ''}{stmt}")
        return
    L.append("            // TODO: implement")


# ── Param string building ─────────────────────────────────────────────

def _build_param_strs(params: list) -> list:
    result = []
    for p in params:
        pt, name, default = p["type"], p["name"], p.get("default")
        if pt == "callback(f32)":
            result.append(f"Action<float> {name}")
        elif pt == "Entity[]":
            result.append(f"Entity[] {name}")
        elif pt == "u8[]":
            result.append(f"byte[] {name}")
        elif pt in schema["types"]:
            result.append(f"{to_pascal(pt)}? {name} = null" if default else f"{to_pascal(pt)} {name}")
        elif pt in schema["enums"]:
            result.append(f"{_enum_cs_name(pt)} {name}")
        else:
            ct = cs_type(pt)
            if default is not None:
                if isinstance(default, float):
                    ds = f"{default}f"
                elif isinstance(default, str) and not default.replace(".", "").replace("-", "").isdigit():
                    ds = f'"{default}"'
                else:
                    ds = str(default)
                result.append(f"{ct} {name} = {ds}")
            else:
                result.append(f"{ct} {name}")
    return result


def _safe_param_strs(params: list) -> list:
    """Strip defaults from non-trailing optional params to avoid compile errors."""
    raw = _build_param_strs(params)
    last_required = -1
    for i, p in enumerate(params):
        is_opt = p.get("default") is not None or p["type"] in schema["types"]
        if not is_opt:
            last_required = i
    return [s.split("=")[0].rstrip() if (i < last_required and "=" in s) else s
            for i, s in enumerate(raw)]


# ── Tool class generation ─────────────────────────────────────────────

def _gen_tool_class(tool_name: str, tm: dict, out_path, is_windowed: bool = False):
    tool = schema["tools"][tool_name]
    class_name = tool_name
    extra = []
    if is_windowed:
        for prop in tool.get("properties", []):
            pm_check = tm.get("properties", {}).get(prop["name"], {})
            if pm_check.get("source") == "cached":
                pt_priv = cs_type(prop["type"])
                field = _to_cs_field(pm_check.get("field", f"_{to_snake(prop['name'])}"))
                extra.append(f"        private {pt_priv} {field};")
    ctor_params = tool.get("constructor", {}).get("params", [])
    ctor_ffi = tm.get("constructor", {}).get("ffi", "goud_context_create")

    if ctor_params:
        cs_ps = []
        for p in ctor_params:
            ct = cs_type(p["type"])
            d = p.get("default")
            if d is None:
                cs_ps.append(f"{ct} {p['name']}")
            elif isinstance(d, str) and not str(d).isdigit():
                cs_ps.append(f'{ct} {p["name"]} = "{d}"')
            else:
                cs_ps.append(f"{ct} {p['name']} = {d}")
        ctor_sig = ", ".join(cs_ps)
        ctor_call = f"NativeMethods.{ctor_ffi}({', '.join(p['name'] for p in ctor_params)})"
    else:
        ctor_sig = ""
        ctor_call = f"NativeMethods.{ctor_ffi}()"

    err_msg = "Failed to create GLFW window" if is_windowed else "Failed to create headless context"
    # Build constructor body with cached field initialization
    ctor_body_lines = [
        f"            _ctx = {ctor_call};",
        f'            if (!_ctx.IsValid) throw new Exception("{err_msg}");',
    ]
    if is_windowed:
        ctor_param_names = {p["name"] for p in ctor_params}
        for prop in tool.get("properties", []):
            pm_init = tm.get("properties", {}).get(prop["name"], {})
            if pm_init.get("source") == "cached":
                field = _to_cs_field(pm_init.get("field", f"_{to_snake(prop['name'])}"))
                if prop["name"] in ctor_param_names:
                    ctor_body_lines.append(f"            {field} = {prop['name']};")
                else:
                    default_val = _cs_default_value(cs_type(prop["type"]))
                    ctor_body_lines.append(f"            {field} = {default_val};")

    lines = [
        f"// {HEADER_COMMENT}",
        "using System;", "using System.Runtime.InteropServices;", "",
        f"namespace {NS}", "{",
        f"    /// <summary>{tool.get('doc', class_name)}</summary>",
        f"    public class {class_name} : IDisposable", "    {",
        "        private GoudContextId _ctx;",
        "        private bool _disposed;",
        *extra, "",
        f"        public {class_name}({ctor_sig})", "        {",
        *ctor_body_lines,
        "        }", "",
    ]

    # Properties (windowed only)
    for prop in tool.get("properties", []):
        pn = to_pascal(prop["name"])
        pt = cs_type(prop["type"])
        pm = tm.get("properties", {}).get(prop["name"], {})
        src = pm.get("source", "")
        if src == "cached":
            field = _to_cs_field(pm.get("field", f"_{to_snake(prop['name'])}"))
            lines.append(f"        public {pt} {pn} => {field};")
        elif src == "computed":
            # Computed properties reference their dependent cached fields
            lines.append(f"        public {pt} {pn} => _deltaTime > 0 ? 1f / _deltaTime : 0f;")
        elif "out_index" in pm:
            idx = pm["out_index"]
            lines += [f"        public {pt} {pn}", "        {", "            get", "            {",
                      "                uint w = 0, h = 0;",
                      f"                NativeMethods.{pm['ffi']}(_ctx, ref w, ref h);",
                      f"                return {'w' if idx == 0 else 'h'};",
                      "            }", "        }"]
        lines.append("")

    # Methods
    for method in tool.get("methods", []):
        mn = to_pascal(method["name"])
        mm = tm.get("methods", {}).get(method["name"], {})
        params = method.get("params", [])
        ret = method.get("returns", "void")
        cs_ret = cs_type(ret.rstrip("?").rstrip("[]"))
        if ret.endswith("[]"):
            actual_ret = f"{cs_ret}[]"
        elif method.get("nullable", False):
            actual_ret = f"{cs_ret}?"
        elif method.get("async"):
            actual_ret = cs_ret
        else:
            actual_ret = cs_ret

        sig = ", ".join(_safe_param_strs(params))
        if method.get("doc"):
            lines.append(f"        /// <summary>{method['doc']}</summary>")
        lines += [f"        public {actual_ret} {mn}({sig})", "        {"]
        _gen_method_body(mn, mm, params, ret, lines, is_windowed)
        lines += ["        }", ""]

    dispose_body = "Destroy()" if is_windowed else "_ctx = GoudContextId.Invalid"
    lines += [f"        public void Dispose() => {dispose_body};", "    }", "}", ""]
    write_generated(out_path, "\n".join(lines))


def gen_game():
    _gen_tool_class("GoudGame", mapping["tools"]["GoudGame"], OUT / "GoudGame.g.cs", is_windowed=True)


def gen_context():
    _gen_tool_class("GoudContext", mapping["tools"]["GoudContext"], OUT / "GoudContext.g.cs", is_windowed=False)


if __name__ == "__main__":
    print("Generating C# SDK...")
    gen_native_methods()
    gen_enums()
    gen_value_types()
    gen_component_wrappers()
    gen_entity()
    gen_game()
    gen_context()
    print("C# SDK generation complete.")
