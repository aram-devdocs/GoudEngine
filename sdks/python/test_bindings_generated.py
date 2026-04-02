#!/usr/bin/env python3
"""Generated-wrapper binding tests for the Python SDK."""

import ctypes

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
    _PACKAGE_DIR,
    _load_module,
    _new_fake_generated_package,
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
    assert "self._raise_network_error_or_runtime(f'goud_network_connect_with_peer failed with status {_status}')" in game_src
    assert "return NetworkConnectResult(_handle.value, _peer_id.value)" in game_src
    assert "_status = self._lib.goud_network_receive(self._ctx, handle," in game_src
    assert "self._raise_network_error_or_runtime(f'goud_network_receive failed with status {_status}')" in game_src
    assert (
        "return NetworkPacket(_out_peer_id.value, bytes(_out_buf[:_status]))" in game_src
        or (
            "_data = bytes(_out_buf[:_status])" in game_src
            and "return NetworkPacket(_out_peer_id.value, _data)" in game_src
        )
    )
    assert "_status = self._lib.goud_network_get_stats_v2(self._ctx, handle, ctypes.byref(_stats))" in game_src
    assert "self._raise_network_error_or_runtime(f'goud_network_get_stats_v2 failed with status {_status}')" in game_src
    assert "return NetworkStats(" in game_src
    assert "_config_ffi = _ffi_module.NetworkSimulationConfig()" in game_src
    assert "return self._lib.goud_network_set_simulation(self._ctx, handle, _config_ffi)" in game_src
    assert "def _raise_network_error_or_runtime(self, message):" in game_src
    assert "from ._errors import GoudError" in game_src
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
    assert "_lib.goud_network_set_simulation.argtypes = [GoudContextId, ctypes.c_int64, NetworkSimulationConfig]" in ffi_src
    assert "_lib.goud_network_clear_simulation.argtypes = [GoudContextId, ctypes.c_int64]" in ffi_src

    print("  Network wrapper API name tests passed")
    return True


def test_generated_provider_capability_imports():
    """Validate _game.py imports the capability symbols used by provider helpers."""
    print("Testing generated provider capability imports...")

    game_src = (_GENERATED_DIR / "_game.py").read_text()

    expected_ffi_symbols = [
        "RenderCapabilities",
        "PhysicsCapabilities",
        "AudioCapabilities",
        "InputCapabilities",
        "NetworkCapabilities",
    ]
    for symbol in expected_ffi_symbols:
        assert symbol in game_src, f"missing capability FFI import or usage for {symbol}"

    expected_value_types = [
        "RenderCapabilities",
        "PhysicsCapabilities",
        "AudioCapabilities",
        "InputCapabilities",
        "NetworkCapabilities",
    ]
    for symbol in expected_value_types:
        assert symbol in game_src, f"missing capability value type import or usage for {symbol}"

    expected_helpers = [
        "def get_render_capabilities(self):",
        "def get_physics_capabilities(self):",
        "def get_audio_capabilities(self):",
        "def get_input_capabilities(self):",
        "def get_network_capabilities(self):",
    ]
    for signature in expected_helpers:
        assert signature in game_src, f"missing provider capability helper {signature}"

    print("  Provider capability import tests passed")
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


def test_generated_debugger_wrapper_api_names():
    """Validate generated debugger wrapper methods and helper exports."""
    print("Testing generated debugger wrapper API names...")

    game_src = (_GENERATED_DIR / "_game.py").read_text()
    ffi_src = (_GENERATED_DIR / "_ffi.py").read_text()
    root_init_src = (_GENERATED_DIR.parent / "__init__.py").read_text()

    expected_methods = [
        "def get_debugger_snapshot_json(self):",
        "def get_debugger_manifest_json(self):",
        "def set_debugger_profiling_enabled(self, enabled):",
        "def set_debugger_selected_entity(self, entity_id):",
        "def clear_debugger_selected_entity(self):",
        "def get_memory_summary(self):",
        "def set_debugger(self, debugger):",
    ]
    for signature in expected_methods:
        assert signature in game_src, f"missing generated debugger wrapper: {signature}"

    assert "_lib.goud_debugger_get_snapshot_json.argtypes = [GoudContextId, ctypes.POINTER(ctypes.c_uint8), ctypes.c_size_t]" in ffi_src
    assert "_lib.goud_debugger_get_manifest_json.argtypes = [ctypes.POINTER(ctypes.c_uint8), ctypes.c_size_t]" in ffi_src
    assert "_lib.goud_debugger_get_memory_summary.argtypes = [GoudContextId, ctypes.POINTER(GoudMemorySummary)]" in ffi_src
    assert "_lib.goud_engine_config_set_debugger.argtypes = [ctypes.c_void_p, ctypes.POINTER(GoudDebuggerConfig)]" in ffi_src
    assert '("route_label", ctypes.c_char_p)' in ffi_src
    assert "def __init__(self, config: 'ContextConfig' = None):" in game_src
    assert "self._ctx = lib.goud_context_create_with_config(ctypes.byref(_config_ffi))" in game_src
    assert "from .debugger import parse_debugger_manifest, parse_debugger_snapshot" in root_init_src

    print("  Debugger wrapper API name tests passed")
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


def test_generated_new_api_names():
    """Validate API names for fixed timestep, text batch, atlas, spatial grid, and generic ECS query."""
    print("Testing generated new API names (PRs #601-605)...")

    game_src = (_GENERATED_DIR / "_game.py").read_text()
    ffi_src = (_GENERATED_DIR / "_ffi.py").read_text()
    types_src = (_GENERATED_DIR / "_types.py").read_text()

    # Fixed timestep (PR #601)
    assert "def interpolation_alpha(self) -> float:" in game_src, "missing interpolation_alpha property"
    assert "def run_with_fixed_update(self, fixed_update, update):" in game_src, "missing run_with_fixed_update wrapper"
    assert "goud_fixed_timestep_begin" in game_src, "run_with_fixed_update should call goud_fixed_timestep_begin"
    assert "goud_fixed_timestep_step" in game_src, "run_with_fixed_update should call goud_fixed_timestep_step"
    assert "goud_fixed_timestep_dt" in game_src, "run_with_fixed_update should call goud_fixed_timestep_dt"
    assert "_lib.goud_fixed_timestep_begin.argtypes" in ffi_src, "missing goud_fixed_timestep_begin argtypes"
    assert "_lib.goud_fixed_timestep_step.argtypes" in ffi_src, "missing goud_fixed_timestep_step argtypes"
    assert "_lib.goud_fixed_timestep_alpha.argtypes" in ffi_src, "missing goud_fixed_timestep_alpha argtypes"
    assert "_lib.goud_fixed_timestep_dt.argtypes" in ffi_src, "missing goud_fixed_timestep_dt argtypes"
    assert "_lib.goud_fixed_timestep_set.argtypes" in ffi_src, "missing goud_fixed_timestep_set argtypes"

    # Text batch (PR #604)
    assert "def draw_text_batch(self, cmds):" in game_src, "missing draw_text_batch wrapper"
    assert "goud_renderer_draw_text_batch" in game_src, "draw_text_batch should call goud_renderer_draw_text_batch"
    assert "_lib.goud_renderer_draw_text_batch.argtypes" in ffi_src, "missing goud_renderer_draw_text_batch argtypes"
    assert "class FfiTextCmd(ctypes.Structure):" in ffi_src, "missing FfiTextCmd struct in _ffi.py"

    # Texture atlas (PR #602)
    assert "_lib.goud_atlas_create.argtypes" in ffi_src, "missing goud_atlas_create argtypes"
    assert "_lib.goud_atlas_add_from_file.argtypes" in ffi_src, "missing goud_atlas_add_from_file argtypes"
    assert "_lib.goud_atlas_finalize.argtypes" in ffi_src, "missing goud_atlas_finalize argtypes"
    assert "class FfiAtlasEntry(ctypes.Structure):" in ffi_src, "missing FfiAtlasEntry struct in _ffi.py"
    assert "class AtlasEntry:" in types_src, "missing AtlasEntry in _types.py"
    assert "class AtlasStats:" in types_src, "missing AtlasStats in _types.py"

    # Spatial grid (PR #603)
    assert "_lib.goud_spatial_grid_create.argtypes" in ffi_src, "missing goud_spatial_grid_create argtypes"
    assert "_lib.goud_spatial_grid_destroy.argtypes" in ffi_src, "missing goud_spatial_grid_destroy argtypes"
    assert "_lib.goud_spatial_grid_insert.argtypes" in ffi_src, "missing goud_spatial_grid_insert argtypes"

    # Spatial hash - AABB broad-phase (#642)
    assert "_lib.goud_spatial_hash_create.argtypes" in ffi_src, "missing goud_spatial_hash_create argtypes"
    assert "_lib.goud_spatial_hash_destroy.argtypes" in ffi_src, "missing goud_spatial_hash_destroy argtypes"
    assert "_lib.goud_spatial_hash_insert.argtypes" in ffi_src, "missing goud_spatial_hash_insert argtypes"
    assert "_lib.goud_spatial_hash_query_range.argtypes" in ffi_src, "missing goud_spatial_hash_query_range argtypes"
    assert "_lib.goud_spatial_hash_query_rect.argtypes" in ffi_src, "missing goud_spatial_hash_query_rect argtypes"

    # Generic ECS query (PR #605)
    assert "def component_count(self, type_id_hash):" in game_src, "missing component_count wrapper"
    assert "def component_get_entities(self, type_id_hash, out_entities, max_count):" in game_src, "missing component_get_entities wrapper"
    assert "def component_get_all(self, type_id_hash, out_entities, out_data_ptrs, max_count):" in game_src, "missing component_get_all wrapper"
    assert "_lib.goud_component_count.argtypes" in ffi_src, "missing goud_component_count argtypes"
    assert "_lib.goud_component_get_entities.argtypes" in ffi_src, "missing goud_component_get_entities argtypes"
    assert "_lib.goud_component_get_all.argtypes" in ffi_src, "missing goud_component_get_all argtypes"

    print("  New API name tests passed")
    return True


def test_phase0_ffi_surface():
    """Verify all Phase 0 feature FFI function declarations and struct layouts exist."""
    print("Testing Phase 0 FFI surface (SpatialGrid, Atlas, Batch, Timestep, Metrics)...")

    ffi_src = (_GENERATED_DIR / "_ffi.py").read_text()

    # -- SpatialGrid API --
    spatial_fns = [
        "goud_spatial_grid_create",
        "goud_spatial_grid_create_with_capacity",
        "goud_spatial_grid_destroy",
        "goud_spatial_grid_clear",
        "goud_spatial_grid_insert",
        "goud_spatial_grid_remove",
        "goud_spatial_grid_update",
        "goud_spatial_grid_query_radius",
        "goud_spatial_grid_entity_count",
    ]

    # -- Atlas API --
    atlas_fns = [
        "goud_atlas_create",
        "goud_atlas_add_from_file",
        "goud_atlas_add_pixels",
        "goud_atlas_finalize",
        "goud_atlas_get_entry",
        "goud_atlas_get_stats",
        "goud_atlas_get_texture",
        "goud_atlas_destroy",
    ]

    # -- Batch API --
    batch_fns = [
        "goud_renderer_draw_sprite_batch",
        "goud_renderer_draw_text_batch",
    ]

    # -- Fixed Timestep API --
    timestep_fns = [
        "goud_fixed_timestep_begin",
        "goud_fixed_timestep_step",
        "goud_fixed_timestep_alpha",
        "goud_fixed_timestep_dt",
        "goud_fixed_timestep_set",
        "goud_fixed_timestep_set_max_steps",
    ]

    # -- Render Metrics API --
    metrics_fns = [
        "goud_renderer_get_frame_metrics",
    ]

    # -- SpatialHash API (AABB broad-phase, #642) --
    spatial_hash_fns = [
        "goud_spatial_hash_create",
        "goud_spatial_hash_create_with_capacity",
        "goud_spatial_hash_destroy",
        "goud_spatial_hash_clear",
        "goud_spatial_hash_insert",
        "goud_spatial_hash_remove",
        "goud_spatial_hash_update",
        "goud_spatial_hash_query_range",
        "goud_spatial_hash_query_rect",
        "goud_spatial_hash_entity_count",
    ]

    all_fns = spatial_fns + spatial_hash_fns + atlas_fns + batch_fns + timestep_fns + metrics_fns
    missing = []
    for fn_name in all_fns:
        key = f"_lib.{fn_name}.argtypes"
        if key not in ffi_src:
            missing.append(fn_name)
    assert not missing, f"Missing Phase 0 FFI function declarations: {missing}"

    # -- Struct layout: FfiRenderMetrics (13 fields) --
    expected_render_metrics_fields = [
        ("draw_call_count", "ctypes.c_uint32"),
        ("sprites_submitted", "ctypes.c_uint32"),
        ("sprites_drawn", "ctypes.c_uint32"),
        ("sprites_culled", "ctypes.c_uint32"),
        ("batches_submitted", "ctypes.c_uint32"),
        ("avg_sprites_per_batch", "ctypes.c_float"),
        ("sprite_render_ms", "ctypes.c_float"),
        ("text_render_ms", "ctypes.c_float"),
        ("ui_render_ms", "ctypes.c_float"),
        ("total_render_ms", "ctypes.c_float"),
        ("text_draw_calls", "ctypes.c_uint32"),
        ("text_glyph_count", "ctypes.c_uint32"),
        ("ui_draw_calls", "ctypes.c_uint32"),
    ]
    assert "class FfiRenderMetrics(ctypes.Structure):" in ffi_src, \
        "FfiRenderMetrics struct not found in _ffi.py"
    for field_name, field_type in expected_render_metrics_fields:
        pattern = f'("{field_name}", {field_type})'
        assert pattern in ffi_src, \
            f"FfiRenderMetrics missing field: {field_name} with type {field_type}"

    # -- Struct existence: FfiSpriteCmd, FfiTextCmd, FfiAtlasEntry, FfiAtlasStats --
    assert "class FfiSpriteCmd(ctypes.Structure):" in ffi_src, \
        "FfiSpriteCmd struct not found in _ffi.py"
    assert "class FfiTextCmd(ctypes.Structure):" in ffi_src, \
        "FfiTextCmd struct not found in _ffi.py"
    assert "class FfiAtlasEntry(ctypes.Structure):" in ffi_src, \
        "FfiAtlasEntry struct not found in _ffi.py"
    assert "class FfiAtlasStats(ctypes.Structure):" in ffi_src, \
        "FfiAtlasStats struct not found in _ffi.py"

    # -- Python-side types: AtlasEntry, AtlasStats --
    types_src = (_GENERATED_DIR / "_types.py").read_text()
    assert "class AtlasEntry:" in types_src, "AtlasEntry not found in _types.py"
    assert "class AtlasStats:" in types_src, "AtlasStats not found in _types.py"

    print("  Phase 0 FFI surface tests passed")
    return True


def test_debugger_helpers():
    """Test the debugger JSON helper functions without requiring the native library."""
    print("Testing debugger helpers...")

    debugger_mod = _load_module("debugger", _PACKAGE_DIR / "debugger.py")

    class _FakeSource:
        def get_debugger_snapshot_json(self):
            return '{"entities": 42}'

        def get_debugger_manifest_json(self):
            return '{"routes": ["a", "b"]}'

    source = _FakeSource()
    snapshot = debugger_mod.parse_debugger_snapshot(source)
    assert snapshot == {"entities": 42}, f"Expected parsed snapshot, got {snapshot}"
    manifest = debugger_mod.parse_debugger_manifest(source)
    assert manifest == {"routes": ["a", "b"]}, f"Expected parsed manifest, got {manifest}"

    print("  Debugger helper tests passed")
    return True


def test_generated_game_runtime_with_fake_lib():
    """Execute generated _game.py wrappers with an isolated fake FFI backend."""
    print("Testing generated _game.py runtime wrappers with fake lib...")

    def _write(ptr, ctype, value):
        ctypes.cast(ptr, ctypes.POINTER(ctype)).contents.value = value

    class _FakeLib:
        def __init__(self):
            self.calls = []

        def _record(self, name, *args):
            self.calls.append((name, *args))
            return 0

        def goud_window_create(self, width, height, title):
            self.calls.append(("goud_window_create", width, height, title))
            return self.ffi.GoudContextId(11)

        def goud_context_create(self):
            return self.ffi.GoudContextId(22)

        def goud_context_create_with_config(self, config_ptr):
            config = ctypes.cast(config_ptr, ctypes.POINTER(self.ffi.GoudContextConfig)).contents
            route_label = config.debugger.route_label.decode("utf-8") if config.debugger.route_label else ""
            self.calls.append((
                "goud_context_create_with_config",
                config.debugger.enabled,
                config.debugger.publish_local_attach,
                route_label,
            ))
            return self.ffi.GoudContextId(23)

        def goud_window_get_size(self, ctx, w_ptr, h_ptr):
            _write(w_ptr, ctypes.c_uint32, 1280)
            _write(h_ptr, ctypes.c_uint32, 720)

        def goud_window_poll_events(self, ctx):
            self.calls.append(("goud_window_poll_events",))
            return 0.016

        def goud_window_should_close(self, ctx):
            return 0

        def goud_input_get_mouse_position(self, ctx, x_ptr, y_ptr):
            _write(x_ptr, ctypes.c_float, 7.5)
            _write(y_ptr, ctypes.c_float, 8.5)

        def goud_input_get_mouse_delta(self, ctx, x_ptr, y_ptr):
            _write(x_ptr, ctypes.c_float, 1.5)
            _write(y_ptr, ctypes.c_float, -2.5)

        def goud_input_get_scroll_delta(self, ctx, x_ptr, y_ptr):
            _write(x_ptr, ctypes.c_float, 0.0)
            _write(y_ptr, ctypes.c_float, -1.0)

        def goud_provider_render_capabilities(self, ctx, ptr):
            out = ctypes.cast(ptr, ctypes.POINTER(self.ffi.RenderCapabilities)).contents
            out.max_texture_units = 16
            out.max_texture_size = 4096
            out.supports_instancing = True
            out.supports_compute = False
            out.supports_msaa = True

        def goud_provider_physics_capabilities(self, ctx, ptr):
            out = ctypes.cast(ptr, ctypes.POINTER(self.ffi.PhysicsCapabilities)).contents
            out.supports_continuous_collision = True
            out.supports_joints = True
            out.max_bodies = 512

        def goud_provider_audio_capabilities(self, ctx, ptr):
            out = ctypes.cast(ptr, ctypes.POINTER(self.ffi.AudioCapabilities)).contents
            out.supports_spatial = True
            out.max_channels = 32

        def goud_provider_input_capabilities(self, ctx, ptr):
            out = ctypes.cast(ptr, ctypes.POINTER(self.ffi.InputCapabilities)).contents
            out.supports_gamepad = True
            out.supports_touch = False
            out.max_gamepads = 4

        def goud_provider_network_capabilities(self, ctx, ptr):
            out = ctypes.cast(ptr, ctypes.POINTER(self.ffi.NetworkCapabilities)).contents
            out.supports_hosting = True
            out.max_connections = 8
            out.max_channels = 4
            out.max_message_size = 64

        def goud_network_connect_with_peer(self, ctx, protocol, address, address_len, port, handle_ptr, peer_ptr):
            _write(handle_ptr, ctypes.c_int64, 901)
            _write(peer_ptr, ctypes.c_uint64, 42)
            return 0

        def goud_network_receive(self, ctx, handle, out_ptr, out_len, peer_ptr):
            payload = b"ok"
            ctypes.memmove(out_ptr, payload, len(payload))
            _write(peer_ptr, ctypes.c_uint64, 77)
            return len(payload)

        def goud_network_get_stats_v2(self, ctx, handle, stats_ptr):
            out = ctypes.cast(stats_ptr, ctypes.POINTER(self.ffi.FfiNetworkStats)).contents
            out.bytes_sent = 1
            out.bytes_received = 2
            out.packets_sent = 3
            out.packets_received = 4
            out.packets_lost = 0
            out.rtt_ms = 5.0
            out.send_bandwidth_bytes_per_sec = 6.0
            out.receive_bandwidth_bytes_per_sec = 7.0
            out.packet_loss_percent = 0.0
            out.jitter_ms = 1.0
            return 0

        def goud_ui_manager_create(self):
            return 1234

        def goud_ui_event_read(self, handle, index, event_ptr):
            out = ctypes.cast(event_ptr, ctypes.POINTER(self.ffi.FfiUiEvent)).contents
            out.event_kind = 2
            out.node_id = 10
            out.previous_node_id = 9
            out.current_node_id = 10
            return 1

        def goud_engine_config_create(self):
            return 555

        def goud_engine_create(self, handle):
            return self.ffi.GoudContextId(33)

        def goud_debugger_get_snapshot_json(self, ctx, out_buf, out_len):
            payload = b'{"route":"ok"}'
            if not out_buf or out_len == 0:
                return -(len(payload) + 1)
            ctypes.memmove(out_buf, payload, len(payload))
            return len(payload)

        def goud_debugger_get_manifest_json(self, out_buf, out_len):
            payload = b'{"routes":[]}'
            if not out_buf or out_len == 0:
                return -(len(payload) + 1)
            ctypes.memmove(out_buf, payload, len(payload))
            return len(payload)

        def goud_debugger_get_memory_summary(self, ctx, summary_ptr):
            summary = ctypes.cast(summary_ptr, ctypes.POINTER(self.ffi.GoudMemorySummary)).contents
            summary.rendering.current_bytes = 11
            summary.rendering.peak_bytes = 22
            summary.total_current_bytes = 11
            summary.total_peak_bytes = 22
            return 0

        def __getattr__(self, name):
            return lambda *args: self._record(name, *args)

    lib = _FakeLib()
    _types_mod, game_mod, ffi_mod = _new_fake_generated_package("_cov_generated_game", lib)
    lib.ffi = ffi_mod

    game = game_mod.GoudGame(320, 200, "Cov")
    assert game.window_width == 1280 and game.window_height == 720
    game.begin_frame()
    assert game.delta_time > 0.0 and game.fps > 0.0
    mouse = game.get_mouse_position()
    assert mouse.x == 7.5 and mouse.y == 8.5
    assert game.get_mouse_delta().y == -2.5 and game.get_scroll_delta().y == -1.0

    assert game.network_host(1, 9001) == 0
    conn = game.network_connect_with_peer(1, "127.0.0.1", 9001)
    assert conn.handle == 901 and conn.peer_id == 42
    assert game.network_receive(901) == b"ok"
    pkt = game.network_receive_packet(901)
    assert pkt.peer_id == 77 and pkt.data == b"ok"
    stats = game.get_network_stats(901)
    assert stats.bytes_sent == 1 and stats.bytes_received == 2
    assert game.network_peer_count(901) == 0
    sim = _types_mod.NetworkSimulationConfig(one_way_latency_ms=5, jitter_ms=1, packet_loss_percent=0.5)
    assert game.set_network_simulation(901, sim) == 0
    assert game.clear_network_simulation(901) == 0
    assert game.set_network_overlay_handle(901) == 0
    assert game.clear_network_overlay_handle() == 0

    rc = game.get_render_capabilities()
    pc = game.get_physics_capabilities()
    ac = game.get_audio_capabilities()
    ic = game.get_input_capabilities()
    nc = game.get_network_capabilities()
    assert rc.max_texture_units == 16 and pc.max_bodies == 512
    assert ac.max_channels == 32 and ic.max_gamepads == 4 and nc.max_message_size == 64

    assert game.audio_play(b"a") == 0
    assert game.audio_play_on_channel(b"a", 2) == 0
    assert game.audio_play_with_settings(b"a", 0.5, 1.0, False, 2) == 0
    assert game.audio_crossfade_to(1, b"a", 0.25, 0) == 0
    assert game.audio_mix_with(1, b"a", 0.2, 0) == 0
    assert game.audio_activate() == 0
    assert game.check_hot_swap_shortcut() is False

    # Exercise additional generated wrappers that are mostly thin pass-throughs.
    assert game.load_texture("assets/a.png") == 0
    game.destroy_texture(1)
    assert game.load_font("assets/a.ttf") == 0
    assert game.destroy_font(1) == 0
    assert game.draw_text(1, "txt", 1.0, 2.0) == 0
    game.draw_sprite(1, 1.0, 2.0, 3.0, 4.0, 0.1, Color.white())
    game.draw_quad(1.0, 2.0, 3.0, 4.0, Color.red())
    game.draw_sprite_rect(1, 1.0, 2.0, 3.0, 4.0, 0.0, 0.0, 0.0, 1.0, 1.0, Color.blue())
    game.set_viewport(0, 0, 320, 200)
    game.enable_depth_test()
    game.disable_depth_test()
    game.clear_depth()
    game.disable_blending()
    _ = game.get_render_stats()
    _ = game.is_key_pressed(1)
    _ = game.is_key_just_pressed(1)
    _ = game.is_key_just_released(1)
    _ = game.is_mouse_button_pressed(0)
    _ = game.is_mouse_button_just_pressed(0)
    _ = game.is_mouse_button_just_released(0)
    fps = game.get_fps_stats()
    assert fps.current_fps == 0.0
    rm = game.get_render_metrics()
    assert rm.draw_call_count == 0
    game.set_fps_overlay_enabled(True)
    game.set_fps_update_interval(0.25)
    _ = game.set_fps_overlay_corner(0)
    assert game.get_debugger_snapshot_json() == '{"route":"ok"}'
    assert game.get_debugger_manifest_json() == '{"routes":[]}'
    game.set_debugger_profiling_enabled(True)
    game.set_debugger_selected_entity(99)
    game.clear_debugger_selected_entity()
    memory = game.get_memory_summary()
    assert memory.rendering.current_bytes == 11 and memory.total_peak_bytes == 22
    _ = game.map_action_key("jump", 32)
    _ = game.is_action_pressed("jump")
    _ = game.is_action_just_pressed("jump")
    _ = game.is_action_just_released("jump")
    _ = game.collision_aabb_aabb(0, 0, 1, 1, 0, 0, 1, 1)
    _ = game.collision_circle_circle(0, 0, 1, 1, 1, 1)
    _ = game.collision_circle_aabb(0, 0, 1, 0, 0, 1, 1)
    _ = game.point_in_rect(0, 0, 0, 0, 1, 1)
    _ = game.point_in_circle(0, 0, 0, 0, 1)
    _ = game.aabb_overlap(0, 0, 1, 1, 0, 0, 1, 1)
    _ = game.circle_overlap(0, 0, 1, 1, 1, 1)
    _ = game.distance(0, 0, 1, 1)
    _ = game.distance_squared(0, 0, 1, 1)
    _ = game.create_cube(1, 1.0, 1.0, 1.0)
    _ = game.create_plane(1, 1.0, 1.0)
    _ = game.create_sphere(1, 1.0)
    _ = game.create_cylinder(1, 1.0, 2.0)
    _ = game.set_object_position(1, 0.0, 1.0, 2.0)
    _ = game.set_object_rotation(1, 0.0, 0.0, 0.0)
    _ = game.set_object_scale(1, 1.0, 1.0, 1.0)
    _ = game.destroy_object(1)
    _ = game.add_light(0, 0, 0, 0, 0, -1, 0, 1, 1, 1, 1, 10, 45)
    _ = game.update_light(1, 0, 0, 0, 0, 0, -1, 0, 1, 1, 1, 1, 10, 45)
    _ = game.remove_light(1)
    _ = game.set_camera_position3_d(0, 0, 1)
    _ = game.set_camera_rotation3_d(0, 0, 0)
    _ = game.configure_grid(True, 10.0, 10)
    _ = game.set_grid_enabled(True)
    _ = game.configure_skybox(True, 0.1, 0.2, 0.3, 1.0)
    _ = game.configure_fog(True, 0.1, 0.2, 0.3, 0.5)
    _ = game.set_fog_enabled(True)
    _ = game.render3_d()

    ent = _types_mod.Entity(123)
    _ = game.spawn_batch(2)
    _ = game.despawn_batch((ctypes.c_uint64 * 1)(ent.to_bits()))
    _ = game.play(ent)
    _ = game.stop(ent)
    _ = game.set_state(ent, "idle")
    _ = game.set_parameter_bool(ent, "grounded", True)
    _ = game.set_parameter_float(ent, "speed", 1.5)
    _ = game.component_register_type(1, "Comp", 16, 8)
    _ = game.component_add(ent, 1, ctypes.POINTER(ctypes.c_uint8)(), 0)
    _ = game.component_has(ent, 1)
    _ = game.component_get(ent, 1)
    _ = game.component_get_mut(ent, 1)
    _ = game.component_add_batch((ctypes.c_uint64 * 1)(ent.to_bits()), 1, ctypes.POINTER(ctypes.c_uint8)(), 0)
    _ = game.component_remove_batch((ctypes.c_uint64 * 1)(ent.to_bits()), 1)
    _ = game.component_has_batch((ctypes.c_uint64 * 1)(ent.to_bits()), 1, (ctypes.c_uint8 * 1)())
    _ = game.component_count(1)
    _ = game.component_get_entities(1, (ctypes.c_uint64 * 4)(), 4)
    _ = game.component_get_all(1, (ctypes.c_uint64 * 4)(), (ctypes.POINTER(ctypes.c_uint8) * 4)(), 4)
    assert game.interpolation_alpha == 0.0
    game.end_frame()
    game.close()
    game.destroy()

    ctx = game_mod.GoudContext()
    assert ctx.is_valid() == 0
    _ = ctx.get_network_capabilities()
    _ = ctx.network_host(1, 9100)
    _ = ctx.network_connect(1, "127.0.0.1", 9100)
    conn2 = ctx.network_connect_with_peer(1, "127.0.0.1", 9100)
    _ = ctx.network_send(conn2.handle, conn2.peer_id, b"x", 0)
    _ = ctx.network_receive(conn2.handle)
    _ = ctx.network_receive_packet(conn2.handle)
    _ = ctx.network_poll(conn2.handle)
    _ = ctx.get_network_stats(conn2.handle)
    _ = ctx.network_peer_count(conn2.handle)
    _ = ctx.set_network_simulation(conn2.handle, sim)
    _ = ctx.clear_network_simulation(conn2.handle)
    _ = ctx.set_network_overlay_handle(conn2.handle)
    _ = ctx.clear_network_overlay_handle()
    assert ctx.get_debugger_snapshot_json() == '{"route":"ok"}'
    assert ctx.get_debugger_manifest_json() == '{"routes":[]}'
    ctx.set_debugger_profiling_enabled(False)
    ctx.set_debugger_selected_entity(7)
    ctx.clear_debugger_selected_entity()
    assert ctx.get_memory_summary().rendering.peak_bytes == 22
    e2 = ctx.spawn_empty()
    _ = ctx.spawn_batch(1)
    _ = ctx.despawn_batch((ctypes.c_uint64 * 1)(e2.to_bits()))
    _ = ctx.clone_entity(e2)
    _ = ctx.clone_entity_recursive(e2)
    _ = ctx.is_alive(e2)
    _ = ctx.entity_count()
    _ = ctx.add_name(e2, "ctx")
    _ = ctx.get_name(e2)
    _ = ctx.has_name(e2)
    _ = ctx.remove_name(e2)
    _ = ctx.component_register_type(1, "Comp", 8, 4)
    _ = ctx.component_add(e2, 1, ctypes.POINTER(ctypes.c_uint8)(), 0)
    _ = ctx.component_remove(e2, 1)
    _ = ctx.component_has(e2, 1)
    _ = ctx.component_get(e2, 1)
    _ = ctx.component_get_mut(e2, 1)
    _ = ctx.component_add_batch((ctypes.c_uint64 * 1)(e2.to_bits()), 1, ctypes.POINTER(ctypes.c_uint8)(), 0)
    _ = ctx.component_remove_batch((ctypes.c_uint64 * 1)(e2.to_bits()), 1)
    _ = ctx.component_has_batch((ctypes.c_uint64 * 1)(e2.to_bits()), 1, (ctypes.c_uint8 * 1)())
    _ = ctx.component_count(1)
    _ = ctx.component_get_entities(1, (ctypes.c_uint64 * 4)(), 4)
    _ = ctx.component_get_all(1, (ctypes.c_uint64 * 4)(), (ctypes.POINTER(ctypes.c_uint8) * 4)(), 4)
    sid2 = ctx.scene_create("ctx-scene")
    _ = ctx.scene_destroy(sid2)
    _ = ctx.scene_get_by_name("ctx-scene")
    _ = ctx.load_scene("ctx-scene", "{}")
    _ = ctx.unload_scene("ctx-scene")
    _ = ctx.set_active_scene(sid2, True)
    _ = ctx.scene_set_active(sid2, False)
    _ = ctx.scene_is_active(sid2)
    _ = ctx.scene_count()
    _ = ctx.scene_set_current(sid2)
    _ = ctx.scene_get_current()
    _ = ctx.scene_transition_to(sid2, sid2, 0, 0.1)
    _ = ctx.scene_transition_progress()
    _ = ctx.scene_transition_is_active()
    _ = ctx.scene_transition_tick(0.016)
    ctx.destroy()

    cfg_ctx = game_mod.GoudContext(
        _types_mod.ContextConfig(
            debugger=_types_mod.DebuggerConfig(
                enabled=True,
                publish_local_attach=True,
                route_label="ctx-route",
            )
        )
    )
    assert cfg_ctx._ctx._bits == 23
    assert ("goud_context_create_with_config", True, True, "ctx-route") in lib.calls
    cfg_ctx.destroy()

    cfg = game_mod.EngineConfig()
    cfg.set_title("T").set_size(800, 600).set_vsync(True).set_fullscreen(False)
    cfg.set_target_fps(60).set_fps_overlay(True).set_physics_debug(False).set_physics_backend2_d(0)
    cfg.set_debugger(_types_mod.DebuggerConfig(enabled=True, publish_local_attach=True, route_label="cov"))
    built = cfg.build()
    assert built._ctx._bits == 33
    cfg.destroy()
    try:
        cfg.build()
        assert False, "build() after consumption should fail"
    except RuntimeError:
        pass

    ui = game_mod.UiManager()
    style = _types_mod.UiStyle(font_family="Inter", texture_path="ui/button.png")
    assert ui.node_count() == 0
    node = ui.create_button(True)
    assert isinstance(node, int)
    assert ui.set_style(node, style) == 0
    assert ui.set_label_text(node, "Hello") == 0
    assert ui.set_image_texture_path(node, "tex.png") == 0
    assert ui.set_slider(node, 0.0, 1.0, 0.5, True) == 0
    ev = ui.event_read(0)
    assert ev is not None and ev.node_id == 10
    ui.destroy()

    w2d = game_mod.PhysicsWorld2D(0.0, -9.8, backend=0)
    assert w2d.create_with_backend(0.0, -9.8, 0) == 0
    assert w2d.set_gravity(0.0, -9.8) == 0
    assert w2d.add_rigid_body(0, 0.0, 0.0, 1.0) == 0
    assert w2d.set_timestep(0.016) == 0
    w2d.destroy()

    w3d = game_mod.PhysicsWorld3D(0.0, -9.8, 0.0)
    assert w3d.set_gravity(0.0, -9.8, 0.0) == 0
    assert w3d.add_rigid_body(0, 0.0, 0.0, 0.0, 1.0) == 0
    assert w3d.set_timestep(0.016) == 0
    w3d.destroy()

    print("  Generated _game.py fake-lib runtime coverage passed")
    return True
