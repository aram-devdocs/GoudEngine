"""Enum generation for Kotlin SDK."""
from __future__ import annotations
from .helpers import HEADER_COMMENT, KOTLIN_OUT, ENUM_SUBDIRS, KOTLIN_TYPES, schema, to_pascal, write_kotlin, kdoc

_KOTLIN_RESERVED_ENTRY_NAMES = {
    "Int", "Float", "Double", "Long", "Short", "Byte", "Boolean", "Char",
    "String", "Unit", "Any", "Nothing", "Array", "Enum",
}


def _escape_entry_name(name: str) -> str:
    if name in _KOTLIN_RESERVED_ENTRY_NAMES:
        return f"`{name}`"
    return name


def _kt_underlying(underlying: str) -> str:
    return KOTLIN_TYPES.get(underlying, "Int")

def gen_enums():
    for enum_name, enum_def in schema["enums"].items():
        underlying = enum_def.get("underlying", "i32")
        kt_ty = _kt_underlying(underlying)
        subdir = ENUM_SUBDIRS.get(enum_name, "core")
        pascal_name = to_pascal(enum_name)

        # Check if any entry clashes with a reserved name
        has_clash = any(vname in _KOTLIN_RESERVED_ENTRY_NAMES for vname in enum_def["values"])
        val_type = f"kotlin.{kt_ty}" if has_clash else kt_ty
        param_type = f"kotlin.{kt_ty}" if has_clash else kt_ty

        lines = [f"// {HEADER_COMMENT}", f"package com.goudengine.{subdir}", ""]

        doc = enum_def.get("doc")
        lines.extend(kdoc(doc))

        lines.append(f"enum class {pascal_name}(val value: {val_type}) {{")
        entries = list(enum_def["values"].items())
        for i, (vname, vval) in enumerate(entries):
            comma = "," if i < len(entries) - 1 else ";"
            escaped = _escape_entry_name(vname)
            lines.append(f"    {escaped}({vval}){comma}")
        lines += ["", "    companion object {",
                   f"        fun fromValue(value: {param_type}): {pascal_name}? =",
                   f"            entries.firstOrNull {{ it.value == value }}",
                   "    }", "}", ""]
        write_kotlin(KOTLIN_OUT / subdir / f"{pascal_name}.kt", "\n".join(lines))
