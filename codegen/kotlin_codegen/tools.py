"""Tool class generation for Kotlin SDK (GoudGame, GoudContext)."""

from __future__ import annotations

from .helpers import (
    HEADER_COMMENT,
    KOTLIN_OUT,
    schema,
    to_pascal,
    to_camel,
    kt_type,
    java_native_class,
    write_kotlin,
)

# Java carrier types that need conversion
_CARRIER_TYPES = {"Color", "Vec2", "Vec3", "Rect", "Transform2D", "Sprite", "Text", "SpriteAnimator"}

# Types that are entity handles in the schema
_ENTITY_TYPES = {"Entity"}

# Enum types from schema
_ENUM_NAMES = set()


def _init_enum_names():
    global _ENUM_NAMES
    _ENUM_NAMES = set(schema.get("enums", {}).keys())


def _is_enum(t: str) -> bool:
    return t.rstrip("?") in _ENUM_NAMES


def _is_entity(t: str) -> bool:
    return t.rstrip("?") in _ENTITY_TYPES


def _is_carrier(t: str) -> bool:
    return t.rstrip("?") in _CARRIER_TYPES


def _param_convert(pname: str, ptype: str) -> str:
    """Convert a Kotlin param to Java native call arg."""
    base = ptype.rstrip("?")
    if _is_entity(base):
        return f"{pname}.id"
    if _is_enum(base):
        return f"{pname}.value"
    if base in ("Transform2D", "Sprite", "Text", "SpriteAnimator"):
        return f"{pname}.native"
    if base in ("Color", "Vec2", "Vec3", "Rect"):
        return f"{pname}.toNative()"
    return pname


def _return_convert(ret: str, expr: str) -> str:
    """Wrap a Java native return value into Kotlin type."""
    base = ret.rstrip("?").rstrip("[]")
    if _is_entity(base):
        return f"com.goudengine.core.EntityHandle({expr})"
    if base == "Color":
        return f"com.goudengine.types.Color.fromNative({expr})"
    if base == "Vec2":
        return f"com.goudengine.types.Vec2.fromNative({expr})"
    if base == "Vec3":
        return f"com.goudengine.types.Vec3({expr}.x, {expr}.y, {expr}.z)"
    if base == "Rect":
        return f"com.goudengine.types.Rect.fromNative({expr})"
    if base in ("Transform2D", "Sprite", "Text", "SpriteAnimator"):
        return f"com.goudengine.components.{base}({expr})"
    return expr


def _needs_return_wrap(ret: str) -> bool:
    base = ret.rstrip("?").rstrip("[]")
    return _is_entity(base) or base in _CARRIER_TYPES


def _kt_return_type(ret: str) -> str:
    """Map schema return type to Kotlin type for tool methods."""
    base = ret.rstrip("?").rstrip("[]")
    if _is_entity(base):
        mapped = "com.goudengine.core.EntityHandle"
    elif base in ("Color",):
        mapped = "com.goudengine.types.Color"
    elif base in ("Vec2",):
        mapped = "com.goudengine.types.Vec2"
    elif base in ("Vec3",):
        mapped = "com.goudengine.types.Vec3"
    elif base in ("Rect",):
        mapped = "com.goudengine.types.Rect"
    elif base in ("Transform2D", "Sprite", "Text", "SpriteAnimator"):
        mapped = f"com.goudengine.components.{base}"
    else:
        mapped = kt_type(base)

    if ret.endswith("[]"):
        return f"Array<{mapped}>"
    if ret.endswith("?"):
        return f"{mapped}?"
    return mapped


def _kt_param_type(ptype: str) -> str:
    """Map schema param type to Kotlin type for tool methods."""
    base = ptype.rstrip("?")
    if _is_entity(base):
        return "com.goudengine.core.EntityHandle"
    if _is_enum(base):
        return f"com.goudengine.{_enum_package(base)}.{to_pascal(base)}"
    if base in ("Color",):
        return "com.goudengine.types.Color"
    if base in ("Vec2",):
        return "com.goudengine.types.Vec2"
    if base in ("Vec3",):
        return "com.goudengine.types.Vec3"
    if base in ("Rect",):
        return "com.goudengine.types.Rect"
    if base in ("Transform2D", "Sprite", "Text", "SpriteAnimator"):
        return f"com.goudengine.components.{base}"
    return kt_type(ptype)


def _enum_package(enum_name: str) -> str:
    from .helpers import ENUM_SUBDIRS
    return ENUM_SUBDIRS.get(enum_name, "core")


def _gen_tool_class(tool_name: str, is_windowed: bool = False):
    _init_enum_names()

    tool = schema["tools"][tool_name]
    ffi_tools = schema.get("ffi_tools", {}).get(tool_name, {})
    ffi_methods = ffi_tools.get("methods", {})
    native_cls = java_native_class(tool_name)

    lines = [
        f"// {HEADER_COMMENT}",
        f"package com.goudengine.{'core' if not is_windowed else 'core'}",
        "",
        f"import com.goudengine.internal.{native_cls}",
        "",
    ]

    class_name = tool_name
    lines.append(f"class {class_name} private constructor(internal val contextId: Long) : AutoCloseable {{")
    lines.append("")

    # Constructor companion
    ctor = tool.get("constructor", {})
    ctor_params = ctor.get("params", [])

    lines.append("    companion object {")

    if is_windowed:
        # GoudGame factory
        kt_ctor_params = ", ".join(
            f"{p['name']}: {kt_type(p['type'])}" + (f" = {_kt_default(p)}" if p.get('default') is not None else "")
            for p in ctor_params
        )
        call_args = ", ".join(p["name"] for p in ctor_params)
        lines.append(f"        fun create({kt_ctor_params}): {class_name} {{")
        lines.append(f"            val ctx = {native_cls}.create({call_args})")
        lines.append(f"            require(ctx != 0L) {{ \"Failed to create {class_name}\" }}")
        lines.append(f"            return {class_name}(ctx)")
        lines.append("        }")
    else:
        lines.append(f"        fun create(): {class_name} {{")
        lines.append(f"            val ctx = {native_cls}.create()")
        lines.append(f"            require(ctx != 0L) {{ \"Failed to create {class_name}\" }}")
        lines.append(f"            return {class_name}(ctx)")
        lines.append("        }")

    lines.append("    }")
    lines.append("")

    # Methods
    for method in tool.get("methods", []):
        mn = method["name"]
        ffi_entry = ffi_methods.get(mn, {})

        # Skip destroy -- handled manually via close
        if mn == "destroy":
            continue

        params = method.get("params", [])
        ret = method.get("returns", "void")
        kt_ret = _kt_return_type(ret)

        # Rename close -> requestClose for windowed
        kt_mn = to_camel(mn)
        if mn == "close" and is_windowed:
            kt_mn = "requestClose"

        # Build param list
        kt_params_list = []
        call_args_list = ["contextId"]
        for p in params:
            pname = p["name"]
            ptype = p["type"]
            kt_params_list.append(f"{pname}: {_kt_param_type(ptype)}")
            call_args_list.append(_param_convert(pname, ptype))

        kt_params = ", ".join(kt_params_list)
        call_args = ", ".join(call_args_list)

        java_mn = to_camel(mn)
        # Apply Java method renames
        from .helpers import java_method_name
        java_mn = java_method_name(mn)

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

    # close/destroy
    lines.append("    fun destroy() {")
    lines.append(f"        {native_cls}.destroy(contextId)")
    lines.append("    }")
    lines.append("")
    lines.append("    override fun close() = destroy()")

    lines.append("}")
    lines.append("")

    write_kotlin(KOTLIN_OUT / "core" / f"{class_name}.kt", "\n".join(lines))


def _kt_default(p: dict) -> str:
    d = p.get("default")
    if d is None:
        return ""
    if isinstance(d, str) and not str(d).isdigit():
        return f'"{d}"'
    return str(d)


def gen_game():
    _gen_tool_class("GoudGame", is_windowed=True)


def gen_context():
    _gen_tool_class("GoudContext", is_windowed=False)
