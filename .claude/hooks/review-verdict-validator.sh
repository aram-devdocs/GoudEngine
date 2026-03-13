#!/usr/bin/env bash
# SubagentStop hook: ensure reviewer outputs end with a clear verdict.
set -euo pipefail

INPUT=$(cat)
AGENT_TYPE=$(echo "$INPUT" | jq -r '.agent_type // empty')
LAST_MESSAGE=$(echo "$INPUT" | jq -r '.last_assistant_message // empty')

case "$AGENT_TYPE" in
  reviewer|spec-reviewer|code-quality-reviewer|security-auditor) ;;
  *) exit 0 ;;
esac

if [[ -z "$LAST_MESSAGE" ]]; then
  echo "GOVERNANCE: $AGENT_TYPE must produce a verdict."
  exit 2
fi

if echo "$LAST_MESSAGE" | grep -Eqi '(^|[^A-Z])(APPROVED|REJECTED|CHANGES REQUESTED)($|[^A-Z])'; then
  exit 0
fi

echo "GOVERNANCE: $AGENT_TYPE must end with APPROVED, REJECTED, or CHANGES REQUESTED."
exit 2
