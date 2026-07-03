#!/usr/bin/env python3
"""Assert the canonical verify pipeline stays wired consistently.

The value of a single data-driven gate (scripts/verify.sh) is only real if the
git hooks and CI actually route through it and the fast staged tier never runs a
check the full gate skips. This validator enforces exactly that so "passes at
commit" always implies "passes at push and in CI".

Checks:
  1. The staged step set is a strict subset of the full step set (name + cmd),
     so nothing runs at commit time that the push/CI gate does not also run.
  2. The pre-commit hook invokes `verify.sh --staged` and the pre-push hook
     invokes `verify.sh` — the hooks are thin wrappers, not parallel step lists.
  3. No residual hand-mirrored step banners survive in the hooks (the old
     "=== [N/16] ===" pattern), which would mean a second, drifting gate.
  4. Every block-tier step names a CI anchor whose workflow file exists.
"""
from __future__ import annotations

import re
import subprocess
import sys
from pathlib import Path

REPO_ROOT = Path(
    subprocess.check_output(["git", "rev-parse", "--show-toplevel"], text=True).strip()
)
VERIFY = REPO_ROOT / "scripts" / "verify.sh"
HOOKS = REPO_ROOT / ".husky" / "hooks"
WORKFLOWS = REPO_ROOT / ".github" / "workflows"


def load_steps(staged: bool) -> list[dict]:
    args = ["bash", str(VERIFY), "--list"]
    if staged:
        args.append("--staged")
    out = subprocess.check_output(args, text=True, cwd=REPO_ROOT)
    rows = out.strip().splitlines()
    header = rows[0].split("\t")
    steps = []
    for line in rows[1:]:
        cols = line.split("\t")
        steps.append(dict(zip(header, cols)))
    return steps


def main() -> int:
    errors: list[str] = []

    full = load_steps(staged=False)
    staged = load_steps(staged=True)

    # 1. staged ⊆ full by (name, cmd).
    full_keys = {(s["name"], s["cmd"]) for s in full}
    for s in staged:
        if (s["name"], s["cmd"]) not in full_keys:
            errors.append(
                f"staged step '{s['name']}' is not present (same cmd) in the full gate"
            )

    # 2. hooks are thin wrappers around verify.sh.
    pre_commit = (HOOKS / "pre-commit").read_text()
    pre_push = (HOOKS / "pre-push").read_text()
    if "verify.sh" not in pre_commit or "--staged" not in pre_commit:
        errors.append("pre-commit hook must invoke 'verify.sh --staged'")
    if "verify.sh" not in pre_push:
        errors.append("pre-push hook must invoke 'verify.sh'")

    # 3. no residual hand-mirrored step banners in the hooks.
    banner = re.compile(r"===\s*\[\d+/\d+\]")
    for name, text in (("pre-commit", pre_commit), ("pre-push", pre_push)):
        if banner.search(text):
            errors.append(
                f"{name} still contains a hand-mirrored '=== [N/M] ===' step list; "
                "it must delegate to verify.sh instead"
            )

    # 4. every block-tier step's CI anchor names a real workflow file.
    for s in full:
        if s["tier"] != "block":
            continue
        anchor = s.get("ci_anchor", "")
        if not anchor:
            errors.append(f"block step '{s['name']}' has no ci_anchor")
            continue
        wf = anchor.split(":", 1)[0]
        if not (WORKFLOWS / wf).exists():
            errors.append(
                f"block step '{s['name']}' anchors to '{wf}', which does not exist "
                f"under .github/workflows/"
            )

    if errors:
        print("✗ gate-parity check failed:", file=sys.stderr)
        for e in errors:
            print(f"  - {e}", file=sys.stderr)
        return 1
    print(f"✓ gate-parity: {len(staged)} staged ⊆ {len(full)} full; hooks wired; anchors valid")
    return 0


if __name__ == "__main__":
    sys.exit(main())
