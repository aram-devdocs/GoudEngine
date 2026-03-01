#!/usr/bin/env bash
# SubagentStop hook: track review gate verdicts
set -euo pipefail

REPO_ROOT="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
STATE_DIR="$REPO_ROOT/.claude/state"
mkdir -p "$STATE_DIR"

INPUT=$(cat)
SESSION_ID=$(echo "$INPUT" | jq -r '.session_id // "unknown"')
AGENT_TYPE=$(echo "$INPUT" | jq -r '.agent_type // empty')
LAST_MESSAGE=$(echo "$INPUT" | jq -r '.last_assistant_message // empty')

if [[ -z "$AGENT_TYPE" || -z "$LAST_MESSAGE" ]]; then
  exit 0
fi

GATE_FILE="$STATE_DIR/${SESSION_ID}.review-gates"

# Extract verdict from the last message
VERDICT=""
if echo "$LAST_MESSAGE" | grep -qi "APPROVED"; then
  VERDICT="APPROVED"
elif echo "$LAST_MESSAGE" | grep -qi "REJECTED"; then
  VERDICT="REJECTED"
elif echo "$LAST_MESSAGE" | grep -qi "CHANGES REQUESTED"; then
  VERDICT="CHANGES REQUESTED"
fi

if [[ -n "$VERDICT" ]]; then
  echo "${AGENT_TYPE}:${VERDICT}" >> "$GATE_FILE"
fi

exit 0
