#!/usr/bin/env bash
# validate-ai-config.sh — Verify AI config integrity and wrapper sync
# Checks symlinks, content parity, generated wrapper drift, and model-table consistency.
# Usage: scripts/validate-ai-config.sh

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$REPO_ROOT"

errors=0
warnings=0

err() { echo "ERROR: $*" >&2; ((errors++)); }
warn() { echo "WARN:  $*" >&2; ((warnings++)); }

# --- 1. Root AGENTS.md exists ---
if [ ! -f "AGENTS.md" ]; then
  err "Root AGENTS.md does not exist"
fi

# --- 2. Root CLAUDE.md points to AGENTS.md ---
if [ -L "CLAUDE.md" ]; then
  target="$(readlink "CLAUDE.md")"
  if [ "$target" != "AGENTS.md" ]; then
    err "Root CLAUDE.md symlink points to '$target', expected 'AGENTS.md'"
  fi
elif [ -f "CLAUDE.md" ]; then
  # Acceptable if content matches (Windows fallback)
  if ! diff -q "CLAUDE.md" "AGENTS.md" >/dev/null 2>&1; then
    err "Root CLAUDE.md is not a symlink and content differs from AGENTS.md"
  else
    warn "Root CLAUDE.md is a copy, not a symlink (OK for Windows)"
  fi
else
  err "Root CLAUDE.md does not exist"
fi

# --- 3. Every subdirectory with AGENTS.md has a CLAUDE.md ---
while IFS= read -r agents_file; do
  dir="$(dirname "$agents_file")"
  claude_file="$dir/CLAUDE.md"
  if [ ! -e "$claude_file" ]; then
    err "$dir has AGENTS.md but no CLAUDE.md"
  elif [ -L "$claude_file" ]; then
    target="$(readlink "$claude_file")"
    if [ "$target" != "AGENTS.md" ]; then
      err "$claude_file symlink points to '$target', expected 'AGENTS.md'"
    fi
  elif [ -f "$claude_file" ]; then
    if ! diff -q "$claude_file" "$agents_file" >/dev/null 2>&1; then
      err "$claude_file content differs from $agents_file"
    fi
  fi
done < <(find . -name "AGENTS.md" -not -path "./AGENTS.md" -not -path "./.git/*" 2>/dev/null)

# --- 4. Every .claude/rules/*.md has a corresponding .agents/rules/*.md ---
if [ -d ".claude/rules" ]; then
  for rule in .claude/rules/*.md; do
    name="$(basename "$rule")"
    source=".agents/rules/$name"
    if [ ! -f "$source" ]; then
      err ".claude/rules/$name has no corresponding .agents/rules/$name"
    elif [ -L "$rule" ]; then
      target="$(readlink "$rule")"
      expected="../../.agents/rules/$name"
      if [ "$target" != "$expected" ]; then
        err ".claude/rules/$name symlink points to '$target', expected '$expected'"
      fi
    fi
  done
fi

# --- 5. Every .agents/rules/*.md has a corresponding .claude/rules/*.md ---
if [ -d ".agents/rules" ]; then
  for rule in .agents/rules/*.md; do
    name="$(basename "$rule")"
    if [ ! -e ".claude/rules/$name" ]; then
      err ".agents/rules/$name has no corresponding .claude/rules/$name"
    fi
  done
fi

# --- 6. No content divergence (for non-symlink setups) ---
# Already checked above in steps 2-4

# --- 7. Generated wrapper drift check ---
if ! sync_output="$(python3 scripts/sync-agent-configs.py --check 2>&1)"; then
  err "Generated agent wrappers are out of sync (run: python3 scripts/sync-agent-configs.py)"
  echo "$sync_output" >&2
fi

# --- 8. Model table consistency (catalog <-> AGENTS/protocol docs) ---
if ! python3 - <<'PY'
import re
import sys
from pathlib import Path

try:
    import tomllib  # type: ignore[attr-defined]

    def parse_toml(text: str):
        return tomllib.loads(text)
except ModuleNotFoundError:  # pragma: no cover
    import toml  # type: ignore[import-not-found]

    def parse_toml(text: str):
        return toml.loads(text)

root = Path(".")
catalog = parse_toml((root / ".agents" / "agent-catalog.toml").read_text(encoding="utf-8"))
roles = catalog["roles"]
expected_models = {
    role: data["claude"]["model"]
    for role, data in roles.items()
    if data.get("claude_enabled")
}

errors = []

# AGENTS.md table consistency.
agents_text = (root / "AGENTS.md").read_text(encoding="utf-8")
section_split = agents_text.split("### Subagent Dispatch Reference", 1)
if len(section_split) != 2:
    errors.append("AGENTS.md: missing 'Subagent Dispatch Reference' section")
else:
    table_block = section_split[1]
    saw_table_row = False
    for line in table_block.splitlines():
        if not line.strip():
            if saw_table_row:
                break
            continue
        if not line.startswith("|"):
            if saw_table_row:
                break
            if line.strip():
                continue
            continue
        saw_table_row = True
        cols = [c.strip() for c in line.strip().strip("|").split("|")]
        if len(cols) < 2 or cols[0] in {"Role", "------"}:
            continue
        role, model = cols[0], cols[1]
        expected = expected_models.get(role)
        if expected and model != expected:
            errors.append(f"AGENTS.md: role '{role}' model '{model}' != catalog '{expected}'")
    if not saw_table_row:
        errors.append("AGENTS.md: could not parse Subagent Dispatch Reference table")

# Orchestrator protocol tree consistency for explicit role(model) entries.
protocol_text = (root / ".agents" / "rules" / "orchestrator-protocol.md").read_text(
    encoding="utf-8"
)
for match in re.finditer(r"(?:[|+]--)[ \t]+([a-z0-9-]+)[ \t]+\(([^)]+)\)", protocol_text):
    role, model = match.group(1), match.group(2)
    expected = expected_models.get(role)
    if expected and model != expected:
        errors.append(
            f".agents/rules/orchestrator-protocol.md: role '{role}' model '{model}' != catalog '{expected}'"
        )

if errors:
    for error in errors:
        print(error, file=sys.stderr)
    sys.exit(1)
PY
then
  err "Model table consistency check failed"
fi

echo ""
echo "=== AI Config Validation ==="
echo "Errors:   $errors"
echo "Warnings: $warnings"

if [ "$errors" -gt 0 ]; then
  echo "FAILED"
  exit 1
fi

echo "PASSED"
