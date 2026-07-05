#!/usr/bin/env python3
"""Validate every SKILL.md under .agents/skills/.

Checks, per skill:
  1. The file opens with a YAML frontmatter block (--- ... ---).
  2. The frontmatter has a non-empty `name` and a non-empty `description`.
  3. `name` matches the skill's directory name.
  4. Every relative resource path the skill references (scripts/..., references/...,
     assets/..., tests/...) resolves to a real file or directory, either relative to
     the skill directory or relative to the repository root.

Pure standard library. Prints per-skill OK/FAIL plus a summary and exits nonzero
on any failure so CI can gate on it.
"""

import re
import sys
from pathlib import Path

# scripts/ lives at the repo root; this file is scripts/validate-skills.py.
REPO_ROOT = Path(__file__).resolve().parent.parent
SKILLS_DIR = REPO_ROOT / ".agents" / "skills"

# Resource prefixes a skill may ship or call. references/, assets/, and tests/ are
# skill-local; scripts/ usually points at a repo-root helper. Both roots are tried.
RESOURCE_PREFIXES = ("scripts", "references", "assets", "tests")

# A path token: one of the prefixes, a slash, then path characters. The lookbehind
# stops us matching the tail of a URL or a longer path (".../scripts/foo").
PATH_RE = re.compile(
    r"(?<![A-Za-z0-9._/-])(?:" + "|".join(RESOURCE_PREFIXES) + r")/[A-Za-z0-9._/-]+"
)

# Trailing characters that are punctuation from prose/markdown, not part of the path.
_TRAILING = ".,:;!?)]}'\"`"


def parse_frontmatter(text):
    """Return a dict of the leading YAML frontmatter, or None if absent/malformed.

    Only the simple `key: value` shape used by these skills is supported; that is
    all the frontmatter contract needs, so no third-party YAML dependency is pulled in.
    """
    lines = text.splitlines()
    if not lines or lines[0].strip() != "---":
        return None
    fields = {}
    for i in range(1, len(lines)):
        line = lines[i]
        if line.strip() == "---":
            return fields
        if not line.strip() or line.lstrip().startswith("#"):
            continue
        if ":" not in line:
            continue
        key, _, value = line.partition(":")
        key = key.strip()
        value = value.strip().strip("'\"")
        if key:
            fields[key] = value
    # No closing delimiter found.
    return None


def referenced_paths(text):
    """Yield unique cleaned resource paths referenced in the skill body."""
    seen = []
    for match in PATH_RE.finditer(text):
        token = match.group(0).rstrip(_TRAILING).rstrip("/")
        # Skip glob-y or placeholder fragments that survived (none expected, but be safe).
        if not token or token.endswith("/"):
            continue
        if token not in seen:
            seen.append(token)
    return seen


def validate_skill(skill_md):
    """Validate one SKILL.md. Return a list of error strings (empty == pass)."""
    errors = []
    skill_dir = skill_md.parent
    dir_name = skill_dir.name

    text = skill_md.read_text(encoding="utf-8")

    fm = parse_frontmatter(text)
    if fm is None:
        errors.append("missing or malformed YAML frontmatter block")
        return errors

    name = fm.get("name", "").strip()
    description = fm.get("description", "").strip()

    if not name:
        errors.append("frontmatter `name` is missing or empty")
    if not description:
        errors.append("frontmatter `description` is missing or empty")
    if name and name != dir_name:
        errors.append(f"frontmatter `name` ({name!r}) != directory name ({dir_name!r})")

    for ref in referenced_paths(text):
        local = skill_dir / ref
        rooted = REPO_ROOT / ref
        if not local.exists() and not rooted.exists():
            errors.append(f"referenced path does not exist: {ref}")

    return errors


def main():
    if not SKILLS_DIR.is_dir():
        print(f"FAIL: skills directory not found: {SKILLS_DIR}")
        return 1

    skill_files = sorted(SKILLS_DIR.glob("*/SKILL.md"))
    if not skill_files:
        print(f"FAIL: no SKILL.md files found under {SKILLS_DIR}")
        return 1

    failed = 0
    for skill_md in skill_files:
        name = skill_md.parent.name
        errors = validate_skill(skill_md)
        if errors:
            failed += 1
            print(f"FAIL {name}")
            for err in errors:
                print(f"       - {err}")
        else:
            print(f"OK   {name}")

    total = len(skill_files)
    print()
    print(f"Summary: {total - failed}/{total} skills passed, {failed} failed.")
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())
