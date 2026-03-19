"""Generator for ValueTypes.g.swift."""

from .context import HEADER_COMMENT, OUT, schema, mapping, write_generated
from .shared_helpers import (
    swift_file_header,
    swift_type,
    swift_field_type,
    swift_default,
    swift_literal,
    to_camel,
    SWIFT_TYPES,
)


# ── Pure-Swift method implementations ──────────────────────────────

_VEC2_METHODS: dict[str, str] = {
    "add":       "public func add(_ other: Vec2) -> Vec2 { Vec2(x: x + other.x, y: y + other.y) }",
    "sub":       "public func sub(_ other: Vec2) -> Vec2 { Vec2(x: x - other.x, y: y - other.y) }",
    "scale":     "public func scale(_ s: Float) -> Vec2 { Vec2(x: x * s, y: y * s) }",
    "length":    "public func length() -> Float { (x * x + y * y).squareRoot() }",
    "normalize": "public func normalize() -> Vec2 { let l = length(); return l == 0 ? Vec2.zero() : Vec2(x: x / l, y: y / l) }",
    "dot":       "public func dot(_ other: Vec2) -> Float { x * other.x + y * other.y }",
    "distance":  "public func distance(_ other: Vec2) -> Float { sub(other).length() }",
    "lerp":      "public func lerp(_ other: Vec2, t: Float) -> Vec2 { Vec2(x: x + (other.x - x) * t, y: y + (other.y - y) * t) }",
}

_OTHER_METHODS: dict[tuple[str, str], str] = {
    ("Color", "withAlpha"): "public func withAlpha(_ a: Float) -> Color { Color(r: r, g: g, b: b, a: a) }",
    ("Color", "lerp"):      "public func lerp(_ other: Color, t: Float) -> Color { Color(r: r + (other.r - r) * t, g: g + (other.g - g) * t, b: b + (other.b - b) * t, a: a + (other.a - a) * t) }",
    ("Rect", "contains"):   "public func contains(_ point: Vec2) -> Bool { point.x >= x && point.x <= x + width && point.y >= y && point.y <= y + height }",
    ("Rect", "intersects"):  "public func intersects(_ other: Rect) -> Bool { x < other.x + other.width && x + width > other.x && y < other.y + other.height && y + height > other.y }",
}

_FACTORY_OVERRIDES: dict[tuple[str, str], str] = {
    ("Color", "rgb"):     "public static func rgb(r: Float, g: Float, b: Float) -> Color { Color(r: r, g: g, b: b, a: 1) }",
    ("Color", "fromHex"): "public static func fromHex(_ hex: Int) -> Color { Color(r: Float((hex >> 16) & 0xFF) / 255, g: Float((hex >> 8) & 0xFF) / 255, b: Float(hex & 0xFF) / 255, a: 1) }",
    ("Color", "fromU8"):  "public static func fromU8(r: UInt8, g: UInt8, b: UInt8, a: UInt8) -> Color { Color(r: Float(r) / 255, g: Float(g) / 255, b: Float(b) / 255, a: Float(a) / 255) }",
}


def gen_value_types() -> None:
    lines = [swift_file_header(), "import Foundation", "import CGoudEngine", ""]

    for type_name, type_def in schema.get("types", {}).items():
        kind = type_def.get("kind")
        if kind == "handle":
            _gen_handle_type(lines, type_name, type_def)
            continue
        if kind != "value":
            continue

        # Special case: Mat3x3
        if type_name == "Mat3x3":
            _gen_mat3x3(lines)
            continue

        fields = type_def.get("fields", [])
        doc = type_def.get("doc", "")
        factories = type_def.get("factories", [])
        methods = type_def.get("methods", [])

        if doc:
            lines.append(f"/// {doc}")
        lines.append(f"public struct {type_name}: Equatable {{")

        # Fields
        for f in fields:
            fname = to_camel(f["name"])
            ftype = swift_field_type(f)
            fdefault = swift_default(f.get("type", "f32"))
            lines.append(f"    public var {fname}: {ftype}")

        lines.append("")

        # Memberwise init
        init_params = []
        for f in fields:
            fname = to_camel(f["name"])
            ftype = swift_field_type(f)
            fdefault = swift_default(f.get("type", "f32"))
            init_params.append(f"{fname}: {ftype} = {fdefault}")
        lines.append(f"    public init({', '.join(init_params)}) {{")
        for f in fields:
            fname = to_camel(f["name"])
            lines.append(f"        self.{fname} = {fname}")
        lines.append("    }")
        lines.append("")

        # FFI conversion: init from FFI struct
        ffi_info = mapping.get("ffi_types", {}).get(type_name, {})
        ffi_name = ffi_info.get("ffi_name")
        if ffi_name:
            lines.append(f"    internal init(ffi: {ffi_name}) {{")
            for f in fields:
                fname = to_camel(f["name"])
                ftype = f.get("type", "f32")
                if ftype == "string":
                    lines.append(f"        self.{fname} = String(cString: ffi.{f['name']})")
                elif ftype in schema.get("types", {}) and schema["types"][ftype].get("kind") == "value":
                    nested_ffi = mapping.get("ffi_types", {}).get(ftype, {}).get("ffi_name")
                    if nested_ffi:
                        lines.append(f"        self.{fname} = {ftype}(ffi: ffi.{f['name']})")
                    else:
                        lines.append(f"        self.{fname} = {ftype}()")
                else:
                    lines.append(f"        self.{fname} = ffi.{f['name']}")
            lines.append("    }")
            lines.append("")

            # toFFI()
            lines.append(f"    internal func toFFI() -> {ffi_name} {{")
            lines.append(f"        var ffi = {ffi_name}()")
            for f in fields:
                fname = to_camel(f["name"])
                ftype = f.get("type", "f32")
                if ftype == "string":
                    pass  # String fields in FFI are tricky; skip for now
                elif ftype in schema.get("types", {}) and schema["types"][ftype].get("kind") == "value":
                    nested_ffi = mapping.get("ffi_types", {}).get(ftype, {}).get("ffi_name")
                    if nested_ffi:
                        lines.append(f"        ffi.{f['name']} = {fname}.toFFI()")
                else:
                    lines.append(f"        ffi.{f['name']} = {fname}")
            lines.append("        return ffi")
            lines.append("    }")
            lines.append("")

        # Methods (pure Swift implementations)
        for m in methods:
            mname = m["name"]
            impl = None
            if type_name == "Vec2":
                impl = _VEC2_METHODS.get(mname)
            else:
                impl = _OTHER_METHODS.get((type_name, mname))
            if impl:
                lines.append(f"    {impl}")

        if methods:
            lines.append("")

        # Static factories
        for factory in factories:
            fname = factory["name"]
            fargs = factory.get("args", [])
            val = factory.get("value")

            override = _FACTORY_OVERRIDES.get((type_name, fname))
            if override:
                lines.append(f"    {override}")
                continue

            if val and not fargs:
                val_parts = []
                for i, f in enumerate(fields):
                    field_name = to_camel(f["name"])
                    v = val[i] if i < len(val) else 0
                    val_parts.append(f"{field_name}: {swift_literal(v, f.get('type', 'f32'))}")
                lines.append(f"    public static func {fname}() -> {type_name} {{ {type_name}({', '.join(val_parts)}) }}")
            elif fargs:
                arg_str = ", ".join(f"{to_camel(a['name'])}: {swift_type(a['type'])}" for a in fargs)
                pass_str = ", ".join(f"{to_camel(a['name'])}: {to_camel(a['name'])}" for a in fargs)
                lines.append(f"    public static func {fname}({arg_str}) -> {type_name} {{ {type_name}({pass_str}) }}")

        lines.append("}")
        lines.append("")

    write_generated(OUT / "ValueTypes.g.swift", "\n".join(lines))


def _gen_handle_type(lines: list[str], type_name: str, type_def: dict) -> None:
    """Generate a handle type (Entity, PhysicsWorld2D handle, etc.)."""
    doc = type_def.get("doc", "")
    if doc:
        lines.append(f"/// {doc}")
    lines.append(f"public struct {type_name}: Equatable, Hashable {{")
    lines.append("    public let bits: UInt64")
    lines.append("")
    lines.append(f"    public init(bits: UInt64) {{")
    lines.append("        self.bits = bits")
    lines.append("    }")

    if type_name == "Entity":
        lines.append("")
        lines.append("    public var index: UInt32 { UInt32(bits & 0xFFFFFFFF) }")
        lines.append("    public var generation: UInt32 { UInt32(bits >> 32) }")
        lines.append("    public var isPlaceholder: Bool { bits == UInt64.max }")
        lines.append("")
        lines.append("    public static let placeholder = Entity(bits: UInt64.max)")

    lines.append("}")
    lines.append("")


def _gen_mat3x3(lines: list[str]) -> None:
    """Generate Mat3x3 with tuple field."""
    lines += [
        "/// 3x3 matrix in column-major order for 2D transforms.",
        "public struct Mat3x3: Equatable {",
        "    public var m: (Float, Float, Float, Float, Float, Float, Float, Float, Float)",
        "",
        "    public init(m: (Float, Float, Float, Float, Float, Float, Float, Float, Float) = (0, 0, 0, 0, 0, 0, 0, 0, 0)) {",
        "        self.m = m",
        "    }",
        "",
        "    public subscript(index: Int) -> Float {",
        "        get {",
        "            withUnsafeBytes(of: m) { buf in",
        "                buf.load(fromByteOffset: index * MemoryLayout<Float>.stride, as: Float.self)",
        "            }",
        "        }",
        "        set {",
        "            withUnsafeMutableBytes(of: &m) { buf in",
        "                buf.storeBytes(of: newValue, toByteOffset: index * MemoryLayout<Float>.stride, as: Float.self)",
        "            }",
        "        }",
        "    }",
        "",
        "    internal init(ffi: FfiMat3x3) {",
        "        self.m = ffi.m",
        "    }",
        "",
        "    internal func toFFI() -> FfiMat3x3 {",
        "        var ffi = FfiMat3x3()",
        "        ffi.m = m",
        "        return ffi",
        "    }",
        "}",
        "",
    ]
