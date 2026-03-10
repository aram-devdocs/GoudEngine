#!/usr/bin/env python3
"""Game napi-rs generation helpers for the TypeScript Node SDK."""

from pathlib import Path

from ts_node_shared import NATIVE_SRC, write_generated

TEMPLATE_PATH = Path(__file__).with_name("ts_node_game.g.rs.tpl")


def gen_napi_rust_game():
    """Generate sdks/typescript/native/src/game.g.rs from the checked-in template."""
    write_generated(NATIVE_SRC / "game.g.rs", TEMPLATE_PATH.read_text())
