"""Enum generation for Kotlin SDK."""
from __future__ import annotations
from .helpers import HEADER_COMMENT, KOTLIN_OUT, ENUM_SUBDIRS, KOTLIN_TYPES, schema, to_pascal, write_kotlin

def _kt_underlying(underlying: str) -> str:
    return KOTLIN_TYPES.get(underlying, "Int")

def gen_enums():
    for enum_name, enum_def in schema["enums"].items():
        underlying = enum_def.get("underlying", "i32")
        kt_ty = _kt_underlying(underlying)
        subdir = ENUM_SUBDIRS.get(enum_name, "core")
        pascal_name = to_pascal(enum_name)
        lines = [f"// {HEADER_COMMENT}", f"package com.goudengine.{subdir}", ""]
        lines.append(f"enum class {pascal_name}(val value: {kt_ty}) {{")
        entries = list(enum_def["values"].items())
        for i, (vname, vval) in enumerate(entries):
            comma = "," if i < len(entries) - 1 else ";"
            lines.append(f"    {vname}({vval}){comma}")
        lines += ["", "    companion object {",
                   f"        fun fromValue(value: {kt_ty}): {pascal_name}? =",
                   f"            entries.firstOrNull {{ it.value == value }}",
                   "    }", "}", ""]
        write_kotlin(KOTLIN_OUT / subdir / f"{pascal_name}.kt", "\n".join(lines))
