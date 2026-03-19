"""Generator for Errors.g.swift."""

from .context import HEADER_COMMENT, OUT, load_errors, schema, write_generated
from .shared_helpers import swift_file_header


def gen_errors() -> None:
    categories, _codes = load_errors(schema)
    if not categories:
        return

    lines = [swift_file_header(), "import Foundation", ""]

    # RecoveryClass enum
    lines += [
        "/// Recovery classification for engine errors.",
        "public enum RecoveryClass: Int {",
        "    case recoverable = 0",
        "    case fatal = 1",
        "    case degraded = 2",
        "}",
        "",
    ]

    # GoudError base class
    lines += [
        "/// Base error type for all GoudEngine errors.",
        "public class GoudError: Error, CustomStringConvertible {",
        "    public let errorCode: Int32",
        "    public let message: String",
        "    public let category: String",
        "    public let subsystem: String",
        "    public let operation: String",
        "    public let recovery: RecoveryClass",
        "    public let recoveryHint: String",
        "",
        "    public init(",
        "        errorCode: Int32,",
        "        message: String,",
        "        category: String,",
        "        subsystem: String,",
        "        operation: String,",
        "        recovery: RecoveryClass,",
        "        recoveryHint: String",
        "    ) {",
        "        self.errorCode = errorCode",
        "        self.message = message",
        "        self.category = category",
        "        self.subsystem = subsystem",
        "        self.operation = operation",
        "        self.recovery = recovery",
        "        self.recoveryHint = recoveryHint",
        "    }",
        "",
        '    public var description: String {',
        '        "\\(category)(\\(errorCode)): \\(message) [\\(subsystem)/\\(operation)] recovery=\\(recovery)"',
        '    }',
        "}",
        "",
    ]

    # Typed subclasses
    for cat in categories:
        cat_name = cat["name"]
        base_class = cat["base_class"]
        range_start = cat["range_start"]
        lines += [
            f"/// {cat_name} error (codes {range_start}+).",
            f"public class {base_class}: GoudError {{}}",
            "",
        ]

    write_generated(OUT / "Errors.g.swift", "\n".join(lines))
