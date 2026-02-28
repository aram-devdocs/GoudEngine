#!/usr/bin/env bash
# SubagentStop hook: ensure reviewers produce a clear verdict
set -euo pipefail

INPUT=$(cat)
AGENT_TYPE=$(echo "$INPUT" | jq -r '.agent_type // empty')
LAST_MESSAGE=$(echo "$INPUT" | jq -r '.last_assistant_message // empty')

# Only validate reviewer agents
case "$AGENT_TYPE" in
  spec-reviewer|code-quality-reviewer) ;;
  *) exit 0 ;;
esac

if [[ -z "$LAST_MESSAGE" ]]; then
  echo "GOVERNANCE: Reviewer ($AGENT_TYPE) must produce a verdict. No output detected."
  exit 2
fi

# Check for a clear verdict
HAS_VERDICT=false
if echo "$LAST_MESSAGE" | grep -qi "APPROVED"; then
  HAS_VERDICT=true
elif echo "$LAST_MESSAGE" | grep -qi "REJECTED"; then
  HAS_VERDICT=true
elif echo "$LAST_MESSAGE" | grep -qi "CHANGES REQUESTED"; then
  HAS_VERDICT=true
fi

if [[ "$HAS_VERDICT" != "true" ]]; then
  echo "GOVERNANCE: Reviewer ($AGENT_TYPE) must end with a clear verdict: APPROVED, REJECTED, or CHANGES REQUESTED. Review output did not contain a verdict."
  exit 2
fi

exit 0
