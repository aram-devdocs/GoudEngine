"""Diagnostic class generation."""

from sdk_common import HEADER_COMMENT, write_generated
from .context import NS, OUT, schema

def gen_diagnostic():
    if "diagnostic" not in schema:
        return
    diag = schema["diagnostic"]
    cls = diag["class_name"]
    lines = [
        f"// {HEADER_COMMENT}",
        "",
        "using System;",
        "using System.Text;",
        "",
        f"namespace {NS}",
        "{",
        "    /// <summary>",
        f"    /// {diag['doc']}",
        "    /// </summary>",
        f"    public static class {cls}",
        "    {",
    ]
    for method in diag["methods"]:
        cs_name = method["name"][0].upper() + method["name"][1:]
        ffi_name = method["ffi"]
        params = method.get("params", [])
        ret = method["returns"]

        lines.append("        /// <summary>")
        lines.append(f"        /// {method['doc']}")
        lines.append("        /// </summary>")

        if method.get("buffer_protocol"):
            lines += [
                f"        public static string {cs_name}",
                "        {",
                "            get",
                "            {",
                "                var buf = new byte[4096];",
                "                unsafe",
                "                {",
                "                    fixed (byte* ptr = buf)",
                "                    {",
                f"                        int written = NativeMethods.{ffi_name}(",
                "                            (IntPtr)ptr, (nuint)buf.Length);",
                "                        if (written <= 0)",
                "                            return string.Empty;",
                "                        return Encoding.UTF8.GetString(buf, 0, written);",
                "                    }",
                "                }",
                "            }",
                "        }",
            ]
        elif ret == "void":
            cs_params = ", ".join(f"{'bool' if p['type'] == 'bool' else p['type']} {p['name']}" for p in params)
            call_args = ", ".join(p["name"] for p in params)
            lines += [
                f"        public static void {cs_name}({cs_params})",
                "        {",
                f"            NativeMethods.{ffi_name}({call_args});",
                "        }",
            ]
        elif ret == "bool":
            lines += [
                f"        public static bool {cs_name} => NativeMethods.{ffi_name}();",
            ]

        lines.append("")

    lines.append("    }")
    lines.append("}")
    lines.append("")

    write_generated(OUT / "Core" / f"{cls}.g.cs", "\n".join(lines))


if __name__ == "__main__":
    print("Generating C# SDK...")
    gen_native_methods()
    gen_enums()
