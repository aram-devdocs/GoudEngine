#!/usr/bin/env python3

from pathlib import Path
import shutil
import subprocess
import sys
import tempfile


ROOT = Path(__file__).resolve().parent.parent
HEADER = ROOT / "codegen" / "generated" / "goud_engine.h"

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
    if not HEADER.exists():
        fail(f"Missing generated header: {HEADER}")

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
        c_path.write_text('#include "goud_engine.h"\n')
        cpp_path.write_text('#include "goud_engine.h"\n')
        include_dir = str(HEADER.parent)
        run_syntax_check([c_compiler, "-std=c11", "-fsyntax-only", "-I", include_dir], str(c_path))
        run_syntax_check([cpp_compiler, "-std=c++17", "-fsyntax-only", "-I", include_dir], str(cpp_path))

    print("C header validation passed.")


if __name__ == "__main__":
    main()
