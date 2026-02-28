#!/usr/bin/env bash
# PreToolUse hook: block root orchestrator from writing implementation files directly
set -euo pipefail

REPO_ROOT="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
STATE_DIR="$REPO_ROOT/.claude/state"

INPUT=$(cat)
SESSION_ID=$(echo "$INPUT" | jq -r '.session_id // "unknown"')
FILE_PATH=$(echo "$INPUT" | jq -r '.tool_input.file_path // .tool_input.file // empty')

# If no file path, allow
if [[ -z "$FILE_PATH" ]]; then
  exit 0
fi

# Read role - if role file doesn't exist or not root, allow
ROLE_FILE="$STATE_DIR/${SESSION_ID}.role"
if [[ ! -f "$ROLE_FILE" ]]; then
  exit 0
fi

ROLE=$(cat "$ROLE_FILE")
if [[ "$ROLE" != "root" ]]; then
  exit 0
fi

# Allow writes to .claude/ directory (config, hooks, agents, rules)
if [[ "$FILE_PATH" == *".claude/"* ]]; then
  exit 0
fi

# Block implementation files
case "$FILE_PATH" in
  *.rs|*.cs|*.py)
    echo '{"decision":"block","reason":"GOVERNANCE: Root orchestrator cannot write implementation files (.rs, .cs, .py) directly. Dispatch a team lead (engine-lead, integration-lead) or quick-fix agent instead."}'
    exit 2
    ;;
esac

exit 0
