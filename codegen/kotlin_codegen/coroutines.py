"""Coroutine suspend extensions for Kotlin SDK."""

from __future__ import annotations

from .helpers import (
    HEADER_COMMENT,
    KOTLIN_OUT,
    write_kotlin,
)


def gen_coroutines():
    """Generate suspend extension functions for GoudGame."""
    lines = [
        f"// {HEADER_COMMENT}",
        "package com.goudengine.core",
        "",
        "import kotlinx.coroutines.Dispatchers",
        "import kotlinx.coroutines.withContext",
        "",
        "suspend fun GoudGame.loadTextureAsync(path: String): Long =",
        "    withContext(Dispatchers.IO) { loadTexture(path) }",
        "",
        "suspend fun GoudGame.loadFontAsync(path: String): Long =",
        "    withContext(Dispatchers.IO) { loadFont(path) }",
        "",
    ]

    write_kotlin(KOTLIN_OUT / "core" / "Coroutines.kt", "\n".join(lines))
