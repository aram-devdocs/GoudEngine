#!/usr/bin/env python3
"""Validate that backtick-quoted paths in .md files exist on disk."""

from __future__ import annotations

import re
from pathlib import Path
import sys

ROOT = Path(__file__).resolve().parent.parent

# Matches backtick-quoted paths like `some/path/` or `some/file.rs`
PATH_PATTERN = re.compile(r'`([a-zA-Z0-9_./-]+/[a-zA-Z0-9_.*/-]*)`')

# Directories to skip when scanning .md files
SKIP_DIRS = {'.git', 'target', 'node_modules', 'docs/book'}

# Paths to ignore (not project-relative references)
IGNORE_PREFIXES = (
    'http', 'https', 'use ', 'cargo ', 'git ', 'npm ', 'pip ',
    'dotnet ', 'python', 'bash ', '#', 'dep:', 'crate::', 'std::',
)

# Known path prefixes that indicate a project-relative reference
# (paths must start with one of these to be checked)
KNOWN_ROOTS = (
    'core/', 'libs/', 'ecs/', 'assets/', 'sdk/', 'rendering/',
    'component_ops/', 'context_registry/', 'ffi/', 'wasm/',
    'goud_engine/', 'sdks/', 'codegen/', 'tools/', 'examples/',
    'docs/', 'scripts/', '.agents/', '.github/', '.claude/',
)


def should_skip(dirpath: Path) -> bool:
    rel = str(dirpath.relative_to(ROOT))
    return any(rel.startswith(skip) or f'/{skip}/' in rel for skip in SKIP_DIRS)


def check_path(path_str: str) -> bool:
    """Check if a path reference exists on disk."""
    # Strip trailing wildcards or patterns
    clean = path_str.rstrip('*').rstrip('/')
    if not clean:
        return True

    target = ROOT / clean
    # Check both exact match and as a prefix (for glob-like references)
    return target.exists() or target.with_suffix('').exists()


def main() -> int:
    errors: list[str] = []

    for md_file in sorted(ROOT.rglob('*.md')):
        if should_skip(md_file.parent):
            continue

        try:
            content = md_file.read_text(encoding='utf-8')
        except (UnicodeDecodeError, PermissionError):
            continue

        for line_num, line in enumerate(content.splitlines(), 1):
            # Skip code blocks
            if line.strip().startswith('```'):
                continue

            for match in PATH_PATTERN.finditer(line):
                path_str = match.group(1)

                # Skip non-path references
                if any(path_str.startswith(p) for p in IGNORE_PREFIXES):
                    continue

                # Only check paths that look like project references
                if not any(path_str.startswith(r) for r in KNOWN_ROOTS):
                    continue

                if not check_path(path_str):
                    rel_file = md_file.relative_to(ROOT)
                    errors.append(f"  {rel_file}:{line_num}: `{path_str}` not found")

    if errors:
        print(f"Found {len(errors)} broken path reference(s) in documentation:")
        for err in errors:
            print(err)
        return 1

    print("All doc path references OK.")
    return 0


if __name__ == '__main__':
    sys.exit(main())
