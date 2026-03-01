#!/usr/bin/env bash
# PostToolUse hook: auto-lint edited files based on extension
#
# Uses fast, file-scoped checks only. Full cargo clippy is deferred to
# pre-commit hooks and CI — running it on every edit is too expensive.
set -euo pipefail

INPUT=$(cat)
FILE=$(echo "$INPUT" | jq -r '.tool_input.file_path // .tool_input.file // empty')
if [[ -z "$FILE" ]]; then
  exit 0
fi

EXT="${FILE##*.}"
EXIT_CODE=0

case "$EXT" in
  rs)
    if command -v rustfmt &>/dev/null && [[ -f "$FILE" ]]; then
      if ! rustfmt --check "$FILE" 2>/dev/null; then
        echo "⚠ rustfmt: formatting issues in $FILE"
        EXIT_CODE=1
      fi
    fi
    if command -v cargo &>/dev/null; then
      if ! cargo check --message-format=short 2>&1 | grep -q "^error"; then
        :
      else
        echo "✗ cargo check: compilation errors detected"
        cargo check --message-format=short 2>&1 | grep "^error" | head -10
        EXIT_CODE=2
      fi
    fi
    ;;
  cs)
    if command -v dotnet &>/dev/null && [[ -f "$FILE" ]]; then
      if ! dotnet format --verify-no-changes --include "$FILE" 2>/dev/null; then
        echo "⚠ dotnet format: formatting issues in $FILE"
        EXIT_CODE=1
      fi
    fi
    ;;
  py)
    if command -v ruff &>/dev/null && [[ -f "$FILE" ]]; then
      RUFF_OUT=$(ruff check "$FILE" 2>&1 || true)
      if [[ -n "$RUFF_OUT" ]]; then
        echo "⚠ ruff: issues in $FILE"
        echo "$RUFF_OUT" | head -10
        EXIT_CODE=1
      fi
    fi
    ;;
esac

exit $EXIT_CODE
