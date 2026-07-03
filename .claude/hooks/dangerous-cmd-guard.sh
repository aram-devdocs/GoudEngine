#!/usr/bin/env bash
# PreToolUse hook: block destructive or dangerous shell commands
set -euo pipefail

INPUT=$(cat)
CMD=$(echo "$INPUT" | jq -r '.tool_input.command // empty')

if [[ -z "$CMD" ]]; then
  exit 0
fi

BLOCKED_PATTERNS=(
  # Destructive removals of true system roots or the bare home dir. Project-path
  # deletes (including absolute paths under /Users, /home, /private/tmp) are
  # allowed — only the enumerated dangerous roots (and their subpaths) match.
  'rm\s+-rf\s+(--no-preserve-root\s+)?/\s*($|;)'
  'rm\s+-rf\s+(--no-preserve-root\s+)?/(usr|etc|bin|sbin|lib|var|boot|dev|sys|proc|opt|root|System|Library|Applications|Volumes)(/|\s|$)'
  'rm\s+-rf\s+~(\s|$)'
  'rm\s+-rf\s+\$HOME(\s|$)'
  'rm\s+-rf\s+\.\s*(\*|$)'
  # Force-push to protected branches
  'git\s+push\s+.*(--force|-f)\b.*\b(main|master)\b'
  'git\s+push\s+.*(--force|-f)\s*$'
  # Bypassing the verification gate is never allowed (see CONTRIBUTING)
  '--no-verify\b'
  '\[skip[ -]ci\]'
  # Publishing is release-automation's job, not an agent's
  'cargo\s+publish'
  # Privilege escalation and pipe-to-shell installs
  'sudo\s'
  '(curl|wget)\b[^|]*\|\s*(ba|z|k)?sh\b'
  # Destructive DB ops
  'DROP\s+TABLE'
  'DROP\s+DATABASE'
  'TRUNCATE\s+TABLE'
  # Broad permission and disk-destroying commands
  'chmod\s+-R\s+777'
  ':\(\)\{\s*:\|:&\s*\};:'
  'mkfs\.'
  'dd\s+if=.*of=/dev/'
  '>\s*/dev/sd[a-z]'
)

for PATTERN in "${BLOCKED_PATTERNS[@]}"; do
  # -e marks the pattern explicitly so entries beginning with `-` (e.g. --no-verify)
  # are not misparsed as grep options.
  if echo "$CMD" | grep -qiE -e "$PATTERN"; then
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
