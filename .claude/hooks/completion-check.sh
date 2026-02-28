#!/usr/bin/env bash
# Stop hook: advisory check that work is in a good state before session ends
set -euo pipefail

INPUT=$(cat)

REPO_ROOT="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
cd "$REPO_ROOT"

WARNINGS=0

echo "=== Completion Check ==="
echo ""

if ! cargo check 2>/dev/null; then
  echo "⚠ cargo check failed — there may be compilation errors"
  WARNINGS=$((WARNINGS + 1))
fi

MODIFIED_RS=$(git diff --name-only HEAD 2>/dev/null | grep '\.rs$' || true)
MODIFIED_FFI=$(echo "$MODIFIED_RS" | grep 'ffi/' || true)
MODIFIED_SDK=$(git diff --name-only HEAD 2>/dev/null | grep '^sdks/' || true)

if [[ -n "$MODIFIED_FFI" && -z "$MODIFIED_SDK" ]]; then
  echo "⚠ FFI files changed but no SDK files updated:"
  echo "$MODIFIED_FFI" | sed 's/^/    /'
  echo "  Consider updating C# and Python SDK bindings."
  WARNINGS=$((WARNINGS + 1))
fi

UNCOMMITTED=$(git status --porcelain 2>/dev/null | wc -l | tr -d ' ')
if [[ "$UNCOMMITTED" -gt 0 ]]; then
  echo "ℹ $UNCOMMITTED uncommitted file(s) in working tree"
fi

TODO_COUNT=$(grep -rn 'todo!()' goud_engine/src/ --include='*.rs' 2>/dev/null | wc -l | tr -d ' ')
if [[ "$TODO_COUNT" -gt 0 ]]; then
  echo "⚠ Found $TODO_COUNT todo!() macro(s) in engine source"
  WARNINGS=$((WARNINGS + 1))
fi

if [[ $WARNINGS -eq 0 ]]; then
  echo "✓ All checks passed"
else
  echo ""
  echo "⚠ $WARNINGS warning(s) found (advisory only)"
fi

echo ""
echo "=== End Completion Check ==="

# Advisory only — never block session end
exit 0
