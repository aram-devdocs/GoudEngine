#!/usr/bin/env python3
"""Shared helpers for the Python SDK binding tests."""

import importlib.util
from pathlib import Path

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
