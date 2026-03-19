"""Generator for `goud/errors.go` -- GoudError and category types."""

from .context import GO_HEADER, OUT, schema, write_generated
from sdk_common import load_errors


def gen_errors() -> None:
    categories, _codes = load_errors(schema)
    if not categories:
        return

    lines = [
        GO_HEADER,
        "",
        "package goud",
        "",
        'import "fmt"',
        "",
        "// RecoveryClass classifies how an error can be recovered.",
        "type RecoveryClass int32",
        "",
        "const (",
        "\tRecoveryClassRecoverable RecoveryClass = 0",
        "\tRecoveryClassFatal       RecoveryClass = 1",
        "\tRecoveryClassDegraded    RecoveryClass = 2",
        ")",
        "",
        "// String returns the recovery class name.",
        "func (rc RecoveryClass) String() string {",
        "\tswitch rc {",
        '\tcase RecoveryClassRecoverable: return "recoverable"',
        '\tcase RecoveryClassFatal:       return "fatal"',
        '\tcase RecoveryClassDegraded:    return "degraded"',
        '\tdefault:                       return "unknown"',
        "\t}",
        "}",
        "",
        "// GoudError is the base error type for all GoudEngine errors.",
        "type GoudError struct {",
        "\tCode       int32",
        "\tMessage    string",
        "\tCategory   string",
        "\tSubsystem  string",
        "\tOperation  string",
        "\tRecovery   RecoveryClass",
        "\tHint       string",
        "}",
        "",
        "// Error implements the error interface.",
        "func (e *GoudError) Error() string {",
        '\treturn fmt.Sprintf("GoudError(code=%d, category=%s, recovery=%s): %s",',
        "\t\te.Code, e.Category, e.Recovery, e.Message)",
        "}",
        "",
    ]

    # Category error types
    for cat in categories:
        cls = cat["base_class"]
        doc = f"{cat['name']} errors (codes {cat['range_start']}-{cat['range_end']})."
        lines.append(f"// {cls} {doc}")
        lines.append(f"type {cls} struct {{ GoudError }}")
        lines.append("")

    # categoryFromCode function
    lines.append("// categoryFromCode maps an error code to its category name.")
    lines.append("func categoryFromCode(code int32) string {")
    sorted_cats = sorted(categories, key=lambda c: c["range_start"], reverse=True)
    for cat in sorted_cats:
        lines.append(f'\tif code >= {cat["range_start"]} {{ return "{cat["name"]}" }}')
    lines.append('\treturn "Unknown"')
    lines.append("}")
    lines.append("")

    write_generated(OUT / "errors.go", "\n".join(lines))
