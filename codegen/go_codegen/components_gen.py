"""Generator for `goud/components.go` -- component types (Transform2D, Sprite, etc.)."""

from .context import GO_HEADER, GO_TYPES, GO_ZERO, OUT, schema, to_go_field, write_generated


def _go_field_type(field: dict) -> str:
    ft = field.get("type", "f32")
    if ft in GO_TYPES:
        return GO_TYPES[ft]
    return "float32"


def _go_zero(field: dict) -> str:
    ft = field.get("type", "f32")
    return GO_ZERO.get(ft, "0")


def gen_components() -> None:
    lines = [
        GO_HEADER,
        "",
        "package goud",
        "",
    ]

    for type_name, type_def in schema["types"].items():
        if type_def.get("kind") != "component":
            continue

        fields = type_def.get("fields", [])
        doc = type_def.get("doc", f"{type_name} component type.")

        lines.append(f"// {type_name} {doc}")
        lines.append(f"type {type_name} struct {{")
        for f in fields:
            fname = to_go_field(f["name"])
            ftype = _go_field_type(f)
            lines.append(f"\t{fname} {ftype}")
        lines.append("}")
        lines.append("")

        # New<Component> constructor with default values
        lines.append(f"// New{type_name} creates a {type_name} with default values.")
        lines.append(f"func New{type_name}() {type_name} {{")
        lines.append(f"\treturn {type_name}{{")
        for f in fields:
            fname = to_go_field(f["name"])
            ft = f.get("type", "f32")
            # Special defaults: scale should be 1.0
            if f["name"] in ("scaleX", "scaleY"):
                lines.append(f"\t\t{fname}: 1.0,")
            else:
                lines.append(f"\t\t{fname}: {_go_zero(f)},")
        lines.append("\t}")
        lines.append("}")
        lines.append("")

    write_generated(OUT / "components.go", "\n".join(lines))
