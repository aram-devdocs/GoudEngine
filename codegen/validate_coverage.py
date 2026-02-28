#!/usr/bin/env python3
"""Validates that ffi_manifest.json covers 100% of FFI functions from ffi_mapping.json."""

import json
import sys
from pathlib import Path

CODEGEN_DIR = Path(__file__).parent


def load_manifest_functions(manifest_path: Path) -> set[str]:
    """Extract function names from ffi_manifest.json.

    Accepts three shapes produced by build.rs:
    - A list of strings: ["goud_window_create", ...]
    - A flat function dict: {"functions": {"goud_window_create": {metadata}, ...}}
    - A module-grouped dict: {"ffi_functions": {"module": {"fn_name": {}, ...}, ...}}
    """
    data = json.loads(manifest_path.read_text())
    if isinstance(data, list):
        return {item for item in data if isinstance(item, str)}
    if isinstance(data, dict):
        funcs = data.get("functions", data.get("ffi_functions", []))
        if isinstance(funcs, list):
            return {item for item in funcs if isinstance(item, str)}
        if isinstance(funcs, dict):
            # Flat dict: keys are function names (values are metadata objects)
            # Detect by checking if any value is a non-dict or has fn-metadata keys
            sample = next(iter(funcs.values()), None) if funcs else None
            is_flat = sample is None or isinstance(sample, dict) and any(
                k in sample for k in ("params", "return_type", "source_file")
            )
            if is_flat:
                return set(funcs.keys())
            # Module-grouped dict: keys are module names, values are fn dicts
            names: set[str] = set()
            for module_val in funcs.values():
                if isinstance(module_val, dict):
                    names.update(module_val.keys())
            return names
    return set()


def collect_ffi_values(obj: object) -> set[str]:
    """Recursively collect values of "ffi" string keys in a nested structure."""
    found: set[str] = set()
    if isinstance(obj, dict):
        for key, val in obj.items():
            if key == "ffi" and isinstance(val, str):
                found.add(val)
            else:
                found |= collect_ffi_values(val)
    elif isinstance(obj, list):
        for item in obj:
            found |= collect_ffi_values(item)
    return found


def load_mapping_functions(mapping: dict) -> set[str]:
    """Extract all FFI function names declared in ffi_mapping.json.

    Primary: second-level keys of "ffi_functions" (canonical function list).
    Secondary: "ffi" string values in the "tools" section (cross-check).
    """
    names: set[str] = set()
    for module_key, module_val in mapping.get("ffi_functions", {}).items():
        if module_key != "_comment" and isinstance(module_val, dict):
            names.update(module_val.keys())
    names |= collect_ffi_values(mapping.get("tools", {}))
    return names


def main() -> None:
    manifest_path = CODEGEN_DIR / "ffi_manifest.json"
    mapping_path = CODEGEN_DIR / "ffi_mapping.json"

    if not manifest_path.exists():
        print(
            "WARNING: ffi_manifest.json not found. "
            "Run `cargo build` first to generate it."
        )
        sys.exit(0)

    mapping = json.loads(mapping_path.read_text())
    manifest_functions = load_manifest_functions(manifest_path)
    mapping_functions = load_mapping_functions(mapping)

    in_manifest_not_mapping = sorted(manifest_functions - mapping_functions)
    in_mapping_not_manifest = sorted(mapping_functions - manifest_functions)

    manifest_count = len(manifest_functions)
    mapped_count = len(manifest_functions - set(in_manifest_not_mapping))
    coverage = (mapped_count / manifest_count * 100.0) if manifest_count else 100.0

    print("FFI Coverage Report")
    print("===================")
    print(f"Manifest functions: {manifest_count}")
    print(f"Mapped functions:   {mapped_count}")
    print(f"Coverage:           {coverage:.1f}%")
    print()

    if not in_manifest_not_mapping and not in_mapping_not_manifest:
        print("Full coverage — all FFI functions are mapped.")
        sys.exit(0)

    if in_manifest_not_mapping:
        print(f"MISSING from mapping ({len(in_manifest_not_mapping)} functions):")
        for fn in in_manifest_not_mapping:
            print(f"  - {fn}")
        print()

    if in_mapping_not_manifest:
        print(f"STALE in mapping ({len(in_mapping_not_manifest)} functions):")
        for fn in in_mapping_not_manifest:
            print(f"  - {fn}")
        print()

    sys.exit(1)


if __name__ == "__main__":
    main()
