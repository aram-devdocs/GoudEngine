#!/usr/bin/env bash
# PreCompact hook: persist session state before context compaction
set -euo pipefail

INPUT=$(cat)

REPO_ROOT="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
cd "$REPO_ROOT"

mkdir -p .claude/memory

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

BRANCH=$(git branch --show-current 2>/dev/null || echo "detached")
TIMESTAMP=$(date -u '+%Y-%m-%dT%H:%M:%SZ')

MODIFIED_FILES=$(git diff --name-only HEAD 2>/dev/null || true)
STAGED_FILES=$(git diff --cached --name-only 2>/dev/null || true)
UNTRACKED_FILES=$(git ls-files --others --exclude-standard 2>/dev/null | head -20 || true)
ACTIVE_RUN_DIR=$(find_active_run_dir "$BRANCH")
ACTIVE_RUN_SECTION="- (none)"
NEXT_TODO_SECTION="- (none)"
if [[ -n "$ACTIVE_RUN_DIR" ]] && [[ -f "$ACTIVE_RUN_DIR/state.json" ]]; then
  ACTIVE_RUN_SECTION=$(cat <<EOT
- Run: $ACTIVE_RUN_DIR
- Phase: $(jq -r '.phase // "unknown"' "$ACTIVE_RUN_DIR/state.json" 2>/dev/null || echo "unknown")
- PR: $(jq -r '.pr.number // "none"' "$ACTIVE_RUN_DIR/state.json" 2>/dev/null || echo "none")
EOT
)
  NEXT_TODO=$(jq -r '.todos[] | select(.status != "done") | "- \(.id): \(.title) [\(.owner)]"' "$ACTIVE_RUN_DIR/state.json" 2>/dev/null | head -1)
  if [[ -n "$NEXT_TODO" ]]; then
    NEXT_TODO_SECTION="$NEXT_TODO"
  fi
fi

cat > .claude/memory/SESSION.md <<EOT
# Session Memory - GoudEngine

## Current State
- Branch: $BRANCH
- Last checkpoint: $TIMESTAMP

## Active gh-issue Run
$ACTIVE_RUN_SECTION

## Modified Files (unstaged)
$(if [[ -n "$MODIFIED_FILES" ]]; then echo "$MODIFIED_FILES" | sed 's/^/- /'; else echo "- (none)"; fi)

## Staged Files
$(if [[ -n "$STAGED_FILES" ]]; then echo "$STAGED_FILES" | sed 's/^/- /'; else echo "- (none)"; fi)

## Untracked Files
$(if [[ -n "$UNTRACKED_FILES" ]]; then echo "$UNTRACKED_FILES" | sed 's/^/- /'; else echo "- (none)"; fi)

## Recent Commits
$(git log --oneline -5 2>/dev/null || echo "- (no commits)")

## Next Steps
$NEXT_TODO_SECTION

## Blockers
- (not captured automatically — review above sections for state)
EOT

echo "Session state saved to .claude/memory/SESSION.md"
