"""Value type generation for Kotlin SDK."""
from __future__ import annotations
from .helpers import HEADER_COMMENT, KOTLIN_OUT, schema, to_pascal, to_camel, kt_type, write_kotlin, kdoc

_VEC2_METHODS = {
    "add": "fun add(other: Vec2): Vec2 = Vec2(x + other.x, y + other.y)",
    "sub": "fun sub(other: Vec2): Vec2 = Vec2(x - other.x, y - other.y)",
    "scale": "fun scale(s: Float): Vec2 = Vec2(x * s, y * s)",
    "length": "fun length(): Float = kotlin.math.sqrt(x * x + y * y)",
    "normalize": "fun normalize(): Vec2 { val l = length(); return if (l == 0f) zero() else Vec2(x / l, y / l) }",
    "dot": "fun dot(other: Vec2): Float = x * other.x + y * other.y",
    "distance": "fun distance(other: Vec2): Float = sub(other).length()",
    "lerp": "fun lerp(other: Vec2, t: Float): Vec2 = Vec2(x + (other.x - x) * t, y + (other.y - y) * t)",
}
_OTHER_METHODS = {
    ("Color", "withAlpha"): "fun withAlpha(a: Float): Color = Color(r, g, b, a)",
    ("Color", "lerp"): "fun lerp(other: Color, t: Float): Color = Color(r + (other.r - r) * t, g + (other.g - g) * t, b + (other.b - b) * t, a + (other.a - a) * t)",
    ("Rect", "contains"): "fun contains(p: Vec2): Boolean = p.x >= x && p.x <= x + width && p.y >= y && p.y <= y + height",
    ("Rect", "intersects"): "fun intersects(o: Rect): Boolean = x < o.x + o.width && x + width > o.x && y < o.y + o.height && y + height > o.y",
}
_FACTORY_OVERRIDES: dict = {
    ("Color", "rgb"): "fun rgb(r: Float, g: Float, b: Float): Color = Color(r, g, b, 1f)",
    ("Color", "fromHex"): "fun fromHex(hex: Int): Color = Color(((hex shr 16) and 0xFF) / 255f, ((hex shr 8) and 0xFF) / 255f, (hex and 0xFF) / 255f, 1f)",
    ("Color", "fromU8"): "fun fromU8(r: Int, g: Int, b: Int, a: Int): Color = Color(r / 255f, g / 255f, b / 255f, a / 255f)",
}

def _gen_factory(type_name, factory, fields):
    fname = to_camel(factory["name"])
    override = _FACTORY_OVERRIDES.get((type_name, fname))
    if override:
        return f"        {override}"
    fargs = factory.get("args", [])
    val = factory.get("value")
    if val and not fargs:
        val_str = ", ".join(f"{v}f" if isinstance(v, (int, float)) else str(v) for v in val)
        return f"        fun {fname}(): {type_name} = {type_name}({val_str})"
    arg_str = ", ".join(f"{a['name']}: {kt_type(a['type'])}" for a in fargs)
    pass_str = ", ".join(a["name"] for a in fargs)
    return f"        fun {fname}({arg_str}): {type_name} = {type_name}({pass_str})"

def gen_value_types():
    for type_name, type_def in schema["types"].items():
        if type_def.get("kind") != "value":
            continue
        if type_name in ("Mat3x3", "UiStyle", "NetworkPacket", "DebuggerCapture",
                         "DebuggerReplayArtifact", "ContextConfig", "MemorySummary"):
            continue
        fields = type_def.get("fields", [])
        if not fields:
            continue
        simple_types = {"f32","f64","u8","u16","u32","u64","i8","i16","i32","i64","bool","string","usize","ptr"}
        if any(f["type"] not in simple_types for f in fields):
            continue
        is_carrier = type_name in ("Color", "Vec2", "Vec3", "Rect", "P2pMeshConfig", "RollbackConfig")
        lines = [f"// {HEADER_COMMENT}", "package com.goudengine.types", ""]
        if is_carrier:
            lines += [f"import com.goudengine.internal.{type_name} as Java{type_name}", ""]
        doc = type_def.get("doc")
        lines.extend(kdoc(doc))
        field_params = ", ".join(f"val {to_camel(f['name'])}: {kt_type(f['type'])}" for f in fields)
        lines.append(f"data class {type_name}({field_params}) {{")
        methods = type_def.get("methods", [])
        if methods:
            lines.append("")
            for meth in methods:
                mn = to_camel(meth["name"])
                impl = _VEC2_METHODS.get(mn) if type_name == "Vec2" else _OTHER_METHODS.get((type_name, mn))
                if impl:
                    lines.append(f"    {impl}")
        if is_carrier:
            lines.append("")
            to_args = ", ".join(to_camel(f["name"]) for f in fields)
            lines.append(f"    fun toNative(): Java{type_name} = Java{type_name}({to_args})")
        factories = type_def.get("factories", [])
        if factories or is_carrier:
            lines += ["", "    companion object {"]
            for factory in factories:
                lines.append(_gen_factory(type_name, factory, fields))
            if is_carrier:
                from_args = ", ".join(f"n.{to_camel(f['name'])}" for f in fields)
                lines.append(f"        fun fromNative(n: Java{type_name}): {type_name} = {type_name}({from_args})")
            lines.append("    }")
        lines += ["}", ""]
        write_kotlin(KOTLIN_OUT / "types" / f"{type_name}.kt", "\n".join(lines))
