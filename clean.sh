#!/usr/bin/env bash
# clean.sh — GoudEngine repository cleanup utility
# Usage: ./clean.sh [--deep | --size | -h]

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
NUGET_OUTPUT="$SCRIPT_DIR/sdks/nuget_package_output"
NUGET_LOCAL="$HOME/nuget-local"
WORKTREES_DIR="$SCRIPT_DIR/.claude/worktrees"
KEEP_NUGET=3

usage() {
  echo "Usage: $(basename "$0") [--deep | --size | -h]"
  echo "  (none)  Lightweight: incremental dirs, bin/obj, __pycache__, .nupkg, screenshots"
  echo "  --deep  Full: everything above + cargo clean, node_modules, NuGet pruning, worktrees"
  echo "  --size  Report only — show sizes, no deletions"
  echo "  -h      Show this help"
}

kb_to_human() {
  local kb="$1"
  if [ "$kb" -ge 1048576 ]; then printf "%.1f GB\n" "$(echo "scale=1; $kb/1048576" | bc)"
  elif [ "$kb" -ge 1024 ];   then printf "%.1f MB\n" "$(echo "scale=1; $kb/1024" | bc)"
  else echo "${kb} KB"; fi
}

path_kb() { [ -e "$1" ] && du -sk "$1" 2>/dev/null | awk '{print $1}' || echo 0; }

find_dirs() { find "$SCRIPT_DIR" "${@}" -type d 2>/dev/null || true; }

# Remove a set of directories found by find args; print reclaimed space.
remove_dirs() {
  local label="$1"; shift
  local total=0
  while IFS= read -r d; do
    [ -z "$d" ] && continue
    local kb; kb=$(path_kb "$d")
    total=$((total + kb))
    rm -rf "$d"
  done < <(find_dirs "$@")
  echo "  $label: $(kb_to_human $total)"
}

clean_incremental() {
  echo "==> Pruning target/debug/incremental older than 7 days..."
  local before; before=$(path_kb "$SCRIPT_DIR/target/debug/incremental")
  find "$SCRIPT_DIR/target/debug/incremental" -mindepth 1 -maxdepth 1 \
    -type d -mtime +7 -exec rm -rf {} + 2>/dev/null || true
  local after; after=$(path_kb "$SCRIPT_DIR/target/debug/incremental")
  echo "  Reclaimed: $(kb_to_human $(( before - after )))"
}

clean_bin_obj() {
  echo "==> Cleaning bin/ and obj/ (excluding target/)..."
  remove_dirs "Reclaimed" \
    -not \( -path "$SCRIPT_DIR/target/*" -prune \) \
    \( -name "bin" -o -name "obj" \)
}

clean_pycache() {
  echo "==> Cleaning __pycache__..."
  remove_dirs "Reclaimed" -name "__pycache__"
}

clean_stale_nupkg() {
  echo "==> Removing stale .nupkg files..."
  local total=0
  while IFS= read -r f; do
    [ -z "$f" ] && continue
    local kb; kb=$(path_kb "$f")
    total=$((total + kb))
    echo "  Removing: $(basename "$f")"; rm -f "$f"
  done < <(find "$NUGET_OUTPUT" -name "*.nupkg" -type f 2>/dev/null || true)
  echo "  Reclaimed: $(kb_to_human $total)"
}

clean_screenshots() {
  echo "==> Removing flappy-web-*.png screenshots..."
  local total=0
  while IFS= read -r f; do
    [ -z "$f" ] && continue
    local kb; kb=$(path_kb "$f")
    total=$((total + kb)); rm -f "$f"
  done < <(find "$SCRIPT_DIR" -maxdepth 1 -name "flappy-web-*.png" -type f 2>/dev/null || true)
  echo "  Reclaimed: $(kb_to_human $total)"
}

clean_cargo() {
  echo "==> Running cargo clean..."
  local before; before=$(path_kb "$SCRIPT_DIR/target")
  (cd "$SCRIPT_DIR" && cargo clean)
  echo "  Reclaimed: $(kb_to_human $(( before - $(path_kb "$SCRIPT_DIR/target") )))"
}

clean_node_modules() {
  echo "==> Removing node_modules/ directories..."
  remove_dirs "Reclaimed" -name "node_modules" -prune
}

prune_nuget_local() {
  echo "==> Pruning ~/nuget-local (keep latest $KEEP_NUGET versions)..."
  [ -d "$NUGET_LOCAL" ] || { echo "  Not found, skipping."; return; }
  local total_count
  total_count=$(find "$NUGET_LOCAL" -name "GoudEngine.*.nupkg" -type f 2>/dev/null | wc -l | tr -d ' ')
  local remove_count=$((total_count - KEEP_NUGET))
  if [ "$remove_count" -le 0 ]; then
    echo "  Nothing to prune ($total_count packages, keeping $KEEP_NUGET)."
    return
  fi
  local reclaimed=0
  find "$NUGET_LOCAL" -name "GoudEngine.*.nupkg" -type f | sort -V | head -n "$remove_count" | while IFS= read -r pkg; do
    local kb; kb=$(path_kb "$pkg")
    reclaimed=$((reclaimed + kb))
    echo "  Removing: $(basename "$pkg")"; rm -f "$pkg"
  done
  echo "  Pruned $remove_count of $total_count packages."
}

clean_worktrees() {
  echo "==> Cleaning target/ dirs under .claude/worktrees/..."
  remove_dirs "Reclaimed" \
    -path "$WORKTREES_DIR/*" -maxdepth 3 -name "target"
}

report_sizes() {
  echo "=== Reclaimable Space Report ==="
  local grand=0
  row() {
    local label="$1" path="$2"
    local kb; kb=$(path_kb "$path")
    grand=$((grand + kb))
    printf "  %-46s %s\n" "$label" "$(kb_to_human $kb)"
  }
  row "target/"                             "$SCRIPT_DIR/target"
  row "target/debug/incremental/"           "$SCRIPT_DIR/target/debug/incremental"
  row "~/nuget-local/"                      "$NUGET_LOCAL"
  row ".claude/worktrees/"                  "$WORKTREES_DIR"

  local nm=0
  while IFS= read -r d; do [ -z "$d" ] && continue; nm=$((nm + $(path_kb "$d"))); done \
    < <(find "$SCRIPT_DIR" -name "node_modules" -type d -prune 2>/dev/null || true)
  grand=$((grand + nm))
  printf "  %-46s %s\n" "node_modules/ (all)" "$(kb_to_human $nm)"

  local bo=0
  while IFS= read -r d; do [ -z "$d" ] && continue; bo=$((bo + $(path_kb "$d"))); done \
    < <(find "$SCRIPT_DIR" -not \( -path "$SCRIPT_DIR/target/*" -prune \) \
          \( -name "bin" -o -name "obj" \) -type d 2>/dev/null || true)
  grand=$((grand + bo))
  printf "  %-46s %s\n" "bin/ + obj/ (excl. target/)" "$(kb_to_human $bo)"

  local pc=0
  while IFS= read -r d; do [ -z "$d" ] && continue; pc=$((pc + $(path_kb "$d"))); done \
    < <(find "$SCRIPT_DIR" -name "__pycache__" -type d 2>/dev/null || true)
  grand=$((grand + pc))
  printf "  %-46s %s\n" "__pycache__/ dirs" "$(kb_to_human $pc)"

  echo ""; printf "  %-46s %s\n" "TOTAL ESTIMATED RECLAIMABLE" "$(kb_to_human $grand)"
  echo ""; echo "Note: target/ rows overlap — run './clean.sh --deep' to reclaim all."
}

# ── main ──────────────────────────────────────────────────────────────────────

case "${1:-}" in
  -h|--help) usage; exit 0 ;;
  --size)    report_sizes; exit 0 ;;
  --deep)    MODE=deep ;;
  "")        MODE=default ;;
  *)         echo "Unknown option: $1" >&2; usage >&2; exit 1 ;;
esac

echo "=== GoudEngine cleanup (mode: $MODE) ===" && echo ""
clean_incremental
clean_bin_obj
clean_pycache
clean_stale_nupkg
clean_screenshots

if [ "$MODE" = "deep" ]; then
  clean_cargo
  clean_node_modules
  prune_nuget_local
  clean_worktrees
fi

echo "" && echo "=== Done. ==="
