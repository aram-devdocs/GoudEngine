"""Shared helper functions for Swift code generation modules."""

from __future__ import annotations

from .context import (
    SWIFT_TYPES,
    SWIFT_FFI_TYPES,
    mapping,
    resolve_swift_ffi_type,
    schema,
    to_camel,
    to_pascal,
    to_screaming_snake,
    to_snake,
)


_SWIFT_KEYWORDS = frozenset({
    "protocol", "class", "struct", "enum", "func", "var", "let", "import",
    "return", "if", "else", "for", "while", "do", "switch", "case", "break",
    "continue", "default", "where", "in", "is", "as", "self", "Self", "true",
    "false", "nil", "try", "catch", "throw", "throws", "guard", "defer",
    "repeat", "typealias", "operator", "subscript", "init", "deinit",
    "extension", "associatedtype", "static", "override", "private", "public",
    "internal", "fileprivate", "open", "mutating", "nonmutating", "inout",
    "lazy", "final", "required", "convenience", "dynamic", "optional",
    "prefix", "postfix", "indirect", "some", "any", "Type",
})


def safe_swift_name(name: str) -> str:
    """Escape a name with backticks if it is a Swift keyword."""
    if name in _SWIFT_KEYWORDS:
        return f"`{name}`"
    return name


def swift_type(schema_type: str) -> str:
    """Map a schema type to its Swift representation."""
    nullable = schema_type.endswith("?")
    base = schema_type.rstrip("?")

    if base.endswith("[]"):
        element = base[:-2]
        if element == "u8":
            result = "Data"
        elif element == "Entity":
            result = "[Entity]"
        elif element == "f32":
            result = "[Float]"
        else:
            mapped = SWIFT_TYPES.get(element, to_pascal(element))
            result = f"[{mapped}]"
        return f"{result}?" if nullable else result

    mapped = SWIFT_TYPES.get(base)
    if mapped is not None:
        return f"{mapped}?" if nullable else mapped

    pascal = to_pascal(base)
    return f"{pascal}?" if nullable else pascal


def swift_field_type(field: dict) -> str:
    """Return the Swift type for a schema field."""
    t = field.get("type", "f32")
    if t == "f32[9]":
        return "(Float, Float, Float, Float, Float, Float, Float, Float, Float)"
    if t == "u8[]":
        return "Data"
    if t == "Entity[]":
        return "[Entity]"
    return swift_type(t)


def swift_default(schema_type: str) -> str:
    """Return a Swift default value for a schema type."""
    if schema_type == "f32[9]":
        return "(0, 0, 0, 0, 0, 0, 0, 0, 0)"
    base = schema_type.rstrip("?")
    if schema_type.endswith("?"):
        return "nil"
    if base in ("f32", "f64"):
        return "0"
    if base in ("u8", "u16", "u32", "u64", "i8", "i16", "i32", "i64", "usize", "ptr"):
        return "0"
    if base == "bool":
        return "false"
    if base == "string":
        return '""'
    if base in ("bytes", "u8[]"):
        return "Data()"
    if base == "Entity[]":
        return "[]"
    if base in schema.get("types", {}):
        kind = schema["types"][base].get("kind")
        if kind == "handle":
            return f"{to_pascal(base)}(bits: 0)"
        return f"{to_pascal(base)}()"
    return "0"


def swift_literal(value, schema_type: str = "f32") -> str:
    """Produce a Swift literal from a schema default value."""
    if isinstance(value, bool):
        return "true" if value else "false"
    if isinstance(value, (int, float)):
        return str(value)
    if isinstance(value, str):
        return f'"{value}"'
    return str(value)


def swift_file_header() -> str:
    """Return the standard generated-file header."""
    from .context import HEADER_COMMENT
    return f"// {HEADER_COMMENT}"


def is_enum(type_name: str) -> bool:
    return type_name in schema.get("enums", {})


def is_value_type(type_name: str) -> bool:
    return type_name in schema.get("types", {}) and schema["types"][type_name].get("kind") == "value"


def is_handle_type(type_name: str) -> bool:
    return type_name in schema.get("types", {}) and schema["types"][type_name].get("kind") == "handle"


def is_component_type(type_name: str) -> bool:
    return type_name in schema.get("types", {}) and schema["types"][type_name].get("kind") == "component"


def method_exists_in_ffi(tool_name: str, method_name: str) -> bool:
    ffi_tools = schema.get("ffi_tools", {})
    tool_ffi = ffi_tools.get(tool_name, {})
    return method_name in tool_ffi.get("methods", {})


def ffi_func_name(tool_name: str, method_name: str) -> str | None:
    """Get the C function name for a tool method."""
    ffi_tools = schema.get("ffi_tools", {})
    tool_ffi = ffi_tools.get(tool_name, {})
    method_info = tool_ffi.get("methods", {}).get(method_name)
    if method_info:
        return method_info.get("ffi")
    return None


def convert_param_to_ffi(pname: str, ptype: str) -> str:
    """Convert a Swift parameter to the FFI call expression."""
    base = ptype.rstrip("?")
    if base == "Entity":
        return f"{pname}.bits"
    if is_handle_type(base):
        return f"{pname}.bits"
    if base == "string":
        return f"{pname}Ptr"
    if base in ("bytes", "u8[]", "Data"):
        return f"{pname}BasePtr"
    if base == "Entity[]":
        return f"{pname}BasePtr"
    if base.endswith("[]"):
        return f"{pname}BasePtr"
    if is_enum(base):
        enum_def = schema.get("enums", {}).get(base, {})
        underlying = enum_def.get("underlying", "i32")
        swift_raw = SWIFT_TYPES.get(underlying, "Int32")
        return f"{swift_raw}({pname}.rawValue)"
    if is_value_type(base):
        from .context import mapping
        ffi_info = mapping.get("ffi_types", {}).get(base, {})
        if ffi_info.get("ffi_name"):
            return f"{pname}.toFFI()"
    return pname


def convert_return_from_ffi(expr: str, ret_type: str) -> str:
    """Wrap an FFI return expression into the Swift SDK type."""
    base = ret_type.rstrip("?")
    if base == "Entity":
        return f"Entity(bits: {expr})"
    if is_handle_type(base):
        return f"{base}(bits: {expr})"
    if base == "bool":
        # C bool imports as Swift Bool directly
        return expr
    if base == "string":
        return f"String(cString: {expr})"
    if base.endswith("[]"):
        return f"{expr} /* TODO: array return needs manual FFI bridge */"
    if is_enum(base):
        return f"{base}(rawValue: Int({expr}))!"
    if is_value_type(base):
        return f"{base}(ffi: {expr})"
    return expr
