#!/usr/bin/env bash
# Canonical verification gate for GoudEngine.
#
# One data-driven step table drives local git hooks AND CI, so "passes locally"
# means "passes CI". The pre-commit and pre-push hooks and the CI workflows all
# route through this script instead of hand-mirroring a step list.
#
# Usage:
#   scripts/verify.sh                 Full gate (pre-push / CI default)
#   scripts/verify.sh --staged        Fast subset (pre-commit); strict subset of full
#   scripts/verify.sh --lane rust     Only steps tagged with a lane (CI job scoping)
#   scripts/verify.sh --list          Print the step table as TSV (for tooling)
#   scripts/verify.sh --list --staged Print only the staged subset
#
# Step tiers:
#   block     — must pass; a failure fails the gate
#   advisory  — always non-fatal; prints findings but never blocks (lets a
#               nascent check surface signal before it is promoted to blocking)
#
# Modes:
#   both  — runs in --staged and full (guarantees staged ⊆ full by construction)
#   full  — runs only in the full gate
set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
# --list must be cheap; skip the cargo-metadata call when only listing.
case " $* " in *" --list "*) export GOUD_SKIP_FMT_PACKAGES=1 ;; esac
# shellcheck source=scripts/_common.sh
. "$SCRIPT_DIR/_common.sh"
cd "$REPO_ROOT" || exit 1

# --- argument parsing --------------------------------------------------------
MODE="full"       # full | staged
LANE=""           # empty = all lanes
DO_LIST=0
while [ $# -gt 0 ]; do
  case "$1" in
    --staged) MODE="staged" ;;
    --lane)   LANE="${2:-}"; shift ;;
    --lane=*) LANE="${1#--lane=}" ;;
    --list)   DO_LIST=1 ;;
    -h|--help) sed -n '2,26p' "$0"; exit 0 ;;
    *) log_err "verify.sh: unknown argument '$1'"; exit 2 ;;
  esac
  shift
done

# --- step table --------------------------------------------------------------
# Parallel arrays keyed by index. Fields: NAME TIER MODE LANE SKIP CI_ANCHOR CMD
# SKIP is a command probed with `have`; if absent the step is skipped with a
# notice (CI runner images are expected to provide every block-tier tool).
STEP_NAME=(); STEP_TIER=(); STEP_MODE=(); STEP_LANE=(); STEP_SKIP=(); STEP_ANCHOR=(); STEP_CMD=()
add_step() {
  STEP_NAME+=("$1"); STEP_TIER+=("$2"); STEP_MODE+=("$3"); STEP_LANE+=("$4")
  STEP_SKIP+=("$5"); STEP_ANCHOR+=("$6"); STEP_CMD+=("$7")
}
#         name              tier      mode   lane     skip_if_missing  ci_anchor              cmd
add_step  fmt               block     both   rust     ""               ci.yml:preflight       'cargo fmt ${FMT_PACKAGES} -- --check'
add_step  clippy            block     both   rust     ""               ci.yml:preflight       'cargo clippy ${FMT_PACKAGES} -- -D warnings'
add_step  lint-layers       block     both   arch     ""               ci.yml:preflight       'cargo run -q -p lint-layers'
add_step  rs-line-limit     block     both   meta     ""               pr-validation.yml      '"$REPO_ROOT/scripts/check-rs-line-limit.sh" --error'
add_step  ai-config         block     both   meta     ""               pr-validation.yml      '"$REPO_ROOT/scripts/validate-ai-config.sh"'
add_step  agents-md         block     both   meta     ""               pr-validation.yml      '"$REPO_ROOT/scripts/check-agents-md.sh"'
add_step  gate-parity       block     both   meta     python3          pr-validation.yml      'python3 "$REPO_ROOT/scripts/check-gate-parity.py"'
add_step  skills-validate   block     both   meta     python3          pr-validation.yml      'python3 "$REPO_ROOT/scripts/validate-skills.py"'
add_step  hooks-test        block     both   meta     ""               pr-validation.yml      '"$REPO_ROOT/scripts/test-hooks.sh"'
add_step  clippy-all        block     full   rust     ""               ci.yml:preflight       'cargo clippy ${FMT_PACKAGES} --all-targets --all-features -- -D warnings'
add_step  build             block     full   rust     ""               ci.yml:build           'cargo build ${FMT_PACKAGES}'
# Native window smoke tests need a real display; the gate skips them (as CI does)
# so it is reproducible headlessly. Run them directly with `cargo test` on a
# machine with a display.
add_step  test              block     full   rust     ""               ci.yml:test            'GOUD_SKIP_NATIVE_SMOKE=1 cargo test ${FMT_PACKAGES}'
add_step  sdk-test          block     full   rust     ""               ci.yml:test            'cargo test --lib sdk'
add_step  cargo-deny        block     full   rust     cargo-deny       security.yml           'cargo deny --all-features check'
# doc-paths is advisory: check-doc-paths.py resolves every backtick `path/` against
# the repo root, so relative module references (layer names like `libs/`, `core/`)
# read as false positives. Kept in the advisory tier until its precision improves.
add_step  doc-paths         advisory  full   docs     python3          pr-validation.yml      'python3 "$REPO_ROOT/scripts/check-doc-paths.py"'
add_step  large-files       block     full   meta     ""               pr-validation.yml      '"$REPO_ROOT/scripts/check-large-files.sh"'
add_step  codegen-drift     block     full   codegen  ""               ci.yml:codegen         '"$REPO_ROOT/scripts/check-generated-artifacts.sh"'
add_step  ts-typecheck      block     full   sdk      npm              ci.yml:typescript      '(cd "$REPO_ROOT/sdks/typescript" && npm run build:ts)'
add_step  markdown          advisory  full   docs     npx              pr-validation.yml      'npx markdownlint-cli2 "**/*.md" "!**/target/**" "!**/node_modules/**"'
add_step  npm-audit         advisory  full   sdk      npm              security.yml           '(cd "$REPO_ROOT/sdks/typescript" && npm audit --audit-level=moderate)'
add_step  dotnet-vuln       advisory  full   sdk      dotnet           security.yml           '(cd "$REPO_ROOT/sdks/csharp" && dotnet list package --vulnerable)'

STEP_COUNT=${#STEP_NAME[@]}

step_in_mode() { # index -> 0 if the step runs in the current MODE
  [ "$MODE" = "full" ] && return 0
  [ "${STEP_MODE[$1]}" = "both" ] && return 0
  return 1
}
step_in_lane() { # index -> 0 if the step matches the requested LANE (or no lane filter)
  [ -z "$LANE" ] && return 0
  [ "${STEP_LANE[$1]}" = "$LANE" ] && return 0
  return 1
}

# --- --list ------------------------------------------------------------------
if [ "$DO_LIST" -eq 1 ]; then
  printf 'name\ttier\tmode\tlane\tci_anchor\tcmd\n'
  for i in $(seq 0 $((STEP_COUNT - 1))); do
    step_in_mode "$i" || continue
    step_in_lane "$i" || continue
    printf '%s\t%s\t%s\t%s\t%s\t%s\n' \
      "${STEP_NAME[$i]}" "${STEP_TIER[$i]}" "${STEP_MODE[$i]}" \
      "${STEP_LANE[$i]}" "${STEP_ANCHOR[$i]}" "${STEP_CMD[$i]}"
  done
  exit 0
fi

# --- run ---------------------------------------------------------------------
log_info "=== GoudEngine verify (${MODE}${LANE:+, lane=$LANE}) ==="
FAILURES=()
ADVISORIES=()
SELECTED=()
for i in $(seq 0 $((STEP_COUNT - 1))); do
  step_in_mode "$i" || continue
  step_in_lane "$i" || continue
  SELECTED+=("$i")
done
TOTAL=${#SELECTED[@]}
[ "$TOTAL" -eq 0 ] && { log_warn "no steps match mode=$MODE lane=$LANE"; exit 0; }

n=0
for i in "${SELECTED[@]}"; do
  n=$((n + 1))
  name="${STEP_NAME[$i]}"; tier="${STEP_TIER[$i]}"; skip="${STEP_SKIP[$i]}"; cmd="${STEP_CMD[$i]}"
  if [ -n "$skip" ] && ! have "$skip"; then
    log_dim "[$n/$TOTAL] $name — skipped (missing: $skip)"
    continue
  fi
  printf '%b[%d/%d] %s%b\n' "$_C_BLUE" "$n" "$TOTAL" "$name" "$_C_RESET"
  start="$(now_ms)"
  # Run the step; capture output so we can attribute failures precisely.
  out="$(eval "$cmd" 2>&1)"; rc=$?
  dur=$(( $(now_ms) - start ))
  if [ "$rc" -eq 0 ]; then
    log_ok "    ok (${dur}ms)"
  elif [ "$tier" = "advisory" ]; then
    log_warn "    advisory: $name reported issues (${dur}ms) — non-blocking"
    ADVISORIES+=("$name")
  else
    log_err "    FAIL: $name (${dur}ms)"
    printf '%s\n' "$out" | sed 's/^/      /'
    FAILURES+=("$name|$cmd")
  fi
done

echo ""
if [ "${#ADVISORIES[@]}" -gt 0 ]; then
  log_warn "Advisory (non-blocking): ${ADVISORIES[*]}"
fi
if [ "${#FAILURES[@]}" -gt 0 ]; then
  log_err "=== verify FAILED: ${#FAILURES[@]} step(s) ==="
  for f in "${FAILURES[@]}"; do
    log_err "  ✗ ${f%%|*}"
    log_dim "    reproduce: ${f#*|}"
  done
  exit 1
fi
log_ok "=== verify passed (${TOTAL} steps) ==="
