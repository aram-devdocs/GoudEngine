#!/usr/bin/env bash
# SessionStart hook: mark session role for governance enforcement
set -euo pipefail

REPO_ROOT="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
STATE_DIR="$REPO_ROOT/.claude/state"
mkdir -p "$STATE_DIR"

INPUT=$(cat)
SESSION_ID=$(echo "$INPUT" | jq -r '.session_id // "unknown"')
AGENT_TYPE=$(echo "$INPUT" | jq -r '.agent_type // empty')

if [[ -z "$AGENT_TYPE" ]]; then
  echo "root" > "$STATE_DIR/${SESSION_ID}.role"
else
  echo "subagent:${AGENT_TYPE}" > "$STATE_DIR/${SESSION_ID}.role"
fi

exit 0
