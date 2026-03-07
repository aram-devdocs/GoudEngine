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


def load_manifest_signatures(manifest_path: Path) -> dict[str, dict]:
    """Extract function signatures from ffi_manifest.json.

    Returns a dict mapping function name to its metadata (params, return_type).
    Only works for the flat function dict shape with metadata objects.
    """
    data = json.loads(manifest_path.read_text())
    if not isinstance(data, dict):
        return {}
    funcs = data.get("functions", {})
    if not isinstance(funcs, dict):
        return {}
    # Check if flat dict with metadata
    sample = next(iter(funcs.values()), None) if funcs else None
    if sample is not None and isinstance(sample, dict) and any(
        k in sample for k in ("params", "return_type", "source_file")
    ):
        return funcs
    return {}


def load_mapping_signatures(mapping: dict) -> dict[str, dict]:
    """Extract function signatures from ffi_mapping.json's ffi_functions section.

    Returns a dict mapping function name to its metadata (params, returns).
    """
    sigs: dict[str, dict] = {}
    for module_key, module_val in mapping.get("ffi_functions", {}).items():
        if module_key != "_comment" and isinstance(module_val, dict):
            for fn_name, fn_meta in module_val.items():
                if isinstance(fn_meta, dict):
                    sigs[fn_name] = fn_meta
    return sigs


def parse_manifest_param_type(param_str: str) -> str:
    """Extract the type from a manifest param string like 'name: type'."""
    parts = param_str.split(":", 1)
    if len(parts) == 2:
        return parts[1].strip()
    return param_str.strip()


# Known type aliases: manifest type -> normalized form
_TYPE_ALIASES: dict[str, str] = {
    "()": "void",
    "*const std::os::raw::c_char": "*const c_char",
    "GoudEntityId": "u64",
    "GoudTextureHandle": "u64",
    "GoudKeyCode": "i32",
    "GoudMouseButton": "i32",
    "GoudErrorCode": "i32",
}


def normalize_type(t: str) -> str:
    """Normalize a type string by resolving known aliases."""
    return _TYPE_ALIASES.get(t, t)


def compare_signatures(
    manifest_sigs: dict[str, dict],
    mapping_sigs: dict[str, dict],
) -> list[str]:
    """Compare parameter counts and types for functions in both sources.

    Returns a list of warning strings for mismatches.
    """
    warnings: list[str] = []
    common_fns = sorted(set(manifest_sigs.keys()) & set(mapping_sigs.keys()))

    for fn_name in common_fns:
        m_meta = manifest_sigs[fn_name]
        p_meta = mapping_sigs[fn_name]

        m_params = m_meta.get("params", [])
        p_params = p_meta.get("params", [])

        # Compare parameter counts
        if len(m_params) != len(p_params):
            warnings.append(
                f"  {fn_name}: param count mismatch "
                f"(manifest={len(m_params)}, mapping={len(p_params)})"
            )
            continue

        # Compare parameter types
        for i, (m_p, p_p) in enumerate(zip(m_params, p_params)):
            m_type = parse_manifest_param_type(m_p) if isinstance(m_p, str) else ""
            p_type = p_p.get("type", "") if isinstance(p_p, dict) else str(p_p)
            if normalize_type(m_type) != normalize_type(p_type):
                m_name = m_p.split(":")[0].strip() if isinstance(m_p, str) else f"param{i}"
                warnings.append(
                    f"  {fn_name}: param '{m_name}' type mismatch "
                    f"(manifest='{m_type}', mapping='{p_type}')"
                )

        # Compare return types
        m_ret = m_meta.get("return_type", "void")
        p_ret = p_meta.get("returns", "void")
        if normalize_type(m_ret) != normalize_type(p_ret):
            warnings.append(
                f"  {fn_name}: return type mismatch "
                f"(manifest='{m_ret}', mapping='{p_ret}')"
            )

    return warnings


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
            names.update(k for k in module_val.keys() if not k.startswith("_"))
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

    # Signature comparison pass (warnings only, not hard failures)
    manifest_sigs = load_manifest_signatures(manifest_path)
    mapping_sigs = load_mapping_signatures(mapping)
    sig_warnings = compare_signatures(manifest_sigs, mapping_sigs)

    if sig_warnings:
        print(f"Signature warnings ({len(sig_warnings)}):")
        for w in sig_warnings:
            print(w)
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
