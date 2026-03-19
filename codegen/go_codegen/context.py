"""Shared context for Go SDK code generation."""

from pathlib import Path

from sdk_common import (
    HEADER_COMMENT,
    SDKS_DIR,
    load_ffi_mapping,
    load_schema,
    to_pascal,
    to_snake,
    to_screaming_snake,
    write_generated,
)

OUT = SDKS_DIR / "go" / "goud"
HELPERS_OUT = SDKS_DIR / "go" / "internal" / "ffi"
schema = load_schema()
mapping = load_ffi_mapping(schema)

# Go type mapping from schema primitives.
GO_TYPES = {
    "f32": "float32",
    "f64": "float64",
    "u8": "uint8",
    "u16": "uint16",
    "u32": "uint32",
    "u64": "uint64",
    "i8": "int8",
    "i16": "int16",
    "i32": "int32",
    "i64": "int64",
    "usize": "uint",
    "bool": "bool",
    "string": "string",
    "bytes": "[]byte",
    "void": "",
    "ptr": "uintptr",
}

# Go zero values for schema types.
GO_ZERO = {
    "f32": "0",
    "f64": "0",
    "u8": "0",
    "u16": "0",
    "u32": "0",
    "u64": "0",
    "i8": "0",
    "i16": "0",
    "i32": "0",
    "i64": "0",
    "usize": "0",
    "bool": "false",
    "string": '""',
    "bytes": "nil",
}

GO_HEADER = f"// {HEADER_COMMENT}"


def to_go_name(name: str) -> str:
    """Convert camelCase/snake_case schema name to Go PascalCase export."""
    return to_pascal(name)


def to_go_field(name: str) -> str:
    """Convert schema field name to Go PascalCase field name."""
    return to_pascal(name)


def to_go_local(name: str) -> str:
    """Convert schema param name to Go camelCase local variable."""
    if not name:
        return name
    # Handle special cases: already camelCase in schema
    s = name[0].lower() + name[1:]
    # Avoid Go keyword collisions
    if s in ("type", "func", "var", "range", "map", "chan", "go", "select",
             "case", "default", "break", "continue", "return", "interface"):
        return s + "_"
    # Avoid shadowing the receiver 'g' used by Game methods
    if s == "g":
        return "gVal"
    return s


__all__ = [
    "Path",
    "HEADER_COMMENT",
    "GO_HEADER",
    "GO_TYPES",
    "GO_ZERO",
    "OUT",
    "HELPERS_OUT",
    "schema",
    "mapping",
    "to_go_name",
    "to_go_field",
    "to_go_local",
    "to_pascal",
    "to_snake",
    "to_screaming_snake",
    "write_generated",
]
