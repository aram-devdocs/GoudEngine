"""Generator for `goud/doc.go` -- package documentation."""

from .context import GO_HEADER, OUT, write_generated


def gen_doc() -> None:
    lines = [
        GO_HEADER,
        "",
        "// Package goud provides the Go SDK wrapper for GoudEngine.",
        "//",
        "// This package is a thin wrapper over the FFI bindings in internal/ffi,",
        "// providing idiomatic Go types and methods. All game logic, rendering,",
        "// physics, and audio processing lives in the Rust engine core; this SDK",
        "// only marshals calls across the FFI boundary.",
        "//",
        "// Basic usage:",
        "//",
        '//\tgame := goud.NewGame(800, 600, "My Game")',
        "//\tdefer game.Destroy()",
        "//\tfor !game.ShouldClose() {",
        "//\t\tgame.BeginFrame(0, 0, 0, 1)",
        "//\t\t// draw and update here",
        "//\t\tgame.EndFrame()",
        "//\t}",
        "package goud",
        "",
    ]

    write_generated(OUT / "doc.go", "\n".join(lines))
