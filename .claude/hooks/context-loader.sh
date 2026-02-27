#!/usr/bin/env bash
# SessionStart hook: load project context for new sessions
set -euo pipefail

REPO_ROOT="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
cd "$REPO_ROOT"

echo "=== GoudEngine Session Context ==="
echo ""

echo "## Git State"
BRANCH=$(git branch --show-current 2>/dev/null || echo "detached")
echo "Branch: $BRANCH"
echo ""
echo "Recent commits:"
git log --oneline -5 2>/dev/null || echo "  (no commits)"
echo ""

CHANGES=$(git status --porcelain 2>/dev/null)
if [[ -n "$CHANGES" ]]; then
  echo "Uncommitted changes:"
  echo "$CHANGES" | head -20
  TOTAL=$(echo "$CHANGES" | wc -l | tr -d ' ')
  if [[ "$TOTAL" -gt 20 ]]; then
    echo "  ... and $((TOTAL - 20)) more files"
  fi
  echo ""
fi

echo "## Cargo Workspace"
if [[ -f Cargo.toml ]]; then
  VERSION=$(grep '^version' Cargo.toml | head -1 | sed 's/.*"\(.*\)"/\1/')
  echo "Engine version: $VERSION"
fi
echo ""

if [[ -f .claude/memory/SESSION.md ]]; then
  echo "## Previous Session State"
  cat .claude/memory/SESSION.md
  echo ""
fi

if [[ -d .claude/specs ]]; then
  SPECS=$(find .claude/specs -name "*.md" -type f 2>/dev/null)
  if [[ -n "$SPECS" ]]; then
    echo "## Active Specs"
    for SPEC in $SPECS; do
      echo "  - $SPEC"
    done
    echo ""
  fi
fi

echo "=== End Context ==="
