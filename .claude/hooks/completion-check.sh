#!/usr/bin/env bash
# Stop hook — a one-time completion gate. On the FIRST Stop of a session it
# blocks (exit 2) when the tree looks unfinished:
#   * uncommitted tracked changes, or
#   * newly added dbg! / println! / todo!() lines in the working diff, or
#   * a newly added #[ignore].
# It writes a per-session marker so the very NEXT Stop passes (exit 0) — the
# session is never trapped. Any error fails open (exit 0).
set -uo pipefail

INPUT=$(cat 2>/dev/null || true)

PROJECT_DIR="${CLAUDE_HOOK_PROJECT_DIR:-}"
[[ -z "$PROJECT_DIR" ]] && PROJECT_DIR=$(printf '%s' "$INPUT" | jq -r '.cwd // empty' 2>/dev/null || true)
[[ -z "$PROJECT_DIR" ]] && PROJECT_DIR="${CLAUDE_PROJECT_DIR:-.}"

SESSION_ID=$(printf '%s' "$INPUT" | jq -r '.session_id // empty' 2>/dev/null || true)
[[ -z "$SESSION_ID" ]] && SESSION_ID="default"

STATE_DIR="${CLAUDE_HOOK_STATE_DIR:-$PROJECT_DIR/.claude/state}"
MARKER="$STATE_DIR/stop-ack-$SESSION_ID"

# Second (or later) Stop for this session — we already advised once, let it end.
[[ -f "$MARKER" ]] && exit 0

# Only reason about changes inside a git work tree; otherwise fail open.
git -C "$PROJECT_DIR" rev-parse --is-inside-work-tree >/dev/null 2>&1 || exit 0

REASONS=()

# Uncommitted tracked changes (staged or unstaged) relative to HEAD.
if [[ -n "$(git -C "$PROJECT_DIR" diff --name-only HEAD 2>/dev/null)" ]]; then
  REASONS+=("uncommitted tracked changes present")
fi

# Added lines (leading + but not the +++ file header) in the working diff.
ADDED=$(git -C "$PROJECT_DIR" diff HEAD 2>/dev/null | grep -E '^\+' | grep -vE '^\+\+\+' || true)
if printf '%s' "$ADDED" | grep -qE '(dbg!|println!|todo!\(\))'; then
  REASONS+=("new dbg!/println!/todo!() in changed code")
fi
if printf '%s' "$ADDED" | grep -qE '#\[ignore\]'; then
  REASONS+=("a #[ignore] was added to changed code")
fi

# Clean enough — allow the stop.
[[ ${#REASONS[@]} -eq 0 ]] && exit 0

# Record the advisory first so this only ever blocks once, then block.
mkdir -p "$STATE_DIR" 2>/dev/null || exit 0
printf 'advised %s\n' "$SESSION_ID" > "$MARKER" 2>/dev/null || true

{
  echo "COMPLETION CHECK: this session looks unfinished:"
  for r in "${REASONS[@]}"; do echo "  - $r"; done
  echo "Commit the work, remove debug/placeholder lines, or confirm intentionally,"
  echo "then stop again — this advisory only blocks once."
} >&2
exit 2
