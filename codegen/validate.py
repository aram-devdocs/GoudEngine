#!/usr/bin/env python3
"""Validates that ffi_mapping.json references match goud_sdk.schema.json.

Checks:
  1. Every method in ffi_mapping references a method in goud_sdk.schema
  2. Every method in goud_sdk.schema has a mapping in ffi_mapping
  3. No orphaned FFI function references
"""

import json
import sys
from pathlib import Path

CODEGEN_DIR = Path(__file__).parent


def main():
    with open(CODEGEN_DIR / "goud_sdk.schema.json") as f:
        schema = json.load(f)
    with open(CODEGEN_DIR / "ffi_mapping.json") as f:
        mapping = json.load(f)

    errors = []

    for tool_name, tool_def in schema.get("tools", {}).items():
        tool_map = mapping.get("tools", {}).get(tool_name)
        if not tool_map:
            errors.append(f"Tool '{tool_name}' in schema has no FFI mapping")
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

    if errors:
        print(f"Validation FAILED ({len(errors)} issues):")
        for e in errors:
            print(f"  {e}")
        sys.exit(1)
    else:
        print("Validation passed: schema and ffi_mapping are in sync.")


if __name__ == "__main__":
    main()
