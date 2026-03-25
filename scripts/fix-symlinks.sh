#!/usr/bin/env bash
# fix-symlinks.sh — Recreate or repair AI config symlinks
# Safe to run on any platform. On systems without symlink support, copies files instead.
# Usage: scripts/fix-symlinks.sh [--quiet] [--check]

set -euo pipefail

CHECK_MODE=0
QUIET=""
for arg in "$@"; do
  case "$arg" in
    --check) CHECK_MODE=1 ;;
    --quiet) QUIET="--quiet" ;;
    *) echo "Usage: $0 [--quiet] [--check]"; exit 1 ;;
  esac
done
REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$REPO_ROOT"

log() {
  [ "$QUIET" = "--quiet" ] || echo "$@"
}

fix_link() {
  local link_path="$1"
  local target="$2"

  if [ -L "$link_path" ] && [ "$(readlink "$link_path")" = "$target" ]; then
    return 0  # Already correct
  fi

  # Remove existing file/broken symlink
  rm -f "$link_path"

  # Try symlink first, fall back to copy
  if ln -s "$target" "$link_path" 2>/dev/null; then
    log "  symlink: $link_path -> $target"
  else
    # Resolve the target relative to the link's directory
    local link_dir
    link_dir="$(dirname "$link_path")"
    cp "$link_dir/$target" "$link_path" 2>/dev/null || {
      log "  WARN: could not create $link_path"
      return 1
    }
    log "  copy: $link_path (from $target)"
  fi
}

verify_link() {
  local link_path="$1"
  local target="$2"

  if [ -L "$link_path" ] && [ "$(readlink "$link_path")" = "$target" ]; then
    return 0
  fi

  log "  MISSING/WRONG: $link_path (expected -> $target)"
  return 1
}

# Select the appropriate function based on mode
if [ "$CHECK_MODE" -eq 1 ]; then
  action() { verify_link "$@"; }
  verb="Checking"
else
  action() { fix_link "$@"; }
  verb="Fixing"
fi

errors=0

# --- Root CLAUDE.md / GEMINI.md -> AGENTS.md ---
log "$verb root symlinks..."
action "CLAUDE.md" "AGENTS.md" || ((errors++))
action "GEMINI.md" "AGENTS.md" || ((errors++))

# --- Subdirectory CLAUDE.md -> AGENTS.md ---
log "$verb subdirectory CLAUDE.md symlinks..."
while IFS= read -r agents_file; do
  dir="$(dirname "$agents_file")"
  claude_file="$dir/CLAUDE.md"
  action "$claude_file" "AGENTS.md" || ((errors++))
done < <(find . -name "AGENTS.md" -not -path "./AGENTS.md" -not -path "./.git/*" 2>/dev/null)

# --- .claude/rules/*.md -> ../../.agents/rules/*.md ---
log "$verb .claude/rules/ symlinks..."
if [ -d ".agents/rules" ]; then
  if [ "$CHECK_MODE" -eq 0 ]; then
    mkdir -p ".claude/rules"
  fi
  for src in .agents/rules/*.md; do
    name="$(basename "$src")"
    action ".claude/rules/$name" "../../.agents/rules/$name" || ((errors++))
  done
fi

# --- .claude/skills/gh-issue -> ../../.agents/skills/gh-issue ---
log "$verb .claude/skills/ symlinks..."
if [ -d ".agents/skills/gh-issue" ]; then
  if [ "$CHECK_MODE" -eq 0 ]; then
    mkdir -p ".claude/skills"
  fi
  action ".claude/skills/gh-issue" "../../.agents/skills/gh-issue" || ((errors++))
fi

if [ "$errors" -gt 0 ]; then
  if [ "$CHECK_MODE" -eq 1 ]; then
    log ""
    log "Found $errors symlink issue(s). Run scripts/fix-symlinks.sh to fix."
  else
    log "Completed with $errors error(s)"
  fi
  exit 1
fi

log "All symlinks OK."
