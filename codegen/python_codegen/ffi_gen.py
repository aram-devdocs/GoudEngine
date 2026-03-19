"""Generator for `generated/_ffi.py`."""

from .context import CTYPES_MAP, HEADER_COMMENT, OUT, mapping, resolve_ctypes_type, schema, to_snake, write_generated
from .shared_helpers import resolve_ffi_param, resolve_ffi_return


def gen_ffi() -> None:
    lines = [
        f'"""{HEADER_COMMENT}"""',
        "",
        "import ctypes",
        "import os",
        "import platform",
        "import shutil",
        "import subprocess",
        "from pathlib import Path",
        "",
        "# ── Library loading ──",
        "",
        "def _has_required_symbol(lib_path: Path, symbol: str) -> bool:",
        "    # Some environments ship stale release artifacts; avoid selecting them when newer symbols exist elsewhere.",
        "    # Fall back to permissive loading if platform-native export inspection is unavailable.",
        "    try:",
        "        nm = shutil.which(\"nm\")",
        "        if nm:",
        "            result = subprocess.run([nm, \"-g\", str(lib_path)], check=False, capture_output=True, text=True)",
        "            if result.returncode == 0:",
        "                return symbol in result.stdout",
        "        if platform.system() == \"Windows\":",
        "            library = ctypes.WinDLL(str(lib_path))",
        "            return hasattr(library, symbol)",
        "    except Exception:",
        "        return True",
        "    return True",
        "",
        "def _env_library_candidates(name: str):",
        '    raw = os.environ.get("GOUD_ENGINE_LIB", "").strip()',
        "    if not raw:",
        "        return []",
        "    path = Path(raw)",
        "    if path.is_file():",
        "        return [path]",
        "    return [path / name]",
        "",
        "def _load_library():",
        '    """Load the GoudEngine shared library."""',
        "    system = platform.system()",
        '    if system == "Darwin":',
        '        ext, prefix = ".dylib", "lib"',
        '    elif system == "Linux":',
        '        ext, prefix = ".so", "lib"',
        '    elif system == "Windows":',
        '        ext, prefix = ".dll", ""',
        "    else:",
        '        raise OSError(f"Unsupported platform: {system}")',
        "",
        '    name = f"{prefix}goud_engine{ext}"',
        "    search = [",
        "        *_env_library_candidates(name),",
        '        Path(__file__).parent / name,',
        '        Path(__file__).parent.parent / name,',
        '        Path(__file__).parent.parent.parent.parent.parent / "target" / "debug" / name,',
        '        Path(__file__).parent.parent.parent.parent.parent / "target" / "release" / name,',
        "    ]",
        "    for p in search:",
        "        if p.exists():",
        "            if not _has_required_symbol(p, \"goud_engine_config_set_physics_debug\"):",
        "                continue",
        "            return ctypes.cdll.LoadLibrary(str(p))",
        '    raise OSError(f"Could not find {name}. Set GOUD_ENGINE_LIB env var.")',
        "",
        "_lib = _load_library()",
        "",
    ]

    lines.append("# ── FFI struct types ──")
    lines.append("")
    lines.append("class GoudContextId(ctypes.Structure):")
    lines.append('    _fields_ = [("_bits", ctypes.c_uint64)]')
    lines.append("")
    lines.append("class GoudResult(ctypes.Structure):")
    lines.append('    _fields_ = [("code", ctypes.c_int32), ("success", ctypes.c_bool)]')
    lines.append("")

    field_ctypes = {
        "f32": "ctypes.c_float",
        "f64": "ctypes.c_double",
        "u8": "ctypes.c_uint8",
        "u16": "ctypes.c_uint16",
        "u32": "ctypes.c_uint32",
        "u64": "ctypes.c_uint64",
        "usize": "ctypes.c_size_t",
        "ptr": "ctypes.c_void_p",
        "bool": "ctypes.c_bool",
        "i8": "ctypes.c_int8",
        "i16": "ctypes.c_int16",
        "i32": "ctypes.c_int32",
        "i64": "ctypes.c_int64",
    }

    for type_name, type_def in mapping["ffi_types"].items():
        ffi_name = type_def["ffi_name"]
        if not ffi_name or ffi_name == "u64":
            continue
        sdk_type = schema["types"].get(type_name)
        if not sdk_type or "fields" not in sdk_type:
            continue
        # Skip value types that share their SDK name — they have no
        # distinct FFI struct and are handled as plain Python classes.
        if sdk_type.get("kind") == "value" and ffi_name == type_name:
            continue
        lines.append(f"class {ffi_name}(ctypes.Structure):")
        fields_list = []
        for f in sdk_type["fields"]:
            fn = to_snake(f["name"])
            ft = f.get("type", "f32")
            if "[" in ft:
                base = ft.split("[")[0]
                count = int(ft.split("[")[1].rstrip("]"))
                ct = resolve_ctypes_type(
                    base,
                    enums=schema.get("enums", {}),
                    default=field_ctypes.get(base, "ctypes.c_float"),
                )
                fields_list.append(f'        ("{fn}", {ct} * {count})')
            else:
                if ft in schema.get("enums", {}):
                    underlying = schema["enums"][ft].get("underlying", "i32")
                    ct = CTYPES_MAP.get(underlying, "ctypes.c_int32")
                elif ft == "string":
                    ct = "ctypes.c_char_p"
                elif ft in mapping.get("ffi_types", {}):
                    ct = mapping["ffi_types"][ft].get("ffi_name", "ctypes.c_float")
                else:
                    ct = resolve_ctypes_type(
                        ft,
                        enums=schema.get("enums", {}),
                        default=field_ctypes.get(ft, "ctypes.c_float"),
                    )
                fields_list.append(f'        ("{fn}", {ct})')
        lines.append("    _fields_ = [")
        lines.append(",\n".join(fields_list))
        lines.append("    ]")
        lines.append("")

    lines.append("# ── Function signatures ──")
    lines.append("")
    lines.append("def _setup():")

    for module, funcs in mapping["ffi_functions"].items():
        if not isinstance(funcs, dict):
            continue
        optional = funcs.get("_feature") == "optional"
        lines.append(f"    # {module}")
        if optional:
            lines.append("    try:")
        indent = "        " if optional else "    "
        for fname, fdef in funcs.items():
            if fname.startswith("_"):
                continue
            if fdef.get("alias_of"):
                alias_fdef = funcs.get(fdef["alias_of"], fdef)
                argtypes = [resolve_ffi_param(p["type"]) for p in alias_fdef.get("params", fdef.get("params", []))]
                ret = alias_fdef.get("returns", fdef.get("returns", "void"))
                restype = resolve_ffi_return(ret)
                at_str = ", ".join(argtypes) if argtypes else ""
                lines.append(f"{indent}_lib.{fname}.argtypes = [{at_str}]")
                lines.append(f"{indent}_lib.{fname}.restype = {restype}")
                continue

            argtypes = [resolve_ffi_param(p["type"]) for p in fdef["params"]]
            restype = resolve_ffi_return(fdef["returns"])
            at_str = ", ".join(argtypes) if argtypes else ""
            lines.append(f"{indent}_lib.{fname}.argtypes = [{at_str}]")
            lines.append(f"{indent}_lib.{fname}.restype = {restype}")
        if optional:
            lines.append("    except AttributeError:")
            lines.append("        pass  # feature not compiled in")
        lines.append("")

    lines.append("_setup()")
    lines.append("")
    lines.append("def get_lib():")
    lines.append("    return _lib")
    lines.append("")

    write_generated(OUT / "_ffi.py", "\n".join(lines))
