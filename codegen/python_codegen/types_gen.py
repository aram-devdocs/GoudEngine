"""Generator for `generated/_types.py`."""

from .context import HEADER_COMMENT, OUT, schema, to_snake, write_generated
from .types_component_gen import gen_component_type
from .types_value_gen import gen_value_type


def _gen_handle_type(type_name: str, type_def: dict, lines: list[str]) -> None:
    lines.append(f"class {type_name}:")
    if type_def.get("doc"):
        lines.append(f'    """{type_def["doc"]}"""')
    lines.append("    def __init__(self, bits: int):")
    lines.append("        self._bits = bits")
    lines.append("")
    for prop in type_def.get("properties", []):
        pname = to_snake(prop["name"])
        if pname == "index":
            lines.append("    @property")
            lines.append(f"    def {pname}(self) -> int:")
            lines.append("        return self._bits & 0xFFFFFFFF")
        elif pname == "generation":
            lines.append("    @property")
            lines.append(f"    def {pname}(self) -> int:")
            lines.append("        return self._bits >> 32")
        elif pname == "is_placeholder":
            lines.append("    @property")
            lines.append(f"    def {pname}(self) -> bool:")
            lines.append("        return self._bits == 0xFFFFFFFFFFFFFFFF")
    for meth in type_def.get("methods", []):
        mname = to_snake(meth["name"])
        if mname == "to_bits":
            lines.append(f"    def {mname}(self) -> int:")
            lines.append("        return self._bits")
    lines.append("")
    lines.append("    def __repr__(self):")
    lines.append('        return f"Entity({self.index}v{self.generation})"')
    lines.append("")


def gen_types() -> None:
    lines = [
        f'"""{HEADER_COMMENT}"""',
        "",
        "import ctypes",
        "import math",
        "",
        "# Lazy FFI loading -- allows pure-Python types (Color, Vec2, etc.) to work",
        "# without the native library. FFI is only loaded on first use by component",
        "# types (Transform2D, Sprite) and their builders.",
        "_ffi_module = None",
        "_lib = None",
        "",
        "",
        "def _ensure_ffi():",
        '    """Load the FFI module and native library on first use."""',
        "    global _ffi_module, _lib",
        "    if _lib is not None:",
        "        return",
        "    from . import _ffi as ffi_mod",
        "    _ffi_module = ffi_mod",
        "    _lib = ffi_mod.get_lib()",
        "",
    ]

    for type_name, type_def in schema["types"].items():
        kind = type_def.get("kind")

        if kind == "handle":
            _gen_handle_type(type_name, type_def, lines)
            continue

        if kind == "component":
            gen_component_type(type_name, type_def, lines)
            continue

        if kind == "value":
            gen_value_type(type_name, type_def, lines)

    write_generated(OUT / "_types.py", "\n".join(lines))
