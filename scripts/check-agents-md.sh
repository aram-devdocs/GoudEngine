#!/usr/bin/env bash
# check-agents-md.sh — Validate AGENTS.md length and stale-fact patterns.
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

MAX_LINES=55
errors=0

# Check length of all AGENTS.md files
while IFS= read -r file; do
  lines=$(wc -l < "$file")
  if [ "$lines" -gt "$MAX_LINES" ]; then
    echo "FAIL: $file has $lines lines (max $MAX_LINES)"
    ((errors++))
  fi
done < <(find . -name "AGENTS.md" -not -path "./.git/*" -not -path "./target/*" -not -path "./node_modules/*" -not -path "./worktrees/*" 2>/dev/null)

# Check for stale patterns in all .md files
stale_patterns=(
  'RendererType'
  'OpenGL Game Engine'
)

while IFS= read -r file; do
  for pattern in "${stale_patterns[@]}"; do
    if grep -q "$pattern" "$file" 2>/dev/null; then
      echo "FAIL: $file contains stale pattern: $pattern"
      ((errors++))
    fi
  done
done < <(find . -name "*.md" -not -path "./.git/*" -not -path "./target/*" -not -path "./node_modules/*" -not -path "./docs/book/*" -not -path "./worktrees/*" -not -name "CHANGELOG.md" -not -name "ALPHA_ROADMAP.md" -not -path "./docs/rfcs/*" -not -path "./docs/src/rfcs/*" 2>/dev/null)

if [ "$errors" -gt 0 ]; then
  echo ""
  echo "Found $errors documentation issue(s)."
  exit 1
fi

echo "All AGENTS.md files OK."
