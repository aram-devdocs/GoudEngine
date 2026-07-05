#!/usr/bin/env bash
# PostToolUse hook (matcher: Write|Edit|MultiEdit) — best-effort auto-format of
# the file the tool just wrote:
#   * *.rs (not *.g.rs, not under target/)  -> rustfmt --edition 2021
#   * *.py under sdks/python|scripts|codegen -> ruff format (when ruff exists)
#
# Never blocks: formatter failures are swallowed and the hook always exits 0. It
# only prints a short note about what it did.
set -uo pipefail

INPUT=$(cat 2>/dev/null || true)

# Relative paths resolve against the project dir; the env override lets tests
# point at a scratch tree instead of the real repo. Defaults to "." in prod,
# which matches how Claude passes repo-relative paths.
PROJECT_DIR="${CLAUDE_HOOK_PROJECT_DIR:-.}"

# Write/Edit/MultiEdit all carry a single .tool_input.file_path.
FILE=$(printf '%s' "$INPUT" | jq -r '.tool_input.file_path // empty' 2>/dev/null || true)
[[ -z "$FILE" ]] && exit 0

case "$FILE" in
  /*) TARGET="$FILE" ;;
  *)  TARGET="$PROJECT_DIR/$FILE" ;;
esac

[[ -f "$TARGET" ]] || exit 0

NOTE=""
case "$TARGET" in
  *.g.rs)      : ;;   # generated Rust — never hand-format
  */target/*)  : ;;   # build artifacts — leave alone
  *.rs)
    if command -v rustfmt >/dev/null 2>&1; then
      if rustfmt --edition 2021 "$TARGET" >/dev/null 2>&1; then
        NOTE="rustfmt applied to $FILE"
      else
        NOTE="rustfmt skipped (non-fatal) for $FILE"
      fi
    fi
    ;;
esac

case "$TARGET" in
  */sdks/python/*.py|*/scripts/*.py|*/codegen/*.py)
    if command -v ruff >/dev/null 2>&1; then
      if ruff format "$TARGET" >/dev/null 2>&1; then
        NOTE="ruff format applied to $FILE"
      else
        NOTE="ruff format skipped (non-fatal) for $FILE"
      fi
    fi
    ;;
esac

[[ -n "$NOTE" ]] && printf 'quality-check: %s\n' "$NOTE"
exit 0
