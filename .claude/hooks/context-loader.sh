#!/usr/bin/env bash
# SessionStart hook: load project context for new sessions
set -euo pipefail

INPUT=$(cat)

REPO_ROOT="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
cd "$REPO_ROOT"

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

echo "=== GoudEngine Session Context ==="
echo ""

echo "## Git State"
BRANCH=$(git branch --show-current 2>/dev/null || echo "detached")
echo "Branch: $BRANCH"
echo ""
echo "Recent commits:"
git log --oneline -5 2>/dev/null || echo "  (no commits)"
echo ""

CHANGES=$(git status --porcelain 2>/dev/null)
if [[ -n "$CHANGES" ]]; then
  echo "Uncommitted changes:"
  echo "$CHANGES" | head -20
  TOTAL=$(echo "$CHANGES" | wc -l | tr -d ' ')
  if [[ "$TOTAL" -gt 20 ]]; then
    echo "  ... and $((TOTAL - 20)) more files"
  fi
  echo ""
fi

echo "## Cargo Workspace"
if [[ -f Cargo.toml ]]; then
  VERSION=$(grep '^version' Cargo.toml | head -1 | sed 's/.*"\(.*\)"/\1/')
  echo "Engine version: $VERSION"
fi
echo ""

if [[ -f .claude/memory/SESSION.md ]]; then
  echo "## Previous Session State"
  cat .claude/memory/SESSION.md
  echo ""
fi

if [[ -d .claude/specs ]]; then
  SPECS=$(find .claude/specs -name "*.md" -type f 2>/dev/null)
  if [[ -n "$SPECS" ]]; then
    echo "## Active Specs"
    for SPEC in $SPECS; do
      echo "  - $SPEC"
    done
    echo ""
  fi
fi

ACTIVE_RUN_DIR=$(find_active_run_dir "$BRANCH")
if [[ -n "$ACTIVE_RUN_DIR" ]]; then
  echo "## Active gh-issue Run"
  echo "Run: $ACTIVE_RUN_DIR"
  if [[ -f "$ACTIVE_RUN_DIR/plan.md" ]]; then
    echo "Plan: $ACTIVE_RUN_DIR/plan.md"
  fi
  if [[ -f "$ACTIVE_RUN_DIR/state.json" ]]; then
    PHASE=$(jq -r '.phase // "unknown"' "$ACTIVE_RUN_DIR/state.json" 2>/dev/null || echo "unknown")
    NEXT_TODO=$(jq -r '.todos[] | select(.status != "done") | "\(.id): \(.title) [\(.owner)]"' "$ACTIVE_RUN_DIR/state.json" 2>/dev/null | head -1)
    echo "Phase: $PHASE"
    if [[ -n "$NEXT_TODO" ]]; then
      echo "Next todo: $NEXT_TODO"
    fi
  fi
  echo ""
fi

echo "=== End Context ==="
