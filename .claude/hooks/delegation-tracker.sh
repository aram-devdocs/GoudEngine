#!/usr/bin/env bash
# PostToolUse hook: log subagent dispatches for governance audit trail
set -euo pipefail

REPO_ROOT="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
STATE_DIR="$REPO_ROOT/.claude/state"
mkdir -p "$STATE_DIR"

INPUT=$(cat)
SESSION_ID=$(echo "$INPUT" | jq -r '.session_id // "unknown"')
AGENT_TYPE=$(echo "$INPUT" | jq -r '.tool_input.subagent_type // empty')
PROMPT=$(echo "$INPUT" | jq -r '.tool_input.prompt // empty' | head -c 200)
TIMESTAMP=$(date -u '+%Y-%m-%dT%H:%M:%SZ')

if [[ -z "$AGENT_TYPE" ]]; then
  exit 0
fi

LOG_FILE="$STATE_DIR/${SESSION_ID}.delegations"
echo "${TIMESTAMP} | ${AGENT_TYPE} | ${PROMPT}" >> "$LOG_FILE"

exit 0
