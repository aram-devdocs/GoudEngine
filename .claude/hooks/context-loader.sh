#!/usr/bin/env bash
# SessionStart hook — surface a compact orientation block for a fresh session:
# current branch, last three commit subjects, uncommitted file count, and a short
# SESSION.md excerpt (only when it is recent, <7 days, so stale notes do not
# mislead). Fails open: any git/read error still exits 0.
set -uo pipefail

INPUT=$(cat 2>/dev/null || true)

# Prefer the test/env override, then the payload cwd, then the project dir.
PROJECT_DIR="${CLAUDE_HOOK_PROJECT_DIR:-}"
[[ -z "$PROJECT_DIR" ]] && PROJECT_DIR=$(printf '%s' "$INPUT" | jq -r '.cwd // empty' 2>/dev/null || true)
[[ -z "$PROJECT_DIR" ]] && PROJECT_DIR="${CLAUDE_PROJECT_DIR:-.}"

MEMORY_DIR="${CLAUDE_HOOK_MEMORY_DIR:-$PROJECT_DIR/.claude/memory}"

BRANCH=$(git -C "$PROJECT_DIR" rev-parse --abbrev-ref HEAD 2>/dev/null || echo "unknown")
COMMITS=$(git -C "$PROJECT_DIR" log -3 --pretty=format:'  - %s' 2>/dev/null || true)
DIRTY=$(git -C "$PROJECT_DIR" status --porcelain 2>/dev/null | grep -c . || true)
[[ -z "$DIRTY" ]] && DIRTY=0

CTX="GoudEngine session context
Branch: $BRANCH
Uncommitted files: $DIRTY
Recent commits:
${COMMITS:-  (none)}"

SESSION="$MEMORY_DIR/SESSION.md"
if [[ -f "$SESSION" ]] && find "$SESSION" -mtime -7 2>/dev/null | grep -q .; then
  EXCERPT=$(head -15 "$SESSION" 2>/dev/null || true)
  if [[ -n "$EXCERPT" ]]; then
    CTX="$CTX

Recent SESSION.md (first 15 lines):
$EXCERPT"
  fi
fi

# Emit as SessionStart additionalContext; fall back to plain stdout if jq fails.
jq -cn --arg ctx "$CTX" \
  '{hookSpecificOutput: {hookEventName: "SessionStart", additionalContext: $ctx}}' \
  2>/dev/null || printf '%s\n' "$CTX"

exit 0
