"""Orchestrator for Lua codegen -- ties all generators together."""

import sys
import os

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
import sdk_common

from .context import LUA_BINDINGS_DIR
from . import enums_gen, types_gen, tools_gen, register_gen


def run():
    """Generate all Lua binding .g.rs files."""
    print("Lua codegen: loading schema and manifests...")
    schema = sdk_common.load_schema()
    ffi_mapping = sdk_common.load_ffi_mapping(schema)
    ffi_manifest = sdk_common.load_ffi_manifest()

    LUA_BINDINGS_DIR.mkdir(parents=True, exist_ok=True)

    # 1. Enums
    enums_content = enums_gen.generate(schema)
    enums_path = LUA_BINDINGS_DIR / "enums.g.rs"
    sdk_common.write_generated(enums_path, enums_content)

    # 2. Types
    types_content = types_gen.generate(schema, ffi_mapping)
    types_path = LUA_BINDINGS_DIR / "types.g.rs"
    sdk_common.write_generated(types_path, types_content)

    # 3. Tools -- generates extern "C" stubs, which only resolve in cdylib
    # builds. Write them but don't register them until a proper linking
    # strategy is in place (e.g., re-exporting FFI functions as pub(crate)).
    tools_content = tools_gen.generate(schema, ffi_manifest)
    tools_path = LUA_BINDINGS_DIR / "tools.g.rs"
    sdk_common.write_generated(tools_path, tools_content)

    # Tools are generated but not wired up yet; registration is deferred.
    has_tools = False
    tool_names_with_methods = []

    # 4. Register
    register_content = register_gen.generate(schema, has_tools, tool_names_with_methods)
    register_path = LUA_BINDINGS_DIR / "register.g.rs"
    sdk_common.write_generated(register_path, register_content)

    print(f"Lua codegen complete. Generated {4} files in {LUA_BINDINGS_DIR}")
