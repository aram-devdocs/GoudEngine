#!/usr/bin/env bash
# Stop hook: ensure plan is fully executed, changes committed, and CI passes
set -euo pipefail

REPO_ROOT="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
STATE_DIR="$REPO_ROOT/.claude/state"
cd "$REPO_ROOT"

INPUT=$(cat)
SESSION_ID=$(echo "$INPUT" | jq -r '.session_id // "unknown"')

PLAN_STOP_MARKER="$STATE_DIR/${SESSION_ID}.plan_stop_checked"
if [[ -f "$PLAN_STOP_MARKER" ]]; then
  exit 0
fi

find_active_run_dir() {
  local branch="$1"
  local base="$REPO_ROOT/.agents/runs/gh-issue"
  local fallback=""
  [[ -d "$base" ]] || return 0

  while IFS= read -r state_file; do
    local state_branch
    local phase
    state_branch="$(jq -r '.branch // empty' "$state_file" 2>/dev/null || echo "")"
    phase="$(jq -r '.phase // empty' "$state_file" 2>/dev/null || echo "")"
    case "$phase" in
      done|complete|cleanup-complete|abandoned)
        continue
        ;;
    esac

    if [[ -n "$branch" ]] && [[ "$branch" != "detached" ]] && [[ "$state_branch" == "$branch" ]]; then
      dirname "$state_file"
      return 0
    fi

    if [[ -z "$fallback" ]]; then
      fallback="$(dirname "$state_file")"
    fi
  done < <(find "$base" -name state.json -type f 2>/dev/null | sort)

  if [[ -z "$branch" || "$branch" == "detached" || "$branch" == "unknown" ]]; then
    printf '%s\n' "$fallback"
  fi
}

ISSUES=""
BRANCH=$(git branch --show-current 2>/dev/null || echo "unknown")
UNCOMMITTED=$(git status --porcelain 2>/dev/null | wc -l | tr -d ' ')

if [[ "$BRANCH" != "main" && "$UNCOMMITTED" -gt 0 ]]; then
  ISSUES="${ISSUES}\n- Uncommitted changes on feature branch '$BRANCH' ($UNCOMMITTED files). Commit before stopping."
fi

if ! cargo check 2>/dev/null; then
  ISSUES="${ISSUES}\n- cargo check fails. Fix compilation errors before stopping."
fi

ACTIVE_RUN_DIR=$(find_active_run_dir "$BRANCH")
PLAN_FILE=""
STATE_FILE=""
if [[ -n "$ACTIVE_RUN_DIR" ]]; then
  PLAN_FILE="$ACTIVE_RUN_DIR/plan.md"
  STATE_FILE="$ACTIVE_RUN_DIR/state.json"
else
  PLAN_FILE=$(find "$REPO_ROOT/.claude/plans/" -name "*.md" -type f 2>/dev/null | head -1)
fi

if [[ -n "$PLAN_FILE" && -f "$PLAN_FILE" ]]; then
  PENDING_TASKS=$(grep -c '^\- \[ \]' "$PLAN_FILE" 2>/dev/null || echo "0")
  if [[ "$PENDING_TASKS" -gt 0 ]]; then
    ISSUES="${ISSUES}\n- Plan file has $PENDING_TASKS incomplete tasks. Finish or acknowledge before stopping."
  fi
fi

if [[ -n "$STATE_FILE" && -f "$STATE_FILE" ]]; then
  PENDING_TODOS=$(jq '[.todos[] | select(.status != "done")] | length' "$STATE_FILE" 2>/dev/null || echo "0")
  if [[ "$PENDING_TODOS" -gt 0 ]]; then
    ISSUES="${ISSUES}\n- Shared gh-issue state still has $PENDING_TODOS incomplete todo(s)."
  fi

  PR_NUMBER=$(jq -r '.pr.number // empty' "$STATE_FILE" 2>/dev/null || echo "")
  STORED_CI=$(jq -r '.pr.ci_state // empty' "$STATE_FILE" 2>/dev/null || echo "")
  STORED_REVIEW=$(jq -r '.pr.review_state // empty' "$STATE_FILE" 2>/dev/null || echo "")
  if [[ -n "$PR_NUMBER" ]]; then
    if [[ "$STORED_CI" == "failure" ]]; then
      ISSUES="${ISSUES}\n- Shared gh-issue state shows CI failing on PR #$PR_NUMBER."
    elif [[ "$STORED_CI" == "pending" ]]; then
      ISSUES="${ISSUES}\n- Shared gh-issue state shows CI still pending on PR #$PR_NUMBER."
    fi
    if [[ "$STORED_REVIEW" == "changes_requested" || "$STORED_REVIEW" == "commented" ]]; then
      ISSUES="${ISSUES}\n- Shared gh-issue state shows review feedback still pending on PR #$PR_NUMBER."
    fi
  fi
fi

if [[ "$BRANCH" != "main" && "$BRANCH" != "unknown" ]]; then
  if git ls-remote --exit-code origin "$BRANCH" &>/dev/null; then
    LIVE_PR=$(gh pr list --head "$BRANCH" --json number --jq '.[0].number' 2>/dev/null || echo "")
    if [[ -n "$LIVE_PR" ]]; then
      CI_STATUS=$(gh pr checks "$LIVE_PR" --json state --jq '[.[].state] | if all(. == "SUCCESS") then "pass" elif any(. == "FAILURE") then "fail" elif any(. == "PENDING") then "pending" else "unknown" end' 2>/dev/null || echo "unknown")
      LIVE_REVIEW=$(gh pr view "$LIVE_PR" --json reviews --jq '[.reviews[].state] | if any(. == "CHANGES_REQUESTED") then "changes_requested" elif any(. == "COMMENTED") then "commented" elif any(. == "APPROVED") then "approved" else "no_reviews" end' 2>/dev/null || echo "unknown")
      case "$CI_STATUS" in
        "fail")
          ISSUES="${ISSUES}\n- GitHub Actions FAILING on PR #$LIVE_PR. Fix CI before stopping."
          ;;
        "pending")
          ISSUES="${ISSUES}\n- GitHub Actions still RUNNING on PR #$LIVE_PR. Wait for CI or check back."
          ;;
      esac
      case "$LIVE_REVIEW" in
        "changes_requested"|"commented")
          ISSUES="${ISSUES}\n- GitHub review feedback is still pending on PR #$LIVE_PR."
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
