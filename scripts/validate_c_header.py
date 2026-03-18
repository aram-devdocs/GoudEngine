#!/usr/bin/env python3

from pathlib import Path
import shutil
import subprocess
import sys
import tempfile


ROOT = Path(__file__).resolve().parent.parent
HEADER = ROOT / "codegen" / "generated" / "goud_engine.h"
C_SDK_INCLUDE = ROOT / "sdks" / "c" / "include"
CPP_SDK_INCLUDE = ROOT / "sdks" / "cpp" / "include"
C_SMOKE_EXAMPLE = ROOT / "examples" / "c" / "smoke" / "main.c"
CPP_SMOKE_EXAMPLE = ROOT / "examples" / "cpp" / "smoke" / "main.cpp"

REQUIRED_MARKERS = [
    "#ifndef GOUD_ENGINE_H",
    "#define GOUD_ENGINE_H",
    "#define GOUD_ENGINE_VERSION ",
    "GOUD_DEPRECATED",
    "GOUD_DEPRECATED_MSG(msg)",
    "/* === Common Types and Constants === */",
    "/* === ECS === */",
    "/* === Assets === */",
    "/* === Renderer === */",
    "/* === Input === */",
    "/* === Audio === */",
]

REPRESENTATIVE_SYMBOLS = {
    "/* === ECS === */": "goud_context_create(",
    "/* === Assets === */": "goud_texture_load(",
    "/* === Renderer === */": "goud_renderer_draw_sprite(",
    "/* === Input === */": "goud_input_key_pressed(",
    "/* === Audio === */": "goud_audio_activate(",
}


def fail(message: str) -> None:
    print(message, file=sys.stderr)
    raise SystemExit(1)


def run_syntax_check(command: list[str], source: str) -> None:
    completed = subprocess.run(
        command + [source],
        capture_output=True,
        text=True,
        check=False,
    )
    if completed.returncode != 0:
        stderr = completed.stderr.strip() or completed.stdout.strip() or "unknown compiler error"
        fail(f"Header syntax check failed for {' '.join(command)}: {stderr}")


def run_link_check(command: list[str]) -> None:
    completed = subprocess.run(
        command,
        capture_output=True,
        text=True,
        check=False,
    )
    if completed.returncode != 0:
        stderr = completed.stderr.strip() or completed.stdout.strip() or "unknown linker error"
        fail(f"Example link check failed for {' '.join(command)}: {stderr}")


def native_library_name() -> str:
    if sys.platform == "darwin":
        return "libgoud_engine.dylib"
    if sys.platform.startswith("linux"):
        return "libgoud_engine.so"
    if sys.platform in {"win32", "cygwin"}:
        return "goud_engine.dll"
    fail(f"Unsupported platform for native SDK validation: {sys.platform}")


def find_native_library() -> Path:
    library_name = native_library_name()
    candidates = [
        ROOT / "target" / profile / library_name for profile in ("debug", "release")
    ]
    existing = [candidate for candidate in candidates if candidate.exists()]
    if not existing:
        fail(
            "Missing native library for SDK validation. "
            f"Looked for: {', '.join(str(candidate) for candidate in candidates)}"
        )
    return max(existing, key=lambda candidate: candidate.stat().st_mtime)


def require_path(path: Path) -> None:
    if not path.exists():
        fail(f"Missing required path: {path}")


def find_between(text: str, start_marker: str) -> str:
    start = text.find(start_marker)
    if start == -1:
        fail(f"Missing section marker: {start_marker}")
    later_markers = [text.find(marker, start + len(start_marker)) for marker in REPRESENTATIVE_SYMBOLS if marker != start_marker]
    later_markers.extend(text.find(marker, start + len(start_marker)) for marker in REQUIRED_MARKERS if marker.startswith("/* ===") and marker != start_marker)
    candidates = [idx for idx in later_markers if idx != -1]
    end = min(candidates) if candidates else len(text)
    return text[start:end]


def main() -> None:
    require_path(HEADER)
    require_path(C_SDK_INCLUDE / "goud_engine.h")
    require_path(C_SDK_INCLUDE / "goud" / "goud.h")
    require_path(CPP_SDK_INCLUDE / "goud_engine.h")
    require_path(CPP_SDK_INCLUDE / "goud" / "goud.h")
    require_path(CPP_SDK_INCLUDE / "goud" / "goud.hpp")
    require_path(C_SMOKE_EXAMPLE)
    require_path(CPP_SMOKE_EXAMPLE)

    text = HEADER.read_text()

    for marker in REQUIRED_MARKERS:
        if marker not in text:
            fail(f"Missing required header marker: {marker}")

    for section, symbol in REPRESENTATIVE_SYMBOLS.items():
        section_text = find_between(text, section)
        if symbol not in section_text:
            fail(f"Expected {symbol} inside {section}")

    if text.count('extern "C" {') != 1:
        fail('Expected exactly one extern "C" opening wrapper in the generated header')

    close_count = text.count('} /* extern "C" */') + text.count('}  // extern "C"')
    if close_count != 1:
        fail('Expected exactly one extern "C" closing wrapper in the generated header')

    input_section = find_between(text, "/* === Input === */")
    for symbol in [
        "typedef int32_t GoudKeyCode;",
        "typedef int32_t GoudMouseButton;",
        "#define KEY_A 65",
        "#define MOUSE_BUTTON_LEFT 0",
    ]:
        if symbol not in input_section:
            fail(f"Expected {symbol} inside /* === Input === */")

    if "GOUD_DEPRECATED_MSG(\"Use goud_renderer_draw_text instead.\")" not in text or "goud_draw_text(" not in text:
        fail("Expected goud_draw_text to be marked deprecated in the generated header")

    c_compiler = shutil.which("cc")
    cpp_compiler = shutil.which("c++") or shutil.which("clang++")
    if not c_compiler or not cpp_compiler:
        fail("Missing required C/C++ compiler for header syntax validation")

    with tempfile.TemporaryDirectory() as tmp:
        c_path = Path(tmp) / "header_smoke.c"
        cpp_path = Path(tmp) / "header_smoke.cpp"
        c_sdk_path = Path(tmp) / "c_sdk_smoke.c"
        cpp_sdk_path = Path(tmp) / "cpp_sdk_smoke.cpp"
        c_example_bin = Path(tmp) / "c_smoke"
        cpp_example_bin = Path(tmp) / "cpp_smoke"
        c_path.write_text('#include "goud_engine.h"\n')
        cpp_path.write_text('#include "goud_engine.h"\n')
        c_sdk_path.write_text('#include <goud/goud.h>\n')
        cpp_sdk_path.write_text('#include <goud/goud.hpp>\n')
        include_dir = str(HEADER.parent)
        c_sdk_include = str(C_SDK_INCLUDE)
        cpp_sdk_include = str(CPP_SDK_INCLUDE)
        run_syntax_check([c_compiler, "-std=c11", "-fsyntax-only", "-I", include_dir], str(c_path))
        run_syntax_check([cpp_compiler, "-std=c++17", "-fsyntax-only", "-I", include_dir], str(cpp_path))
        run_syntax_check(
            [c_compiler, "-std=c11", "-fsyntax-only", "-I", c_sdk_include],
            str(c_sdk_path),
        )
        run_syntax_check(
            [
                cpp_compiler,
                "-std=c++17",
                "-fsyntax-only",
                "-I",
                cpp_sdk_include,
            ],
            str(cpp_sdk_path),
        )

        library_path = find_native_library()
        library_dir = str(library_path.parent)

        run_link_check(
            [
                c_compiler,
                "-std=c11",
                "-I",
                c_sdk_include,
                str(C_SMOKE_EXAMPLE),
                "-L",
                library_dir,
                "-lgoud_engine",
                "-o",
                str(c_example_bin),
            ]
        )
        run_link_check(
            [
                cpp_compiler,
                "-std=c++17",
                "-I",
                cpp_sdk_include,
                str(CPP_SMOKE_EXAMPLE),
                "-L",
                library_dir,
                "-lgoud_engine",
                "-o",
                str(cpp_example_bin),
            ]
        )

    print("C and C++ header validation passed.")


if __name__ == "__main__":
    main()
