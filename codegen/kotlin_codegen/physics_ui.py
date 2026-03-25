"""Physics, EngineConfig, UiManager, and sub-tool generation for Kotlin SDK."""

from __future__ import annotations

import re

from .helpers import (
    HEADER_COMMENT,
    KOTLIN_OUT,
    JAVA_DST,
    schema,
    to_pascal,
    to_camel,
    kt_type,
    java_native_class,
    write_kotlin,
)
from .tools import _init_enum_names, _kt_param_type, _kt_return_type, _param_convert, _return_convert, _needs_return_wrap, _UNSUPPORTED_TYPES


def _uses_unsupported(method: dict) -> bool:
    """Return True if the method references any unsupported type."""
    ret = method.get("returns", "void")
    all_types = [ret] + [p["type"] for p in method.get("params", [])]
    for t in all_types:
        base = t.rstrip("?").rstrip("[]")
        if base in _UNSUPPORTED_TYPES:
            return True
        if "[]" in t:
            return True
        if "callback" in t.lower():
            return True
    return False


def _read_java_native_methods(native_cls: str) -> set:
    """Read the Java native class file and extract declared method names."""
    java_file = JAVA_DST / f"{native_cls}.java"
    if not java_file.exists():
        return set()
    content = java_file.read_text()
    return set(re.findall(r'public static native \S+ (\w+)\(', content))


# Sub-tools that delegate to a parent context
_SUB_TOOLS = {
    "AnimationController", "Tween", "Skeleton", "AnimationEvents",
    "AnimationLayerStack", "Network", "Plugin", "Audio",
}

# Physics tools that create their own context
_PHYSICS_TOOLS = {"PhysicsWorld2D", "PhysicsWorld3D"}

# Skip create/createWithBackend for physics tools
_PHYSICS_SKIP_METHODS = {"create", "createWithBackend"}


def _gen_sub_tool(tool_name: str):
    """Generate a sub-tool class that wraps JNI native methods."""
    _init_enum_names()

    tool = schema["tools"][tool_name]
    ffi_tools = schema.get("ffi_tools", {}).get(tool_name, {})
    ffi_methods = ffi_tools.get("methods", {})
    native_cls = java_native_class(tool_name)

    # Read Java native methods
    java_methods = _read_java_native_methods(native_cls)

    lines = [
        f"// {HEADER_COMMENT}",
        "package com.goudengine.core",
        "",
        f"import com.goudengine.internal.{native_cls}",
        "import com.goudengine.types.BoundingBox3D",
        "import com.goudengine.types.CharacterMoveResult",
        "",
        f"class {tool_name}(private val contextId: Long) {{",
        "",
    ]

    for method in tool.get("methods", []):
        mn = method["name"]

        # Skip unsupported methods
        if _uses_unsupported(method):
            continue

        ffi_entry = ffi_methods.get(mn, {})
        if ffi_entry.get("batch_in") or ffi_entry.get("batch_out_results") or ffi_entry.get("buffer_protocol"):
            continue

        # Skip methods not in ffi_methods mapping
        if mn not in ffi_methods:
            continue

        params = method.get("params", [])
        ret = method.get("returns", "void")
        kt_ret = _kt_return_type(ret)
        kt_mn = to_camel(mn)

        from .helpers import java_method_name
        java_mn = java_method_name(mn)

        # Verify Java native method exists
        if java_methods and java_mn not in java_methods:
            continue

        is_no_context = ffi_entry.get("no_context", False)

        kt_params_list = []
        call_args_list = [] if is_no_context else ["contextId"]
        for p in params:
            pname = p["name"]
            ptype = p["type"]
            kt_params_list.append(f"{pname}: {_kt_param_type(ptype)}")
            call_args_list.append(_param_convert(pname, ptype))

        kt_params = ", ".join(kt_params_list)
        call_args = ", ".join(call_args_list)

        if ret == "void":
            lines.append(f"    fun {kt_mn}({kt_params}) {{")
            lines.append(f"        {native_cls}.{java_mn}({call_args})")
            lines.append("    }")
        elif _needs_return_wrap(ret):
            lines.append(f"    fun {kt_mn}({kt_params}): {kt_ret} {{")
            lines.append(f"        val r = {native_cls}.{java_mn}({call_args})")
            lines.append(f"        return {_return_convert(ret, 'r')}")
            lines.append("    }")
        else:
            lines.append(f"    fun {kt_mn}({kt_params}): {kt_ret} =")
            lines.append(f"        {native_cls}.{java_mn}({call_args})")
        lines.append("")

    lines.append("}")
    lines.append("")

    write_kotlin(KOTLIN_OUT / "core" / f"{tool_name}.kt", "\n".join(lines))


def _gen_physics_tool(tool_name: str):
    """Generate a physics tool class that creates its own context."""
    _init_enum_names()

    tool = schema["tools"][tool_name]
    ffi_tools = schema.get("ffi_tools", {}).get(tool_name, {})
    ffi_methods = ffi_tools.get("methods", {})
    native_cls = java_native_class(tool_name)

    # Read Java native methods
    java_methods = _read_java_native_methods(native_cls)

    ctor_params = tool.get("constructor", {}).get("params", [])
    kt_ctor_params = ", ".join(
        f"{p['name']}: {kt_type(p['type'])}" for p in ctor_params
    )

    # Collect type imports needed for return types.
    type_imports = set()
    for method in tool.get("methods", []):
        ret = method.get("returns", "void")
        if ret in schema.get("types", {}):
            type_imports.add(ret)
    import_lines = [f"import com.goudengine.types.{t}" for t in sorted(type_imports)]

    lines = [
        f"// {HEADER_COMMENT}",
        "package com.goudengine.core",
        "",
        f"import com.goudengine.internal.{native_cls}",
        "import com.goudengine.internal.GoudContextNative",
    ] + import_lines + [
        "",
        f"class {tool_name} private constructor(private val contextId: Long) : AutoCloseable {{",
        "",
        "    companion object {",
        f"        fun create({kt_ctor_params}): {tool_name} {{",
        "            val ctx = GoudContextNative.create()",
        f"            require(ctx != 0L) {{ \"Failed to create context for {tool_name}\" }}",
    ]

    call_args = ", ".join(["ctx"] + [p["name"] for p in ctor_params])
    lines.append(f"            val status = {native_cls}.create({call_args})")
    lines += [
        "            if (status != 0) {",
        "                GoudContextNative.destroy(ctx)",
        f"                error(\"Failed to create {tool_name} (status $status)\")",
        "            }",
        f"            return {tool_name}(ctx)",
        "        }",
        "    }",
        "",
    ]

    for method in tool.get("methods", []):
        mn = method["name"]
        if mn in _PHYSICS_SKIP_METHODS:
            continue

        # Skip unsupported methods
        if _uses_unsupported(method):
            continue

        ffi_entry = ffi_methods.get(mn, {})
        if ffi_entry.get("batch_in") or ffi_entry.get("batch_out_results") or ffi_entry.get("buffer_protocol"):
            continue

        if mn not in ffi_methods and mn != "destroy":
            continue

        params = method.get("params", [])
        ret = method.get("returns", "void")
        kt_ret = _kt_return_type(ret)
        kt_mn = to_camel(mn)

        from .helpers import java_method_name
        java_mn = java_method_name(mn)

        # Verify Java native method exists
        if java_methods and java_mn not in java_methods and mn != "destroy":
            continue

        if mn == "destroy":
            lines.append(f"    fun destroy(): {kt_ret} {{")
            lines.append(f"        val status = {native_cls}.destroy(contextId)")
            lines.append("        GoudContextNative.destroy(contextId)")
            lines.append("        return status")
            lines.append("    }")
            lines.append("")
            continue

        kt_params_list = []
        call_args_list = ["contextId"]
        for p in params:
            pname = p["name"]
            ptype = p["type"]
            kt_params_list.append(f"{pname}: {_kt_param_type(ptype)}")
            call_args_list.append(_param_convert(pname, ptype))

        kt_params = ", ".join(kt_params_list)
        call_args = ", ".join(call_args_list)

        if ret == "void":
            lines.append(f"    fun {kt_mn}({kt_params}) {{")
            lines.append(f"        {native_cls}.{java_mn}({call_args})")
            lines.append("    }")
        elif _needs_return_wrap(ret):
            lines.append(f"    fun {kt_mn}({kt_params}): {kt_ret} {{")
            lines.append(f"        val r = {native_cls}.{java_mn}({call_args})")
            lines.append(f"        return {_return_convert(ret, 'r')}")
            lines.append("    }")
        else:
            lines.append(f"    fun {kt_mn}({kt_params}): {kt_ret} =")
            lines.append(f"        {native_cls}.{java_mn}({call_args})")
        lines.append("")

    lines.append("    override fun close() { destroy() }")
    lines.append("}")
    lines.append("")

    write_kotlin(KOTLIN_OUT / "core" / f"{tool_name}.kt", "\n".join(lines))


def _gen_engine_config():
    """Generate EngineConfig builder class."""
    _init_enum_names()

    tool = schema["tools"]["EngineConfig"]
    ffi_tools = schema.get("ffi_tools", {}).get("EngineConfig", {})
    ffi_methods = ffi_tools.get("methods", {})
    native_cls = java_native_class("EngineConfig")

    # Read Java native methods
    java_methods = _read_java_native_methods(native_cls)

    lines = [
        f"// {HEADER_COMMENT}",
        "package com.goudengine.core",
        "",
        f"import com.goudengine.internal.{native_cls}",
        "",
        "class EngineConfig private constructor(private var handle: Long) : AutoCloseable {",
        "",
        "    companion object {",
        "        fun create(): EngineConfig {",
        f"            val h = {native_cls}.create()",
        "            return EngineConfig(h)",
        "        }",
        "    }",
        "",
    ]

    for method in tool.get("methods", []):
        mn = method["name"]
        params = method.get("params", [])
        ret = method.get("returns", "void")
        kt_mn = to_camel(mn)

        # Skip unsupported methods
        if _uses_unsupported(method):
            continue

        from .helpers import java_method_name
        java_mn = java_method_name(mn)

        # Verify Java native method exists
        if java_methods and java_mn not in java_methods and mn not in ("build", "destroy"):
            continue

        if mn == "build":
            lines.append("    fun build(): GoudGame {")
            lines.append(f"        val ctx = {native_cls}.build(handle)")
            lines.append("        handle = 0L")
            lines.append("        require(ctx != 0L) { \"Failed to build engine from config\" }")
            lines.append("        return GoudGame(ctx)")
            lines.append("    }")
            lines.append("")
        elif mn == "destroy":
            lines.append("    fun destroy() {")
            lines.append(f"        if (handle != 0L) {{ {native_cls}.destroy(handle); handle = 0L }}")
            lines.append("    }")
            lines.append("")
        else:
            kt_params_list = []
            call_args_list = ["handle"]
            for p in params:
                pname = p["name"]
                ptype = p["type"]
                kt_params_list.append(f"{pname}: {_kt_param_type(ptype)}")
                call_args_list.append(_param_convert(pname, ptype))

            kt_params = ", ".join(kt_params_list)
            call_args = ", ".join(call_args_list)

            lines.append(f"    fun {kt_mn}({kt_params}): EngineConfig {{")
            lines.append(f"        {native_cls}.{java_mn}({call_args})")
            lines.append("        return this")
            lines.append("    }")
            lines.append("")

    lines.append("    override fun close() = destroy()")
    lines.append("}")
    lines.append("")

    write_kotlin(KOTLIN_OUT / "core" / "EngineConfig.kt", "\n".join(lines))


def _gen_ui_manager():
    """Generate UiManager class."""
    _init_enum_names()

    tool_def = schema.get("tools", {}).get("UiManager")
    if not tool_def:
        return

    ffi_tools = schema.get("ffi_tools", {}).get("UiManager", {})
    ffi_methods = ffi_tools.get("methods", {})
    native_cls = java_native_class("UiManager")

    # Read Java native methods
    java_methods = _read_java_native_methods(native_cls)

    lines = [
        f"// {HEADER_COMMENT}",
        "package com.goudengine.core",
        "",
        f"import com.goudengine.internal.{native_cls}",
        "",
        "class UiManager private constructor(private var handle: Long) : AutoCloseable {",
        "",
        "    companion object {",
        "        fun create(): UiManager {",
        f"            val h = {native_cls}.create()",
        "            require(h != 0L) { \"Failed to create UiManager\" }",
        "            return UiManager(h)",
        "        }",
        "    }",
        "",
    ]

    for method in tool_def.get("methods", []):
        mn = method["name"]
        params = method.get("params", [])
        ret = method.get("returns", "void")
        kt_ret = _kt_return_type(ret)
        kt_mn = to_camel(mn)

        # Skip unsupported methods
        if _uses_unsupported(method):
            continue

        # Skip destroy - handled below
        if mn == "destroy":
            continue

        from .helpers import java_method_name
        java_mn = java_method_name(mn)

        # Verify Java native method exists
        if java_methods and java_mn not in java_methods:
            continue

        kt_params_list = []
        call_args_list = ["handle"]
        for p in params:
            pname = p["name"]
            ptype = p["type"]
            kt_params_list.append(f"{pname}: {_kt_param_type(ptype)}")
            call_args_list.append(_param_convert(pname, ptype))

        kt_params = ", ".join(kt_params_list)
        call_args = ", ".join(call_args_list)

        if ret == "void":
            lines.append(f"    fun {kt_mn}({kt_params}) {{")
            lines.append(f"        {native_cls}.{java_mn}({call_args})")
            lines.append("    }")
        else:
            lines.append(f"    fun {kt_mn}({kt_params}): {kt_ret} =")
            lines.append(f"        {native_cls}.{java_mn}({call_args})")
        lines.append("")

    # Emit destroy - either calling native or as a no-op
    has_destroy = not java_methods or "destroy" in java_methods
    lines.append("    fun destroy() {")
    if has_destroy:
        lines.append(f"        if (handle != 0L) {{ {native_cls}.destroy(handle); handle = 0L }}")
    else:
        lines.append("        handle = 0L")
    lines.append("    }")
    lines.append("")

    lines.append("    override fun close() = destroy()")
    lines.append("}")
    lines.append("")

    write_kotlin(KOTLIN_OUT / "core" / "UiManager.kt", "\n".join(lines))


def gen_physics_ui():
    """Generate all physics, engine config, UI manager, and sub-tool classes."""
    for tool_name in _PHYSICS_TOOLS:
        if tool_name in schema.get("tools", {}):
            _gen_physics_tool(tool_name)

    _gen_engine_config()
    _gen_ui_manager()

    for tool_name in _SUB_TOOLS:
        if tool_name in schema.get("tools", {}):
            _gen_sub_tool(tool_name)
