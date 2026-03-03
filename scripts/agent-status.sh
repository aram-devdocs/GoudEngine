#!/bin/bash
#
# agent-status.sh
#
# Shows the current state of the autonomous agent pipeline.
# Displays queued, planning, review, approved, in-progress, and blocked issues plus open agent PRs.
#
# Usage:
#   ./scripts/agent-status.sh

set -euo pipefail

REPO="${GITHUB_REPOSITORY:-aram-devdocs/GoudEngine}"

echo "=== Agent Pipeline Status ==="
echo ""

echo "--- Queued (agent-ready) ---"
gh issue list --label "agent-ready" --state open --repo "$REPO" \
  --json number,title,milestone \
  --jq '.[] | "  #\(.number) \(.title) [\(.milestone.title // "no milestone")]"' 2>/dev/null || echo "  (none)"
echo ""

echo "--- Planning (agent-planning) ---"
gh issue list --label "agent-planning" --state open --repo "$REPO" \
  --json number,title,milestone \
  --jq '.[] | "  #\(.number) \(.title) [\(.milestone.title // "no milestone")]"' 2>/dev/null || echo "  (none)"
echo ""

echo "--- Plan Review (agent-plan-review) ---"
gh issue list --label "agent-plan-review" --state open --repo "$REPO" \
  --json number,title,milestone \
  --jq '.[] | "  #\(.number) \(.title) [\(.milestone.title // "no milestone")]"' 2>/dev/null || echo "  (none)"
echo ""

echo "--- Plan Approved (agent-plan-approved) ---"
gh issue list --label "agent-plan-approved" --state open --repo "$REPO" \
  --json number,title,milestone \
  --jq '.[] | "  #\(.number) \(.title) [\(.milestone.title // "no milestone")]"' 2>/dev/null || echo "  (none)"
echo ""

echo "--- In Progress (agent-working) ---"
gh issue list --label "agent-working" --state open --repo "$REPO" \
  --json number,title,milestone \
  --jq '.[] | "  #\(.number) \(.title) [\(.milestone.title // "no milestone")]"' 2>/dev/null || echo "  (none)"
echo ""

echo "--- Blocked (agent-blocked) ---"
gh issue list --label "agent-blocked" --state open --repo "$REPO" \
  --json number,title,milestone \
  --jq '.[] | "  #\(.number) \(.title) [\(.milestone.title // "no milestone")]"' 2>/dev/null || echo "  (none)"
echo ""

echo "--- Open Agent PRs ---"
gh pr list --repo "$REPO" --state open \
  --json number,title,headRefName \
  --jq '.[] | select(.headRefName | startswith("agent/")) | "  PR #\(.number) \(.title) (\(.headRefName))"' 2>/dev/null || echo "  (none)"
echo ""

# Summary counts
ready=$(gh issue list --label "agent-ready" --state open --repo "$REPO" --json number --jq 'length' 2>/dev/null || echo 0)
planning=$(gh issue list --label "agent-planning" --state open --repo "$REPO" --json number --jq 'length' 2>/dev/null || echo 0)
review=$(gh issue list --label "agent-plan-review" --state open --repo "$REPO" --json number --jq 'length' 2>/dev/null || echo 0)
approved=$(gh issue list --label "agent-plan-approved" --state open --repo "$REPO" --json number --jq 'length' 2>/dev/null || echo 0)
working=$(gh issue list --label "agent-working" --state open --repo "$REPO" --json number --jq 'length' 2>/dev/null || echo 0)
blocked=$(gh issue list --label "agent-blocked" --state open --repo "$REPO" --json number --jq 'length' 2>/dev/null || echo 0)

echo "--- Summary ---"
echo "  Queued: $ready | Planning: $planning | Plan Review: $review | Approved: $approved | Executing: $working | Blocked: $blocked"
