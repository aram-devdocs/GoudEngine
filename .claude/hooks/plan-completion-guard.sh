#!/usr/bin/env bash
# Stop hook: ensure plan is fully executed, changes committed, and CI passes
set -euo pipefail

REPO_ROOT="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
STATE_DIR="$REPO_ROOT/.claude/state"
cd "$REPO_ROOT"

INPUT=$(cat)
SESSION_ID=$(echo "$INPUT" | jq -r '.session_id // "unknown"')

# Prevent infinite loop — only block once per check type
PLAN_STOP_MARKER="$STATE_DIR/${SESSION_ID}.plan_stop_checked"
if [[ -f "$PLAN_STOP_MARKER" ]]; then
  exit 0
fi

ISSUES=""

# --- Check 1: Uncommitted changes on feature branch ---
BRANCH=$(git branch --show-current 2>/dev/null || echo "unknown")
UNCOMMITTED=$(git status --porcelain 2>/dev/null | wc -l | tr -d ' ')

if [[ "$BRANCH" != "main" && "$UNCOMMITTED" -gt 0 ]]; then
  ISSUES="${ISSUES}\n- Uncommitted changes on feature branch '$BRANCH' ($UNCOMMITTED files). Commit before stopping."
fi

# --- Check 2: Does cargo check pass? ---
if ! cargo check 2>/dev/null; then
  ISSUES="${ISSUES}\n- cargo check fails. Fix compilation errors before stopping."
fi

# --- Check 3: Plan file exists with incomplete phases? ---
PLAN_FILE=$(find "$REPO_ROOT/.claude/plans/" -name "*.md" -type f 2>/dev/null | head -1)
if [[ -n "$PLAN_FILE" ]]; then
  # Check for unchecked task items in the plan
  PENDING_TASKS=$(grep -c '^\- \[ \]' "$PLAN_FILE" 2>/dev/null || echo "0")
  if [[ "$PENDING_TASKS" -gt 0 ]]; then
    ISSUES="${ISSUES}\n- Plan file has $PENDING_TASKS incomplete tasks. Finish or acknowledge before stopping."
  fi
fi

# --- Check 4: If on a PR branch, check GitHub Actions ---
if [[ "$BRANCH" != "main" && "$BRANCH" != "unknown" ]]; then
  # Check if branch has a remote and a PR
  if git ls-remote --exit-code origin "$BRANCH" &>/dev/null; then
    PR_NUMBER=$(gh pr list --head "$BRANCH" --json number --jq '.[0].number' 2>/dev/null || echo "")
    if [[ -n "$PR_NUMBER" ]]; then
      # Check CI status on the PR
      CI_STATUS=$(gh pr checks "$PR_NUMBER" --json state --jq '[.[].state] | if all(. == "SUCCESS") then "pass" elif any(. == "FAILURE") then "fail" elif any(. == "PENDING") then "pending" else "unknown" end' 2>/dev/null || echo "unknown")
      case "$CI_STATUS" in
        "fail")
          ISSUES="${ISSUES}\n- GitHub Actions FAILING on PR #$PR_NUMBER. Fix CI before stopping."
          ;;
        "pending")
          ISSUES="${ISSUES}\n- GitHub Actions still RUNNING on PR #$PR_NUMBER. Wait for CI or check back."
          ;;
      esac
    fi
  fi
fi

if [[ -n "$ISSUES" ]]; then
  touch "$PLAN_STOP_MARKER"
  echo "PLAN COMPLETION CHECK:"
  echo -e "$ISSUES"
  echo ""
  echo "Address these issues before ending, or stop again to override."
  exit 2
fi

exit 0
