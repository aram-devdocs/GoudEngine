#!/usr/bin/env bash
# PreToolUse hook: block destructive or dangerous shell commands
set -euo pipefail

CMD="${TOOL_INPUT_COMMAND:-}"

if [[ -z "$CMD" ]]; then
  exit 0
fi

BLOCKED_PATTERNS=(
  'rm\s+-rf\s+/'
  'rm\s+-rf\s+~'
  'rm\s+-rf\s+\$HOME'
  'git\s+push\s+.*--force\s+.*main'
  'git\s+push\s+.*--force\s+.*master'
  'git\s+push\s+-f\s+.*main'
  'git\s+push\s+-f\s+.*master'
  'cargo\s+publish'
  'DROP\s+TABLE'
  'DROP\s+DATABASE'
  'TRUNCATE\s+TABLE'
  'chmod\s+-R\s+777'
  ':\(\)\{\s*:\|:&\s*\};:'
  'mkfs\.'
  'dd\s+if=.*of=/dev/'
  'cargo\s+publish\s+--no-verify'
  'git\s+push\s+--force'
  'git\s+push\s+-f'
  'git\s+push\s+.*--force\s+.*main'
  'git\s+push\s+.*--force\s+.*master'
  'git\s+push\s+-f\s+.*main'
  'git\s+push\s+-f\s+.*master'
)

for PATTERN in "${BLOCKED_PATTERNS[@]}"; do
  if echo "$CMD" | grep -qiE "$PATTERN"; then
    echo "✗ BLOCKED: dangerous command detected"
    echo "  Command: $CMD"
    echo "  Pattern: $PATTERN"
    echo ""
    echo "This command has been blocked by the dangerous-cmd-guard hook."
    echo "If you believe this is safe, run it manually outside the agent."
    exit 2
  fi
done

exit 0
