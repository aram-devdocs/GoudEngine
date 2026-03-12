"""Generator for `generated/_diagnostic.py`."""

from .context import HEADER_COMMENT, OUT, PYTHON_TYPES, schema, to_snake, write_generated


def gen_diagnostic() -> None:
    if "diagnostic" not in schema:
        return
    diag = schema["diagnostic"]
    lines = [
        f'"""{HEADER_COMMENT}"""',
        "",
        "import ctypes",
        "",
        "from ._ffi import _lib",
        "",
        "",
        f"class {diag['class_name']}:",
        f'    """{diag["doc"]}"""',
        "",
    ]
    for method in diag["methods"]:
        py_name = to_snake(method["name"])
        ffi_name = method["ffi"]
        params = method.get("params", [])
        ret = method["returns"]

        param_sig = ", ".join(f"{p['name']}: {PYTHON_TYPES.get(p['type'], p['type'])}" for p in params)
        call_args = ", ".join(p["name"] for p in params)

        lines.append("    @staticmethod")
        lines.append(f"    def {py_name}({param_sig}) -> {PYTHON_TYPES.get(ret, ret)}:")
        lines.append(f'        """{method["doc"]}"""')

        if method.get("buffer_protocol"):
            lines += [
                "        buf = (ctypes.c_uint8 * 4096)()",
                f"        written = _lib.{ffi_name}(buf, 4096)",
                "        if written <= 0:",
                '            return ""',
                '        return bytes(buf[:written]).decode("utf-8", errors="replace")',
            ]
        elif ret == "void":
            lines.append(f"        _lib.{ffi_name}({call_args})")
        elif ret == "bool":
            lines.append(f"        return bool(_lib.{ffi_name}({call_args}))")
        else:
            lines.append(f"        return _lib.{ffi_name}({call_args})")
        lines.append("")

    write_generated(OUT / "_diagnostic.py", "\n".join(lines))
