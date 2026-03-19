"""Generator for `goud/keys.go` -- enum const blocks (Key, MouseButton, etc.)."""

from .context import GO_HEADER, GO_TYPES, OUT, schema, to_screaming_snake, write_generated


def gen_keys() -> None:
    lines = [
        GO_HEADER,
        "",
        "package goud",
        "",
    ]

    for enum_name, enum_def in schema["enums"].items():
        underlying = enum_def.get("underlying", "i32")
        go_type = GO_TYPES.get(underlying, "int32")
        doc = enum_def.get("doc", f"{enum_name} enum constants.")

        lines.append(f"// {enum_name} {doc}")
        lines.append(f"type {enum_name} {go_type}")
        lines.append("")
        lines.append(f"// {enum_name} constants.")
        lines.append("const (")
        for vname, vval in enum_def["values"].items():
            const_name = f"{enum_name}{vname}"
            lines.append(f"\t{const_name} {enum_name} = {vval}")
        lines.append(")")
        lines.append("")

    write_generated(OUT / "keys.go", "\n".join(lines))
