"""Error type generation for Kotlin SDK."""
from __future__ import annotations
from sdk_common import load_errors
from .helpers import HEADER_COMMENT, KOTLIN_OUT, schema, write_kotlin

def gen_errors():
    categories, codes = load_errors(schema)
    if not categories:
        return
    lines = [f"// {HEADER_COMMENT}", "package com.goudengine.core", "",
             "/** Recovery classification for engine errors. */",
             "enum class RecoveryClass(val value: Int) {", "    Recoverable(0),", "    Fatal(1),", "    Degraded(2);", "",
             "    companion object {", "        fun fromValue(value: Int): RecoveryClass =",
             "            entries.firstOrNull { it.value == value } ?: Recoverable", "    }", "}", "",
             "/** Base exception for all GoudEngine errors. */",
             "open class GoudException(", "    val errorCode: Int,", "    override val message: String,",
             "    val category: String,", "    val subsystem: String,", "    val operation: String,",
             "    val recovery: RecoveryClass,", "    val recoveryHint: String,", ") : Exception(message)", ""]
    for cat in categories:
        cls_name = cat["base_class"].replace("Error", "Exception")
        lines += [f"class {cls_name}(", "    errorCode: Int,", "    message: String,", "    subsystem: String,",
                  "    operation: String,", "    recovery: RecoveryClass,", "    recoveryHint: String,",
                  f") : GoudException(errorCode, message, \"{cat['name']}\", subsystem, operation, recovery, recoveryHint)", ""]
    lines.append("")
    write_kotlin(KOTLIN_OUT / "core" / "Errors.kt", "\n".join(lines))
