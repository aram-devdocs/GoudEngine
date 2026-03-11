"""Parameter string helpers for tool wrapper generation."""

from .context import schema
from .helpers import cs_type, _enum_cs_name


def _cs_enum_default(enum_key: str, default) -> str:
    """Render a C# enum default from schema default values."""
    enum_name = _enum_cs_name(enum_key)
    enum_values = schema.get("enums", {}).get(enum_key, {}).get("values", {})

    if isinstance(default, str):
        if default in enum_values:
            return f"{enum_name}.{default}"
        if default.lstrip("-").isdigit():
            default = int(default)
    elif isinstance(default, float) and default.is_integer():
        default = int(default)

    for member, value in enum_values.items():
        if value == default:
            return f"{enum_name}.{member}"

    return f"({enum_name}){default}"


def _build_param_strs(params: list) -> list:
    result = []
    for p in params:
        pt, name, default = p["type"], p["name"], p.get("default")
        if pt == "callback(f32)":
            result.append(f"Action<float> {name}")
        elif pt == "Entity[]":
            result.append(f"Entity[] {name}")
        elif pt == "u8[]":
            result.append(f"byte[] {name}")
        elif pt in schema["types"]:
            result.append(f"{p['type']}? {name} = null" if default else f"{p['type']} {name}")
        elif pt in schema["enums"]:
            enum_name = _enum_cs_name(pt)
            if default is not None:
                result.append(f"{enum_name} {name} = {_cs_enum_default(pt, default)}")
            else:
                result.append(f"{enum_name} {name}")
        else:
            ct = cs_type(pt)
            if default is not None:
                if isinstance(default, float):
                    ds = f"{default}f"
                elif isinstance(default, str) and not default.replace(".", "").replace("-", "").isdigit():
                    ds = f'"{default}"'
                else:
                    ds = str(default)
                result.append(f"{ct} {name} = {ds}")
            else:
                result.append(f"{ct} {name}")
    return result


def _safe_param_strs(params: list) -> list:
    """Strip defaults from non-trailing optional params to avoid compile errors."""
    raw = _build_param_strs(params)
    last_required = -1
    for i, p in enumerate(params):
        is_opt = p.get("default") is not None or p["type"] in schema["types"]
        if not is_opt:
            last_required = i
    return [
        s.split("=")[0].rstrip() if (i < last_required and "=" in s) else s
        for i, s in enumerate(raw)
    ]
