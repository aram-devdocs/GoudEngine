"""Shared context for Python SDK code generation."""

from pathlib import Path

from sdk_common import (
    HEADER_COMMENT,
    CTYPES_MAP,
    PYTHON_TYPES,
    SDKS_DIR,
    load_errors,
    load_ffi_mapping,
    load_schema,
    resolve_ctypes_type,
    to_screaming_snake,
    to_snake,
    write_generated,
)

OUT = SDKS_DIR / "python" / "goudengine" / "generated"
schema = load_schema()
mapping = load_ffi_mapping(schema)

__all__ = [
    "Path",
    "HEADER_COMMENT",
    "CTYPES_MAP",
    "PYTHON_TYPES",
    "OUT",
    "schema",
    "mapping",
    "load_errors",
    "resolve_ctypes_type",
    "to_screaming_snake",
    "to_snake",
    "write_generated",
]
