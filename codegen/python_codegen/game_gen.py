"""Generator for `generated/_game.py`."""

from .context import HEADER_COMMENT, OUT, mapping, schema, write_generated
from .game_extras_gen import gen_engine_config, gen_ui_manager
from .game_tool_gen import gen_tool_class


def gen_game() -> None:
    lines = [
        f'"""{HEADER_COMMENT}"""',
        "",
        "import ctypes",
        "import json",
        "from . import _ffi as _ffi_module",
        "from ._ffi import (get_lib, GoudContextId, FfiVec2, FfiTransform2D, FfiSprite, FfiColor, FfiUiStyle, FfiUiEvent,",
        "    FfiNetworkStats, GoudRenderStats, GoudContact,",
        "    GoudMemoryCategoryStats, GoudMemorySummary)",
        "from ._types import (Entity, Vec2, Color, Transform2D, Sprite, RenderStats, UiStyle, UiEvent,",
        "    RenderCapabilities, PhysicsCapabilities, AudioCapabilities, InputCapabilities, NetworkStats,",
        "    NetworkSimulationConfig, NetworkConnectResult, NetworkPacket, NetworkCapabilities,",
        "    DebuggerConfig, ContextConfig, MemoryCategoryStats, MemorySummary,",
        "    DebuggerCapture, DebuggerReplayArtifact)",
        "from ._errors import GoudError",
        "from ._keys import DebuggerStepKind, Key, MouseButton, PhysicsBackend2D",
        "",
        "# Type IDs for built-in component types (hash of type name)",
        "_TYPEID_TRANSFORM2D = hash('Transform2D') & 0xFFFFFFFFFFFFFFFF",
        "_TYPEID_SPRITE = hash('Sprite') & 0xFFFFFFFFFFFFFFFF",
        "",
        "def _read_string_buffer(call):",
        '    """Read a string from a negative-required-size buffer-protocol FFI function."""',
        "    required = call(None, 0)",
        "    if required == 0:",
        '        return ""',
        "    if required == -1:",
        "        raise RuntimeError('buffer-protocol FFI call failed')",
        "    buf_len = -required if required < 0 else required + 1",
        "    while True:",
        "        buf = (ctypes.c_uint8 * buf_len)()",
        "        written = call(ctypes.cast(buf, ctypes.POINTER(ctypes.c_uint8)), buf_len)",
        "        if written == -1:",
        "            raise RuntimeError('buffer-protocol FFI call failed')",
        "        if written < 0:",
        "            buf_len = -written",
        "            continue",
        "        if written == 0:",
        '            return ""',
        '        return bytes(buf[:written]).decode("utf-8", errors="replace")',
        "",
    ]

    gen_tool_class("GoudGame", lines)

    if "GoudContext" in schema.get("tools", {}) and "GoudContext" in mapping.get("tools", {}):
        lines.append("")
        gen_tool_class("GoudContext", lines)

    if "PhysicsWorld2D" in schema.get("tools", {}) and "PhysicsWorld2D" in mapping.get("tools", {}):
        lines.append("")
        gen_tool_class("PhysicsWorld2D", lines)

    if "PhysicsWorld3D" in schema.get("tools", {}) and "PhysicsWorld3D" in mapping.get("tools", {}):
        lines.append("")
        gen_tool_class("PhysicsWorld3D", lines)

    if "EngineConfig" in schema.get("tools", {}) and "EngineConfig" in mapping.get("tools", {}):
        lines.append("")
        gen_engine_config(lines)

    if "UiManager" in schema.get("tools", {}) and "UiManager" in mapping.get("tools", {}):
        lines.append("")
        gen_ui_manager(lines)

    write_generated(OUT / "_game.py", "\n".join(lines))
