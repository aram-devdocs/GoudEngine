"""Entity handle generation for Kotlin SDK."""
from __future__ import annotations
from .helpers import HEADER_COMMENT, KOTLIN_OUT, write_kotlin

def gen_entity():
    lines = [f"// {HEADER_COMMENT}", "package com.goudengine.core", "",
             "@JvmInline", "value class EntityHandle(val id: Long) {",
             "    val index: Int get() = (id and 0xFFFFFFFFL).toInt()",
             "    val generation: Int get() = (id ushr 32).toInt()",
             "    val isPlaceholder: Boolean get() = id == -1L", "",
             "    override fun toString(): String = \"Entity(${index}v${generation})\"", "",
             "    companion object {", "        val PLACEHOLDER = EntityHandle(-1L)", "    }",
             "}", ""]
    write_kotlin(KOTLIN_OUT / "core" / "EntityHandle.kt", "\n".join(lines))
