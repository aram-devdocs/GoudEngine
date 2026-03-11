#!/usr/bin/env python3
"""Validates that the resolved FFI contract matches goud_sdk.schema.json.

Checks:
  1. Every resolved method mapping references a method in goud_sdk.schema
  2. Every method in goud_sdk.schema has a resolved FFI mapping
  3. No orphaned FFI function references
"""

import json
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))
from sdk_common import load_ffi_mapping, load_schema

CODEGEN_DIR = Path(__file__).parent


def main():
    schema = load_schema()
    with open(CODEGEN_DIR / "ffi_mapping.json") as f:
        raw_mapping = json.load(f)
    mapping = load_ffi_mapping(schema)

    errors = []

    legacy_tools = raw_mapping.get("tools")
    if legacy_tools is not None and legacy_tools != schema.get("ffi_tools", {}):
        errors.append(
            "  ffi_mapping.json legacy 'tools' section diverges from "
            "schema-owned 'ffi_tools'"
        )

    legacy_type_methods = raw_mapping.get("type_methods")
    if legacy_type_methods is not None and legacy_type_methods != schema.get(
        "ffi_type_methods", {}
    ):
        errors.append(
            "  ffi_mapping.json legacy 'type_methods' section diverges from "
            "schema-owned 'ffi_type_methods'"
        )

    manual_signature_fields: list[str] = []
    for module_name, module_def in raw_mapping.get("ffi_functions", {}).items():
        if module_name == "_comment" or not isinstance(module_def, dict):
            continue
        for fn_name, fn_meta in module_def.items():
            if fn_name.startswith("_") or not isinstance(fn_meta, dict):
                continue
            forbidden = sorted({"params", "returns", "unsafe"} & set(fn_meta))
            if forbidden:
                manual_signature_fields.append(
                    f"  {fn_name}: raw ffi_mapping.json still contains {', '.join(forbidden)}"
                )
            unknown = sorted(set(fn_meta) - {"alias_of", "params", "returns", "unsafe"})
            if unknown:
                manual_signature_fields.append(
                    f"  {fn_name}: raw ffi_mapping.json has unsupported metadata {', '.join(unknown)}"
                )

    errors.extend(manual_signature_fields)

    for tool_name, tool_def in schema.get("tools", {}).items():
        tool_map = mapping.get("tools", {}).get(tool_name)
        if not tool_map:
            errors.append(f"Tool '{tool_name}' in schema has no resolved FFI mapping")
            continue

        schema_methods = {m["name"] for m in tool_def.get("methods", [])}
        mapped_methods = set(tool_map.get("methods", {}).keys())
        lifecycle_methods = set(tool_map.get("lifecycle", {}).keys())
        mapped_methods |= lifecycle_methods
        if tool_map.get("lifecycle"):
            mapped_methods.add("run")

        for m in schema_methods - mapped_methods:
            errors.append(f"  {tool_name}.{m}: in schema but missing from ffi_mapping")
        for m in mapped_methods - schema_methods:
            errors.append(f"  {tool_name}.{m}: in ffi_mapping but missing from schema")

    audio_activate_mapping = (
        mapping.get("tools", {})
        .get("GoudGame", {})
        .get("methods", {})
        .get("audioActivate", {})
        .get("ffi")
    )
    if audio_activate_mapping != "goud_audio_activate":
        errors.append(
            "  GoudGame.audioActivate: expected ffi mapping to goud_audio_activate"
        )

    if errors:
        print(f"Validation FAILED ({len(errors)} issues):")
        for e in errors:
            print(f"  {e}")
        sys.exit(1)
    print("Validation passed: schema and resolved ffi_mapping are in sync.")


if __name__ == "__main__":
    main()
