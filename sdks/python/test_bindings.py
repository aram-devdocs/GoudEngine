#!/usr/bin/env python3
"""
GoudEngine Python SDK Test Suite

Tests the generated Python SDK data types and enums without requiring
the native library to be built. Run with:
    python3 sdks/python/test_bindings.py
"""

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))

from test_bindings_common import _ERRORS_PATH, _PACKAGE_DIR, _load_module
from test_bindings_generated import (
    test_debugger_helpers,
    test_generated_audio_activate_maps_to_activate_ffi,
    test_generated_audio_wrapper_api_names,
    test_generated_debugger_wrapper_api_names,
    test_generated_game_runtime_with_fake_lib,
    test_generated_network_wrapper_api_names,
    test_generated_new_api_names,
    test_generated_provider_capability_imports,
    test_generated_scene_wrapper_api_names,
    test_generated_ui_manager_wrapper_api_names,
    test_generated_ui_style_color_contract,
    test_generated_ui_style_string_contract,
    test_imports,
)
from test_bindings_networking import (
    test_generated_context_entity_component_runtime_safe,
    test_generated_network_wrapper_exports,
    test_generated_network_wrapper_send_contract_source,
)
from test_bindings_types import (
    test_color,
    test_color_vec2_extended_helpers,
    test_entity,
    test_enums,
    test_generated_value_types_runtime_safe,
    test_generated_types_ffi_runtime_with_fake_lib,
    test_rect,
    test_sprite,
    test_transform2d,
    test_vec2,
)


def test_errors():
    """Test GoudError hierarchy: imports, attributes, subclassing, and category mapping."""
    print("Testing errors module...")

    errors_mod = _load_module("errors", _ERRORS_PATH)

    GoudError = errors_mod.GoudError
    GoudContextError = errors_mod.GoudContextError
    GoudResourceError = errors_mod.GoudResourceError
    GoudGraphicsError = errors_mod.GoudGraphicsError
    GoudEntityError = errors_mod.GoudEntityError
    GoudInputError = errors_mod.GoudInputError
    GoudSystemError = errors_mod.GoudSystemError
    GoudProviderError = errors_mod.GoudProviderError
    GoudInternalError = errors_mod.GoudInternalError
    RecoveryClass = errors_mod.RecoveryClass
    _category_from_code = errors_mod._category_from_code
    _CATEGORY_CLASS_MAP = errors_mod._CATEGORY_CLASS_MAP

    assert GoudError is not None, "GoudError failed to import"
    assert GoudContextError is not None, "GoudContextError failed to import"
    assert GoudResourceError is not None, "GoudResourceError failed to import"
    assert GoudGraphicsError is not None, "GoudGraphicsError failed to import"
    assert GoudEntityError is not None, "GoudEntityError failed to import"
    assert GoudInputError is not None, "GoudInputError failed to import"
    assert GoudSystemError is not None, "GoudSystemError failed to import"
    assert GoudProviderError is not None, "GoudProviderError failed to import"
    assert GoudInternalError is not None, "GoudInternalError failed to import"
    assert RecoveryClass is not None, "RecoveryClass failed to import"

    err = GoudError(
        error_code=1,
        message="context not initialised",
        category="Context",
        subsystem="engine",
        operation="init",
        recovery=RecoveryClass.FATAL,
        recovery_hint="Call the initialization function first",
    )
    assert err.error_code == 1, f"Expected error_code=1, got {err.error_code}"
    assert err.category == "Context", f"Expected category='Context', got {err.category!r}"
    assert err.subsystem == "engine", f"Expected subsystem='engine', got {err.subsystem!r}"
    assert err.operation == "init", f"Expected operation='init', got {err.operation!r}"
    assert err.recovery == RecoveryClass.FATAL, f"Expected recovery=FATAL, got {err.recovery}"
    assert err.recovery_hint == "Call the initialization function first", \
        f"recovery_hint mismatch: {err.recovery_hint!r}"
    assert str(err) == "context not initialised", f"str(err) mismatch: {str(err)!r}"

    assert RecoveryClass.RECOVERABLE == 0, "RECOVERABLE should be 0"
    assert RecoveryClass.FATAL == 1, "FATAL should be 1"
    assert RecoveryClass.DEGRADED == 2, "DEGRADED should be 2"

    ctx_err = GoudContextError(
        error_code=1, message="ctx", category="Context",
        subsystem="", operation="", recovery=0, recovery_hint="",
    )
    assert isinstance(ctx_err, GoudError), "GoudContextError should be instance of GoudError"
    assert isinstance(ctx_err, GoudContextError), "GoudContextError instance check failed"

    res_err = GoudResourceError(
        error_code=100, message="res", category="Resource",
        subsystem="", operation="", recovery=0, recovery_hint="",
    )
    assert isinstance(res_err, GoudError), "GoudResourceError should be instance of GoudError"

    gfx_err = GoudGraphicsError(
        error_code=200, message="gfx", category="Graphics",
        subsystem="", operation="", recovery=0, recovery_hint="",
    )
    assert isinstance(gfx_err, GoudError), "GoudGraphicsError should be instance of GoudError"

    ent_err = GoudEntityError(
        error_code=300, message="ent", category="Entity",
        subsystem="", operation="", recovery=0, recovery_hint="",
    )
    assert isinstance(ent_err, GoudError), "GoudEntityError should be instance of GoudError"

    inp_err = GoudInputError(
        error_code=400, message="inp", category="Input",
        subsystem="", operation="", recovery=0, recovery_hint="",
    )
    assert isinstance(inp_err, GoudError), "GoudInputError should be instance of GoudError"

    sys_err = GoudSystemError(
        error_code=500, message="sys", category="System",
        subsystem="", operation="", recovery=0, recovery_hint="",
    )
    assert isinstance(sys_err, GoudError), "GoudSystemError should be instance of GoudError"

    prv_err = GoudProviderError(
        error_code=600, message="prv", category="Provider",
        subsystem="", operation="", recovery=0, recovery_hint="",
    )
    assert isinstance(prv_err, GoudError), "GoudProviderError should be instance of GoudError"

    int_err = GoudInternalError(
        error_code=900, message="int", category="Internal",
        subsystem="", operation="", recovery=0, recovery_hint="",
    )
    assert isinstance(int_err, GoudError), "GoudInternalError should be instance of GoudError"

    assert ctx_err.category == "Context", f"GoudContextError category should be 'Context'"
    assert res_err.category == "Resource", f"GoudResourceError category should be 'Resource'"
    assert gfx_err.category == "Graphics", f"GoudGraphicsError category should be 'Graphics'"
    assert ent_err.category == "Entity", f"GoudEntityError category should be 'Entity'"
    assert inp_err.category == "Input", f"GoudInputError category should be 'Input'"
    assert sys_err.category == "System", f"GoudSystemError category should be 'System'"
    assert prv_err.category == "Provider", f"GoudProviderError category should be 'Provider'"
    assert int_err.category == "Internal", f"GoudInternalError category should be 'Internal'"

    init_src = (_PACKAGE_DIR / "__init__.py").read_text()
    assert (
        "from .errors import" in init_src
        or "from .generated._errors import" in init_src
    ), "__init__.py should re-export error classes from the package errors module"

    assert _category_from_code(1) == "Context", \
        f"_category_from_code(1) should return 'Context', got {_category_from_code(1)!r}"
    assert _category_from_code(100) == "Resource", \
        f"_category_from_code(100) should return 'Resource', got {_category_from_code(100)!r}"
    assert _category_from_code(200) == "Graphics", \
        f"_category_from_code(200) should return 'Graphics', got {_category_from_code(200)!r}"
    assert _category_from_code(300) == "Entity", \
        f"_category_from_code(300) should return 'Entity', got {_category_from_code(300)!r}"
    assert _category_from_code(400) == "Input", \
        f"_category_from_code(400) should return 'Input', got {_category_from_code(400)!r}"
    assert _category_from_code(500) == "System", \
        f"_category_from_code(500) should return 'System', got {_category_from_code(500)!r}"
    assert _category_from_code(600) == "Provider", \
        f"_category_from_code(600) should return 'Provider', got {_category_from_code(600)!r}"
    assert _category_from_code(900) == "Internal", \
        f"_category_from_code(900) should return 'Internal', got {_category_from_code(900)!r}"

    assert _CATEGORY_CLASS_MAP["Context"] is GoudContextError, \
        f"_CATEGORY_CLASS_MAP['Context'] should map to GoudContextError"
    assert _CATEGORY_CLASS_MAP["Resource"] is GoudResourceError, \
        f"_CATEGORY_CLASS_MAP['Resource'] should map to GoudResourceError"
    assert _CATEGORY_CLASS_MAP["Graphics"] is GoudGraphicsError, \
        f"_CATEGORY_CLASS_MAP['Graphics'] should map to GoudGraphicsError"
    assert _CATEGORY_CLASS_MAP["Entity"] is GoudEntityError, \
        f"_CATEGORY_CLASS_MAP['Entity'] should map to GoudEntityError"
    assert _CATEGORY_CLASS_MAP["Input"] is GoudInputError, \
        f"_CATEGORY_CLASS_MAP['Input'] should map to GoudInputError"
    assert _CATEGORY_CLASS_MAP["System"] is GoudSystemError, \
        f"_CATEGORY_CLASS_MAP['System'] should map to GoudSystemError"
    assert _CATEGORY_CLASS_MAP["Provider"] is GoudProviderError, \
        f"_CATEGORY_CLASS_MAP['Provider'] should map to GoudProviderError"
    assert _CATEGORY_CLASS_MAP["Internal"] is GoudInternalError, \
        f"_CATEGORY_CLASS_MAP['Internal'] should map to GoudInternalError"

    print("  Error tests passed")
    return True


def main():
    """Run all tests."""
    print("=" * 60)
    print(" GoudEngine Python SDK Tests")
    print("=" * 60)

    tests = [
        test_imports,
        test_generated_scene_wrapper_api_names,
        test_generated_audio_wrapper_api_names,
        test_generated_audio_activate_maps_to_activate_ffi,
        test_generated_network_wrapper_api_names,
        test_generated_provider_capability_imports,
        test_generated_network_wrapper_exports,
        test_generated_network_wrapper_send_contract_source,
        test_generated_context_entity_component_runtime_safe,
        test_generated_ui_manager_wrapper_api_names,
        test_generated_debugger_wrapper_api_names,
        test_generated_new_api_names,
        test_debugger_helpers,
        test_generated_ui_style_color_contract,
        test_generated_ui_style_string_contract,
        test_generated_game_runtime_with_fake_lib,
        test_vec2,
        test_color_vec2_extended_helpers,
        test_color,
        test_rect,
        test_transform2d,
        test_sprite,
        test_entity,
        test_generated_value_types_runtime_safe,
        test_generated_types_ffi_runtime_with_fake_lib,
        test_enums,
        test_errors,
    ]

    passed = 0
    failed = 0

    for test in tests:
        try:
            if test():
                passed += 1
            else:
                failed += 1
        except Exception as exc:
            print(f"  {test.__name__} failed with exception: {exc}")
            import traceback
            traceback.print_exc()
            failed += 1

    print("\n" + "=" * 60)
    print(f" Results: {passed} passed, {failed} failed")
    print("=" * 60)

    return 0 if failed == 0 else 1


if __name__ == "__main__":
    sys.exit(main())
