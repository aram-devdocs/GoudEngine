#!/usr/bin/env bash
# Stop hook: verify governance was followed before session ends
set -euo pipefail

REPO_ROOT="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
STATE_DIR="$REPO_ROOT/.claude/state"

INPUT=$(cat)
SESSION_ID=$(echo "$INPUT" | jq -r '.session_id // "unknown"')

# Prevent infinite loop — only block once
STOP_MARKER="$STATE_DIR/${SESSION_ID}.stop_hook_active"
if [[ -f "$STOP_MARKER" ]]; then
  exit 0
fi

# Check if implementation files were changed
CHANGED_FILES=$(git diff --name-only HEAD 2>/dev/null || true)
IMPL_FILES=$(echo "$CHANGED_FILES" | grep -E '\.(rs|cs|py)$' | grep -v '.claude/' || true)

if [[ -z "$IMPL_FILES" ]]; then
  # No implementation files changed, no governance needed
  exit 0
fi

VIOLATIONS=""

# Check 1: Were subagents dispatched?
DELEGATION_LOG="$STATE_DIR/${SESSION_ID}.delegations"
if [[ ! -f "$DELEGATION_LOG" ]] || [[ ! -s "$DELEGATION_LOG" ]]; then
  VIOLATIONS="${VIOLATIONS}\n- Implementation files changed but no subagents were dispatched (delegation-first violation)"
fi

# Check 2: Did review gates pass?
GATE_FILE="$STATE_DIR/${SESSION_ID}.review-gates"
if [[ ! -f "$GATE_FILE" ]]; then
  VIOLATIONS="${VIOLATIONS}\n- Implementation files changed but no review gates were completed"
elif ! grep -q "APPROVED" "$GATE_FILE" 2>/dev/null; then
  VIOLATIONS="${VIOLATIONS}\n- Review gates exist but none show APPROVED status"
fi

# Check 3: If FFI files changed, was security-auditor dispatched?
FFI_FILES=$(echo "$IMPL_FILES" | grep 'ffi/' || true)
if [[ -n "$FFI_FILES" ]]; then
  if [[ -f "$DELEGATION_LOG" ]] && ! grep -q "security-auditor" "$DELEGATION_LOG" 2>/dev/null; then
    VIOLATIONS="${VIOLATIONS}\n- FFI files changed but security-auditor was not dispatched"
  fi
fi

if [[ -n "$VIOLATIONS" ]]; then
  # Mark that we've blocked once to prevent infinite loops
  touch "$STOP_MARKER"
  echo "GOVERNANCE VIOLATIONS DETECTED:"
  echo -e "$VIOLATIONS"
  echo ""
  echo "Implementation files changed:"
  echo "$IMPL_FILES" | sed 's/^/  /'
  echo ""
  echo "Address these violations before ending the session, or run the session stop again to override."
  exit 2
fi

exit 0
