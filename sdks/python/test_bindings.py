#!/usr/bin/env python3
"""
GoudEngine Python SDK Test Suite

Tests the generated Python SDK data types and enums without requiring
the native library to be built. Run with:
    python3 sdks/python/test_bindings.py
"""

import sys
import math
import importlib.util
from pathlib import Path

# Ensure goud_engine package is importable
sys.path.insert(0, str(Path(__file__).parent))

# ---------------------------------------------------------------------------
# Load pure-Python modules directly by file path so we never trigger the
# package __init__.py chain, which would attempt to load the native library
# via _ffi.py before it has been built.
# ---------------------------------------------------------------------------
_GENERATED_DIR = Path(__file__).parent / "goud_engine" / "generated"
_PACKAGE_DIR = Path(__file__).parent / "goud_engine"
_NETWORKING_PATH = _PACKAGE_DIR / "networking.py"
_ROOT_INIT_PATH = _PACKAGE_DIR / "__init__.py"
_LEGACY_ERRORS_PATH = Path(__file__).parent / "goud_engine" / "errors.py"
_ERRORS_PATH = (
    _LEGACY_ERRORS_PATH
    if _LEGACY_ERRORS_PATH.exists()
    else _GENERATED_DIR / "_errors.py"
)


def _load_module(name, path):
    spec = importlib.util.spec_from_file_location(name, path)
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    return mod


_types_mod = _load_module("_types", _GENERATED_DIR / "_types.py")
_keys_mod = _load_module("_keys", _GENERATED_DIR / "_keys.py")

Color = _types_mod.Color
Vec2 = _types_mod.Vec2
Rect = _types_mod.Rect
Transform2D = _types_mod.Transform2D
Sprite = _types_mod.Sprite
Entity = _types_mod.Entity
NetworkConnectResult = _types_mod.NetworkConnectResult
NetworkPacket = _types_mod.NetworkPacket
NetworkSimulationConfig = _types_mod.NetworkSimulationConfig
UiStyle = _types_mod.UiStyle
Key = _keys_mod.Key
MouseButton = _keys_mod.MouseButton


def test_imports():
    """Test that all public types can be imported from their generated modules."""
    print("Testing imports...")

    assert Color is not None, "Color failed to import"
    assert Vec2 is not None, "Vec2 failed to import"
    assert Rect is not None, "Rect failed to import"
    assert Transform2D is not None, "Transform2D failed to import"
    assert Sprite is not None, "Sprite failed to import"
    assert Entity is not None, "Entity failed to import"
    assert Key is not None, "Key failed to import"
    assert MouseButton is not None, "MouseButton failed to import"

    # GoudGame wraps FFI; verify the source file exists (cannot be imported
    # without the native library, which requires a Cargo build).
    game_path = _GENERATED_DIR / "_game.py"
    assert game_path.exists(), f"GoudGame source not found at {game_path}"

    print("  All imports successful")
    return True


def test_generated_scene_wrapper_api_names():
    """Validate generated scene wrapper method names without loading native lib."""
    print("Testing generated scene wrapper API names...")

    game_src = (_GENERATED_DIR / "_game.py").read_text()

    # New idiomatic wrappers
    assert "def load_scene(self, name, json):" in game_src, "missing load_scene wrapper"
    assert "def unload_scene(self, name):" in game_src, "missing unload_scene wrapper"
    assert "def set_active_scene(self, scene_id, active):" in game_src, "missing set_active_scene wrapper"

    # Legacy API kept for compatibility
    assert "def scene_create(self, name):" in game_src, "missing legacy scene_create API"
    assert "def scene_destroy(self, scene_id):" in game_src, "missing legacy scene_destroy API"
    assert "def scene_set_current(self, scene_id):" in game_src, "missing legacy scene_set_current API"

    print("  Scene wrapper API name tests passed")
    return True


def test_generated_audio_wrapper_api_names():
    """Validate generated audio wrapper method names without loading native lib."""
    print("Testing generated audio wrapper API names...")

    game_src = (_GENERATED_DIR / "_game.py").read_text()
    ffi_src = (_GENERATED_DIR / "_ffi.py").read_text()

    expected_game_methods = [
        "def audio_play(self, data):",
        "def audio_play_on_channel(self, data, channel):",
        "def audio_play_with_settings(self, data, volume, speed, looping, channel):",
        "def audio_stop(self, player_id):",
        "def audio_pause(self, player_id):",
        "def audio_resume(self, player_id):",
        "def audio_stop_all(self):",
        "def audio_set_global_volume(self, volume):",
        "def audio_get_global_volume(self):",
        "def audio_set_channel_volume(self, channel, volume):",
        "def audio_get_channel_volume(self, channel):",
        "def audio_is_playing(self, player_id):",
        "def audio_active_count(self):",
        "def audio_cleanup_finished(self):",
        "def audio_play_spatial3d(self, data, source_x, source_y, source_z, listener_x, listener_y, listener_z, max_distance, rolloff):",
        "def audio_update_spatial3d(self, player_id, source_x, source_y, source_z, listener_x, listener_y, listener_z, max_distance, rolloff):",
        "def audio_set_listener_position3d(self, x, y, z):",
        "def audio_set_source_position3d(self, player_id, x, y, z, max_distance, rolloff):",
        "def audio_set_player_volume(self, player_id, volume):",
        "def audio_set_player_speed(self, player_id, speed):",
        "def audio_crossfade(self, from_player_id, to_player_id, mix):",
        "def audio_crossfade_to(self, from_player_id, data, duration_sec, channel):",
        "def audio_mix_with(self, primary_player_id, data, secondary_volume, secondary_channel):",
        "def audio_update_crossfades(self, delta_sec):",
        "def audio_active_crossfade_count(self):",
        "def audio_activate(self):",
    ]
    for signature in expected_game_methods:
        assert signature in game_src, f"missing generated audio wrapper: {signature}"

    assert "def load_audio_clip(self" not in game_src, "deprecated load_audio_clip wrapper should not be generated"
    assert "def unload_audio_clip(self" not in game_src, "deprecated unload_audio_clip wrapper should not be generated"
    assert "_lib.load_audio_clip" not in ffi_src, "deprecated load_audio_clip ffi symbol should not be generated"
    assert "_lib.unload_audio_clip" not in ffi_src, "deprecated unload_audio_clip ffi symbol should not be generated"

    # Ensure ctypes declarations include the expected FFI export.
    assert "_lib.goud_audio_activate.argtypes = [GoudContextId]" in ffi_src, \
        "missing goud_audio_activate argtypes declaration"
    assert "_lib.goud_audio_activate.restype = ctypes.c_int32" in ffi_src, \
        "missing goud_audio_activate restype declaration"

    print("  Audio wrapper API name tests passed")
    return True


def test_generated_audio_activate_maps_to_activate_ffi():
    """Ensure audio_activate wrapper calls goud_audio_activate, not cleanup."""
    print("Testing generated audio_activate FFI mapping...")

    game_src = (_GENERATED_DIR / "_game.py").read_text()
    start = game_src.find("def audio_activate(self):")
    assert start != -1, "missing audio_activate wrapper"
    snippet = game_src[start:start + 220]

    assert "goud_audio_activate" in snippet, \
        "audio_activate wrapper should call goud_audio_activate"
    assert "goud_audio_cleanup_finished" not in snippet, \
        "audio_activate wrapper must not call goud_audio_cleanup_finished"

    print("  audio_activate mapping test passed")
    return True

def test_generated_network_wrapper_api_names():
    """Validate generated network wrapper API names and mappings without native load."""
    print("Testing generated network wrapper API names...")

    game_src = (_GENERATED_DIR / "_game.py").read_text()
    ffi_src = (_GENERATED_DIR / "_ffi.py").read_text()

    expected_game_methods = [
        "def network_host(self, protocol, port):",
        "def network_connect(self, protocol, address, port):",
        "def network_connect_with_peer(self, protocol, address, port):",
        "def network_disconnect(self, handle):",
        "def network_send(self, handle, peer_id, data, channel):",
        "def network_receive(self, handle):",
        "def network_receive_packet(self, handle):",
        "def network_poll(self, handle):",
        "def get_network_stats(self, handle):",
        "def network_peer_count(self, handle):",
        "def set_network_simulation(self, handle, config):",
        "def clear_network_simulation(self, handle):",
        "def set_network_overlay_handle(self, handle):",
        "def clear_network_overlay_handle(self):",
    ]
    for signature in expected_game_methods:
        assert signature in game_src, f"missing generated network wrapper: {signature}"

    assert "return self._lib.goud_network_host(self._ctx, protocol, port)" in game_src
    assert "return self._lib.goud_network_connect(self._ctx, protocol," in game_src
    assert "_status = self._lib.goud_network_connect_with_peer(self._ctx, protocol," in game_src
    assert "return NetworkConnectResult(_handle.value, _peer_id.value)" in game_src
    assert "_status = self._lib.goud_network_receive(self._ctx, handle," in game_src
    assert (
        "return NetworkPacket(_out_peer_id.value, bytes(_out_buf[:_status]))" in game_src
        or (
            "_data = bytes(_out_buf[:_status])" in game_src
            and "return NetworkPacket(_out_peer_id.value, _data)" in game_src
        )
    )
    assert "_status = self._lib.goud_network_get_stats_v2(self._ctx, handle, ctypes.byref(_stats))" in game_src
    assert "raise RuntimeError(f'goud_network_get_stats_v2 failed with status {_status}')" in game_src
    assert "return NetworkStats(" in game_src
    assert "_config_ffi = _ffi_module.FfiNetworkSimulationConfig()" in game_src
    assert "return self._lib.goud_network_set_simulation(self._ctx, handle, _config_ffi)" in game_src
    assert "class NetworkConnectResult:" in (_GENERATED_DIR / "_types.py").read_text()
    assert "class NetworkPacket:" in (_GENERATED_DIR / "_types.py").read_text()
    assert "class GoudContext:" in game_src
    assert "def network_connect_with_peer(self, protocol, address, port):" in game_src
    assert "def network_receive_packet(self, handle):" in game_src

    assert "_lib.goud_network_get_stats_v2.argtypes = [GoudContextId, ctypes.c_int64, ctypes.POINTER(FfiNetworkStats)]" in ffi_src
    assert "_lib.goud_network_connect_with_peer.argtypes = [GoudContextId, ctypes.c_int32, ctypes.POINTER(ctypes.c_uint8), ctypes.c_int32, ctypes.c_uint16, ctypes.POINTER(ctypes.c_int64), ctypes.POINTER(ctypes.c_uint64)]" in ffi_src
    assert "_lib.goud_network_set_overlay_handle.argtypes = [GoudContextId, ctypes.c_int64]" in ffi_src
    assert "_lib.goud_network_clear_overlay_handle.argtypes = [GoudContextId]" in ffi_src
    assert "_lib.goud_network_set_simulation.argtypes = [GoudContextId, ctypes.c_int64, FfiNetworkSimulationConfig]" in ffi_src
    assert "_lib.goud_network_clear_simulation.argtypes = [GoudContextId, ctypes.c_int64]" in ffi_src

    print("  Network wrapper API name tests passed")
    return True


def test_network_wrapper_exports_and_runtime_api():
    """Validate handwritten networking wrappers without loading the native library."""
    print("Testing handwritten networking wrapper exports...")

    networking_src = _NETWORKING_PATH.read_text()
    root_init_src = _ROOT_INIT_PATH.read_text()

    expected_exports = [
        "from .networking import NetworkManager, NetworkEndpoint",
        "from .generated._types import (",
        '"NetworkManager",',
        '"NetworkEndpoint",',
        '"NetworkConnectResult",',
        '"NetworkPacket",',
    ]
    for export_line in expected_exports:
        assert export_line in root_init_src, f"missing root networking export: {export_line}"

    expected_defs = [
        "class NetworkManager:",
        "class NetworkEndpoint:",
        "def host(self, protocol, port):",
        "def connect(self, protocol, address, port):",
        "def receive(self):",
        "def send(self, data, channel=0):",
        "def send_to(self, peer_id, data, channel=0):",
        "def poll(self):",
        "def disconnect(self):",
        "def get_stats(self):",
        "def peer_count(self):",
        "def set_simulation(self, config):",
        "def clear_simulation(self):",
        "def set_overlay_target(self):",
        "def clear_overlay_target(self):",
    ]
    for definition in expected_defs:
        assert definition in networking_src, f"missing networking wrapper member: {definition}"

    networking_mod = _load_module("_networking", _NETWORKING_PATH)
    NetworkManager = networking_mod.NetworkManager

    class FakeNetworkContext:
        def __init__(self):
            self.sent = []
            self.received = []
            self.overlay_target = None

        def network_host(self, protocol, port):
            self.hosted = (protocol, port)
            return 11

        def network_connect_with_peer(self, protocol, address, port):
            self.connected = (protocol, address, port)
            return NetworkConnectResult(22, 77)

        def network_receive_packet(self, handle):
            if self.received:
                return self.received.pop(0)
            return None

        def network_send(self, handle, peer_id, data, channel):
            self.sent.append((handle, peer_id, bytes(data), channel))
            return 0

        def network_poll(self, handle):
            self.last_polled = handle
            return 0

        def network_disconnect(self, handle):
            self.last_disconnected = handle
            return 0

        def get_network_stats(self, handle):
            self.last_stats_handle = handle
            return "stats"

        def network_peer_count(self, handle):
            self.last_peer_count_handle = handle
            return 2

        def set_network_simulation(self, handle, config):
            self.last_simulation = (handle, config)
            return 0

        def clear_network_simulation(self, handle):
            self.last_cleared_simulation = handle
            return 0

        def set_network_overlay_handle(self, handle):
            self.overlay_target = handle
            return 0

        def clear_network_overlay_handle(self):
            self.overlay_target = None
            return 0

    fake = FakeNetworkContext()
    manager = NetworkManager(fake)

    host_endpoint = manager.host(2, 40123)
    assert host_endpoint.handle == 11
    assert host_endpoint.default_peer_id is None

    client_endpoint = manager.connect(2, "127.0.0.1", 40123)
    assert client_endpoint.handle == 22
    assert client_endpoint.default_peer_id == 77

    fake.received.append(NetworkPacket(55, b"payload"))
    packet = host_endpoint.receive()
    assert packet.peer_id == 55
    assert packet.data == b"payload"

    assert client_endpoint.send(b"ping") == 0
    assert fake.sent[-1] == (22, 77, b"ping", 0)
    assert host_endpoint.send_to(55, b"pong", channel=1) == 0
    assert fake.sent[-1] == (11, 55, b"pong", 1)

    try:
        host_endpoint.send(b"no-default")
        raise AssertionError("host endpoint send should fail without a default peer")
    except RuntimeError as exc:
        assert "default peer" in str(exc).lower()

    sim = NetworkSimulationConfig(one_way_latency_ms=12, jitter_ms=3, packet_loss_percent=1.5)
    assert client_endpoint.set_simulation(sim) == 0
    assert client_endpoint.clear_simulation() == 0
    assert client_endpoint.set_overlay_target() == 0
    assert fake.overlay_target == 22
    assert client_endpoint.clear_overlay_target() == 0
    assert fake.overlay_target is None
    assert client_endpoint.poll() == 0
    assert client_endpoint.disconnect() == 0
    assert client_endpoint.get_stats() == "stats"
    assert client_endpoint.peer_count() == 2

    print("  Handwritten networking wrapper tests passed")
    return True


def test_handwritten_network_wrapper_exports():
    """Validate handwritten network wrapper exports and source API shape."""
    print("Testing handwritten network wrapper exports...")

    init_src = (_PACKAGE_DIR / "__init__.py").read_text()
    networking_path = _PACKAGE_DIR / "networking.py"
    assert networking_path.exists(), "missing handwritten networking wrapper module"

    networking_src = networking_path.read_text()
    assert "from .networking import NetworkManager, NetworkEndpoint" in init_src, \
        "__init__.py must export NetworkManager and NetworkEndpoint"
    assert "class NetworkManager:" in networking_src, "missing NetworkManager class"
    assert "class NetworkEndpoint:" in networking_src, "missing NetworkEndpoint class"
    assert "def host(self, protocol, port):" in networking_src, "missing NetworkManager.host()"
    assert "def connect(self, protocol, address, port):" in networking_src, "missing NetworkManager.connect()"
    assert "def receive(self):" in networking_src, "missing NetworkEndpoint.receive()"
    assert "def send(self, data, channel = 0):" in networking_src, "missing NetworkEndpoint.send()"
    assert "def send_to(self, peer_id, data, channel = 0):" in networking_src, "missing NetworkEndpoint.send_to()"
    assert "def disconnect(self):" in networking_src, "missing NetworkEndpoint.disconnect()"
    assert "network_connect_with_peer" in networking_src, "connect() must use network_connect_with_peer"

    print("  Handwritten network wrapper export tests passed")
    return True


def test_handwritten_network_wrapper_send_contract_source():
    """Validate endpoint send() and receive() wrapper contracts."""
    print("Testing handwritten network wrapper send() contract...")

    networking_path = _PACKAGE_DIR / "networking.py"
    networking_src = networking_path.read_text()
    assert "if self.default_peer_id is None:" in networking_src, \
        "send() must check for missing default peer ID"
    assert "raise ValueError" in networking_src, \
        "send() must fail clearly when no default peer ID exists"
    assert "return self.send_to(self.default_peer_id, data, channel)" in networking_src, \
        "send() must route through send_to() using default peer ID"

    networking_mod = _load_module("_networking", networking_path)
    NetworkManager = networking_mod.NetworkManager

    class _ConnectResult:
        def __init__(self, handle, peer_id):
            self.handle = handle
            self.peer_id = peer_id

    class _FakePacket:
        def __init__(self, peer_id, data):
            self.peer_id = peer_id
            self.data = data

    class _FakeBackend:
        def __init__(self):
            self.calls = []
            self._packet = _FakePacket(42, b"payload")

        def network_host(self, protocol, port):
            self.calls.append(("network_host", protocol, port))
            return 1001

        def network_connect_with_peer(self, protocol, address, port):
            self.calls.append(("network_connect_with_peer", protocol, address, port))
            return _ConnectResult(1002, 77)

        def network_send(self, handle, peer_id, data, channel):
            self.calls.append(("network_send", handle, peer_id, data, channel))
            return 0

        def network_poll(self, handle):
            self.calls.append(("network_poll", handle))
            return 0

        def network_receive_packet(self, handle):
            self.calls.append(("network_receive_packet", handle))
            packet = self._packet
            self._packet = None
            return packet

        def network_disconnect(self, handle):
            self.calls.append(("network_disconnect", handle))
            return 0

    backend = _FakeBackend()
    manager = NetworkManager(backend)
    host_endpoint = manager.host(2, 40000)
    assert host_endpoint.default_peer_id is None, "host() should not seed a default peer ID"

    try:
        host_endpoint.send(b"hello")
        assert False, "send() must raise ValueError when no default peer ID exists"
    except ValueError as exc:
        assert "default peer ID" in str(exc), "send() error should clearly mention default peer ID"

    connect_endpoint = manager.connect(2, "127.0.0.1", 40000)
    assert connect_endpoint.default_peer_id == 77, "connect() should seed default peer ID"
    assert connect_endpoint.send(b"hello") == 0
    assert connect_endpoint.send_to(88, b"other", 1) == 0

    packet = connect_endpoint.receive()
    assert packet is not None, "receive() should return packet when available"
    assert packet.peer_id == 42
    assert packet.data == b"payload"
    assert connect_endpoint.receive() is None, "receive() should return None when no packet exists"

    print("  Handwritten network wrapper send() contract tests passed")
    return True

    print("  Handwritten network wrapper send() contract tests passed")
    return True


def test_generated_ui_manager_wrapper_api_names():
    """Validate generated UiManager wrappers and exports without native lib."""
    print("Testing generated UiManager wrapper API names...")

    game_src = (_GENERATED_DIR / "_game.py").read_text()
    ffi_src = (_GENERATED_DIR / "_ffi.py").read_text()
    init_src = (_GENERATED_DIR / "__init__.py").read_text()

    assert "class UiManager:" in game_src, "missing UiManager wrapper class"
    assert "from ._game import GoudGame, GoudContext, PhysicsWorld2D, PhysicsWorld3D, EngineConfig, UiManager" in init_src, \
        "generated __init__.py must export UiManager"
    assert '"UiManager",' in init_src, "generated __all__ must include UiManager"

    expected_low_level = [
        "def create_node(self, component_type):",
        "def remove_node(self, node_id):",
        "def set_parent(self, child_id, parent_id):",
        "def get_parent(self, node_id):",
        "def get_child_count(self, node_id):",
        "def get_child_at(self, node_id, index):",
        "def set_widget(self, node_id, widget_kind):",
        "def set_style(self, node_id, style):",
        "def set_label_text(self, node_id, text):",
        "def set_button_enabled(self, node_id, enabled):",
        "def set_image_texture_path(self, node_id, path):",
        "def set_slider(self, node_id, min_value, max_value, value, enabled):",
        "def event_count(self):",
        "def event_read(self, index):",
    ]
    for signature in expected_low_level:
        assert signature in game_src, f"missing UiManager low-level wrapper: {signature}"

    expected_helpers = [
        "def create_panel(self):",
        "def create_label(self, text):",
        "def create_button(self, enabled = True):",
        "def create_image(self, path):",
        "def create_slider(self, min_value, max_value, value, enabled = True):",
    ]
    for signature in expected_helpers:
        assert signature in game_src, f"missing UiManager convenience wrapper: {signature}"

    assert "def set_event_callback(" not in game_src, \
        "UiManager must not expose a public raw callback registration wrapper"

    assert "class FfiUiStyle(ctypes.Structure):" in ffi_src, "missing FfiUiStyle in _ffi.py"
    assert '("background_color", FfiColor)' in ffi_src, "FfiUiStyle.background_color must be FfiColor"
    assert '("foreground_color", FfiColor)' in ffi_src, "FfiUiStyle.foreground_color must be FfiColor"
    assert '("border_color", FfiColor)' in ffi_src, "FfiUiStyle.border_color must be FfiColor"
    assert '("font_family_ptr", ctypes.c_void_p)' in ffi_src, "FfiUiStyle.font_family_ptr must be c_void_p"
    assert '("font_family_len", ctypes.c_size_t)' in ffi_src, "FfiUiStyle.font_family_len must be c_size_t"
    assert '("texture_path_ptr", ctypes.c_void_p)' in ffi_src, "FfiUiStyle.texture_path_ptr must be c_void_p"
    assert '("texture_path_len", ctypes.c_size_t)' in ffi_src, "FfiUiStyle.texture_path_len must be c_size_t"
    assert "_lib.goud_ui_set_style.argtypes = [ctypes.c_void_p, ctypes.c_uint64, ctypes.POINTER(FfiUiStyle)]" in ffi_src, \
        "goud_ui_set_style argtypes must use pointer to FfiUiStyle"
    assert "_lib.goud_ui_set_event_callback.argtypes = [ctypes.c_void_p, ctypes.c_void_p, ctypes.c_void_p]" in ffi_src, \
        "goud_ui_set_event_callback argtypes must preserve callback/user_data pointers"
    assert "_lib.goud_ui_event_read.argtypes = [ctypes.c_void_p, ctypes.c_uint32, ctypes.POINTER(FfiUiEvent)]" in ffi_src, \
        "goud_ui_event_read argtypes must use pointer to FfiUiEvent"
    assert "_lib.goud_ui_events_read.argtypes = [ctypes.c_void_p, ctypes.c_uint32, ctypes.POINTER(FfiUiEvent)]" in ffi_src, \
        "goud_ui_events_read argtypes must use pointer to FfiUiEvent"

    print("  UiManager wrapper API tests passed")
    return True


def test_generated_ui_style_color_contract():
    """Ensure generated UiStyle color fields match UiManager.set_style expectations."""
    print("Testing generated UiStyle color contract...")

    types_src = (_GENERATED_DIR / "_types.py").read_text()
    game_src = (_GENERATED_DIR / "_game.py").read_text()

    assert "background_color: 'Color' = None" in types_src, \
        "UiStyle.background_color should be generated as an optional Color input"
    assert "foreground_color: 'Color' = None" in types_src, \
        "UiStyle.foreground_color should be generated as an optional Color input"
    assert "border_color: 'Color' = None" in types_src, \
        "UiStyle.border_color should be generated as an optional Color input"

    style = UiStyle()
    assert isinstance(style.background_color, Color), \
        "UiStyle() should materialize a Color for background_color"
    assert isinstance(style.foreground_color, Color), \
        "UiStyle() should materialize a Color for foreground_color"
    assert isinstance(style.border_color, Color), \
        "UiStyle() should materialize a Color for border_color"

    custom = UiStyle(background_color=Color.red())
    assert custom.background_color.r == 1 and custom.background_color.g == 0, \
        "UiStyle should preserve caller-provided Color instances"

    assert "style.background_color.r" in game_src, \
        "UiManager.set_style should continue serializing Color channel fields"
    assert "style.foreground_color.g" in game_src, \
        "UiManager.set_style should continue serializing foreground Color channels"
    assert "style.border_color.b" in game_src, \
        "UiManager.set_style should continue serializing border Color channels"

    print("  UiStyle color contract tests passed")
    return True


def test_generated_ui_style_string_contract():
    """Ensure UiStyle exposes managed strings and set_style marshals them safely."""
    print("Testing generated UiStyle string contract...")

    types_src = (_GENERATED_DIR / "_types.py").read_text()
    game_src = (_GENERATED_DIR / "_game.py").read_text()

    assert "font_family: str = ''" in types_src, \
        "UiStyle.font_family should be a managed string field"
    assert "texture_path: str = ''" in types_src, \
        "UiStyle.texture_path should be a managed string field"
    assert "font_family_ptr" not in types_src, \
        "UiStyle must not expose raw font_family_ptr"
    assert "font_family_len" not in types_src, \
        "UiStyle must not expose raw font_family_len"
    assert "texture_path_ptr" not in types_src, \
        "UiStyle must not expose raw texture_path_ptr"
    assert "texture_path_len" not in types_src, \
        "UiStyle must not expose raw texture_path_len"

    style = UiStyle(font_family="Inter", texture_path="ui/button.png")
    assert style.font_family == "Inter", "UiStyle should preserve font_family strings"
    assert style.texture_path == "ui/button.png", "UiStyle should preserve texture_path strings"

    assert "font_family.encode('utf-8')" in game_src, \
        "UiManager.set_style must UTF-8 encode font_family during the call"
    assert "texture_path.encode('utf-8')" in game_src, \
        "UiManager.set_style must UTF-8 encode texture_path during the call"
    assert "ctypes.create_string_buffer(font_family_bytes, len(font_family_bytes))" in game_src, \
        "UiManager.set_style must pin font_family bytes for the FFI call"
    assert "ctypes.create_string_buffer(texture_path_bytes, len(texture_path_bytes))" in game_src, \
        "UiManager.set_style must pin texture_path bytes for the FFI call"
    assert "ffi_style.font_family_ptr = ctypes.cast(font_family_buf, ctypes.c_void_p).value if font_family_bytes else None" in game_src, \
        "UiManager.set_style must populate the FFI font pointer from the managed buffer"
    assert "ffi_style.texture_path_ptr = ctypes.cast(texture_path_buf, ctypes.c_void_p).value if texture_path_bytes else None" in game_src, \
        "UiManager.set_style must populate the FFI texture pointer from the managed buffer"

    print("  UiStyle string contract tests passed")
    return True


def test_vec2():
    """Test Vec2 construction, factories, arithmetic, and math methods."""
    print("Testing Vec2...")

    def approx(a, b, eps=0.001):
        return abs(a - b) < eps

    # Construction
    v = Vec2(3.0, 4.0)
    assert v.x == 3.0, f"Expected x=3.0, got {v.x}"
    assert v.y == 4.0, f"Expected y=4.0, got {v.y}"

    # Default construction
    v_default = Vec2()
    assert v_default.x == 0.0 and v_default.y == 0.0, "Default Vec2 should be (0, 0)"

    # Factories
    assert Vec2.zero().x == 0.0 and Vec2.zero().y == 0.0, "zero() should return (0, 0)"
    assert Vec2.one().x == 1.0 and Vec2.one().y == 1.0, "one() should return (1, 1)"
    assert Vec2.up().x == 0.0 and Vec2.up().y == -1.0, "up() should return (0, -1)"
    assert Vec2.down().x == 0.0 and Vec2.down().y == 1.0, "down() should return (0, 1)"
    assert Vec2.left().x == -1.0 and Vec2.left().y == 0.0, "left() should return (-1, 0)"
    assert Vec2.right().x == 1.0 and Vec2.right().y == 0.0, "right() should return (1, 0)"

    # Named methods
    a = Vec2(1.0, 2.0)
    b = Vec2(3.0, 4.0)

    result = a.add(b)
    assert result.x == 4.0 and result.y == 6.0, f"add() failed: {result}"

    result = a.sub(b)
    assert result.x == -2.0 and result.y == -2.0, f"sub() failed: {result}"

    result = a.scale(3.0)
    assert result.x == 3.0 and result.y == 6.0, f"scale() failed: {result}"

    # Operator overloads
    result = a + b
    assert result.x == 4.0 and result.y == 6.0, f"__add__ failed: {result}"

    result = b - a
    assert result.x == 2.0 and result.y == 2.0, f"__sub__ failed: {result}"

    result = a * 2.0
    assert result.x == 2.0 and result.y == 4.0, f"__mul__ failed: {result}"

    result = b / 2.0
    assert result.x == 1.5 and result.y == 2.0, f"__truediv__ failed: {result}"

    result = -a
    assert result.x == -1.0 and result.y == -2.0, f"__neg__ failed: {result}"

    # Length
    v = Vec2(3.0, 4.0)
    assert v.length() == 5.0, f"length() expected 5.0, got {v.length()}"

    # Normalize
    n = v.normalize()
    assert approx(n.length(), 1.0), f"normalize() result has non-unit length: {n.length()}"

    # Dot product
    a = Vec2(1.0, 0.0)
    b = Vec2(0.0, 1.0)
    assert a.dot(b) == 0.0, f"dot() of perpendicular vectors should be 0, got {a.dot(b)}"
    assert a.dot(a) == 1.0, f"dot() of unit vector with itself should be 1, got {a.dot(a)}"

    # Distance
    p = Vec2(0.0, 0.0)
    q = Vec2(3.0, 4.0)
    assert p.distance(q) == 5.0, f"distance() expected 5.0, got {p.distance(q)}"

    # Lerp
    start = Vec2(0.0, 0.0)
    end = Vec2(10.0, 20.0)
    mid = start.lerp(end, 0.5)
    assert mid.x == 5.0 and mid.y == 10.0, f"lerp(0.5) failed: {mid}"
    at_start = start.lerp(end, 0.0)
    assert at_start.x == 0.0 and at_start.y == 0.0, f"lerp(0.0) should equal start: {at_start}"
    at_end = start.lerp(end, 1.0)
    assert at_end.x == 10.0 and at_end.y == 20.0, f"lerp(1.0) should equal end: {at_end}"

    print("  Vec2 tests passed")
    return True


def test_color():
    """Test Color construction, factories, and with_alpha."""
    print("Testing Color...")

    def approx(a, b, eps=0.001):
        return abs(a - b) < eps

    # Construction
    c = Color(0.5, 0.6, 0.7, 0.8)
    assert approx(c.r, 0.5), f"Expected r=0.5, got {c.r}"
    assert approx(c.g, 0.6), f"Expected g=0.6, got {c.g}"
    assert approx(c.b, 0.7), f"Expected b=0.7, got {c.b}"
    assert approx(c.a, 0.8), f"Expected a=0.8, got {c.a}"

    # Default construction
    c_default = Color()
    assert c_default.r == 0.0 and c_default.g == 0.0 and c_default.b == 0.0 and c_default.a == 0.0, \
        "Default Color should be (0, 0, 0, 0)"

    # Factory: white
    white = Color.white()
    assert approx(white.r, 1.0) and approx(white.g, 1.0) and approx(white.b, 1.0) and approx(white.a, 1.0), \
        f"white() should be (1,1,1,1), got {white}"

    # Factory: black
    black = Color.black()
    assert approx(black.r, 0.0) and approx(black.g, 0.0) and approx(black.b, 0.0) and approx(black.a, 1.0), \
        f"black() should be (0,0,0,1), got {black}"

    # Factory: red
    red = Color.red()
    assert approx(red.r, 1.0) and approx(red.g, 0.0) and approx(red.b, 0.0) and approx(red.a, 1.0), \
        f"red() should be (1,0,0,1), got {red}"

    # Factory: green
    green = Color.green()
    assert approx(green.r, 0.0) and approx(green.g, 1.0) and approx(green.b, 0.0) and approx(green.a, 1.0), \
        f"green() should be (0,1,0,1), got {green}"

    # Factory: blue
    blue = Color.blue()
    assert approx(blue.r, 0.0) and approx(blue.g, 0.0) and approx(blue.b, 1.0) and approx(blue.a, 1.0), \
        f"blue() should be (0,0,1,1), got {blue}"

    # Factory: rgb
    c = Color.rgb(0.2, 0.4, 0.6)
    assert approx(c.r, 0.2) and approx(c.g, 0.4) and approx(c.b, 0.6) and approx(c.a, 1.0), \
        f"rgb() should set alpha to 1.0, got {c}"

    # Factory: rgba
    c = Color.rgba(0.1, 0.2, 0.3, 0.4)
    assert approx(c.r, 0.1) and approx(c.g, 0.2) and approx(c.b, 0.3) and approx(c.a, 0.4), \
        f"rgba() failed: {c}"

    # Factory: from_hex (extracts R, G, B from 24-bit integer; alpha is always 1.0)
    c = Color.from_hex(0xFF0000)
    assert approx(c.r, 1.0) and approx(c.g, 0.0) and approx(c.b, 0.0) and approx(c.a, 1.0), \
        f"from_hex(0xFF0000) should be red with full alpha, got {c}"

    c = Color.from_hex(0x00FF00)
    assert approx(c.r, 0.0) and approx(c.g, 1.0) and approx(c.b, 0.0), \
        f"from_hex(0x00FF00) should be green, got {c}"

    c = Color.from_hex(0x0000FF)
    assert approx(c.r, 0.0) and approx(c.g, 0.0) and approx(c.b, 1.0), \
        f"from_hex(0x0000FF) should be blue, got {c}"

    # with_alpha
    base = Color.red()
    semi = base.with_alpha(0.5)
    assert approx(semi.r, 1.0) and approx(semi.g, 0.0) and approx(semi.b, 0.0) and approx(semi.a, 0.5), \
        f"with_alpha(0.5) on red should give (1,0,0,0.5), got {semi}"
    # Original should be unchanged
    assert approx(base.a, 1.0), "with_alpha should not mutate the original"

    print("  Color tests passed")
    return True


def test_rect():
    """Test Rect creation, contains, and intersects."""
    print("Testing Rect...")

    # Construction
    r = Rect(10, 20, 100, 50)
    assert r.x == 10, f"Expected x=10, got {r.x}"
    assert r.y == 20, f"Expected y=20, got {r.y}"
    assert r.width == 100, f"Expected width=100, got {r.width}"
    assert r.height == 50, f"Expected height=50, got {r.height}"

    # Default construction
    r_default = Rect()
    assert r_default.x == 0.0 and r_default.y == 0.0 and r_default.width == 0.0 and r_default.height == 0.0, \
        "Default Rect should be (0, 0, 0, 0)"

    # contains: point inside
    assert r.contains(Vec2(50, 40)), "Point (50,40) should be inside Rect(10,20,100,50)"
    # contains: point on left edge (inclusive)
    assert r.contains(Vec2(10, 45)), "Left edge point should be inside"
    # contains: point on top edge (inclusive)
    assert r.contains(Vec2(50, 20)), "Top edge point should be inside"
    # contains: point outside left
    assert not r.contains(Vec2(9, 40)), "Point (9,40) should be outside left edge"
    # contains: point outside right
    assert not r.contains(Vec2(111, 45)), "Point (111,45) should be outside right edge"
    # contains: point above rect
    assert not r.contains(Vec2(50, 19)), "Point above rect should be outside"
    # contains: origin, clearly outside
    assert not r.contains(Vec2(0, 0)), "Origin should be outside Rect(10,20,...)"

    # intersects: overlapping rects
    r2 = Rect(50, 40, 100, 50)
    assert r.intersects(r2), "Overlapping rects should intersect"

    # intersects: adjacent rects touching at edge boundary (strict less-than)
    r3 = Rect(110, 20, 100, 50)  # Starts at right edge of r (10+100=110), no pixel overlap
    assert not r.intersects(r3), "Adjacent (touching) rects should not intersect"

    # intersects: fully separated
    r4 = Rect(200, 200, 50, 50)
    assert not r.intersects(r4), "Non-overlapping rects should not intersect"

    # intersects: one rect contained within another
    r5 = Rect(0, 0, 500, 500)
    r6 = Rect(50, 50, 10, 10)
    assert r5.intersects(r6), "Contained rect should intersect its container"

    print("  Rect tests passed")
    return True


def test_transform2d():
    """Test Transform2D flat fields and construction.

    Factory methods (default, from_position, etc.) are now FFI-backed and
    require the native library. They are tested separately when the library
    is available.
    """
    print("Testing Transform2D...")

    # Flat field construction
    t = Transform2D(position_x=10.0, position_y=20.0, rotation=0.5, scale_x=2.0, scale_y=3.0)
    assert t.position_x == 10.0, f"Expected position_x=10.0, got {t.position_x}"
    assert t.position_y == 20.0, f"Expected position_y=20.0, got {t.position_y}"
    assert t.rotation == 0.5, f"Expected rotation=0.5, got {t.rotation}"
    assert t.scale_x == 2.0, f"Expected scale_x=2.0, got {t.scale_x}"
    assert t.scale_y == 3.0, f"Expected scale_y=3.0, got {t.scale_y}"

    # Direct field mutation
    t.position_x = 99.0
    assert t.position_x == 99.0, "Direct field assignment to position_x should work"
    t.rotation = math.pi
    assert abs(t.rotation - math.pi) < 0.001, "Direct field assignment to rotation should work"

    # Factory methods are FFI-backed; skip if native library unavailable
    try:
        t = Transform2D.default()
        assert t.position_x == 0.0 and t.position_y == 0.0, \
            f"default() position should be (0,0), got ({t.position_x},{t.position_y})"
        assert t.rotation == 0.0, f"default() rotation should be 0.0, got {t.rotation}"
        assert t.scale_x == 1.0 and t.scale_y == 1.0, \
            f"default() scale should be (1,1), got ({t.scale_x},{t.scale_y})"

        t = Transform2D.from_position(100.0, 50.0)
        assert t.position_x == 100.0, f"from_position() x failed: {t.position_x}"
        assert t.position_y == 50.0, f"from_position() y failed: {t.position_y}"
        assert t.rotation == 0.0, "from_position() should set rotation to 0"
        assert t.scale_x == 1.0 and t.scale_y == 1.0, "from_position() should set scale to (1,1)"

        t = Transform2D.from_rotation(math.pi / 2)
        assert abs(t.rotation - math.pi / 2) < 0.001, \
            f"from_rotation(pi/2) failed: {t.rotation}"
        assert t.position_x == 0.0 and t.position_y == 0.0, \
            "from_rotation() should set position to (0,0)"
        assert t.scale_x == 1.0 and t.scale_y == 1.0, \
            "from_rotation() should set scale to (1,1)"

        t = Transform2D.from_scale(3.0, 4.0)
        assert t.scale_x == 3.0, f"from_scale() x failed: {t.scale_x}"
        assert t.scale_y == 4.0, f"from_scale() y failed: {t.scale_y}"
        assert t.position_x == 0.0 and t.position_y == 0.0, \
            "from_scale() should set position to (0,0)"
        assert t.rotation == 0.0, "from_scale() should set rotation to 0"
    except ImportError:
        print("    (skipped FFI-backed factories: native library not available)")

    print("  Transform2D tests passed")
    return True


def test_sprite():
    """Test Sprite creation and flat field access."""
    print("Testing Sprite...")

    # Construction with defaults
    s = Sprite()
    assert s.texture_handle == 0, f"Default texture_handle should be 0, got {s.texture_handle}"
    assert s.flip_x == False, f"Default flip_x should be False, got {s.flip_x}"
    assert s.flip_y == False, f"Default flip_y should be False, got {s.flip_y}"
    assert s.anchor_x == 0.0, f"Default anchor_x should be 0.0, got {s.anchor_x}"
    assert s.anchor_y == 0.0, f"Default anchor_y should be 0.0, got {s.anchor_y}"

    # Construction with texture handle
    s = Sprite(texture_handle=42)
    assert s.texture_handle == 42, f"Expected texture_handle=42, got {s.texture_handle}"

    # Construction with all flat fields
    s = Sprite(texture_handle=7, flip_x=True, flip_y=False, anchor_x=0.5, anchor_y=1.0)
    assert s.texture_handle == 7, f"Expected texture_handle=7, got {s.texture_handle}"
    assert s.flip_x == True, f"Expected flip_x=True, got {s.flip_x}"
    assert s.flip_y == False, f"Expected flip_y=False, got {s.flip_y}"
    assert s.anchor_x == 0.5, f"Expected anchor_x=0.5, got {s.anchor_x}"
    assert s.anchor_y == 1.0, f"Expected anchor_y=1.0, got {s.anchor_y}"

    # Mutable field assignment
    s = Sprite(texture_handle=10)
    s.flip_x = True
    assert s.flip_x == True, "flip_x field assignment should work"
    s.flip_y = True
    assert s.flip_y == True, "flip_y field assignment should work"
    s.anchor_x = 0.25
    assert s.anchor_x == 0.25, "anchor_x field assignment should work"
    s.anchor_y = 0.75
    assert s.anchor_y == 0.75, "anchor_y field assignment should work"
    s.texture_handle = 99
    assert s.texture_handle == 99, "texture_handle field assignment should work"

    print("  Sprite tests passed")
    return True


def test_entity():
    """Test Entity bits encoding, index, generation, is_placeholder, and to_bits."""
    print("Testing Entity...")

    # Entity encodes index in lower 32 bits, generation in upper 32 bits
    index_val = 5
    generation_val = 2
    bits = (generation_val << 32) | index_val
    e = Entity(bits)

    assert e.index == index_val, f"Expected index={index_val}, got {e.index}"
    assert e.generation == generation_val, f"Expected generation={generation_val}, got {e.generation}"
    assert e.is_placeholder == False, "Non-sentinel entity should not be a placeholder"
    assert e.to_bits() == bits, f"to_bits() should return original bits, got {e.to_bits()}"

    # Entity with index=0, generation=0
    e_zero = Entity(0)
    assert e_zero.index == 0, "Entity(0).index should be 0"
    assert e_zero.generation == 0, "Entity(0).generation should be 0"
    assert e_zero.is_placeholder == False, "Entity(0) is not the placeholder sentinel"

    # Placeholder sentinel: all 64 bits set
    PLACEHOLDER_BITS = 0xFFFFFFFFFFFFFFFF
    e_placeholder = Entity(PLACEHOLDER_BITS)
    assert e_placeholder.is_placeholder == True, \
        f"Entity(0xFFFFFFFFFFFFFFFF) should be a placeholder, got {e_placeholder.is_placeholder}"
    assert e_placeholder.to_bits() == PLACEHOLDER_BITS, \
        "to_bits() on placeholder should return 0xFFFFFFFFFFFFFFFF"

    # Large generation value
    e_large = Entity((1000 << 32) | 999)
    assert e_large.index == 999, f"Expected index=999, got {e_large.index}"
    assert e_large.generation == 1000, f"Expected generation=1000, got {e_large.generation}"

    print("  Entity tests passed")
    return True


def test_enums():
    """Test Key and MouseButton enum constant values."""
    print("Testing enums...")

    # Key constants matching GLFW values
    assert Key.ESCAPE == 256, f"Key.ESCAPE should be 256, got {Key.ESCAPE}"
    assert Key.SPACE == 32, f"Key.SPACE should be 32, got {Key.SPACE}"
    assert Key.W == 87, f"Key.W should be 87, got {Key.W}"
    assert Key.A == 65, f"Key.A should be 65, got {Key.A}"
    assert Key.S == 83, f"Key.S should be 83, got {Key.S}"
    assert Key.D == 68, f"Key.D should be 68, got {Key.D}"
    assert Key.ENTER == 257, f"Key.ENTER should be 257, got {Key.ENTER}"
    assert Key.LEFT == 263, f"Key.LEFT should be 263, got {Key.LEFT}"
    assert Key.RIGHT == 262, f"Key.RIGHT should be 262, got {Key.RIGHT}"
    assert Key.UP == 265, f"Key.UP should be 265, got {Key.UP}"
    assert Key.DOWN == 264, f"Key.DOWN should be 264, got {Key.DOWN}"

    # MouseButton constants
    assert MouseButton.LEFT == 0, f"MouseButton.LEFT should be 0, got {MouseButton.LEFT}"
    assert MouseButton.RIGHT == 1, f"MouseButton.RIGHT should be 1, got {MouseButton.RIGHT}"
    assert MouseButton.MIDDLE == 2, f"MouseButton.MIDDLE should be 2, got {MouseButton.MIDDLE}"

    print("  Enum tests passed")
    return True


def test_errors():
    """Test GoudError hierarchy: imports, attributes, subclassing, and category mapping."""
    print("Testing errors module...")

    errors_mod = _load_module("errors", _ERRORS_PATH)

    # Verify all public classes can be imported
    GoudError = errors_mod.GoudError
    GoudContextError = errors_mod.GoudContextError
    GoudResourceError = errors_mod.GoudResourceError
    GoudGraphicsError = errors_mod.GoudGraphicsError
    GoudEntityError = errors_mod.GoudEntityError
    GoudInputError = errors_mod.GoudInputError
    GoudSystemError = errors_mod.GoudSystemError
    GoudProviderError = errors_mod.GoudProviderError
    GoudInternalError = errors_mod.GoudInternalError
    RecoveryClass = errors_mod.RecoveryClass
    _category_from_code = errors_mod._category_from_code
    _CATEGORY_CLASS_MAP = errors_mod._CATEGORY_CLASS_MAP

    assert GoudError is not None, "GoudError failed to import"
    assert GoudContextError is not None, "GoudContextError failed to import"
    assert GoudResourceError is not None, "GoudResourceError failed to import"
    assert GoudGraphicsError is not None, "GoudGraphicsError failed to import"
    assert GoudEntityError is not None, "GoudEntityError failed to import"
    assert GoudInputError is not None, "GoudInputError failed to import"
    assert GoudSystemError is not None, "GoudSystemError failed to import"
    assert GoudProviderError is not None, "GoudProviderError failed to import"
    assert GoudInternalError is not None, "GoudInternalError failed to import"
    assert RecoveryClass is not None, "RecoveryClass failed to import"

    # Verify GoudError has expected attributes when constructed
    err = GoudError(
        error_code=1,
        message="context not initialised",
        category="Context",
        subsystem="engine",
        operation="init",
        recovery=RecoveryClass.FATAL,
        recovery_hint="Call the initialization function first",
    )
    assert err.error_code == 1, f"Expected error_code=1, got {err.error_code}"
    assert err.category == "Context", f"Expected category='Context', got {err.category!r}"
    assert err.subsystem == "engine", f"Expected subsystem='engine', got {err.subsystem!r}"
    assert err.operation == "init", f"Expected operation='init', got {err.operation!r}"
    assert err.recovery == RecoveryClass.FATAL, f"Expected recovery=FATAL, got {err.recovery}"
    assert err.recovery_hint == "Call the initialization function first", \
        f"recovery_hint mismatch: {err.recovery_hint!r}"
    assert str(err) == "context not initialised", f"str(err) mismatch: {str(err)!r}"

    # Verify RecoveryClass constants
    assert RecoveryClass.RECOVERABLE == 0, "RECOVERABLE should be 0"
    assert RecoveryClass.FATAL == 1, "FATAL should be 1"
    assert RecoveryClass.DEGRADED == 2, "DEGRADED should be 2"

    # Verify each subclass is a subclass of GoudError (isinstance check)
    ctx_err = GoudContextError(
        error_code=1, message="ctx", category="Context",
        subsystem="", operation="", recovery=0, recovery_hint="",
    )
    assert isinstance(ctx_err, GoudError), "GoudContextError should be instance of GoudError"
    assert isinstance(ctx_err, GoudContextError), "GoudContextError instance check failed"

    res_err = GoudResourceError(
        error_code=100, message="res", category="Resource",
        subsystem="", operation="", recovery=0, recovery_hint="",
    )
    assert isinstance(res_err, GoudError), "GoudResourceError should be instance of GoudError"

    gfx_err = GoudGraphicsError(
        error_code=200, message="gfx", category="Graphics",
        subsystem="", operation="", recovery=0, recovery_hint="",
    )
    assert isinstance(gfx_err, GoudError), "GoudGraphicsError should be instance of GoudError"

    ent_err = GoudEntityError(
        error_code=300, message="ent", category="Entity",
        subsystem="", operation="", recovery=0, recovery_hint="",
    )
    assert isinstance(ent_err, GoudError), "GoudEntityError should be instance of GoudError"

    inp_err = GoudInputError(
        error_code=400, message="inp", category="Input",
        subsystem="", operation="", recovery=0, recovery_hint="",
    )
    assert isinstance(inp_err, GoudError), "GoudInputError should be instance of GoudError"

    sys_err = GoudSystemError(
        error_code=500, message="sys", category="System",
        subsystem="", operation="", recovery=0, recovery_hint="",
    )
    assert isinstance(sys_err, GoudError), "GoudSystemError should be instance of GoudError"

    prv_err = GoudProviderError(
        error_code=600, message="prv", category="Provider",
        subsystem="", operation="", recovery=0, recovery_hint="",
    )
    assert isinstance(prv_err, GoudError), "GoudProviderError should be instance of GoudError"

    int_err = GoudInternalError(
        error_code=900, message="int", category="Internal",
        subsystem="", operation="", recovery=0, recovery_hint="",
    )
    assert isinstance(int_err, GoudError), "GoudInternalError should be instance of GoudError"

    # Verify each subclass maps to its expected category name
    assert ctx_err.category == "Context", f"GoudContextError category should be 'Context'"
    assert res_err.category == "Resource", f"GoudResourceError category should be 'Resource'"
    assert gfx_err.category == "Graphics", f"GoudGraphicsError category should be 'Graphics'"
    assert ent_err.category == "Entity", f"GoudEntityError category should be 'Entity'"
    assert inp_err.category == "Input", f"GoudInputError category should be 'Input'"
    assert sys_err.category == "System", f"GoudSystemError category should be 'System'"
    assert prv_err.category == "Provider", f"GoudProviderError category should be 'Provider'"
    assert int_err.category == "Internal", f"GoudInternalError category should be 'Internal'"

    # Verify errors module is also exported from the package __init__
    # We must avoid triggering FFI load, so load __init__ indirectly via importlib
    init_path = Path(__file__).parent / "goud_engine" / "__init__.py"
    init_src = init_path.read_text()
    assert (
        "from .errors import" in init_src
        or "from .generated._errors import" in init_src
    ), "__init__.py should re-export error classes from the package errors module"

    # Test dispatch path: _category_from_code maps error codes to category strings
    assert _category_from_code(1) == "Context", \
        f"_category_from_code(1) should return 'Context', got {_category_from_code(1)!r}"
    assert _category_from_code(100) == "Resource", \
        f"_category_from_code(100) should return 'Resource', got {_category_from_code(100)!r}"
    assert _category_from_code(200) == "Graphics", \
        f"_category_from_code(200) should return 'Graphics', got {_category_from_code(200)!r}"
    assert _category_from_code(300) == "Entity", \
        f"_category_from_code(300) should return 'Entity', got {_category_from_code(300)!r}"
    assert _category_from_code(400) == "Input", \
        f"_category_from_code(400) should return 'Input', got {_category_from_code(400)!r}"
    assert _category_from_code(500) == "System", \
        f"_category_from_code(500) should return 'System', got {_category_from_code(500)!r}"
    assert _category_from_code(600) == "Provider", \
        f"_category_from_code(600) should return 'Provider', got {_category_from_code(600)!r}"
    assert _category_from_code(900) == "Internal", \
        f"_category_from_code(900) should return 'Internal', got {_category_from_code(900)!r}"

    # Test dispatch path: _CATEGORY_CLASS_MAP maps category strings to exception classes
    assert _CATEGORY_CLASS_MAP["Context"] is GoudContextError, \
        f"_CATEGORY_CLASS_MAP['Context'] should map to GoudContextError"
    assert _CATEGORY_CLASS_MAP["Resource"] is GoudResourceError, \
        f"_CATEGORY_CLASS_MAP['Resource'] should map to GoudResourceError"
    assert _CATEGORY_CLASS_MAP["Graphics"] is GoudGraphicsError, \
        f"_CATEGORY_CLASS_MAP['Graphics'] should map to GoudGraphicsError"
    assert _CATEGORY_CLASS_MAP["Entity"] is GoudEntityError, \
        f"_CATEGORY_CLASS_MAP['Entity'] should map to GoudEntityError"
    assert _CATEGORY_CLASS_MAP["Input"] is GoudInputError, \
        f"_CATEGORY_CLASS_MAP['Input'] should map to GoudInputError"
    assert _CATEGORY_CLASS_MAP["System"] is GoudSystemError, \
        f"_CATEGORY_CLASS_MAP['System'] should map to GoudSystemError"
    assert _CATEGORY_CLASS_MAP["Provider"] is GoudProviderError, \
        f"_CATEGORY_CLASS_MAP['Provider'] should map to GoudProviderError"
    assert _CATEGORY_CLASS_MAP["Internal"] is GoudInternalError, \
        f"_CATEGORY_CLASS_MAP['Internal'] should map to GoudInternalError"

    print("  Error tests passed")
    return True


def main():
    """Run all tests."""
    print("=" * 60)
    print(" GoudEngine Python SDK Tests")
    print("=" * 60)

    tests = [
        test_imports,
        test_generated_scene_wrapper_api_names,
        test_generated_audio_wrapper_api_names,
        test_generated_audio_activate_maps_to_activate_ffi,
        test_generated_network_wrapper_api_names,
        test_handwritten_network_wrapper_exports,
        test_handwritten_network_wrapper_send_contract_source,
        test_generated_ui_manager_wrapper_api_names,
        test_generated_ui_style_color_contract,
        test_generated_ui_style_string_contract,
        test_vec2,
        test_color,
        test_rect,
        test_transform2d,
        test_sprite,
        test_entity,
        test_enums,
        test_errors,
    ]

    passed = 0
    failed = 0

    for test in tests:
        try:
            if test():
                passed += 1
            else:
                failed += 1
        except Exception as e:
            print(f"  {test.__name__} failed with exception: {e}")
            import traceback
            traceback.print_exc()
            failed += 1

    print("\n" + "=" * 60)
    print(f" Results: {passed} passed, {failed} failed")
    print("=" * 60)

    return 0 if failed == 0 else 1


if __name__ == "__main__":
    sys.exit(main())
