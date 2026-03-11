"""Component wrapper generation."""

from sdk_common import to_pascal, write_generated
from .context import HEADER_COMMENT, NS, OUT, schema, mapping, _FFI_TO_SDK_FIELDS, _FFI_TO_SDK_RETURN
from .helpers import (
    cs_type,
    ffi_type,
    _cs_identifier,
    _cs_default_value,
    _cs_out_var_type,
    _normalize_manifest_ffi_type,
    _ffi_fn_def,
    _ffi_uses_ptr_len,
    _ffi_param_type_at,
    _cs_len_cast_expr,
)

def _gen_comp_factory(type_name, factory_name, ffi_info, schema_factory, lines):
    """Generate a static factory method for a component wrapper struct."""
    ffi_fn = ffi_info["ffi"]
    ffi_def = _ffi_fn_def(ffi_fn)
    ffi_ret = _normalize_manifest_ffi_type(ffi_def.get("returns", "void"))
    ffi_name = mapping["ffi_types"].get(type_name, {}).get("ffi_name", "")
    pascal_name = to_pascal(factory_name)
    args = schema_factory.get("args", []) if schema_factory else []
    cs_params = ", ".join(f"{cs_type(a['type'])} {a['name']}" for a in args)
    ffi_args = ", ".join(a["name"] for a in args)
    fields = _FFI_TO_SDK_FIELDS.get(ffi_name, [])

    if schema_factory and schema_factory.get("doc"):
        lines.append(f"        /// <summary>{schema_factory['doc']}</summary>")

    if fields:
        field_refs = ", ".join(f"__r.{f}" for f in fields)
        body = f"var __r = NativeMethods.{ffi_fn}({ffi_args}); return new {type_name}({field_refs});"
    else:
        # No field decomposition -- use internal constructor (e.g., FfiSprite)
        body = f"return new {type_name}(NativeMethods.{ffi_fn}({ffi_args}));"

    lines.append(f"        public static {type_name} {pascal_name}({cs_params})")
    lines.append(f"        {{ {body} }}")
    lines.append("")


def _get_method_names(type_def: dict) -> set:
    """Collect all PascalCase method names for a component type."""
    names = set()
    for m in type_def.get("methods", []):
        names.add(to_pascal(m["name"]))
    return names


def _gen_comp_method(type_name, method_name, ffi_info, schema_method, lines):
    """Generate an instance or static method for a component wrapper struct."""
    ffi_fn = ffi_info["ffi"]
    ffi_def = _ffi_fn_def(ffi_fn)
    ffi_ret = _normalize_manifest_ffi_type(ffi_def.get("returns", "void"))
    self_param = ffi_info.get("self_param", "")
    is_static = ffi_info.get("static", False)
    ffi_name = mapping["ffi_types"].get(type_name, {}).get("ffi_name", "")
    pascal_name = to_pascal(method_name)
    params = schema_method.get("params", []) if schema_method else []
    schema_ret = schema_method.get("returns", "void") if schema_method else "void"
    cs_params = ", ".join(f"{cs_type(p['type'])} {p['name']}" for p in params)

    # Determine C# return type
    cs_ret = cs_type(schema_ret) if schema_ret != "void" else "void"
    if schema_ret in ("Transform2D", "Sprite"):
        cs_ret = schema_ret

    if schema_method and schema_method.get("doc"):
        lines.append(f"        /// <summary>{schema_method['doc']}</summary>")

    is_mut_ptr = "*mut" in self_param
    is_const_ptr = "*const" in self_param
    is_by_value = self_param and not is_mut_ptr and not is_const_ptr

    if is_static:
        # Static method (e.g., normalizeAngle)
        ffi_args = ", ".join(p["name"] for p in params)
        lines.append(f"        public static {cs_ret} {pascal_name}({cs_params})")
        if ffi_ret == "void":
            lines.append(f"        {{ NativeMethods.{ffi_fn}({ffi_args}); }}")
        else:
            lines.append(f"        {{ return NativeMethods.{ffi_fn}({ffi_args}); }}")
        lines.append("")

    elif is_mut_ptr and ffi_ret == "void":
        # Mutation method: pass ref _inner
        extra_args = ", ".join(p["name"] for p in params)
        all_args = f"ref _inner, {extra_args}" if extra_args else "ref _inner"
        lines.append(f"        public void {pascal_name}({cs_params})")
        lines.append(f"        {{ NativeMethods.{ffi_fn}({all_args}); }}")
        lines.append("")

    elif (is_const_ptr or is_mut_ptr) and ffi_ret != "void":
        # Query method: pass ref _inner, return converted result
        extra_args = ", ".join(p["name"] for p in params)
        all_args = f"ref _inner, {extra_args}" if extra_args else "ref _inner"
        sdk_ret = _FFI_TO_SDK_RETURN.get(ffi_ret)

        # Special case for value-returning component methods whose FFI changed to out-params
        # (for example, getters that now return bool and fill an out struct).
        out_ffi_type = mapping["ffi_types"].get(schema_ret, {}).get("ffi_name")
        out_ptr_name = f"*mut {out_ffi_type}" if out_ffi_type else ""
        ffi_params = ffi_def.get("params", [])[1:]
        has_expected_out_param = any(
            p.get("type") == out_ptr_name and out_ffi_type for p in ffi_params
        )
        if out_ffi_type and has_expected_out_param and ffi_ret == "bool":
            out_var = "_out"
            lines.append(f"        public {cs_ret} {pascal_name}({cs_params})")
            lines.append("        {")
            lines.append(f"            {out_ffi_type} {out_var} = default;")
            # Append the by-ref out parameter (expected by generated FFI wrapper).
            all_args = f"{all_args}, ref {out_var}"
            if out_ffi_type in _FFI_TO_SDK_FIELDS:
                fields = _FFI_TO_SDK_FIELDS[out_ffi_type]
                field_refs = ", ".join(f"{out_var}.{f}" for f in fields)
                lines.append(f"            NativeMethods.{ffi_fn}({all_args});")
                lines.append(f"            return new {schema_ret}({field_refs});")
            else:
                lines.append(f"            NativeMethods.{ffi_fn}({all_args});")
                lines.append(f"            return {out_var};")
            lines.append("        }")
            lines.append("")
            return

        if sdk_ret and ffi_ret in _FFI_TO_SDK_FIELDS:
            fields = _FFI_TO_SDK_FIELDS[ffi_ret]
            field_refs = ", ".join(f"__r.{f}" for f in fields)
            lines.append(f"        public {cs_ret} {pascal_name}({cs_params})")
            lines.append(f"        {{ var __r = NativeMethods.{ffi_fn}({all_args}); return new {sdk_ret}({field_refs}); }}")
        elif sdk_ret:
            # Return type needs wrapping (e.g., FfiMat3x3 -> Mat3x3)
            lines.append(f"        public {cs_ret} {pascal_name}({cs_params})")
            lines.append(f"        {{ var __r = NativeMethods.{ffi_fn}({all_args}); var __w = new {sdk_ret}(); __w.Inner = __r; return __w; }}")
        else:
            lines.append(f"        public {cs_ret} {pascal_name}({cs_params})")
            lines.append(f"        {{ return NativeMethods.{ffi_fn}({all_args}); }}")
        lines.append("")

    elif is_by_value:
        # By-value self: struct passed by value, returns new struct
        ffi_all_params = ffi_def.get("params", [])
        ffi_arg_parts = ["_inner"]
        sdk_param_idx = 0
        for i, fp in enumerate(ffi_all_params):
            if i == 0:
                continue  # skip self
            if sdk_param_idx < len(params):
                p = params[sdk_param_idx]
                if p["type"] in mapping["ffi_types"]:
                    ffi_arg_parts.append(f"{p['name']}._inner")
                else:
                    ffi_arg_parts.append(p["name"])
                sdk_param_idx += 1
            else:
                ffi_arg_parts.append(fp["name"])
        ffi_args = ", ".join(ffi_arg_parts)

        sdk_ret = _FFI_TO_SDK_RETURN.get(ffi_ret)
        if sdk_ret and ffi_ret in _FFI_TO_SDK_FIELDS:
            fields = _FFI_TO_SDK_FIELDS[ffi_ret]
            field_refs = ", ".join(f"__r.{f}" for f in fields)
            lines.append(f"        public {cs_ret} {pascal_name}({cs_params})")
            lines.append(f"        {{ var __r = NativeMethods.{ffi_fn}({ffi_args}); return new {sdk_ret}({field_refs}); }}")
        elif sdk_ret:
            # Return type is a component (e.g., FfiSprite -> Sprite) but no field decomposition
            lines.append(f"        public {cs_ret} {pascal_name}({cs_params})")
            lines.append(f"        {{ return new {sdk_ret}(NativeMethods.{ffi_fn}({ffi_args})); }}")
        elif ffi_ret == "void":
            lines.append(f"        public void {pascal_name}({cs_params})")
            lines.append(f"        {{ NativeMethods.{ffi_fn}({ffi_args}); }}")
        else:
            lines.append(f"        public {cs_ret} {pascal_name}({cs_params})")
            lines.append(f"        {{ return NativeMethods.{ffi_fn}({ffi_args}); }}")
        lines.append("")

    elif is_const_ptr:
        # Const ptr, void return
        extra_args = ", ".join(p["name"] for p in params)
        all_args = f"ref _inner, {extra_args}" if extra_args else "ref _inner"
        lines.append(f"        public void {pascal_name}({cs_params})")
        lines.append(f"        {{ NativeMethods.{ffi_fn}({all_args}); }}")
        lines.append("")

    else:
        lines.append(f"        public {cs_ret} {pascal_name}({cs_params})")
        lines.append('        { throw new System.NotImplementedException(); }')
        lines.append("")


def gen_component_wrappers():
    """Generate component wrapper structs (Transform2D, Sprite) with FFI calls."""
    type_methods = mapping.get("type_methods", {})

    for type_name, type_def in schema["types"].items():
        if type_def.get("kind") != "component":
            continue
        ffi_name = mapping["ffi_types"].get(type_name, {}).get("ffi_name")
        if not ffi_name:
            continue

        tm = type_methods.get(type_name, {})
        fields = type_def.get("fields", [])
        method_names = _get_method_names(type_def)

        lines = [
            f"// {HEADER_COMMENT}",
            "using System;",
            "using System.Runtime.InteropServices;",
            "",
            f"namespace {NS}",
            "{",
        ]
        if type_def.get("doc"):
            lines.append(f"    /// <summary>{type_def['doc']}</summary>")
        lines += [f"    public struct {type_name}", "    {",
                  f"        internal {ffi_name} _inner;", ""]

        # Public properties wrapping inner fields
        # Skip properties whose PascalCase name collides with a method name
        for f in fields:
            pn = to_pascal(f["name"])
            ct = cs_type(f["type"])
            if pn in method_names:
                continue  # Skip: would collide with method of same name
            lines += [
                f"        public {ct} {pn}",
                "        {",
                f"            get => _inner.{pn};",
                f"            set => _inner.{pn} = value;",
                "        }",
            ]
        lines.append("")

        # Constructor from individual fields
        ctor_params = ", ".join(
            f"{cs_type(f['type'])} {to_pascal(f['name']).lower()}" for f in fields
        )
        lines += [f"        public {type_name}({ctor_params})", "        {",
                  f"            _inner = new {ffi_name}();"]
        for f in fields:
            pn = to_pascal(f["name"])
            lines.append(f"            _inner.{pn} = {pn.lower()};")
        lines += ["        }", ""]

        # Internal constructor from FFI struct
        lines += [
            f"        internal {type_name}({ffi_name} inner)",
            "        {",
            "            _inner = inner;",
            "        }", "",
        ]

        # Factory methods
        factories_map = tm.get("factories", {})
        schema_factories = {f["name"]: f for f in type_def.get("factories", [])}
        for fname, ffi_info in factories_map.items():
            _gen_comp_factory(type_name, fname, ffi_info, schema_factories.get(fname), lines)

        # Instance/static methods
        methods_map = tm.get("methods", {})
        schema_methods = {m["name"]: m for m in type_def.get("methods", [])}
        for mname, ffi_info in methods_map.items():
            _gen_comp_method(type_name, mname, ffi_info, schema_methods.get(mname), lines)

        # ToString - use _inner.X for fields whose properties were skipped
        field_parts = []
        for f in fields:
            pn = to_pascal(f["name"])
            if pn in method_names:
                field_parts.append(f"{{_inner.{pn}}}")
            else:
                field_parts.append(f"{{{pn}}}")
        field_interp = ", ".join(field_parts)
        lines.append(f'        public override string ToString() => $"{type_name}({field_interp})";')
        lines += ["    }", ""]

        # Builder class
        builder_def = type_def.get("builder")
        builder_map = tm.get("builder", {})
        if builder_def and builder_map:
            builder_name = f"{type_name}Builder"
            lines += [
                f"    /// <summary>{builder_def.get('doc', '')}</summary>",
                f"    public class {builder_name} : IDisposable",
                "    {",
                "        private IntPtr _ptr;", "",
            ]
            schema_builder_methods = {m["name"]: m for m in builder_def.get("methods", [])}

            for bm_name, bm_info in builder_map.items():
                bm_ffi = bm_info["ffi"]
                bm_schema = schema_builder_methods.get(bm_name, {})
                bm_params = bm_schema.get("params", [])
                bm_pascal = to_pascal(bm_name)
                cs_bm_params = ", ".join(f"{cs_type(p['type'])} {p['name']}" for p in bm_params)

                if bm_schema.get("doc"):
                    lines.append(f"        /// <summary>{bm_schema['doc']}</summary>")

                if bm_name in ("new", "default", "atPosition"):
                    # Static factory for builder
                    ffi_args = ", ".join(p["name"] for p in bm_params)
                    lines.append(f"        public static {builder_name} {bm_pascal}({cs_bm_params})")
                    lines.append(f"        {{ return new {builder_name}(NativeMethods.{bm_ffi}({ffi_args})); }}")
                    lines.append("")
                elif bm_name == "build":
                    # Consumes pointer, returns component
                    ffi_fields = _FFI_TO_SDK_FIELDS.get(ffi_name, [])
                    lines.append(f"        public {type_name} Build()")
                    if ffi_fields:
                        fr = ", ".join(f"__r.{f}" for f in ffi_fields)
                        lines.append(f"        {{ var __r = NativeMethods.{bm_ffi}(_ptr); _ptr = IntPtr.Zero; return new {type_name}({fr}); }}")
                    else:
                        lines.append(f"        {{ var __r = NativeMethods.{bm_ffi}(_ptr); _ptr = IntPtr.Zero; return new {type_name}(__r); }}")
                    lines.append("")
                elif bm_name == "free":
                    lines.append("        public void Free()")
                    lines.append(f"        {{ if (_ptr != IntPtr.Zero) {{ NativeMethods.{bm_ffi}(_ptr); _ptr = IntPtr.Zero; }} }}")
                    lines.append("")
                else:
                    # Chaining method
                    extra = ", ".join(p["name"] for p in bm_params)
                    args = f"_ptr, {extra}" if extra else "_ptr"
                    lines.append(f"        public {builder_name} {bm_pascal}({cs_bm_params})")
                    lines.append(f"        {{ _ptr = NativeMethods.{bm_ffi}({args}); return this; }}")
                    lines.append("")

            lines += [
                f"        private {builder_name}(IntPtr ptr) {{ _ptr = ptr; }}", "",
                "        public void Dispose() => Free();",
            ]
            lines += ["    }", ""]

        lines += ["}", ""]
        write_generated(OUT / "Components" / f"{type_name}.g.cs", "\n".join(lines))


# ── Entity.g.cs ─────────────────────────────────────────────────────
