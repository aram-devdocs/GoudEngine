#!/usr/bin/env python3
"""Top-level napi-rs generation helpers for the TypeScript Node SDK."""

from ts_node_audio import gen_napi_rust_audio
from ts_node_napi_basic import (
    gen_napi_rust_components,
    gen_napi_rust_entity,
    gen_napi_rust_types,
)
from ts_node_napi_game import gen_napi_rust_game
from ts_node_shared import NATIVE_SRC, RUST_HEADER, write_generated


def gen_napi_rust_lib():
    lines = [
        RUST_HEADER,
        "#[allow(dead_code)]",
        '#[path = "audio.g.rs"]',
        "mod audio;",
        "#[allow(dead_code)]",
        '#[path = "components.g.rs"]',
        "mod components;",
        '#[path = "entity.g.rs"]',
        "mod entity;",
        '#[path = "game.g.rs"]',
        "mod game;",
        "#[allow(dead_code)]",
        '#[path = "types.g.rs"]',
        "mod types;",
        "",
    ]
    write_generated(NATIVE_SRC / "lib.rs", "\n".join(lines))


def gen_napi_rust():
    gen_napi_rust_types()
    gen_napi_rust_entity()
    gen_napi_rust_components()
    gen_napi_rust_game()
    gen_napi_rust_audio()
    gen_napi_rust_lib()
