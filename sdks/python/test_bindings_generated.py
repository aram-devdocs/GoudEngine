#!/usr/bin/env python3
"""Generated-wrapper binding tests for the Python SDK."""

from test_bindings_common import (
    Color,
    Entity,
    Key,
    MouseButton,
    Rect,
    Sprite,
    Transform2D,
    UiStyle,
    Vec2,
    _GENERATED_DIR,
)


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

    game_path = _GENERATED_DIR / "_game.py"
    assert game_path.exists(), f"GoudGame source not found at {game_path}"

    print("  All imports successful")
    return True


def test_generated_scene_wrapper_api_names():
    """Validate generated scene wrapper method names without loading native lib."""
    print("Testing generated scene wrapper API names...")

    game_src = (_GENERATED_DIR / "_game.py").read_text()

    assert "def load_scene(self, name, json):" in game_src, "missing load_scene wrapper"
    assert "def unload_scene(self, name):" in game_src, "missing unload_scene wrapper"
    assert "def set_active_scene(self, scene_id, active):" in game_src, "missing set_active_scene wrapper"

    assert "def scene_create(self, name):" in game_src, "missing legacy scene_create API"
    assert "def scene_destroy(self, scene_id):" in game_src, "missing legacy scene_destroy API"
    assert "def scene_set_current(self, scene_id):" in game_src, "missing legacy scene_set_current API"

    print("  Scene wrapper API name tests passed")
    return True


def test_generated_audio_wrapper_api_names():
    """Validate generated audio wrapper API names without loading native lib."""
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
    types_src = (_GENERATED_DIR / "_types.py").read_text()
    assert "class NetworkConnectResult:" in types_src
    assert "class NetworkPacket:" in types_src
    assert "def __init__(self, peer_id: int = 0, data: bytes = b''):" in types_src, \
        "NetworkPacket.data should generate as bytes-compatible storage"
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
