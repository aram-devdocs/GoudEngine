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

    # readStringBuffer + categoryFromCode helpers
    lines += [
        "import CGoudEngine",
        "",
        "func readStringBuffer(_ call: (UnsafeMutablePointer<UInt8>?, Int) -> Int32) -> String {",
        "    let bufLen: Int = 256",
        "    var buf = [UInt8](repeating: 0, count: bufLen)",
        "    let written = call(&buf, bufLen)",
        '    guard written > 0 else { return \"\" }',
        '    return String(bytes: buf[..<Int(written)], encoding: .utf8) ?? \"\"',
        "}",
        "",
    ]

    lines.append("private func categoryFromCode(_ code: Int32) -> String {")
    sorted_cats = sorted(categories, key=lambda c: c["range_start"], reverse=True)
    for cat in sorted_cats:
        lines.append(f'    if code >= {cat["range_start"]} {{ return \"{cat["name"]}\" }}')
    lines.append('    return \"Unknown\"')
    lines.append("}")
    lines.append("")

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
        "",
        "    /// Query FFI error state and build a GoudError.",
        "    /// Returns nil if no error is set (code == 0).",
        "    public static func fromLastError() -> GoudError? {",
        "        let code = goud_last_error_code()",
        "        guard code != 0 else { return nil }",
        "        let message = readStringBuffer { buf, len in goud_last_error_message(buf, len) }",
        "        let subsystem = readStringBuffer { buf, len in goud_last_error_subsystem(buf, len) }",
        "        let operation = readStringBuffer { buf, len in goud_last_error_operation(buf, len) }",
        "        let recovery = RecoveryClass(rawValue: Int(goud_error_recovery_class(code))) ?? .fatal",
        "        let hint = readStringBuffer { buf, len in goud_error_recovery_hint(code, buf, len) }",
        "        let category = categoryFromCode(code)",
        "        return GoudError(",
        "            errorCode: code, message: message, category: category,",
        "            subsystem: subsystem, operation: operation,",
        "            recovery: recovery, recoveryHint: hint",
        "        )",
        "    }",
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
