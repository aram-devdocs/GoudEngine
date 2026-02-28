#!/usr/bin/env bash
# PreToolUse hook: block writes containing secrets or credentials
set -euo pipefail

INPUT=$(cat)
FILE=$(echo "$INPUT" | jq -r '.tool_input.file_path // .tool_input.file // empty')
CONTENT=$(echo "$INPUT" | jq -r '.tool_input.content // .tool_input.new_string // empty')

if [[ -z "$CONTENT" ]]; then
  exit 0
fi

# Skip test fixtures, docs examples, lock files, and skill/rule definitions
case "$FILE" in
  *test_fixtures/*|*test_data/*|*docs/examples/*|*.lock)
    exit 0
    ;;
  */skills/*/SKILL.md|*/agents/*.md|*/rules/*.md|*/rules/*.mdc)
    exit 0
    ;;
esac

PATTERNS=(
  'AKIA[0-9A-Z]{16}'                         # AWS access key
  'aws_secret_access_key\s*=\s*[A-Za-z0-9/+=]{40}'
  'ghp_[A-Za-z0-9]{36}'                      # GitHub personal token
  'gho_[A-Za-z0-9]{36}'                      # GitHub OAuth token
  'github_pat_[A-Za-z0-9_]{82}'              # GitHub fine-grained token
  'sk-[A-Za-z0-9]{48}'                       # OpenAI API key
  'xoxb-[0-9]{10,13}-[A-Za-z0-9-]+'          # Slack bot token
  'xoxp-[0-9]{10,13}-[A-Za-z0-9-]+'          # Slack user token
  '-----BEGIN (RSA |EC |DSA )?PRIVATE KEY-----'
  'password\s*=\s*["\x27][^"\x27]{8,}["\x27]'
  'ANTHROPIC_API_KEY\s*=\s*["\x27]sk-ant-'
)

FOUND=0
for PATTERN in "${PATTERNS[@]}"; do
  if echo "$CONTENT" | grep -qiE "$PATTERN"; then
    echo "✗ BLOCKED: potential secret detected matching pattern: $PATTERN"
    echo "  File: $FILE"
    FOUND=1
  fi
done

if [[ $FOUND -eq 1 ]]; then
  echo ""
  echo "If this is a false positive (test fixture, documentation example),"
  echo "add the file path to the allowlist in .claude/hooks/secret-scanner.sh"
  exit 2
fi

exit 0
