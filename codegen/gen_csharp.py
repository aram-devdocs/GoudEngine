#!/usr/bin/env python3
"""Generates the complete C# SDK from the universal schema."""

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))
from sdk_common import (
    HEADER_COMMENT, SDKS_DIR, load_schema, load_ffi_mapping,
    to_pascal, write_generated, CSHARP_TYPES, CSHARP_FFI_TYPES,
)

OUT = SDKS_DIR / "csharp" / "generated"
schema = load_schema()
mapping = load_ffi_mapping()
NS = "GoudEngine"


def cs_type(t: str) -> str:
    return CSHARP_TYPES.get(t, to_pascal(t))


def ffi_type(t: str) -> str:
    return CSHARP_FFI_TYPES.get(t, t)


def _ffi_return_type(ffi_fn_name: str) -> str:
    """Return the FFI return type string for a named function."""
    for _mod, funcs in mapping["ffi_functions"].items():
        if isinstance(funcs, dict) and ffi_fn_name in funcs:
            return funcs[ffi_fn_name].get("returns", "void")
    return "void"


def _enum_cs_name(key: str) -> str:
    if key == "Key":
        return "Keys"
    if key == "MouseButton":
        return "MouseButtons"
    return to_pascal(key)


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
    for type_name, type_def in mapping["ffi_types"].items():
        ffi_name = type_def["ffi_name"]
        if ffi_name in ("u64", "FfiSprite"):
            continue
        sdk_type = schema["types"].get(type_name)
        if not sdk_type or "fields" not in sdk_type:
            continue
        lines += ["    [StructLayout(LayoutKind.Sequential)]", f"    public struct {ffi_name}", "    {"]
        for f in sdk_type["fields"]:
            cs = CSHARP_TYPES.get(f["type"], "float")
            if f["type"] == "bool":
                lines.append("        [MarshalAs(UnmanagedType.U1)]")
            lines.append(f"        public {cs} {to_pascal(f['name'])};")
        lines += ["    }", ""]
    lines += ["    public static class NativeMethods", "    {",
              '        private const string DllName = "libgoud_engine";', ""]
    for module, funcs in mapping["ffi_functions"].items():
        if not isinstance(funcs, dict):
            continue
        lines.append(f"        // {module}")
        for fname, fdef in funcs.items():
            params = [f"{ffi_type(p['type'])} {p['name']}" for p in fdef["params"]]
            raw_ret = fdef["returns"]
            ret = ffi_type(raw_ret)
            # bool return needs [return: MarshalAs] attribute, not inline
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


# ── Value / component types ──────────────────────────────────────────

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

# Factories whose argument list does not cover all constructor fields need
# explicit body strings. Key: (TypeName, FactoryPascalName).
_FACTORY_OVERRIDES: dict = {
    # Transform2D partial-arg factories — fill missing fields from Default() values.
    ("Transform2D", "FromPosition"): "public static Transform2D FromPosition(float x, float y) => new Transform2D(x, y, 0f, 1f, 1f);",
    ("Transform2D", "FromRotation"): "public static Transform2D FromRotation(float radians) => new Transform2D(0f, 0f, radians, 1f, 1f);",
    ("Transform2D", "FromScale"):    "public static Transform2D FromScale(float x, float y) => new Transform2D(0f, 0f, 0f, x, y);",
    # Color partial-arg factories.
    ("Color", "Rgb"):     "public static Color Rgb(float r, float g, float b) => new Color(r, g, b, 1f);",
    ("Color", "FromHex"): "public static Color FromHex(uint hex) => new Color(((hex >> 16) & 0xFF) / 255f, ((hex >> 8) & 0xFF) / 255f, (hex & 0xFF) / 255f, 1f);",
    ("Color", "FromU8"):  "public static Color FromU8(byte r, byte g, byte b, byte a) => new Color(r / 255f, g / 255f, b / 255f, a / 255f);",
}


def _factory_line(type_name: str, fname: str, fargs: list, fields: list) -> str:
    """Return the C# factory method line for a given factory."""
    # Check for an explicit override first.
    override = _FACTORY_OVERRIDES.get((type_name, fname))
    if override:
        return f"        {override}"
    # Simple case: factory args cover all constructor fields exactly.
    arg_str = ", ".join(f"{cs_type(a['type'])} {a['name']}" for a in fargs)
    pass_str = ", ".join(a["name"] for a in fargs)
    return f"        public static {type_name} {fname}({arg_str}) => new {type_name}({pass_str});"


def gen_value_types():
    for type_name, type_def in schema["types"].items():
        kind = type_def.get("kind")
        if kind not in ("value", "component"):
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
        subdir = "Components" if kind == "component" else "Math"
        write_generated(OUT / subdir / f"{type_name}.g.cs", "\n".join(lines))


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


# ── Method body generation ───────────────────────────────────────────

def _comp_snake(comp: str) -> str:
    return comp[0].lower() + comp[1:] if comp else ""


def _transform2d_flat(var: str) -> str:
    return f"{var}.PositionX, {var}.PositionY, {var}.Rotation, {var}.ScaleX, {var}.ScaleY"


def _sprite_flat(var: str) -> str:
    return (f"{var}.TextureHandle, {var}.Color.R, {var}.Color.G, {var}.Color.B, {var}.Color.A, "
            f"{var}.FlipX, {var}.FlipY, {var}.AnchorX, {var}.AnchorY")


def _gen_component_body(mm: dict, ret: str, L: list):
    """Generate method body for component FFI strategies.

    Component FFI functions (goud_component_xxx_yyy) are not yet implemented
    in the Rust core or exposed in the codegen NativeMethods. Until those FFI
    exports exist, all component operations throw NotImplementedException so
    the SDK compiles and a clear runtime error is raised instead of a missing
    symbol at link time.
    """
    strategy = mm["ffi_strategy"]
    # Determine whether the method returns a value that needs a return statement.
    # Void-return strategies: component_add, component_set, name_add.
    void_strategies = {"component_add", "component_set", "name_add"}
    if strategy in void_strategies:
        L.append('            throw new System.NotImplementedException("Component FFI not yet implemented.");')
    else:
        L.append('            throw new System.NotImplementedException("Component FFI not yet implemented.");')


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
        # NativeMethods signature: goud_entity_spawn_batch(ctx, count, ref ulong out_entities)
        # Must pin the managed array and pass ref to first element.
        ffi_fn = mm["ffi"]
        L += ["            var buf = new ulong[count];",
              "            uint filled;",
              "            unsafe { fixed (ulong* p = buf) { filled = NativeMethods." + ffi_fn + "(_ctx, count, ref buf[0]); } }",
              "            var result = new Entity[filled];",
              "            for (uint i = 0; i < filled; i++) result[i] = new Entity(buf[i]);",
              "            return result;"]
        return
    if mm.get("batch_in"):
        # NativeMethods signature: goud_entity_despawn_batch(ctx, IntPtr entity_ids, count)
        # Must pin the managed array and pass its address as IntPtr.
        ffi_fn = mm["ffi"]
        L += ["            var buf = new ulong[entities.Length];",
              "            for (int i = 0; i < entities.Length; i++) buf[i] = entities[i].ToBits();",
              "            unsafe",
              "            {",
              "                fixed (ulong* p = buf)",
              "                {",
              f"                    return NativeMethods.{ffi_fn}(_ctx, (System.IntPtr)p, (uint)entities.Length);",
              "                }",
              "            }"]
        return
    if mm.get("returns_entity"):
        L.append(f"            return new Entity(NativeMethods.{mm['ffi']}(_ctx));")
        return
    if "entity_params" in mm and "ffi" in mm:
        ffi_fn = mm["ffi"]
        ffi_ret = _ffi_return_type(ffi_fn)
        suffix = ".Success" if ret == "bool" and ffi_ret == "GoudResult" else ""
        L.append(f"            return NativeMethods.{ffi_fn}(_ctx, entity.ToBits()){suffix};")
        return
    if "enum_params" in mm and mm.get("ffi"):
        ek = list(mm["enum_params"].keys())[0]
        ffi_fn = mm["ffi"]
        if mm.get("string_params"):
            sp = mm["string_params"][0]
            L.append(f"            return NativeMethods.{ffi_fn}(_ctx, {sp}, (int){ek});")
        else:
            L.append(f"            return NativeMethods.{ffi_fn}(_ctx, (int){ek});")
        return
    if "out_params" in mm and mm.get("returns_struct") == "RenderStats":
        ffi_fn = mm["ffi"]
        L += ["            var stats = new GoudRenderStats();",
              f"            NativeMethods.{ffi_fn}(_ctx, ref stats);",
              "            return new RenderStats(stats.DrawCalls, stats.Triangles, stats.TextureBinds);"]
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
        ffi_args = ", ".join(p["name"] for p in params)
        all_args = ffi_args if no_ctx else (f"_ctx, {ffi_args}" if ffi_args else "_ctx")
        stmt = f"NativeMethods.{mm['ffi']}({all_args});"
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
    extra = ["        private float _deltaTime;"] if is_windowed else []
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
        f"            _ctx = {ctor_call};",
        f'            if (!_ctx.IsValid) throw new Exception("{err_msg}");',
        "        }", "",
    ]

    # Properties (windowed only)
    for prop in tool.get("properties", []):
        pn = to_pascal(prop["name"])
        pt = cs_type(prop["type"])
        pm = tm.get("properties", {}).get(prop["name"], {})
        src = pm.get("source", "")
        if src == "cached":
            lines.append(f"        public {pt} {pn} => _deltaTime;")
        elif src == "computed":
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
    gen_entity()
    gen_game()
    gen_context()
    print("C# SDK generation complete.")
