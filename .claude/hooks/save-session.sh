#!/usr/bin/env bash
# PreCompact hook — checkpoint a resumable SESSION.md before context is
# compacted. Every section is rewritten from a fixed skeleton except "## Log:",
# which is append-only so history accumulates across compactions. Uses the short
# commit hash (never date/time, which may be unavailable) as the checkpoint
# marker. Fails open: exits 0 even if the write fails.
set -uo pipefail

INPUT=$(cat 2>/dev/null || true)

PROJECT_DIR="${CLAUDE_HOOK_PROJECT_DIR:-}"
[[ -z "$PROJECT_DIR" ]] && PROJECT_DIR=$(printf '%s' "$INPUT" | jq -r '.cwd // empty' 2>/dev/null || true)
[[ -z "$PROJECT_DIR" ]] && PROJECT_DIR="${CLAUDE_PROJECT_DIR:-.}"

MEMORY_DIR="${CLAUDE_HOOK_MEMORY_DIR:-$PROJECT_DIR/.claude/memory}"
SESSION="$MEMORY_DIR/SESSION.md"

mkdir -p "$MEMORY_DIR" 2>/dev/null || exit 0

BRANCH=$(git -C "$PROJECT_DIR" rev-parse --abbrev-ref HEAD 2>/dev/null || echo "unknown")
SHA=$(git -C "$PROJECT_DIR" rev-parse --short HEAD 2>/dev/null || echo "nocommit")
SUBJECT=$(git -C "$PROJECT_DIR" log -1 --pretty=format:'%s' 2>/dev/null || echo "(no commits)")

# Preserve prior "## Log:" lines so the log stays append-only across runs.
PRIOR_LOG=""
if [[ -f "$SESSION" ]]; then
  PRIOR_LOG=$(awk 'f{print} /^## Log:/{f=1}' "$SESSION" 2>/dev/null || true)
fi

{
  printf '# GoudEngine Session Memory\n\n'
  printf 'Approved plan: <link or path to the approved plan>\n\n'
  printf '## Definition of Done\n'
  printf -- '- [ ] Implementation complete\n'
  printf -- '- [ ] Verification / codegen green\n'
  printf -- '- [ ] Reviewed and merged\n\n'
  printf '## Phase status:\n'
  printf -- '- Branch: %s\n' "$BRANCH"
  printf -- '- Last commit: %s %s\n\n' "$SHA" "$SUBJECT"
  printf 'Resume order: re-read this file, confirm the branch, then continue from the first unchecked Definition of Done item.\n\n'
  printf '## Log:\n'
  [[ -n "$PRIOR_LOG" ]] && printf '%s\n' "$PRIOR_LOG"
  printf -- '- checkpoint at %s (%s)\n' "$SHA" "$BRANCH"
} > "$SESSION" 2>/dev/null || exit 0

exit 0
