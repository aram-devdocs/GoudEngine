"""Builder class generation for Kotlin SDK."""
from __future__ import annotations
from .helpers import HEADER_COMMENT, KOTLIN_OUT, ENUM_SUBDIRS, schema, to_pascal, to_camel, kt_type, java_builder_native_class, write_kotlin


def _collect_enum_imports(builder_def):
    """Collect imports for enum types used in builder params."""
    imports = set()
    _enum_names = set(schema.get("enums", {}).keys())
    for m in builder_def.get("methods", []):
        for p in m.get("params", []):
            base = p["type"].rstrip("?")
            if base in _enum_names:
                subdir = ENUM_SUBDIRS.get(base, "core")
                imports.add(f"import com.goudengine.{subdir}.{to_pascal(base)}")
    return sorted(imports)


def gen_builders():
    type_methods = schema.get("ffi_type_methods", {})
    _enum_names = set(schema.get("enums", {}).keys())

    for type_name, type_def in schema["types"].items():
        if type_def.get("kind") != "component":
            continue
        builder_def = type_def.get("builder")
        if not builder_def:
            continue
        tm = type_methods.get(type_name, {})
        builder_map = tm.get("builder", {})
        if not builder_map:
            continue
        native_cls = java_builder_native_class(type_name)
        builder_name = f"{type_name}Builder"
        lines = [f"// {HEADER_COMMENT}", "package com.goudengine.components", "",
                 f"import com.goudengine.internal.{native_cls}",
                 f"import com.goudengine.internal.{type_name} as Java{type_name}", ""]

        # Add enum imports
        for imp in _collect_enum_imports(builder_def):
            lines.append(imp)
        if _collect_enum_imports(builder_def):
            lines.append("")

        lines += [f"class {builder_name} private constructor(private var handle: Long) : AutoCloseable {{", ""]
        schema_builder_methods = {m["name"]: m for m in builder_def.get("methods", [])}
        for bm_name, bm_info in builder_map.items():
            bm_schema = schema_builder_methods.get(bm_name, {})
            bm_params = bm_schema.get("params", [])
            java_mn = "create" if bm_name == "new" else ("defaultValue" if bm_name == "default" else to_camel(bm_name))
            kt_params_list, call_args_list = [], []
            for p in bm_params:
                pname = "value" if p["name"] == "handle" else p["name"]
                ptype = p["type"]
                base = ptype.rstrip("?")

                if base in _enum_names:
                    kt_params_list.append(f"{pname}: {to_pascal(base)}")
                    call_args_list.append(f"{pname}.value")
                else:
                    kt_params_list.append(f"{pname}: {kt_type(ptype)}")
                    call_args_list.append(pname)
            kt_params = ", ".join(kt_params_list)
            call_args = ", ".join(call_args_list)
            if bm_name in ("new", "default", "atPosition"):
                pass
            elif bm_name == "build":
                lines += [f"    fun build(): {type_name} {{", f"        val result = {native_cls}.build(handle)",
                          "        handle = 0L", f"        return {type_name}(result)", "    }", ""]
            elif bm_name == "free":
                lines += ["    fun free() {", f"        if (handle != 0L) {{ {native_cls}.free(handle); handle = 0L }}", "    }", ""]
            else:
                ffi_args = f"handle, {call_args}" if call_args else "handle"
                lines += [f"    fun {to_camel(bm_name)}({kt_params}): {builder_name} {{",
                          f"        handle = {native_cls}.{java_mn}({ffi_args})", "        return this", "    }", ""]
        lines += ["    override fun close() = free()", "", "    companion object {"]
        for bm_name, bm_info in builder_map.items():
            if bm_name not in ("new", "default", "atPosition"):
                continue
            bm_schema = schema_builder_methods.get(bm_name, {})
            bm_params = bm_schema.get("params", [])
            java_mn = "create" if bm_name == "new" else ("defaultValue" if bm_name == "default" else to_camel(bm_name))
            kt_params_list, call_args_list = [], []
            for p in bm_params:
                pname = "value" if p["name"] == "handle" else p["name"]
                ptype = p["type"]
                base = ptype.rstrip("?")

                if base in _enum_names:
                    kt_params_list.append(f"{pname}: {to_pascal(base)}")
                    call_args_list.append(f"{pname}.value")
                else:
                    kt_params_list.append(f"{pname}: {kt_type(ptype)}")
                    call_args_list.append(pname)
            kt_params = ", ".join(kt_params_list)
            call_args = ", ".join(call_args_list)
            lines += [f"        fun {to_camel(bm_name)}({kt_params}): {builder_name} =",
                      f"            {builder_name}({native_cls}.{java_mn}({call_args}))", ""]
        lines += ["    }", "}", ""]
        write_kotlin(KOTLIN_OUT / "components" / f"{builder_name}.kt", "\n".join(lines))
