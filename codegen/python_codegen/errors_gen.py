"""Generator for `generated/_errors.py`."""

from .context import HEADER_COMMENT, OUT, load_errors, schema, write_generated


def gen_errors() -> None:
    categories, _codes = load_errors(schema)
    if not categories:
        return

    lines = [
        f'"""{HEADER_COMMENT}',
        "",
        "Typed error classes for GoudEngine Python SDK.",
        "",
        "Maps FFI error codes to language-idiomatic exceptions with code,",
        "message, context, and recovery information. All recovery logic",
        "lives in Rust; these classes only marshal the data.",
        '"""',
        "",
        "import ctypes",
        "",
        "",
        "class RecoveryClass:",
        '    """Recovery classification matching Rust RecoveryClass enum."""',
        "    RECOVERABLE = 0",
        "    FATAL = 1",
        "    DEGRADED = 2",
        "",
        '    _NAMES = {0: "recoverable", 1: "fatal", 2: "degraded"}',
        "",
        "    @classmethod",
        "    def name(cls, value):",
        '        return cls._NAMES.get(value, "unknown")',
        "",
        "",
        "class GoudError(Exception):",
        '    """Base exception for all GoudEngine errors."""',
        "",
        "    def __init__(self, error_code, message, category, subsystem,",
        "                 operation, recovery, recovery_hint):",
        "        super().__init__(message)",
        "        self.error_code = error_code",
        "        self.category = category",
        "        self.subsystem = subsystem",
        "        self.operation = operation",
        "        self.recovery = recovery",
        "        self.recovery_hint = recovery_hint",
        "",
        "    def __repr__(self):",
        "        return (",
        '            f"{type(self).__name__}(code={self.error_code}, "',
        '            f"category={self.category!r}, "',
        '            f"recovery={RecoveryClass.name(self.recovery)})"',
        "        )",
        "",
        "    @classmethod",
        "    def from_last_error(cls, lib):",
        '        """Query FFI error state and build the correct typed exception.',
        "",
        '        Returns None if no error is set (code == 0).',
        '        """',
        "        code = lib.goud_last_error_code()",
        "        if code == 0:",
        "            return None",
        "",
        "        message = _read_string(lib.goud_last_error_message)",
        "        subsystem = _read_string(lib.goud_last_error_subsystem)",
        "        operation = _read_string(lib.goud_last_error_operation)",
        "",
        "        recovery = lib.goud_error_recovery_class(code)",
        "        hint = _read_hint(lib, code)",
        "",
        "        category = _category_from_code(code)",
        "        subclass = _CATEGORY_CLASS_MAP.get(category, GoudError)",
        "",
        "        return subclass(",
        "            error_code=code,",
        "            message=message,",
        "            category=category,",
        "            subsystem=subsystem,",
        "            operation=operation,",
        "            recovery=recovery,",
        "            recovery_hint=hint,",
        "        )",
        "",
        "",
    ]

    for cat in categories:
        cls = cat["base_class"]
        doc = f'{cat["name"]} errors (codes {cat["range_start"]}-{cat["range_end"]}).'
        lines += [
            f"class {cls}(GoudError):",
            f'    """{doc}"""',
            "    pass",
            "",
            "",
        ]

    lines.append("_CATEGORY_CLASS_MAP = {")
    for cat in categories:
        lines.append(f'    "{cat["name"]}": {cat["base_class"]},')
    lines += ["}", "", ""]

    lines.append("def _category_from_code(code):")
    sorted_cats = sorted(categories, key=lambda c: c["range_start"], reverse=True)
    for cat in sorted_cats:
        lines.append(f'    if code >= {cat["range_start"]}:')
        lines.append(f'        return "{cat["name"]}"')
    lines += ['    return "Unknown"', "", ""]

    lines += [
        "def _read_string(ffi_fn):",
        '    """Call a buffer-writing FFI function and return the string."""',
        "    buf = (ctypes.c_uint8 * 256)()",
        "    written = ffi_fn(buf, 256)",
        "    if written <= 0:",
        '        return ""',
        '    return bytes(buf[:written]).decode("utf-8", errors="replace")',
        "",
        "",
    ]

    lines += [
        "def _read_hint(lib, code):",
        '    """Call goud_error_recovery_hint and return the string."""',
        "    buf = (ctypes.c_uint8 * 256)()",
        "    written = lib.goud_error_recovery_hint(code, buf, 256)",
        "    if written <= 0:",
        '        return ""',
        '    return bytes(buf[:written]).decode("utf-8", errors="replace")',
        "",
    ]

    write_generated(OUT / "_errors.py", "\n".join(lines))
