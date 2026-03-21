"""Generator for `generated/__init__.py` and package root `__init__.py`."""

from .context import HEADER_COMMENT, OUT, mapping, schema, write_generated


def gen_init() -> None:
    has_context = "GoudContext" in schema.get("tools", {}) and "GoudContext" in mapping.get("tools", {})
    has_physics_world_2d = "PhysicsWorld2D" in schema.get("tools", {}) and "PhysicsWorld2D" in mapping.get("tools", {})
    has_physics_world_3d = "PhysicsWorld3D" in schema.get("tools", {}) and "PhysicsWorld3D" in mapping.get("tools", {})
    has_engine_config = "EngineConfig" in schema.get("tools", {}) and "EngineConfig" in mapping.get("tools", {})
    has_ui_manager = "UiManager" in schema.get("tools", {}) and "UiManager" in mapping.get("tools", {})

    type_imports = "Color, Vec2, Rect, Transform2D, Sprite, Entity"
    builder_imports = []
    for tn in ("Transform2D", "Sprite"):
        td = schema["types"].get(tn, {})
        if td.get("builder"):
            builder_imports.append(f"{tn}Builder")
    if builder_imports:
        type_imports += ", " + ", ".join(builder_imports)

    game_imports = ["GoudGame"]
    if has_context:
        game_imports.append("GoudContext")
    if has_physics_world_2d:
        game_imports.append("PhysicsWorld2D")
    if has_physics_world_3d:
        game_imports.append("PhysicsWorld3D")
    if has_engine_config:
        game_imports.append("EngineConfig")
    if has_ui_manager:
        game_imports.append("UiManager")

    enum_imports = sorted(schema.get("enums", {}).keys())

    has_diagnostic = "diagnostic" in schema
    networking_type_exports = [
        name
        for name in (
            "NetworkCapabilities",
            "NetworkConnectResult",
            "NetworkPacket",
            "NetworkSimulationConfig",
            "NetworkStats",
        )
        if name in schema.get("types", {})
    ]
    has_networking_wrappers = (OUT.parent / "networking.py").exists()
    has_debugger_helpers = (OUT.parent / "debugger.py").exists()

    lines = [
        f'"""{HEADER_COMMENT}"""',
        "",
        f"from ._types import {type_imports}",
    ]
    if enum_imports:
        lines.append(f"from ._keys import {', '.join(enum_imports)}")
    lines.extend([f"from ._game import {', '.join(game_imports)}"])
    if has_diagnostic:
        lines.append("from ._diagnostic import DiagnosticMode")
    lines += ["", "__all__ = ["]
    for gi in game_imports:
        lines.append(f'    "{gi}",')
    lines.append('    "Entity",')
    lines.append('    "Color", "Vec2", "Rect", "Transform2D", "Sprite",')
    for bi in builder_imports:
        lines.append(f'    "{bi}",')
    for ei in enum_imports:
        lines.append(f'    "{ei}",')
    if has_diagnostic:
        lines.append('    "DiagnosticMode",')

    lines.append("]")
    lines.append("")
    write_generated(OUT / "__init__.py", "\n".join(lines))

    root_init = [
        f'"""{HEADER_COMMENT}"""',
        "",
        "from .generated import *  # noqa: F401,F403",
        "from .generated import __all__ as _generated_all  # noqa: F401",
    ]
    if networking_type_exports:
        root_init.extend([
            "from .generated._types import (  # noqa: F401",
            *(f"    {name}," for name in networking_type_exports),
            ")",
        ])
    if has_networking_wrappers:
        root_init.append("from .networking import NetworkManager, NetworkEndpoint  # noqa: F401")
    if has_debugger_helpers:
        root_init.append(
            "from .debugger import parse_debugger_manifest, parse_debugger_snapshot  # noqa: F401"
        )
    root_init.append("")

    extra_exports: list[str] = []
    if has_networking_wrappers:
        extra_exports.extend(["NetworkManager", "NetworkEndpoint"])
    if has_debugger_helpers:
        extra_exports.extend(["parse_debugger_manifest", "parse_debugger_snapshot"])
    extra_exports.extend(networking_type_exports)

    if "errors" in schema:
        root_init.extend([
            "from .generated._errors import (  # noqa: F401",
            "    GoudError,",
        ])
        extra_exports.append("GoudError")
        for cat in schema["errors"].get("categories", []):
            cls = cat["base_class"]
            root_init.append(f"    {cls},")
            extra_exports.append(cls)
        root_init.extend(["    RecoveryClass,", ")", ""])
        extra_exports.append("RecoveryClass")

    # FNV-1a hash helper for generic component type registration.
    root_init.extend([
        "",
        "def component_type_hash(type_name: str) -> int:",
        '    """Compute an FNV-1a 64-bit hash for a component type name.',
        "",
        "    The result matches the Rust and C# codegen implementations, so it can",
        "    be passed directly to ``component_register_type`` and related FFI calls.",
        '    """',
        "    h = 0xCBF29CE484222325",
        '    for b in type_name.encode("utf-8"):',
        "        h ^= b",
        "        h = (h * 0x100000001B3) & 0xFFFFFFFFFFFFFFFF",
        "    return h",
        "",
    ])
    extra_exports.append("component_type_hash")

    root_init.append("__all__ = list(_generated_all) + [")
    for export in extra_exports:
        root_init.append(f'    "{export}",')
    root_init.extend(["]", ""])
    root = OUT.parent / "__init__.py"
    write_generated(root, "\n".join(root_init))
