"""Component wrapper generation for Kotlin SDK."""
from __future__ import annotations
from .helpers import HEADER_COMMENT, KOTLIN_OUT, ENUM_SUBDIRS, schema, mapping, to_pascal, to_camel, kt_type, java_type_native_class, write_kotlin, kdoc

def _collect_java_imports(type_name, type_def):
    imports = set()
    imports.add(f"import com.goudengine.internal.{type_name} as Java{type_name}")
    native_cls = java_type_native_class(type_name)
    imports.add(f"import com.goudengine.internal.{native_cls}")
    for meth in type_def.get("methods", []):
        ret = meth.get("returns", "void")
        if ret in ("Vec2", "Mat3x3", "Color", "Rect"):
            imports.add(f"import com.goudengine.internal.{ret}")
        for p in meth.get("params", []):
            if p["type"] in ("Transform2D", "Sprite"):
                imports.add(f"import com.goudengine.internal.{p['type']} as Java{p['type']}")
    return sorted(imports)


def _collect_enum_imports(type_def):
    """Collect imports for enum types used in fields and methods."""
    imports = set()
    _enum_names = set(schema.get("enums", {}).keys())
    for f in type_def.get("fields", []):
        base = f["type"].rstrip("?")
        if base in _enum_names:
            subdir = ENUM_SUBDIRS.get(base, "core")
            imports.add(f"import com.goudengine.{subdir}.{to_pascal(base)}")
    for meth in type_def.get("methods", []):
        for p in meth.get("params", []):
            base = p["type"].rstrip("?")
            if base in _enum_names:
                subdir = ENUM_SUBDIRS.get(base, "core")
                imports.add(f"import com.goudengine.{subdir}.{to_pascal(base)}")
        ret = meth.get("returns", "void").rstrip("?")
        if ret in _enum_names:
            subdir = ENUM_SUBDIRS.get(ret, "core")
            imports.add(f"import com.goudengine.{subdir}.{to_pascal(ret)}")
    return sorted(imports)


def _java_method(name):
    from .helpers import java_method_name
    return java_method_name(name)


def _is_enum(t):
    return t.rstrip("?") in set(schema.get("enums", {}).keys())


def gen_components():
    type_methods = schema.get("ffi_type_methods", {})
    _enum_names = set(schema.get("enums", {}).keys())

    for type_name, type_def in schema["types"].items():
        if type_def.get("kind") != "component":
            continue
        tm = type_methods.get(type_name, {})
        fields = type_def.get("fields", [])
        native_cls = java_type_native_class(type_name)
        lines = [f"// {HEADER_COMMENT}", "package com.goudengine.components", ""]
        for imp in _collect_java_imports(type_name, type_def):
            lines.append(imp)
        for imp in _collect_enum_imports(type_def):
            lines.append(imp)
        lines += ["import com.goudengine.types.Vec2 as KtVec2", "import com.goudengine.types.Color as KtColor", ""]

        doc = type_def.get("doc")
        lines.extend(kdoc(doc))

        lines += [f"class {type_name}(internal var native: Java{type_name}) {{", ""]

        # Track property JVM signatures to avoid getter/setter clashes
        property_jvm_sigs = set()

        for f in fields:
            fn = to_camel(f["name"])
            ft_raw = f["type"]
            is_enum_field = _is_enum(ft_raw)
            ft = kt_type(ft_raw)

            if is_enum_field:
                enum_pascal = to_pascal(ft_raw.rstrip("?"))
                # Property with enum conversion
                lines += [
                    f"    var {fn}: {enum_pascal}",
                    f"        get() = {enum_pascal}.fromValue(native.{fn})!!",
                    f"        set(value) {{ native.{fn} = value.value }}",
                    "",
                ]
            else:
                lines += [f"    var {fn}: {ft}", f"        get() = native.{fn}", f"        set(value) {{ native.{fn} = value }}", ""]

            # Record JVM getter/setter signatures for clash detection
            property_jvm_sigs.add(f"get{fn[0].upper()}{fn[1:]}")
            property_jvm_sigs.add(f"set{fn[0].upper()}{fn[1:]}")

        factories_map = tm.get("factories", {})
        schema_factories = {fac["name"]: fac for fac in type_def.get("factories", [])}
        if factories_map:
            lines.append("    companion object {")
            for fname, ffi_info in factories_map.items():
                java_mn = _java_method(fname)
                schema_fac = schema_factories.get(fname, {})
                fargs = schema_fac.get("args", [])
                kt_params = ", ".join(f"{a['name']}: {kt_type(a['type'])}" for a in fargs)
                call_args = ", ".join(a["name"] for a in fargs)
                lines += [f"        fun {to_camel(fname)}({kt_params}): {type_name} =",
                          f"            {type_name}({native_cls}.{java_mn}({call_args}))", ""]
            lines += ["    }", ""]
        methods_map = tm.get("methods", {})
        schema_methods = {m["name"]: m for m in type_def.get("methods", [])}
        for mname, ffi_info in methods_map.items():
            java_mn = _java_method(mname)
            schema_meth = schema_methods.get(mname, {})
            params = schema_meth.get("params", [])
            ret = schema_meth.get("returns", "void")
            is_static = ffi_info.get("static", False)

            # Skip getter/setter methods that clash with property JVM signatures
            camel_name = to_camel(mname)
            if camel_name in property_jvm_sigs:
                continue

            kt_ret = kt_type(ret) if ret != "void" else "Unit"
            kt_params_list = []
            call_args_list = ["native"] if not is_static else []
            for p in params:
                pname, ptype = p["name"], p["type"]
                kt_params_list.append(f"{pname}: {kt_type(ptype)}")
                call_args_list.append(f"{pname}.native" if ptype in ("Transform2D","Sprite") else pname)
            kt_params = ", ".join(kt_params_list)
            call_args = ", ".join(call_args_list)
            if is_static:
                pass
            elif ret == "void":
                lines += [f"    fun {to_camel(mname)}({kt_params}) {{", f"        {native_cls}.{java_mn}({call_args})", f"        val n = native"]
                for field in fields:
                    fn = to_camel(field["name"])
                    lines.append(f"        this.{fn} = n.{fn}")
                lines.append("    }")
            elif ret in ("Vec2",):
                lines += [f"    fun {to_camel(mname)}({kt_params}): KtVec2 {{", f"        val r = {native_cls}.{java_mn}({call_args})", "        return KtVec2(r.x, r.y)", "    }"]
            elif ret in ("Color",):
                lines += [f"    fun {to_camel(mname)}({kt_params}): KtColor {{", f"        val r = {native_cls}.{java_mn}({call_args})", "        return KtColor(r.r, r.g, r.b, r.a)", "    }"]
            elif ret == "Mat3x3":
                lines += [f"    fun {to_camel(mname)}({kt_params}): com.goudengine.internal.Mat3x3 {{", f"        return {native_cls}.{java_mn}({call_args})", "    }"]
            elif ret == "Transform2D" or ret == type_name:
                lines += [f"    fun {to_camel(mname)}({kt_params}): {type_name} {{", f"        return {type_name}({native_cls}.{java_mn}({call_args}))", "    }"]
            else:
                lines += [f"    fun {to_camel(mname)}({kt_params}): {kt_ret} {{", f"        return {native_cls}.{java_mn}({call_args})", "    }"]
            lines.append("")
        static_methods = {mn: fi for mn, fi in methods_map.items() if fi.get("static", False)}
        if static_methods and not factories_map:
            lines.append("    companion object {")
        for mname, ffi_info in static_methods.items():
            java_mn = _java_method(mname)
            schema_meth = schema_methods.get(mname, {})
            params = schema_meth.get("params", [])
            ret = schema_meth.get("returns", "void")
            kt_ret = kt_type(ret) if ret != "void" else "Unit"
            kt_params = ", ".join(f"{p['name']}: {kt_type(p['type'])}" for p in params)
            call_args = ", ".join(p["name"] for p in params)
            lines += [f"    fun {to_camel(mname)}({kt_params}): {kt_ret} =", f"        {native_cls}.{java_mn}({call_args})", ""]
        lines.append("    override fun toString(): String =")
        field_str = ", ".join(f"${{{to_camel(f['name'])}}}" for f in fields)
        lines += [f'        "{type_name}({field_str})"', "}", ""]
        write_kotlin(KOTLIN_OUT / "components" / f"{type_name}.kt", "\n".join(lines))
