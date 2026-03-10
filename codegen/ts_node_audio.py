#!/usr/bin/env python3
"""Audio-specific napi-rs generation helpers for the TypeScript Node SDK."""

from pathlib import Path

from ts_node_shared import NATIVE_SRC, write_generated

TEMPLATE_PATH = Path(__file__).with_name("ts_node_audio.g.rs.tpl")


def gen_napi_rust_audio():
    """Generate sdks/typescript/native/src/audio.g.rs from the checked-in template."""
    write_generated(NATIVE_SRC / "audio.g.rs", TEMPLATE_PATH.read_text())
