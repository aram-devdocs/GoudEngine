#!/usr/bin/env python3
"""Error and diagnostic generation helpers for the TypeScript Node SDK."""

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))
from sdk_common import load_errors
from ts_node_shared import GEN, HEADER_COMMENT, schema, write_generated


def gen_errors():
    categories, codes = load_errors(schema)
    if not categories:
        return

    lines = [
        f"// {HEADER_COMMENT}",
        "",
        "export enum RecoveryClass {",
        "  Recoverable = 0,",
        "  Fatal = 1,",
        "  Degraded = 2,",
        "}",
        "",
        "/** Base error for all GoudEngine errors. */",
        "export class GoudError extends Error {",
        "  public readonly code: number;",
        "  public readonly category: string;",
        "  public readonly subsystem: string;",
        "  public readonly operation: string;",
        "  public readonly recovery: RecoveryClass;",
        "  public readonly recoveryHint: string;",
        "",
        "  constructor(",
        "    code: number,",
        "    message: string,",
        "    category: string,",
        "    subsystem: string,",
        "    operation: string,",
        "    recovery: RecoveryClass,",
        "    recoveryHint: string,",
        "  ) {",
        "    super(message);",
        "    this.name = new.target.name;",
        "    this.code = code;",
        "    this.category = category;",
        "    this.subsystem = subsystem;",
        "    this.operation = operation;",
        "    this.recovery = recovery;",
        "    this.recoveryHint = recoveryHint;",
        "",
        "    // Maintain proper prototype chain for instanceof checks",
        "    Object.setPrototypeOf(this, new.target.prototype);",
        "  }",
        "",
        "  /**",
        "   * Build the correct typed error subclass from a code and message.",
        "   * Subsystem and operation are optional context strings.",
        "   */",
        "  static fromCode(",
        "    code: number,",
        "    message: string,",
        '    subsystem: string = "",',
        '    operation: string = "",',
        "  ): GoudError {",
        "    const category = categoryFromCode(code);",
        "    const recovery = recoveryFromCategory(category);",
        "    const hint = hintFromCode(code);",
        "    const Subclass = CATEGORY_CLASS_MAP[category] ?? GoudError;",
        "",
        "    return new Subclass(",
        "      code, message, category, subsystem, operation, recovery, hint,",
        "    );",
        "  }",
        "}",
        "",
    ]

    for cat in categories:
        lines.append(f"export class {cat['base_class']} extends GoudError {{}}")
    lines.append("")

    lines.append("const CATEGORY_CLASS_MAP: Record<string, typeof GoudError> = {")
    for cat in categories:
        lines.append(f"  {cat['name']}: {cat['base_class']},")
    lines += ["};", ""]

    lines.append("function categoryFromCode(code: number): string {")
    for cat in sorted(categories, key=lambda c: c["range_start"], reverse=True):
        lines.append(f'  if (code >= {cat["range_start"]}) return "{cat["name"]}";')
    lines += ['  return "Unknown";', "}", ""]

    fatal_cats = {c["category"] for c in codes if c["recovery"] == "fatal"}
    lines.append("/**")
    lines.append(" * Default recovery class derived from code range. This is a fallback")
    lines.append(" * for environments where the native FFI is not available (e.g., web).")
    lines.append(" * Desktop environments should prefer the value from")
    lines.append(" * goud_error_recovery_class.")
    lines.append(" */")
    lines.append("function recoveryFromCategory(category: string): RecoveryClass {")
    lines.append("  switch (category) {")
    for cat_name in sorted(fatal_cats):
        lines.append(f'    case "{cat_name}":')
    lines.append("      return RecoveryClass.Fatal;")
    lines.append("    default:")
    lines.append("      return RecoveryClass.Recoverable;")
    lines += ["  }", "}", ""]

    lines.append("/** Static hint lookup matching the codegen schema. */")
    lines.append("function hintFromCode(code: number): string {")
    lines.append('  return HINTS[code] ?? "";')
    lines += ["}", ""]

    lines.append("const HINTS: Record<number, string> = {")
    for c in codes:
        lines.append(f'  {c["code"]}: "{c["hint"]}",')
    lines += ["};", ""]

    write_generated(GEN / "errors.g.ts", "\n".join(lines))


def gen_diagnostic():
    if "diagnostic" not in schema:
        return
    diag = schema["diagnostic"]
    cls = diag["class_name"]
    lines = [
        f"// {HEADER_COMMENT}",
        "",
        "/**",
        f" * {diag['doc']}",
        " *",
        " * In web/WASM builds these are no-ops.",
        " */",
        f"export class {cls} {{",
        "  private static _enabled = false;",
        "",
    ]
    for method in diag["methods"]:
        name = method["name"]
        ffi = method["ffi"]
        doc = method["doc"]
        if method.get("buffer_protocol"):
            lines += [f"  /** {doc} */", f"  static get {name}(): string {{", "    try {", "      const native = require('../node/index.g.js');", f"      if (typeof native.{ffi} === 'function') {{", f'        return native.{ffi}() ?? "";', "      }", "    } catch {", "    }", '    return "";', "  }"]
        elif method["returns"] == "void":
            param = method["params"][0]["name"]
            lines += [f"  /** {doc} */", f"  static {name}({param}: boolean): void {{", "    try {", "      const native = require('../node/index.g.js');", f"      if (typeof native.{ffi} === 'function') {{", f"        native.{ffi}({param});", "      }", "    } catch {", "    }", f"    {cls}._enabled = {param};", "  }"]
        elif method["returns"] == "bool":
            lines += [f"  /** {doc} */", f"  static get {name}(): boolean {{", "    try {", "      const native = require('../node/index.g.js');", f"      if (typeof native.{ffi} === 'function') {{", f"        return native.{ffi}();", "      }", "    } catch {", "    }", f"    return {cls}._enabled;", "  }"]
        lines.append("")
    lines += ["}", ""]
    write_generated(GEN / "diagnostic.g.ts", "\n".join(lines))
