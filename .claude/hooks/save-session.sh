#!/usr/bin/env bash
# PreCompact hook: persist session state before context compaction
set -euo pipefail

INPUT=$(cat)

REPO_ROOT="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
cd "$REPO_ROOT"

mkdir -p .claude/memory

BRANCH=$(git branch --show-current 2>/dev/null || echo "detached")
TIMESTAMP=$(date -u '+%Y-%m-%dT%H:%M:%SZ')

MODIFIED_FILES=$(git diff --name-only HEAD 2>/dev/null || true)
STAGED_FILES=$(git diff --cached --name-only 2>/dev/null || true)
UNTRACKED_FILES=$(git ls-files --others --exclude-standard 2>/dev/null | head -20 || true)

cat > .claude/memory/SESSION.md << EOF
# Session Memory - GoudEngine

## Current State
- Branch: $BRANCH
- Last checkpoint: $TIMESTAMP

## Modified Files (unstaged)
$(if [[ -n "$MODIFIED_FILES" ]]; then echo "$MODIFIED_FILES" | sed 's/^/- /'; else echo "- (none)"; fi)

## Staged Files
$(if [[ -n "$STAGED_FILES" ]]; then echo "$STAGED_FILES" | sed 's/^/- /'; else echo "- (none)"; fi)

## Untracked Files
$(if [[ -n "$UNTRACKED_FILES" ]]; then echo "$UNTRACKED_FILES" | sed 's/^/- /'; else echo "- (none)"; fi)

## Recent Commits
$(git log --oneline -5 2>/dev/null || echo "- (no commits)")

## Next Steps
- (not captured automatically — check git diff and recent commits for context)

## Blockers
- (not captured automatically — review above sections for state)
EOF

echo "Session state saved to .claude/memory/SESSION.md"
