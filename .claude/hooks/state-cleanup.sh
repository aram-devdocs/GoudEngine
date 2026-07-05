#!/usr/bin/env bash
# SessionEnd hook — best-effort housekeeping for agent state:
#   * remove *.role markers older than 7 days, plus any unknown.role,
#   * prune administrative records for worktrees that no longer exist,
#   * drop now-empty directories under worktrees/.
# Always exits 0; cleanup is never fatal.
set -uo pipefail

INPUT=$(cat 2>/dev/null || true)

PROJECT_DIR="${CLAUDE_HOOK_PROJECT_DIR:-}"
[[ -z "$PROJECT_DIR" ]] && PROJECT_DIR=$(printf '%s' "$INPUT" | jq -r '.cwd // empty' 2>/dev/null || true)
[[ -z "$PROJECT_DIR" ]] && PROJECT_DIR="${CLAUDE_PROJECT_DIR:-.}"

STATE_DIR="${CLAUDE_HOOK_STATE_DIR:-$PROJECT_DIR/.claude/state}"

# Stale role markers (>7 days) and the never-valid unknown.role.
if [[ -d "$STATE_DIR" ]]; then
  find "$STATE_DIR" -maxdepth 1 -name '*.role' -mtime +7 -delete 2>/dev/null || true
  find "$STATE_DIR" -maxdepth 1 -name 'unknown.role' -delete 2>/dev/null || true
fi

# Prune records for worktrees deleted off disk.
git -C "$PROJECT_DIR" worktree prune 2>/dev/null || true

# Remove empty worktree directories left behind after pruning.
if [[ -d "$PROJECT_DIR/worktrees" ]]; then
  find "$PROJECT_DIR/worktrees" -mindepth 1 -type d -empty -delete 2>/dev/null || true
fi

exit 0
