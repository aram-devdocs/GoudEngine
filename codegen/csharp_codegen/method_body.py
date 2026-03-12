"""Method body generation helpers for tool wrappers."""

from sdk_common import CSHARP_TYPES, to_pascal
from .context import schema, mapping, _FFI_TO_SDK_RETURN
from .component_body import _WINDOWED_BODIES, _gen_component_body
from .helpers import (
    cs_type,
    _cs_identifier,
    _cs_default_value,
    _cs_out_var_type,
    _cs_value_param_ffi_setup,
    _ffi_fn_def,
    _ffi_return_type,
    _ffi_uses_ptr_len,
    _ffi_param_type_at,
    _cs_len_cast_expr,
)


def _cs_sdk_value_expr(source_expr: str, schema_type: str) -> str:
    """Build a C# SDK value-constructor expression from an FFI value expression."""
    type_def = schema.get("types", {}).get(schema_type, {})
    if type_def.get("kind") != "value":
        return source_expr

    field_exprs = []
    for field in type_def.get("fields", []):
        field_type = field["type"]
        access_expr = f"{source_expr}.{to_pascal(field['name'])}"
        if field_type in schema.get("types", {}) and schema["types"][field_type].get("kind") == "value":
            field_exprs.append(_cs_sdk_value_expr(access_expr, field_type))
        else:
            field_exprs.append(access_expr)
    return f"new {schema_type}({', '.join(field_exprs)})"


def _gen_method_body(mn: str, mm: dict, params: list, ret: str, L: list, is_windowed: bool):
    """Emit the body statements for one method into list L."""
    if mn == "Destroy":
        fn = "goud_window_destroy" if is_windowed else "goud_context_destroy"
        L.append(f"            if (!_disposed) {{ NativeMethods.{fn}(_ctx); _disposed = true; }}")
        if ret == "bool": L.append("            return true;")
        return
    if mn == "IsValid":
        L.append("            return NativeMethods.goud_context_is_valid(_ctx);"); return
    if mn in _WINDOWED_BODIES:
        L.extend(_WINDOWED_BODIES[mn])
        return
    if "ffi_strategy" in mm:
        _gen_component_body(mm, ret, L)
        return
    if mm.get("returns_nullable_struct"):
        struct_name = mm["returns_nullable_struct"]
        ffi_fn = mm["ffi"]
        ctx_prefix = "" if mm.get("no_context") else "_ctx, "
        ffi_args = ", ".join(p["name"] for p in params)
        L += ["            var contact = new GoudContact();",
              f"            if (!NativeMethods.{ffi_fn}({ctx_prefix}{ffi_args}, ref contact)) return null;",
              f"            return new {struct_name}(contact.PointX, contact.PointY, contact.NormalX, contact.NormalY, contact.Penetration);"]
        return
    if mm.get("batch_out"):
        ffi_fn = mm["ffi"]
        L += ["            var buf = new ulong[count];",
              "            uint filled;",
              "            unsafe { fixed (ulong* p = buf) { filled = NativeMethods." + ffi_fn + "(_ctx, count, ref buf[0]); } }",
              "            var result = new Entity[filled];",
              "            for (uint i = 0; i < filled; i++) result[i] = new Entity(buf[i]);",
              "            return result;"]
        return
    if mm.get("batch_in"):
        ffi_fn = mm["ffi"]
        # Collect extra params beyond the entities array
        extra_params = [p for p in params if p["type"] != "Entity[]" and p["name"] != "entities"]
        # Build extra args, converting u8[] to pinned IntPtr
        has_byte_array = any(p["type"] == "u8[]" for p in extra_params)
        non_byte_extra = [p for p in extra_params if p["type"] != "u8[]"]
        byte_array_param = next((p for p in extra_params if p["type"] == "u8[]"), None)
        L += ["            var buf = new ulong[entities.Length];",
              "            for (int i = 0; i < entities.Length; i++) buf[i] = entities[i].ToBits();"]
        if byte_array_param:
            ba_name = byte_array_param["name"]
            non_byte_args = ", ".join(p["name"] for p in non_byte_extra)
            non_byte_suffix = f", {non_byte_args}" if non_byte_args else ""
            L += ["            unsafe",
                  "            {",
                  "                fixed (ulong* p = buf)",
                  f"                fixed (byte* outp = {ba_name})",
                  "                {",
                  f"                    return NativeMethods.{ffi_fn}(_ctx, (System.IntPtr)p, (uint)entities.Length{non_byte_suffix}, (System.IntPtr)outp);",
                  "                }",
                  "            }"]
        else:
            extra_args_str = ", ".join(p["name"] for p in extra_params)
            extra_suffix = f", {extra_args_str}" if extra_args_str else ""
            L += ["            unsafe",
                  "            {",
                  "                fixed (ulong* p = buf)",
                  "                {",
                  f"                    return NativeMethods.{ffi_fn}(_ctx, (System.IntPtr)p, (uint)entities.Length{extra_suffix});",
                  "                }",
                  "            }"]
        return
    if mm.get("returns_entity"):
        if "entity_params" in mm:
            # Convert entity parameters to bits
            entity_args = ", ".join(f"{p}.ToBits()" for p in mm["entity_params"])
            L.append(f"            return new Entity(NativeMethods.{mm['ffi']}(_ctx, {entity_args}));")
        else:
            L.append(f"            return new Entity(NativeMethods.{mm['ffi']}(_ctx));")
        return
    if "entity_params" in mm and "ffi" in mm:
        ffi_fn = mm["ffi"]
        no_ctx = mm.get("no_context", False)
        ffi_ret = _ffi_return_type(ffi_fn)
        suffix = ".Success" if ret == "bool" and ffi_ret == "GoudResult" else ""
        entity_set = set(mm["entity_params"])
        string_set = set(mm.get("string_params", []))
        uses_ptr_len = _ffi_uses_ptr_len(ffi_fn)
        has_marshaled_string = any(
            p["type"] == "string" and p["name"] in string_set for p in params
        ) and uses_ptr_len

        if has_marshaled_string:
            L.append("            unsafe")
            L.append("            {")
            fixed_lines = []
            ffi_arg_parts = [] if no_ctx else ["_ctx"]
            ffi_param_index = 0 if no_ctx else 1
            for p in params:
                pname = p["name"]
                if pname in entity_set:
                    ffi_arg_parts.append(f"{pname}.ToBits()")
                    ffi_param_index += 1
                elif p["type"] == "string" and pname in string_set:
                    bvar = f"{pname}Bytes"
                    pvar = f"{pname}Ptr"
                    L.append(f"                var {bvar} = System.Text.Encoding.UTF8.GetBytes({pname});")
                    fixed_lines.append(f"byte* {pvar} = {bvar}")
                    ffi_arg_parts.append(f"(IntPtr){pvar}")
                    len_type = _ffi_param_type_at(ffi_fn, ffi_param_index + 1)
                    ffi_arg_parts.append(_cs_len_cast_expr(len_type, f"{bvar}.Length"))
                    ffi_param_index += 2
                else:
                    ffi_arg_parts.append(pname)
                    ffi_param_index += 1
            fixed_expr = "\n                ".join(f"fixed ({fl})" for fl in fixed_lines)
            L.append(f"                {fixed_expr}")
            L.append("                {")
            call = f"NativeMethods.{ffi_fn}({', '.join(ffi_arg_parts)}){suffix};"
            if ret != "void":
                L.append(f"                    return {call}")
            else:
                L.append(f"                    {call}")
            L.append("                }")
            L.append("            }")
        else:
            ffi_parts = [] if no_ctx else ["_ctx"]
            for p in params:
                if p["name"] in entity_set:
                    ffi_parts.append(f"{p['name']}.ToBits()")
                else:
                    ffi_parts.append(p["name"])
            call = f"NativeMethods.{ffi_fn}({', '.join(ffi_parts)}){suffix};"
            if ret != "void":
                L.append(f"            return {call}")
            else:
                L.append(f"            {call}")
        return
    if "enum_params" in mm and mm.get("ffi"):
        ek = list(mm["enum_params"].keys())[0]
        ffi_fn = mm["ffi"]
        prefix = "return " if ret != "void" else ""
        if mm.get("string_params"):
            sp = mm["string_params"][0]
            L.append(f"            {prefix}NativeMethods.{ffi_fn}(_ctx, {sp}, (int){ek});")
        else:
            L.append(f"            {prefix}NativeMethods.{ffi_fn}(_ctx, (int){ek});")
        return
    if "out_params" in mm and "returns_struct" in mm:
        struct_name = mm["returns_struct"]
        ffi_fn = mm["ffi"]
        no_ctx = mm.get("no_context", False)
        status_nullable_struct = bool(mm.get("status_nullable_struct"))
        status_struct = bool(mm.get("status_struct"))
        entity_set = set(mm.get("entity_params", []))
        enum_set = set(mm.get("enum_params", {}).keys())
        string_set = set(mm.get("string_params", []))
        uses_ptr_len = _ffi_uses_ptr_len(ffi_fn)
        out_params = mm["out_params"]
        out_locals = [f"_{op['name']}" for op in out_params]
        has_marshaled_string = any(
            p["type"] == "string" and p["name"] in string_set for p in params
        ) and uses_ptr_len

        for op, local in zip(out_params, out_locals):
            var_ty = _cs_out_var_type(op["type"])
            L.append(f"            {var_ty} {local} = {_cs_default_value(var_ty)};")
        if has_marshaled_string:
            L.append("            unsafe")
            L.append("            {")
            fixed_lines = []
            ffi_parts = [] if no_ctx else ["_ctx"]
            ffi_param_index = 0 if no_ctx else 1
            for p in params:
                pname = p["name"]
                if pname in entity_set:
                    ffi_parts.append(f"{pname}.ToBits()")
                    ffi_param_index += 1
                elif p["type"] == "string" and pname in string_set:
                    bvar = f"{pname}Bytes"
                    pvar = f"{pname}Ptr"
                    L.append(f"                var {bvar} = System.Text.Encoding.UTF8.GetBytes({pname});")
                    fixed_lines.append(f"byte* {pvar} = {bvar}")
                    ffi_parts.append(f"{bvar}.Length == 0 ? IntPtr.Zero : (IntPtr){pvar}")
                    len_type = _ffi_param_type_at(ffi_fn, ffi_param_index + 1)
                    ffi_parts.append(_cs_len_cast_expr(len_type, f"{bvar}.Length"))
                    ffi_param_index += 2
                elif pname in enum_set:
                    ffi_parts.append(f"(int){pname}")
                    ffi_param_index += 1
                else:
                    ffi_parts.append(pname)
                    ffi_param_index += 1
            ffi_parts.extend(f"ref {local}" for local in out_locals)
            fixed_expr = "\n                ".join(f"fixed ({fl})" for fl in fixed_lines)
            L.append(f"                {fixed_expr}")
            L.append("                {")
            if status_nullable_struct or status_struct:
                L.append(f"                    var _status = NativeMethods.{ffi_fn}({', '.join(ffi_parts)});")
                L.append("                    if (_status < 0)")
                L.append("                    {")
                L.append("                        var _ex = GoudException.FromLastError();")
                L.append("                        if (_ex != null) throw _ex;")
                L.append(f'                        throw new InvalidOperationException($"{ffi_fn} failed with status {{_status}}.");')
                L.append("                    }")
                if status_nullable_struct:
                    L.append("                    if (_status == 0) return null;")
            else:
                L.append(f"                    NativeMethods.{ffi_fn}({', '.join(ffi_parts)});")
            L.append("                }")
            L.append("            }")
        else:
            ffi_parts = [] if no_ctx else ["_ctx"]
            for p in params:
                pname = p["name"]
                if pname in entity_set:
                    ffi_parts.append(f"{pname}.ToBits()")
                elif pname in enum_set:
                    ffi_parts.append(f"(int){pname}")
                else:
                    ffi_parts.append(pname)
            ffi_parts.extend(f"ref {local}" for local in out_locals)

            if status_nullable_struct or status_struct:
                L.append(f"            var _status = NativeMethods.{ffi_fn}({', '.join(ffi_parts)});")
                L.append("            if (_status < 0)")
                L.append("            {")
                L.append("                var _ex = GoudException.FromLastError();")
                L.append("                if (_ex != null) throw _ex;")
                L.append(f'                throw new InvalidOperationException($"{ffi_fn} failed with status {{_status}}.");')
                L.append("            }")
                if status_nullable_struct:
                    L.append("            if (_status == 0) return null;")
            else:
                L.append(f"            NativeMethods.{ffi_fn}({', '.join(ffi_parts)});")

        if len(out_params) == 1 and out_params[0]["type"] not in CSHARP_TYPES:
            src = out_locals[0]
            L.append(f"            return {_cs_sdk_value_expr(src, struct_name)};")
        else:
            field_args = ", ".join(out_locals)
            L.append(f"            return new {struct_name}({field_args});")
        return
    if "out_params" in mm and "returns_scalar" in mm:
        ffi_fn = mm["ffi"]
        no_ctx = mm.get("no_context", False)
        entity_set = set(mm.get("entity_params", []))
        enum_set = set(mm.get("enum_params", {}).keys())
        out = mm["out_params"][0]
        out_ty = _cs_out_var_type(out["type"])
        out_name = f"_{out['name']}"
        L.append(f"            {out_ty} {out_name} = {_cs_default_value(out_ty)};")
        ffi_parts = [] if no_ctx else ["_ctx"]
        for p in params:
            pname = p["name"]
            if pname in entity_set:
                ffi_parts.append(f"{pname}.ToBits()")
            elif pname in enum_set:
                ffi_parts.append(f"(int){pname}")
            else:
                ffi_parts.append(pname)
        ffi_parts.append(f"ref {out_name}")
        L.append(f"            NativeMethods.{ffi_fn}({', '.join(ffi_parts)});")
        L.append(f"            return {out_name};")
        return
    if mm.get("out_buffer"):
        ffi_fn = mm["ffi"]
        no_ctx = mm.get("no_context", False)
        entity_set = set(mm.get("entity_params", []))
        enum_set = set(mm.get("enum_params", {}).keys())
        returns_struct = mm.get("returns_struct")
        status_nullable_struct = bool(mm.get("status_nullable_struct"))
        if not no_ctx:
            L.append("            int _bufferSize = GetNetworkReceiveBufferSize();")
        else:
            L.append("            const int _bufferSize = 65536;")
        L.append("            var buf = new byte[_bufferSize];")
        L.append("            ulong _peerId = 0;")
        L.append("            unsafe")
        L.append("            {")
        L.append("                fixed (byte* bufPtr = buf)")
        L.append("                {")
        ffi_parts = [] if no_ctx else ["_ctx"]
        for p in params:
            pname = p["name"]
            if pname in entity_set:
                ffi_parts.append(f"{pname}.ToBits()")
            elif pname in enum_set or p["type"] in schema.get("enums", {}):
                underlying = schema["enums"][p["type"]].get("underlying", "i32")
                ffi_parts.append(f"({cs_type(underlying)}){pname}")
            else:
                ffi_parts.append(pname)
        ffi_parts.extend(["(IntPtr)bufPtr", "buf.Length", "ref _peerId"])
        L.append(f"                    int _written = NativeMethods.{ffi_fn}({', '.join(ffi_parts)});")
        L.append("                    if (_written < 0)")
        L.append("                    {")
        L.append("                        var _ex = GoudException.FromLastError();")
        L.append("                        if (_ex != null) throw _ex;")
        L.append(f'                        throw new InvalidOperationException($"{ffi_fn} failed with status {{_written}}.");')
        L.append("                    }")
        if returns_struct and status_nullable_struct:
            L.append("                    if (_written == 0) return null;")
        else:
            L.append("                    if (_written == 0) return Array.Empty<byte>();")
        L.append("                    var result = new byte[_written];")
        L.append("                    Array.Copy(buf, result, _written);")
        if returns_struct:
            rs_fields = schema["types"][returns_struct]["fields"]
            field_args = []
            for field in rs_fields:
                if field["type"] in ("bytes", "u8[]"):
                    field_args.append("result")
                elif field["name"] == "peerId":
                    field_args.append("_peerId")
                else:
                    field_args.append(f"default({cs_type(field['type'])})")
            L.append(f"                    return new {returns_struct}({', '.join(field_args)});")
        else:
            L.append("                    return result;")
        L.append("                }")
        L.append("            }")
        return
    if mm.get("buffer_protocol"):
        ffi_fn = mm["ffi"]
        no_ctx = mm.get("no_context", False)
        entity_set = set(mm.get("entity_params", []))
        enum_set = set(mm.get("enum_params", {}).keys())
        ffi_args_parts = []
        ffi_param_index = 0 if no_ctx else 1
        for p in params:
            pname = p["name"]
            if pname in entity_set:
                ffi_args_parts.append(f"{pname}.ToBits()")
            elif pname in enum_set or p["type"] in schema.get("enums", {}):
                expected = _ffi_param_type_at(ffi_fn, ffi_param_index)
                if expected.startswith("Ffi") and expected[3:] in schema.get("enums", {}):
                    ffi_args_parts.append(pname)
                else:
                    underlying = schema["enums"][p["type"]].get("underlying", "i32")
                    ffi_args_parts.append(f"({cs_type(underlying)}){pname}")
            else:
                ffi_args_parts.append(pname)
            ffi_param_index += 1

        def _buffer_call(ptr_expr: str, len_expr: str) -> str:
            prefix = "" if no_ctx else "_ctx, "
            arg_prefix = ", ".join(ffi_args_parts)
            args = prefix
            if arg_prefix:
                args += f"{arg_prefix}, "
            args += f"{ptr_expr}, {len_expr}"
            return f"NativeMethods.{ffi_fn}({args})"

        L += [
            "            unsafe",
            "            {",
            f"                int _required = {_buffer_call('IntPtr.Zero', '(nuint)0')};",
            "                if (_required == -1)",
            "                {",
            "                    var _ex = GoudException.FromLastError();",
            "                    if (_ex != null) throw _ex;",
            f'                    throw new InvalidOperationException("{ffi_fn} failed.");',
            "                }",
            "                if (_required == 0) return string.Empty;",
            "                int _bufferSize = _required < 0 ? -_required : _required + 1;",
            "                while (true)",
            "                {",
            "                    var _buf = new byte[_bufferSize];",
            "                    fixed (byte* _ptr = _buf)",
            "                    {",
            f"                        int _written = {_buffer_call('(IntPtr)_ptr', '(nuint)_buf.Length')};",
            "                        if (_written == -1)",
            "                        {",
            "                            var _ex = GoudException.FromLastError();",
            "                            if (_ex != null) throw _ex;",
            f'                            throw new InvalidOperationException("{ffi_fn} failed.");',
            "                        }",
            "                        if (_written < 0)",
            "                        {",
            "                            _bufferSize = -_written;",
            "                            continue;",
            "                        }",
            "                        if (_written == 0) return string.Empty;",
            "                        return System.Text.Encoding.UTF8.GetString(_buf, 0, _written);",
            "                    }",
            "                }",
            "            }",
        ]
        return
    if "out_params" in mm:
        for op in mm["out_params"]:
            L.append(f"            float {op['name']} = 0;")
        refs = ", ".join(f"ref {op['name']}" for op in mm["out_params"])
        vals = ", ".join(op["name"] for op in mm["out_params"])
        L += [f"            NativeMethods.{mm['ffi']}(_ctx, {refs});",
              f"            return new Vec2({vals});"]
        return
    if "append_args" in mm:
        extra = ", ".join(str(a).lower() for a in mm["append_args"])
        L.append(f"            NativeMethods.{mm['ffi']}(_ctx, {extra});")
        return
    if "ffi" in mm:
        no_ctx = mm.get("no_context", False)
        ffi_fn = mm["ffi"]
        # Special case: componentRegisterType needs string->IntPtr marshalling
        if ffi_fn == "goud_component_register_type":
            L += ["            unsafe",
                  "            {",
                  "                var nameBytes = System.Text.Encoding.UTF8.GetBytes(name);",
                  "                fixed (byte* np = nameBytes)",
                  "                {",
                  f"                    return NativeMethods.{ffi_fn}(typeIdHash, (IntPtr)np, (nuint)nameBytes.Length, size, align);",
                  "                }",
                  "            }"]
            return
        if ffi_fn == "goud_network_set_simulation":
            value_setup = _cs_value_param_ffi_setup("NetworkSimulationConfig", "config")
            if value_setup:
                value_lines, value_arg = value_setup
                L.extend(value_lines)
                L.append(f"            return NativeMethods.{ffi_fn}(_ctx, handle, {value_arg});")
            else:
                L.append(f"            return NativeMethods.{ffi_fn}(_ctx, handle, config);")
            return
        # Generic ptr+len marshalling for UTF-8 strings and raw bytes.
        # Applies when the FFI function uses *const u8 (ptr+len), not *const c_char.
        buffer_params = [p for p in params if p["type"] in ("string", "bytes", "u8[]")]
        if buffer_params and _ffi_uses_ptr_len(ffi_fn):
            L.append("            unsafe")
            L.append("            {")
            fixed_lines = []
            ffi_arg_parts = [] if no_ctx else ["_ctx"]
            ffi_param_index = 0 if no_ctx else 1
            for p in params:
                if p["type"] in ("string", "bytes", "u8[]"):
                    bvar = f"{p['name']}Bytes"
                    pvar = f"{p['name']}Ptr"
                    if p["type"] == "string":
                        L.append(f"                var {bvar} = System.Text.Encoding.UTF8.GetBytes({p['name']});")
                    else:
                        L.append(f"                var {bvar} = {p['name']} ?? Array.Empty<byte>();")
                    fixed_lines.append(f"byte* {pvar} = {bvar}")
                    ffi_arg_parts.append(f"{bvar}.Length == 0 ? IntPtr.Zero : (IntPtr){pvar}")
                    len_type = _ffi_param_type_at(ffi_fn, ffi_param_index + 1)
                    ffi_arg_parts.append(_cs_len_cast_expr(len_type, f"{bvar}.Length"))
                    ffi_param_index += 2
                else:
                    if p["type"] in schema.get("enums", {}):
                        expected = _ffi_param_type_at(ffi_fn, ffi_param_index)
                        if expected.startswith("Ffi") and expected[3:] in schema.get("enums", {}):
                            ffi_arg_parts.append(p["name"])
                        else:
                            underlying = schema["enums"][p["type"]].get("underlying", "i32")
                            ffi_arg_parts.append(f"({cs_type(underlying)}){p['name']}")
                    else:
                        ffi_arg_parts.append(p["name"])
                    ffi_param_index += 1
            fixed_expr = "\n                ".join(f"fixed ({fl})" for fl in fixed_lines)
            L.append(f"                {fixed_expr}")
            L.append("                {")
            call = f"NativeMethods.{ffi_fn}({', '.join(ffi_arg_parts)});"
            L.append(f"                    {'return ' if ret != 'void' else ''}{call}")
            L.append("                }")
            L.append("            }")
            return
        ffi_args_parts = []
        ffi_param_index = 0 if no_ctx else 1
        for p in params:
            if p["type"] in schema.get("enums", {}):
                expected = _ffi_param_type_at(ffi_fn, ffi_param_index)
                if expected.startswith("Ffi") and expected[3:] in schema.get("enums", {}):
                    ffi_args_parts.append(p["name"])
                else:
                    underlying = schema["enums"][p["type"]].get("underlying", "i32")
                    ffi_args_parts.append(f"({cs_type(underlying)}){p['name']}")
            else:
                ffi_args_parts.append(p["name"])
            ffi_param_index += 1
        ffi_args = ", ".join(ffi_args_parts)
        all_args = ffi_args if no_ctx else (f"_ctx, {ffi_args}" if ffi_args else "_ctx")
        call_expr = f"NativeMethods.{ffi_fn}({all_args})"
        if mm.get("returns_bool_from_i32"):
            L.append(f"            return {call_expr} != 0;")
        else:
            stmt = f"{call_expr};"
            L.append(f"            {'return ' if ret != 'void' else ''}{stmt}")
        return
    L.append("            // TODO: implement")


# ── Tool class generation ─────────────────────────────────────────────
