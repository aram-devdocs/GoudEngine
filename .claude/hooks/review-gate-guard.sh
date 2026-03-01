#!/usr/bin/env bash
# PreToolUse hook: enforce spec-reviewer before code-quality-reviewer gate
set -euo pipefail

REPO_ROOT="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
STATE_DIR="$REPO_ROOT/.claude/state"

INPUT=$(cat)
SESSION_ID=$(echo "$INPUT" | jq -r '.session_id // "unknown"')
SUBAGENT_TYPE=$(echo "$INPUT" | jq -r '.tool_input.subagent_type // empty')

# Only guard code-quality-reviewer dispatch
if [[ "$SUBAGENT_TYPE" != "code-quality-reviewer" ]]; then
  exit 0
fi

# Check for spec-reviewer approval
GATE_FILE="$STATE_DIR/${SESSION_ID}.review-gates"
if [[ ! -f "$GATE_FILE" ]]; then
  echo '{"decision":"block","reason":"GOVERNANCE: Cannot dispatch code-quality-reviewer before spec-reviewer has APPROVED. Run spec-reviewer first."}'
  exit 2
fi

if ! grep -q "spec-reviewer:APPROVED" "$GATE_FILE" 2>/dev/null; then
  CURRENT_STATE=$(cat "$GATE_FILE")
  echo '{"decision":"block","reason":"GOVERNANCE: spec-reviewer has not APPROVED yet. Current review gate state: '"$CURRENT_STATE"'. Run spec-reviewer and get APPROVED before dispatching code-quality-reviewer."}'
  exit 2
fi

exit 0
