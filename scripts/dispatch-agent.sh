#!/bin/bash
#
# dispatch-agent.sh
#
# Labels issues as "agent-ready" for autonomous agent pickup.
# Supports specific issue numbers or all open issues in a milestone.
#
# Usage:
#   ./scripts/dispatch-agent.sh 123 456              # Specific issues
#   ./scripts/dispatch-agent.sh --milestone alpha-phase-0  # All open in milestone
#   ./scripts/dispatch-agent.sh --dry-run 123         # Preview without labeling
#   ./scripts/dispatch-agent.sh --dry-run --milestone alpha-phase-0

set -euo pipefail

REPO="${GITHUB_REPOSITORY:-aram-devdocs/GoudEngine}"
DRY_RUN=false
MILESTONE=""
ISSUES=()

usage() {
  echo "Usage: $0 [--dry-run] [--milestone <name>] [issue_numbers...]"
  echo ""
  echo "Options:"
  echo "  --dry-run              Preview without labeling"
  echo "  --milestone <name>     Label all open issues in milestone"
  echo ""
  echo "Examples:"
  echo "  $0 123 456                          # Label specific issues"
  echo "  $0 --milestone alpha-phase-0        # Label all in milestone"
  echo "  $0 --dry-run --milestone alpha-phase-0  # Preview"
  exit 1
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --dry-run)
      DRY_RUN=true
      shift
      ;;
    --milestone)
      MILESTONE="$2"
      shift 2
      ;;
    --help|-h)
      usage
      ;;
    *)
      ISSUES+=("$1")
      shift
      ;;
  esac
done

if [[ -z "$MILESTONE" && ${#ISSUES[@]} -eq 0 ]]; then
  echo "ERROR: Provide issue numbers or --milestone"
  usage
fi

label_issue() {
  local issue_number="$1"

  # Check current labels
  local labels
  labels=$(gh issue view "$issue_number" --repo "$REPO" --json labels --jq '.labels[].name' 2>/dev/null || true)

  if echo "$labels" | grep -q "^agent-ready$"; then
    echo "  SKIP #$issue_number (already agent-ready)"
    return
  fi
  if echo "$labels" | grep -q "^agent-working$"; then
    echo "  SKIP #$issue_number (agent-working)"
    return
  fi

  if [[ "$DRY_RUN" == "true" ]]; then
    local title
    title=$(gh issue view "$issue_number" --repo "$REPO" --json title --jq '.title' 2>/dev/null || echo "???")
    echo "  DRY-RUN: would label #$issue_number — $title"
  else
    gh issue edit "$issue_number" --add-label "agent-ready" --repo "$REPO"
    echo "  LABELED #$issue_number as agent-ready"
  fi
}

if [[ -n "$MILESTONE" ]]; then
  echo "Dispatching all open issues in milestone: $MILESTONE"
  issue_numbers=$(gh issue list --milestone "$MILESTONE" --state open --repo "$REPO" --json number --jq '.[].number' 2>/dev/null)

  if [[ -z "$issue_numbers" ]]; then
    echo "  No open issues found in milestone '$MILESTONE'"
    exit 0
  fi

  while IFS= read -r num; do
    label_issue "$num"
  done <<< "$issue_numbers"
else
  echo "Dispatching specific issues: ${ISSUES[*]}"
  for num in "${ISSUES[@]}"; do
    label_issue "$num"
  done
fi

echo "Done."
