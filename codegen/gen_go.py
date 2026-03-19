"""Generator for Go SDK cgo bindings (sdks/go/internal/ffi/ffi.go)."""

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))

import sdk_common

# ── Go-specific type normalizer ──
# Preserves C typedef names so cgo can resolve them from the header.
# Only resolves Rust-path aliases and void-unit.

_GO_TYPE_ALIASES = {
    "()": "void",
    "crate::ffi::context::GoudContextId": "GoudContextId",
    "*const std::os::raw::c_char": "*const c_char",
    "UiManagerHandle": "*mut UiManager",
}


def _normalize(raw: str) -> str:
    t = raw.strip()
    alias = _GO_TYPE_ALIASES.get(t)
    if alias is not None:
        return _normalize(alias)
    if t.startswith("Option<") and t.endswith(">"):
        return f"Option<{_normalize(t[7:-1])}>"
    if t.startswith("*mut ") or t.startswith("*const "):
        q, inner = t.split(" ", 1)
        return f"{q} {_normalize(inner)}"
    return t


# ── Type classification ──
# C scalar typedefs (backed by an integer) -- zero value is 0
_C_SCALAR_TYPEDEFS = {
    "GoudEntityId", "GoudTextureHandle", "GoudFontHandle",  # uint64_t
    "GoudErrorCode", "GoudKeyCode", "GoudMouseButton",  # int32_t
}

# C enum types -- cgo wraps as Go uint32, pass directly
_C_ENUM_TYPES = {
    "FfiPlaybackMode", "GoudDebuggerStepKind",
}

# C pointer typedefs
_C_PTR_TYPEDEFS = {
    "EngineConfigHandle",  # void*
}

# C struct types (returned by value, zero value is C.Type{})
_C_STRUCT_TYPES = {
    "GoudContextId", "GoudResult",
    "FfiColor", "FfiVec2", "FfiRect", "FfiMat3x3",
    "FfiTransform2D", "FfiSprite", "FfiSpriteAnimator", "FfiText",
    "NetworkSimulationConfig",
}

# Primitive Rust -> Go
_R2GO = {
    "f32": "float32", "f64": "float64",
    "u8": "uint8", "u16": "uint16", "u32": "uint32", "u64": "uint64",
    "i8": "int8", "i16": "int16", "i32": "int32", "i64": "int64",
    "bool": "bool", "usize": "uint",
}

# Primitive Rust -> C cast
_R2C = {
    "f32": "C.float", "f64": "C.double",
    "u8": "C.uint8_t", "u16": "C.uint16_t", "u32": "C.uint32_t", "u64": "C.uint64_t",
    "i8": "C.int8_t", "i16": "C.int16_t", "i32": "C.int32_t", "i64": "C.int64_t",
    "bool": "C._Bool", "usize": "C.size_t",
}

# Known pointer param mappings
_PTR_MAP = {
    "*const c_char": "*C.char",
    "*const u8": "*C.uint8_t",
    "*mut u8": "*C.uint8_t",
    "*const u64": "*C.uint64_t",
    "*mut f32": "*C.float",
    "*mut i32": "*C.int32_t",
    "*mut i64": "*C.int64_t",
    "*mut u32": "*C.uint32_t",
    "*mut u64": "*C.uint64_t",
    "*mut c_void": "unsafe.Pointer",
    "*mut *const u8": "**C.uint8_t",
}


def _snake_to_pascal(name: str) -> str:
    return "".join(w.capitalize() for w in name.split("_"))


def _is_ptr(t: str) -> bool:
    return t.startswith("*const ") or t.startswith("*mut ")


def _is_cb(t: str) -> bool:
    return "Callback" in t


def _has_opaque_cb(params):
    return any(t.startswith("Option<") and _is_cb(t) for _, t in params)


def _go_param_type(t: str) -> str:
    if t in _R2GO:
        return _R2GO[t]
    if t in _PTR_MAP:
        return _PTR_MAP[t]
    if _is_cb(t):
        return "unsafe.Pointer"
    # C enums: cgo wraps enum params as plain uint32
    if t in _C_ENUM_TYPES:
        return "uint32"
    if t in _C_SCALAR_TYPEDEFS:
        return f"C.{t}"
    if t in _C_PTR_TYPEDEFS:
        return f"C.{t}"
    if _is_ptr(t):
        inner = t.split(" ", 1)[1]
        if inner == "c_void":
            return "unsafe.Pointer"
        if inner == "c_char":
            return "*C.char"
        if inner.startswith("*"):
            return "unsafe.Pointer"
        return f"*C.{inner}"
    return f"C.{t}"


def _go_ret_type(t: str) -> str:
    if t == "void":
        return ""
    if t in _R2GO:
        return _R2GO[t]
    if t in _C_SCALAR_TYPEDEFS or t in _C_ENUM_TYPES:
        return f"C.{t}"
    if t in _C_PTR_TYPEDEFS:
        return f"C.{t}"
    if _is_ptr(t):
        inner = t.split(" ", 1)[1]
        if inner == "c_void":
            return "unsafe.Pointer"
        if inner == "c_char":
            return "*C.char"
        if inner == "u8":
            return "*C.uint8_t"
        return f"*C.{inner}"
    return f"C.{t}"


def _c_cast(name: str, t: str) -> str:
    if t in _R2C:
        return f"{_R2C[t]}({name})"
    # C enums: cgo wraps as Go uint32, pass directly
    if t in _C_ENUM_TYPES:
        return name
    return name


def _go_cast_ret(t: str, expr: str) -> str:
    if t == "void":
        return expr
    if t in _R2GO:
        return f"{_R2GO[t]}({expr})"
    return expr


def _nil_return_val(go_ret: str, ffi_ret: str) -> str:
    if not go_ret:
        return ""
    if go_ret.startswith("*") or go_ret == "unsafe.Pointer":
        return "nil"
    if go_ret == "bool":
        return "false"
    # C struct types need a struct literal zero value
    if ffi_ret in _C_STRUCT_TYPES:
        return f"C.{ffi_ret}{{}}"
    # All scalar C typedefs and Go numeric types use 0
    return "0"


_GO_KW = frozenset({
    "break", "case", "chan", "const", "continue", "default", "defer",
    "else", "fallthrough", "for", "func", "go", "goto", "if", "import",
    "interface", "map", "package", "range", "return", "select", "struct",
    "switch", "type", "var",
})


def _safe(n: str) -> str:
    return n + "_" if n in _GO_KW else n


def generate() -> None:
    functions = sdk_common.load_ffi_manifest()

    needs_unsafe = False
    func_lines: list[str] = []

    for fn_name in sorted(functions):
        fdef = functions[fn_name]
        raw_params = fdef.get("params", [])
        ret_type = _normalize(fdef.get("return_type", "void"))

        params = []
        for p in raw_params:
            if isinstance(p, str):
                pn, _, pt = p.partition(":")
                params.append((pn.strip(), _normalize(pt.strip())))
            elif isinstance(p, dict):
                params.append((p["name"], _normalize(p["type"])))

        if _has_opaque_cb(params):
            continue

        go_name = _snake_to_pascal(fn_name)

        go_params = []
        ptr_params = []
        for pn, pt in params:
            gtype = _go_param_type(pt)
            sn = _safe(pn)
            go_params.append(f"{sn} {gtype}")
            if _is_ptr(pt) and not _is_cb(pt):
                ptr_params.append(sn)
            if "unsafe.Pointer" in gtype:
                needs_unsafe = True

        go_ret = _go_ret_type(ret_type)
        if go_ret == "unsafe.Pointer":
            needs_unsafe = True

        c_args = [_c_cast(_safe(pn), pt) for pn, pt in params]
        c_call = f"C.{fn_name}({', '.join(c_args)})"

        has_ret = go_ret != ""
        nil_val = _nil_return_val(go_ret, ret_type)

        func_lines.append(f"// {go_name} wraps {fn_name}.")
        ret_sig = f" {go_ret}" if has_ret else ""
        func_lines.append(f"func {go_name}({', '.join(go_params)}){ret_sig} {{")

        for sn in ptr_params:
            if has_ret:
                func_lines.append(f"\tif {sn} == nil {{")
                func_lines.append(f"\t\treturn {nil_val}")
                func_lines.append("\t}")
            else:
                func_lines.append(f"\tif {sn} == nil {{")
                func_lines.append("\t\treturn")
                func_lines.append("\t}")

        if has_ret:
            func_lines.append(f"\treturn {_go_cast_ret(ret_type, c_call)}")
        else:
            func_lines.append(f"\t{c_call}")

        func_lines.append("}")
        func_lines.append("")

    lines = [
        "package ffi",
        "",
        f"// {sdk_common.HEADER_COMMENT}",
        "",
        '// #include "goud_engine.h"',
        'import "C"',
    ]
    if needs_unsafe:
        lines.append('import "unsafe"')
    lines.append("")
    lines.extend(func_lines)

    sdk_common.write_generated(
        sdk_common.SDKS_DIR / "go" / "internal" / "ffi" / "ffi.go",
        "\n".join(lines),
    )


if __name__ == "__main__":
    generate()
