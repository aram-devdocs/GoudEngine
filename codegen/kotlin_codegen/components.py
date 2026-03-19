"""Component wrapper generation for Kotlin SDK."""
from __future__ import annotations
from .helpers import HEADER_COMMENT, KOTLIN_OUT, ENUM_SUBDIRS, schema, mapping, to_pascal, to_camel, kt_type, kdoc, java_type_native_class, write_kotlin

def _collect_java_imports(type_name, type_def):
    imports = set()
    # Import the carrier class with alias to avoid name clash with the wrapper
    imports.add(f"import com.goudengine.internal.{type_name} as Java{type_name}")
    native_cls = java_type_native_class(type_name)
    imports.add(f"import com.goudengine.internal.{native_cls}")
    for meth in type_def.get("methods", []):
        ret = meth.get("returns", "void")
        if ret in ("Vec2", "Mat3x3", "Color", "Rect"):
            imports.add(f"import com.goudengine.internal.{ret}")
        for p in meth.get("params", []):
            if p["type"] in ("Transform2D", "Sprite", "Text", "SpriteAnimator"):
                if p["type"] != type_name:
                    imports.add(f"import com.goudengine.internal.{p['type']} as Java{p['type']}")
    return sorted(imports)

def _java_method(name):
    from .helpers import java_method_name
    return java_method_name(name)

def _collect_enum_imports(type_def):
    """Collect imports for enum types used in component fields."""
    enum_names = set(schema.get("enums", {}).keys())
    imports = set()
    for f in type_def.get("fields", []):
        ftype = f["type"]
        if ftype in enum_names:
            subdir = ENUM_SUBDIRS.get(ftype, "core")
            imports.add(f"import com.goudengine.{subdir}.{to_pascal(ftype)}")
    for m in type_def.get("methods", []):
        for p in m.get("params", []):
            ptype = p["type"]
            if ptype in enum_names:
                subdir = ENUM_SUBDIRS.get(ptype, "core")
                imports.add(f"import com.goudengine.{subdir}.{to_pascal(ptype)}")
    return sorted(imports)

def gen_components():
    type_methods = schema.get("ffi_type_methods", {})
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
        comp_doc = type_def.get("doc")
        lines.extend(kdoc(comp_doc))
        lines += [f"class {type_name}(internal var native: Java{type_name}) {{", ""]
        enum_names = set(schema.get("enums", {}).keys())
        for f in fields:
            fn = to_camel(f["name"])
            ftype = f["type"]
            ft = kt_type(ftype)
            if ftype in enum_names:
                # Enum fields: Java carrier stores as int, convert to/from enum
                pascal_enum = to_pascal(ftype)
                lines += [f"    var {fn}: {pascal_enum}",
                          f"        get() = {pascal_enum}.fromValue(native.{fn}) ?: {pascal_enum}.entries.first()",
                          f"        set(value) {{ native.{fn} = value.value }}", ""]
            else:
                lines += [f"    var {fn}: {ft}", f"        get() = native.{fn}", f"        set(value) {{ native.{fn} = value }}", ""]
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
        # Collect field names to detect getter/setter clashes
        field_names = {to_camel(f["name"]) for f in fields}
        methods_map = tm.get("methods", {})
        schema_methods = {m["name"]: m for m in type_def.get("methods", [])}
        for mname, ffi_info in methods_map.items():
            java_mn = _java_method(mname)
            schema_meth = schema_methods.get(mname, {})
            params = schema_meth.get("params", [])
            ret = schema_meth.get("returns", "void")
            is_static = ffi_info.get("static", False)
            kt_ret = kt_type(ret) if ret != "void" else "Unit"

            # Skip getter/setter methods that clash with Kotlin properties
            camel_mn = to_camel(mname)
            if camel_mn.startswith("get") or camel_mn.startswith("set"):
                prop_name = camel_mn[3:4].lower() + camel_mn[4:] if len(camel_mn) > 3 else ""
                if prop_name in field_names:
                    continue
            if camel_mn.startswith("has"):
                prop_name = camel_mn  # hasX property vs hasX() method
                if prop_name in field_names:
                    continue
            kt_params_list = []
            call_args_list = ["native"] if not is_static else []
            for p in params:
                pname, ptype = p["name"], p["type"]
                kt_params_list.append(f"{pname}: {kt_type(ptype)}")
                call_args_list.append(f"{pname}.native" if ptype in ("Transform2D","Sprite","Text","SpriteAnimator") else pname)
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
            elif ret in ("Transform2D", "Sprite", "Text", "SpriteAnimator"):
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
