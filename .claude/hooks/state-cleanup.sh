#!/usr/bin/env bash
# SessionEnd hook: clean up session-scoped state files
set -euo pipefail

REPO_ROOT="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
STATE_DIR="$REPO_ROOT/.claude/state"

INPUT=$(cat)
SESSION_ID=$(echo "$INPUT" | jq -r '.session_id // "unknown"')

if [[ "$SESSION_ID" == "unknown" ]]; then
  exit 0
fi

# Remove all state files for this session
rm -f "$STATE_DIR/${SESSION_ID}".* 2>/dev/null || true

exit 0
