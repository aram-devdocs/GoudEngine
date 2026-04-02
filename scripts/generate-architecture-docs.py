#!/usr/bin/env python3
"""Generate architecture doc sections from code sources."""

from __future__ import annotations

import argparse
import re
from pathlib import Path
import sys

ROOT = Path(__file__).resolve().parent.parent
ARCHITECTURE_MD = ROOT / "ARCHITECTURE.md"

GEN_PATTERN = re.compile(
    r'(<!-- gen:(\w[\w-]*) -->\n).*?(<!-- /gen:\2 -->)',
    re.DOTALL,
)


# =============================================================================
# Collectors
# =============================================================================


def collect_layer_hierarchy() -> str:
    """Parse tools/lint_layers.rs for the 5-layer architecture table."""
    lint_path = ROOT / "tools" / "lint_layers.rs"
    source = lint_path.read_text(encoding="utf-8")

    # Extract enum variants with layer numbers and descriptions.
    # Pattern: Foundation, // Layer 1: core/
    enum_re = re.compile(r'^\s+(\w+),?\s*//\s*Layer\s+(\d+):\s*(.*)', re.MULTILINE)
    variants: list[tuple[str, int, str]] = []
    for m in enum_re.finditer(source):
        variants.append((m.group(1), int(m.group(2)), m.group(3).strip()))

    # Extract Display impl names.
    display_re = re.compile(r'Layer::(\w+)\s*=>\s*write!\(f,\s*"([^"]+)"\)', re.MULTILINE)
    display_names: dict[str, str] = {}
    for m in display_re.finditer(source):
        display_names[m.group(1)] = m.group(2)

    # Extract directory prefixes from classify_file.
    classify_fn_re = re.compile(
        r'fn classify_file\(.*?\{(.*?)^\}', re.DOTALL | re.MULTILINE
    )
    classify_match = classify_fn_re.search(source)
    classify_body = classify_match.group(1) if classify_match else ""

    # Map layer variant to directories by parsing the if/else chain.
    # Each branch: starts_with("dir/") ... Some(Layer::Variant)
    dir_re = re.compile(r'starts_with\("([^"]+)/"\)')
    layer_re = re.compile(r'Some\(Layer::(\w+)\)')

    # Split by Some(Layer::...) to associate directories with layers.
    layer_dirs: dict[str, list[str]] = {}
    # Process line-by-line groups between Some(Layer::...) occurrences.
    blocks = re.split(r'(Some\(Layer::\w+\))', classify_body)
    current_dirs: list[str] = []
    for block in blocks:
        layer_m = layer_re.match(block.strip())
        if layer_m:
            variant = layer_m.group(1)
            layer_dirs[variant] = current_dirs
            current_dirs = []
        else:
            current_dirs.extend(m.group(1) for m in dir_re.finditer(block))

    # Build table rows.
    lines: list[str] = [
        "| Layer | Name | Directories | May Import From |",
        "|-------|------|-------------|-----------------|",
    ]
    for variant, num, _desc in sorted(variants, key=lambda v: v[1]):
        name = display_names.get(variant, variant)
        dirs = layer_dirs.get(variant, [])
        dir_str = ", ".join(f"`{d}/`" for d in sorted(dirs)) if dirs else "(root)"
        # Layers that may be imported from: all layers with lower numbers.
        lower = [
            display_names.get(v, v)
            for v, n, _ in sorted(variants, key=lambda v: v[1])
            if n < num
        ]
        import_str = ", ".join(lower) if lower else "(none)"
        lines.append(f"| {num} | {name} | {dir_str} | {import_str} |")

    return "\n".join(lines)


def collect_sdk_table() -> str:
    """List SDK directories under sdks/."""
    sdks_dir = ROOT / "sdks"
    exclude = {"csharp.tests", "nuget_package_output"}

    name_map: dict[str, str] = {
        "c": "C",
        "cpp": "C++",
        "csharp": "C#",
        "go": "Go",
        "kotlin": "Kotlin",
        "lua": "Lua",
        "python": "Python",
        "rust": "Rust",
        "swift": "Swift",
        "typescript": "TypeScript",
    }

    sdks: list[tuple[str, str]] = []
    for entry in sorted(sdks_dir.iterdir()):
        if not entry.is_dir():
            continue
        if entry.name in exclude:
            continue
        display = name_map.get(entry.name, entry.name)
        sdks.append((display, f"sdks/{entry.name}/"))

    lines: list[str] = [
        "| SDK | Path |",
        "|-----|------|",
    ]
    for display, path in sdks:
        lines.append(f"| {display} | `{path}` |")

    lines.append("")
    lines.append(f"Total: {len(sdks)} SDK languages.")
    return "\n".join(lines)


def collect_codegen_generators() -> str:
    """Glob codegen/gen_*.py files and describe each."""
    codegen_dir = ROOT / "codegen"

    desc_map: dict[str, str] = {
        "gen_csharp": "C# SDK bindings",
        "gen_python": "Python SDK bindings",
        "gen_ts_node": "TypeScript Node.js bindings",
        "gen_ts_web": "TypeScript Web/WASM bindings",
        "gen_cpp": "C++ SDK bindings",
        "gen_go": "Go cgo bindings",
        "gen_go_sdk": "Go SDK wrapper",
        "gen_jni": "JNI bindings",
        "gen_kotlin": "Kotlin SDK bindings",
        "gen_lua": "Lua SDK bindings",
        "gen_swift": "Swift SDK bindings",
        "gen_sdk_readmes": "SDK README files from template",
        "gen_sdk_scaffolding": "SDK package scaffolding",
    }

    generators = sorted(codegen_dir.glob("gen_*.py"))

    lines: list[str] = [
        "| Generator | Output |",
        "|-----------|--------|",
    ]
    for gen in generators:
        stem = gen.stem
        desc = desc_map.get(stem, stem)
        lines.append(f"| `{gen.name}` | {desc} |")

    lines.append("")
    lines.append(f"Total: {len(generators)} generators.")
    return "\n".join(lines)


def collect_codegen_steps() -> str:
    """Parse codegen.sh for step echo lines and return the count."""
    codegen_sh = ROOT / "codegen.sh"
    content = codegen_sh.read_text(encoding="utf-8")

    step_re = re.compile(r'echo\s+".*\[\d+[a-z]?/\d+\]')
    steps = step_re.findall(content)

    return f"The full pipeline runs **{len(steps)} steps** (see `codegen.sh` for details)."


def collect_asset_loaders() -> str:
    """List subdirectories of goud_engine/src/assets/loaders/."""
    loaders_dir = ROOT / "goud_engine" / "src" / "assets" / "loaders"

    loaders = sorted(
        entry.name
        for entry in loaders_dir.iterdir()
        if entry.is_dir()
    )

    names = ", ".join(loaders)
    return f"Registered loaders (**{len(loaders)} total**): {names}."


def collect_ffi_modules() -> str:
    """List FFI modules in goud_engine/src/ffi/."""
    ffi_dir = ROOT / "goud_engine" / "src" / "ffi"
    exclude = {"mod.rs", "AGENTS.md", "CLAUDE.md", "error.rs"}

    modules: set[str] = set()
    for entry in ffi_dir.iterdir():
        if entry.name in exclude:
            continue
        if entry.is_dir():
            modules.add(entry.name)
        elif entry.suffix == ".rs":
            modules.add(entry.stem)

    names = ", ".join(sorted(modules))
    return f"FFI modules (**{len(modules)} total**): {names}."


def collect_providers() -> str:
    """List provider .rs files in goud_engine/src/core/providers/."""
    providers_dir = ROOT / "goud_engine" / "src" / "core" / "providers"
    exclude = {"mod.rs", "types.rs", "types3d.rs", "input_types.rs", "builder.rs"}

    providers: list[str] = []
    for entry in sorted(providers_dir.iterdir()):
        if entry.is_dir():
            continue
        if entry.name in exclude:
            continue
        if entry.suffix == ".rs":
            providers.append(entry.stem)

    names = ", ".join(providers)
    return f"Provider traits: {names}."


def collect_feature_flags() -> str:
    """Parse goud_engine/Cargo.toml for the [features] section."""
    cargo_toml = ROOT / "goud_engine" / "Cargo.toml"
    content = cargo_toml.read_text(encoding="utf-8")

    # Find the [features] section.
    features_re = re.compile(
        r'^\[features\]\s*\n(.*?)(?=^\[|\Z)', re.MULTILINE | re.DOTALL
    )
    m = features_re.search(content)
    if not m:
        return "No features section found."

    feature_block = m.group(1)

    lines: list[str] = [
        "| Feature | Dependencies |",
        "|---------|-------------|",
    ]

    feature_line_re = re.compile(r'^(\S+)\s*=\s*\[(.*?)\]\s*$', re.MULTILINE)
    for fm in feature_line_re.finditer(feature_block):
        name = fm.group(1)
        raw_deps = fm.group(2).strip()
        if raw_deps:
            # Parse quoted strings, strip dep: prefix.
            dep_re = re.compile(r'"([^"]*)"')
            deps = []
            for d in dep_re.findall(raw_deps):
                clean = d.replace("dep:", "")
                deps.append(clean)
            dep_str = ", ".join(f"`{d}`" for d in deps)
        else:
            dep_str = "(empty)"
        lines.append(f"| `{name}` | {dep_str} |")

    return "\n".join(lines)


def collect_default_backend() -> str:
    """Parse game_config.rs Default impl for render and window backend."""
    config_path = ROOT / "goud_engine" / "src" / "sdk" / "game_config.rs"
    content = config_path.read_text(encoding="utf-8")

    render_re = re.compile(r'render_backend:\s*(\w+)::(\w+)')
    window_re = re.compile(r'window_backend:\s*(\w+)::(\w+)')

    render_m = render_re.search(content)
    window_m = window_re.search(content)

    render_type = render_m.group(1) if render_m else "Unknown"
    render_variant = render_m.group(2) if render_m else "Unknown"
    window_type = window_m.group(1) if window_m else "Unknown"
    window_variant = window_m.group(2) if window_m else "Unknown"

    return (
        f"Default render backend: **{render_variant}** (`{render_type}::{render_variant}`). "
        f"Default window backend: **{window_variant}** (`{window_type}::{window_variant}`)."
    )


def collect_gameconfig_fields() -> str:
    """Parse pub struct GameConfig fields with doc comments."""
    config_path = ROOT / "goud_engine" / "src" / "sdk" / "game_config.rs"
    content = config_path.read_text(encoding="utf-8")

    # Extract the GameConfig struct body.
    struct_re = re.compile(
        r'pub struct GameConfig\s*\{(.*?)^\}', re.DOTALL | re.MULTILINE
    )
    m = struct_re.search(content)
    if not m:
        return "GameConfig struct not found."

    body = m.group(1)

    lines: list[str] = [
        "| Field | Type | Description |",
        "|-------|------|-------------|",
    ]

    # Parse fields: doc comments (/// ...) followed by pub field_name: Type,
    field_re = re.compile(
        r'((?:\s*///[^\n]*\n)+)\s*pub\s+(\w+)\s*:\s*([^,\n]+)',
        re.MULTILINE,
    )

    for fm in field_re.finditer(body):
        doc_block = fm.group(1)
        field_name = fm.group(2)
        field_type = fm.group(3).strip().rstrip(',')

        # Extract doc comment text, join lines.
        doc_lines = []
        for line in doc_block.strip().splitlines():
            text = line.strip().lstrip('/').strip()
            if text:
                doc_lines.append(text)
        description = " ".join(doc_lines)

        lines.append(f"| `{field_name}` | `{field_type}` | {description} |")

    return "\n".join(lines)


# =============================================================================
# Marker replacement
# =============================================================================


def replace_generated_sections(content: str, sections: dict[str, str]) -> str:
    def replacer(match: re.Match) -> str:
        open_tag = match.group(1)
        section_id = match.group(2)
        close_tag = match.group(3)
        if section_id in sections:
            return f"{open_tag}{sections[section_id]}\n{close_tag}"
        return match.group(0)
    return GEN_PATTERN.sub(replacer, content)


# =============================================================================
# Write / check
# =============================================================================


def write_or_check(path: Path, content: str, check: bool) -> None:
    if check:
        current = path.read_text(encoding="utf-8") if path.exists() else ""
        if current != content:
            # Show the first difference for debugging
            import difflib
            diff = list(difflib.unified_diff(
                current.splitlines(keepends=True),
                content.splitlines(keepends=True),
                fromfile="committed",
                tofile="generated",
                n=2,
            ))
            if diff:
                print("".join(diff[:30]), file=sys.stderr)
            raise SystemExit(f"Generated file is out of date: {path.relative_to(ROOT)}")
        return
    path.write_text(content, encoding="utf-8")


# =============================================================================
# Main
# =============================================================================


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true", help="fail if generated sections are out of date")
    args = parser.parse_args()

    sections = {
        "layer-hierarchy": collect_layer_hierarchy(),
        "sdk-table": collect_sdk_table(),
        "codegen-generators": collect_codegen_generators(),
        "codegen-steps": collect_codegen_steps(),
        "asset-loaders": collect_asset_loaders(),
        "ffi-modules": collect_ffi_modules(),
        "providers": collect_providers(),
        "feature-flags": collect_feature_flags(),
        "default-backend": collect_default_backend(),
        "gameconfig-fields": collect_gameconfig_fields(),
    }

    content = ARCHITECTURE_MD.read_text(encoding="utf-8")
    updated = replace_generated_sections(content, sections)

    write_or_check(ARCHITECTURE_MD, updated, args.check)
    if not args.check:
        print(f"Generated sections in {ARCHITECTURE_MD.relative_to(ROOT)}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
