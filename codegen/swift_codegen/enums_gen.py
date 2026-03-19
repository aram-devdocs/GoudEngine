"""Generator for Enums.g.swift."""

from .context import HEADER_COMMENT, OUT, schema, to_screaming_snake, write_generated
from .shared_helpers import swift_file_header, SWIFT_TYPES


def gen_enums() -> None:
    enums = schema.get("enums", {})
    if not enums:
        return

    lines = [swift_file_header(), "import Foundation", ""]

    for enum_name, enum_def in enums.items():
        doc = enum_def.get("doc", "")
        values = enum_def.get("values", {})
        underlying = enum_def.get("underlying", "i32")
        swift_raw = SWIFT_TYPES.get(underlying, "Int32")

        if doc:
            lines.append(f"/// {doc}")
        lines.append(f"public enum {enum_name}: {swift_raw} {{")
        for vname, vval in values.items():
            swift_case = to_screaming_snake(vname)
            lines.append(f"    case {swift_case} = {vval}")
        lines.append("}")
        lines.append("")

    write_generated(OUT / "Enums.g.swift", "\n".join(lines))
