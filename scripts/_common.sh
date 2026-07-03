#!/usr/bin/env bash
# Shared helpers for GoudEngine validation scripts.
#
# Source this from other scripts:
#   . "$(dirname "$0")/_common.sh"
#
# Provides: REPO_ROOT, colored logging, timing, and the FMT_PACKAGES list
# (all workspace crates except the ones whose sources are codegen output that
# does not exist on a fresh checkout).

# Resolve the repository root once.
REPO_ROOT="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
export REPO_ROOT

# --- logging -----------------------------------------------------------------
# Colors only when stdout is a TTY.
if [ -t 1 ]; then
  _C_RESET='\033[0m'; _C_DIM='\033[2m'; _C_RED='\033[31m'
  _C_GREEN='\033[32m'; _C_YELLOW='\033[33m'; _C_BLUE='\033[34m'
else
  _C_RESET=''; _C_DIM=''; _C_RED=''; _C_GREEN=''; _C_YELLOW=''; _C_BLUE=''
fi

log_info() { printf '%b%s%b\n' "$_C_BLUE" "$*" "$_C_RESET"; }
log_ok()   { printf '%b%s%b\n' "$_C_GREEN" "$*" "$_C_RESET"; }
log_warn() { printf '%b%s%b\n' "$_C_YELLOW" "$*" "$_C_RESET" >&2; }
log_err()  { printf '%b%s%b\n' "$_C_RED" "$*" "$_C_RESET" >&2; }
log_dim()  { printf '%b%s%b\n' "$_C_DIM" "$*" "$_C_RESET"; }

# --- packages ----------------------------------------------------------------
# Workspace crates to run fmt/clippy/build/test against. Excludes crates whose
# .g.rs sources are codegen output (gitignored, absent on a fresh checkout or
# worktree), which would otherwise make cargo fmt/clippy fail.
GOUD_EXCLUDED_CRATES="${GOUD_EXCLUDED_CRATES:-goud-engine-node}"

fmt_packages() {
  cargo metadata --no-deps --format-version 1 2>/dev/null \
    | python3 -c '
import sys, json, os
excluded = set(os.environ.get("GOUD_EXCLUDED_CRATES", "").split())
d = json.load(sys.stdin)
print(" ".join("-p " + p["name"] for p in d["packages"] if p["name"] not in excluded))
'
}

# Cache the result in FMT_PACKAGES for callers that want it eagerly.
: "${FMT_PACKAGES:=}"
if [ -z "${FMT_PACKAGES}" ] && [ "${GOUD_SKIP_FMT_PACKAGES:-0}" != "1" ]; then
  FMT_PACKAGES="$(fmt_packages)"
fi
export FMT_PACKAGES

# --- misc --------------------------------------------------------------------
have() { command -v "$1" >/dev/null 2>&1; }

# Portable millisecond clock (python3 is a hard dependency of the toolchain).
now_ms() { python3 -c 'import time; print(int(time.time()*1000))'; }
