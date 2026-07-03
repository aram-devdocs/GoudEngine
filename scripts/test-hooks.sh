#!/usr/bin/env bash
# Fixture-driven test runner for .claude/hooks.
#
# For every hook that has a fixtures/ directory, this:
#   1. resolves the matching <hook>.sh,
#   2. syntax-checks it with `bash -n`,
#   3. replays each fixture's JSON on stdin and asserts the exit code encoded in
#      the fixture filename (expect<N>).
#
# Stateful hooks (context-loader, save-session, completion-check, state-cleanup,
# quality-check) honor CLAUDE_HOOK_* overrides. A throwaway scratch repo is wired
# through those overrides so the tests never touch the real repo, memory, state,
# or source files.
#
# Prints PASS/FAIL per case, a summary line, and exits nonzero if anything fails.
set -uo pipefail

ROOT=$(cd "$(dirname "$0")/.." && pwd)
HOOKS_DIR="$ROOT/.claude/hooks"
FIXTURES_DIR="$HOOKS_DIR/fixtures"

PASS=0
FAIL=0
FAILED_CASES=()

# --- Isolated scratch environment -------------------------------------------
SCRATCH=$(mktemp -d 2>/dev/null || echo "/tmp/goud-hook-test.$$")
cleanup() { rm -rf "$SCRATCH" 2>/dev/null || true; }
trap cleanup EXIT

setup_scratch() {
  mkdir -p "$SCRATCH/src" "$SCRATCH/scripts" \
           "$SCRATCH/.claude/state" "$SCRATCH/.claude/memory" 2>/dev/null || return 0

  git -C "$SCRATCH" init -q 2>/dev/null || return 0
  git -C "$SCRATCH" config user.email test@example.com 2>/dev/null || true
  git -C "$SCRATCH" config user.name "hook test" 2>/dev/null || true

  printf 'fn main() {}\n' > "$SCRATCH/src/foo.rs"
  git -C "$SCRATCH" add -A 2>/dev/null || true
  git -C "$SCRATCH" -c commit.gpgsign=false commit -qm baseline 2>/dev/null || true

  # An unfinished-looking, unformatted tracked change: dirty tree + println! +
  # #[ignore] so completion-check has real triggers and quality-check's rustfmt
  # has something to reformat.
  printf 'fn main(){\nprintln!("debug");\n}\n#[ignore]\nfn t(){}\n' > "$SCRATCH/src/foo.rs"
  # Exercise quality-check's skip branch and a non-code path.
  printf 'fn   x(){}\n' > "$SCRATCH/src/bindings.g.rs"
  printf '# scratch\n' > "$SCRATCH/README.md"
  # Unformatted python for ruff.
  printf 'x=1\n' > "$SCRATCH/scripts/tool.py"

  # Pre-create the marker completion-check's acked fixture expects.
  printf 'advised\n' > "$SCRATCH/.claude/state/stop-ack-acked-sess" 2>/dev/null || true
}

setup_scratch
export CLAUDE_HOOK_PROJECT_DIR="$SCRATCH"
export CLAUDE_HOOK_STATE_DIR="$SCRATCH/.claude/state"
export CLAUDE_HOOK_MEMORY_DIR="$SCRATCH/.claude/memory"

# --- Run fixtures ------------------------------------------------------------
for fixdir in "$FIXTURES_DIR"/*/; do
  [[ -d "$fixdir" ]] || continue
  hook_name=$(basename "$fixdir")
  hook="$HOOKS_DIR/$hook_name.sh"

  if [[ ! -f "$hook" ]]; then
    echo "SKIP  $hook_name (no matching hook script)"
    continue
  fi

  if ! bash -n "$hook" 2>/dev/null; then
    echo "FAIL  $hook_name (bash -n syntax error)"
    FAIL=$((FAIL + 1)); FAILED_CASES+=("$hook_name:syntax")
    continue
  fi

  for fx in "$fixdir"*.json; do
    [[ -f "$fx" ]] || continue
    base=$(basename "$fx")
    want=$(printf '%s' "$base" | grep -oE 'expect[0-9]+' | grep -oE '[0-9]+')
    if [[ -z "$want" ]]; then
      echo "SKIP  $hook_name/$base (no expect<N> in filename)"
      continue
    fi

    got=$(cat "$fx" | bash "$hook" >/dev/null 2>&1; echo $?)

    if [[ "$got" == "$want" ]]; then
      echo "PASS  $hook_name/$base (exit $got)"
      PASS=$((PASS + 1))
    else
      echo "FAIL  $hook_name/$base (want $want, got $got)"
      FAIL=$((FAIL + 1)); FAILED_CASES+=("$hook_name/$base")
    fi
  done
done

echo "--------------------------------------------------"
echo "Total: $((PASS + FAIL))   PASS: $PASS   FAIL: $FAIL"
if [[ $FAIL -gt 0 ]]; then
  echo "Failed: ${FAILED_CASES[*]}"
  exit 1
fi
exit 0
