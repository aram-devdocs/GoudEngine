"""Generator for `generated/_keys.py`."""

from .context import HEADER_COMMENT, OUT, schema, to_screaming_snake, write_generated


def gen_keys() -> None:
    lines = [f'"""{HEADER_COMMENT}"""', ""]

    for enum_name, enum_def in schema["enums"].items():
        class_name = enum_name
        lines.append(f"class {class_name}:")
        if enum_def.get("doc"):
            lines.append(f'    """{enum_def["doc"]}"""')
        for vname, vval in enum_def["values"].items():
            lines.append(f"    {to_screaming_snake(vname)} = {vval}")
        lines.append("")

    write_generated(OUT / "_keys.py", "\n".join(lines))
