"""Shared context for Swift SDK code generation."""

from pathlib import Path

from sdk_common import (
    HEADER_COMMENT,
    SWIFT_TYPES,
    SWIFT_FFI_TYPES,
    SDKS_DIR,
    load_errors,
    load_ffi_mapping,
    load_schema,
    resolve_swift_ffi_type,
    to_camel,
    to_pascal,
    to_screaming_snake,
    to_snake,
    write_generated,
)

OUT = SDKS_DIR / "swift" / "Sources" / "GoudEngine" / "generated"
schema = load_schema()
mapping = load_ffi_mapping(schema)

__all__ = [
    "Path",
    "HEADER_COMMENT",
    "SWIFT_TYPES",
    "SWIFT_FFI_TYPES",
    "OUT",
    "schema",
    "mapping",
    "load_errors",
    "resolve_swift_ffi_type",
    "to_camel",
    "to_pascal",
    "to_screaming_snake",
    "to_snake",
    "write_generated",
]
