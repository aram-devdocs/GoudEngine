#!/usr/bin/env bash
# validate-ai-config.sh — Verify AI config file integrity
# Checks symlinks, content parity, and structural requirements.
# Usage: scripts/validate-ai-config.sh

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$REPO_ROOT"

errors=0
warnings=0

err() { echo "ERROR: $*" >&2; ((errors++)); }
warn() { echo "WARN:  $*" >&2; ((warnings++)); }

# --- 1. Root AGENTS.md exists ---
if [ ! -f "AGENTS.md" ]; then
  err "Root AGENTS.md does not exist"
fi

# --- 2. Root CLAUDE.md points to AGENTS.md ---
if [ -L "CLAUDE.md" ]; then
  target="$(readlink "CLAUDE.md")"
  if [ "$target" != "AGENTS.md" ]; then
    err "Root CLAUDE.md symlink points to '$target', expected 'AGENTS.md'"
  fi
elif [ -f "CLAUDE.md" ]; then
  # Acceptable if content matches (Windows fallback)
  if ! diff -q "CLAUDE.md" "AGENTS.md" >/dev/null 2>&1; then
    err "Root CLAUDE.md is not a symlink and content differs from AGENTS.md"
  else
    warn "Root CLAUDE.md is a copy, not a symlink (OK for Windows)"
  fi
else
  err "Root CLAUDE.md does not exist"
fi

# --- 3. Every subdirectory with AGENTS.md has a CLAUDE.md ---
while IFS= read -r agents_file; do
  dir="$(dirname "$agents_file")"
  claude_file="$dir/CLAUDE.md"
  if [ ! -e "$claude_file" ]; then
    err "$dir has AGENTS.md but no CLAUDE.md"
  elif [ -L "$claude_file" ]; then
    target="$(readlink "$claude_file")"
    if [ "$target" != "AGENTS.md" ]; then
      err "$claude_file symlink points to '$target', expected 'AGENTS.md'"
    fi
  elif [ -f "$claude_file" ]; then
    if ! diff -q "$claude_file" "$agents_file" >/dev/null 2>&1; then
      err "$claude_file content differs from $agents_file"
    fi
  fi
done < <(find . -name "AGENTS.md" -not -path "./AGENTS.md" -not -path "./.git/*" 2>/dev/null)

# --- 4. Every .claude/rules/*.md has a corresponding .agents/rules/*.md ---
if [ -d ".claude/rules" ]; then
  for rule in .claude/rules/*.md; do
    name="$(basename "$rule")"
    source=".agents/rules/$name"
    if [ ! -f "$source" ]; then
      err ".claude/rules/$name has no corresponding .agents/rules/$name"
    elif [ -L "$rule" ]; then
      target="$(readlink "$rule")"
      expected="../../.agents/rules/$name"
      if [ "$target" != "$expected" ]; then
        err ".claude/rules/$name symlink points to '$target', expected '$expected'"
      fi
    fi
  done
fi

# --- 5. Every .agents/rules/*.md has a corresponding .claude/rules/*.md ---
if [ -d ".agents/rules" ]; then
  for rule in .agents/rules/*.md; do
    name="$(basename "$rule")"
    if [ ! -e ".claude/rules/$name" ]; then
      err ".agents/rules/$name has no corresponding .claude/rules/$name"
    fi
  done
fi

# --- 6. No content divergence (for non-symlink setups) ---
# Already checked above in steps 2-4

echo ""
echo "=== AI Config Validation ==="
echo "Errors:   $errors"
echo "Warnings: $warnings"

if [ "$errors" -gt 0 ]; then
  echo "FAILED"
  exit 1
fi

echo "PASSED"
