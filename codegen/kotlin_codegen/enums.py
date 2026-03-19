"""Enum generation for Kotlin SDK."""
from __future__ import annotations
from .helpers import HEADER_COMMENT, KOTLIN_OUT, ENUM_SUBDIRS, KOTLIN_TYPES, schema, to_pascal, kdoc, write_kotlin

# Kotlin reserved words and built-in type names that cannot be used as enum entry names.
_KOTLIN_RESERVED_ENTRY_NAMES = {
    "Int", "Float", "Double", "Long", "Short", "Byte", "Boolean", "Char",
    "String", "Unit", "Any", "Nothing", "Array", "Enum",
}

def _kt_underlying(underlying: str) -> str:
    return KOTLIN_TYPES.get(underlying, "Int")

def _escape_entry_name(name: str) -> str:
    """Escape enum entry names that clash with Kotlin built-in types."""
    if name in _KOTLIN_RESERVED_ENTRY_NAMES:
        return f"`{name}`"
    return name

def gen_enums():
    for enum_name, enum_def in schema["enums"].items():
        underlying = enum_def.get("underlying", "i32")
        kt_ty = _kt_underlying(underlying)
        subdir = ENUM_SUBDIRS.get(enum_name, "core")
        pascal_name = to_pascal(enum_name)
        # Check if any entry name clashes with the value type (e.g. Int, Long)
        entry_names = set(enum_def["values"].keys())
        has_type_clash = bool(entry_names & _KOTLIN_RESERVED_ENTRY_NAMES)
        # Use FQN for the type if there's a clash to avoid ambiguity
        fqn_ty = f"kotlin.{kt_ty}" if has_type_clash else kt_ty
        enum_doc = enum_def.get("doc")
        lines = [f"// {HEADER_COMMENT}", f"package com.goudengine.{subdir}", ""]
        lines.extend(kdoc(enum_doc))
        lines.append(f"enum class {pascal_name}(val value: {fqn_ty}) {{")
        entries = list(enum_def["values"].items())
        for i, (vname, vval) in enumerate(entries):
            comma = "," if i < len(entries) - 1 else ";"
            escaped = _escape_entry_name(vname)
            lines.append(f"    {escaped}({vval}){comma}")
        lines += ["", "    companion object {",
                   f"        fun fromValue(value: {fqn_ty}): {pascal_name}? =",
                   f"            entries.firstOrNull {{ it.value == value }}",
                   "    }", "}", ""]
        write_kotlin(KOTLIN_OUT / subdir / f"{pascal_name}.kt", "\n".join(lines))
