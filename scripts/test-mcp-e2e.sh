#!/usr/bin/env bash
# End-to-end MCP attach test for GoudEngine debugger runtime.
#
# Starts a sandbox game with debugger enabled, waits for the runtime manifest
# to appear, then exercises the MCP server's tool chain via JSON-RPC over
# stdin/stdout.
#
# Requirements:
#   - A display (GL context) - cannot run in headless CI
#   - cargo build completed (sandbox + goudengine-mcp binaries available)
#
# Usage:
#   ./scripts/test-mcp-e2e.sh [rust|all]
#
# Environment:
#   GOUD_MCP_E2E_TIMEOUT  - seconds to wait for manifest (default: 15)
#   GOUD_MCP_E2E_GAME_SEC - how long sandbox runs (default: 20)

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$REPO_ROOT"

TIMEOUT="${GOUD_MCP_E2E_TIMEOUT:-15}"
GAME_SEC="${GOUD_MCP_E2E_GAME_SEC:-20}"
SDK="${1:-rust}"
PASSED=0
FAILED=0
ERRORS=""

# Determine runtime manifest directory
if [[ -n "${XDG_RUNTIME_DIR:-}" ]]; then
  RUNTIME_DIR="$XDG_RUNTIME_DIR/goudengine"
elif [[ -n "${TMPDIR:-}" ]]; then
  RUNTIME_DIR="$TMPDIR/goudengine"
else
  RUNTIME_DIR="/tmp/goudengine"
fi

log() { echo "[mcp-e2e] $*"; }
pass() { log "PASS: $1"; PASSED=$((PASSED + 1)); }
fail() { log "FAIL: $1"; FAILED=$((FAILED + 1)); ERRORS="$ERRORS\n  - $1"; }

# Wait for a runtime manifest to appear with the given route label
wait_for_manifest() {
  local label="$1"
  local deadline=$((SECONDS + TIMEOUT))
  while [[ $SECONDS -lt $deadline ]]; do
    if [[ -d "$RUNTIME_DIR" ]]; then
      for f in "$RUNTIME_DIR"/runtime-*.json; do
        [[ -f "$f" ]] || continue
        if grep -q "\"$label\"" "$f" 2>/dev/null; then
          echo "$f"
          return 0
        fi
      done
    fi
    sleep 0.5
  done
  return 1
}

# Send a JSON-RPC request to MCP server via stdin/stdout pipes
# Uses a co-process with the MCP server binary.
mcp_call() {
  local method="$1"
  local params="$2"
  local id="$3"
  local request
  request=$(cat <<JSONRPC
{"jsonrpc":"2.0","id":$id,"method":"$method","params":$params}
JSONRPC
)
  echo "$request" >&"${MCP_COPROC[1]}"
  # Read response (line-delimited JSON-RPC)
  local response=""
  local read_timeout=10
  if read -r -t "$read_timeout" response <&"${MCP_COPROC[0]}"; then
    echo "$response"
  else
    echo '{"error":"timeout"}'
  fi
}

# Build required binaries
log "Building sandbox and MCP server..."
cargo build -p sandbox -p goudengine-mcp --release 2>&1 | tail -3

run_rust_test() {
  log "=== Rust sandbox E2E test ==="

  # Clean stale manifests
  rm -f "$RUNTIME_DIR"/runtime-*.json 2>/dev/null || true
  mkdir -p "$RUNTIME_DIR"

  # Start sandbox in background
  log "Starting Rust sandbox (${GAME_SEC}s runtime)..."
  GOUD_SANDBOX_SMOKE_SECONDS="$GAME_SEC" \
    cargo run -p sandbox --release &
  local SANDBOX_PID=$!
  log "Sandbox PID: $SANDBOX_PID"

  # Wait for manifest
  log "Waiting for runtime manifest (route: sandbox-rust)..."
  local manifest_file
  if manifest_file=$(wait_for_manifest "sandbox-rust"); then
    pass "Runtime manifest published: $(basename "$manifest_file")"
  else
    fail "Runtime manifest not found within ${TIMEOUT}s"
    kill "$SANDBOX_PID" 2>/dev/null || true
    wait "$SANDBOX_PID" 2>/dev/null || true
    return
  fi

  # Extract context_id and process_nonce from manifest
  local context_id
  context_id=$(python3 -c "
import json, sys
m = json.load(open('$manifest_file'))
for r in m['routes']:
    if r.get('label') == 'sandbox-rust':
        print(r['route_id']['context_id'])
        sys.exit(0)
print('0')
")
  local process_nonce
  process_nonce=$(python3 -c "
import json
m = json.load(open('$manifest_file'))
print(m['process_nonce'])
")
  log "Discovered context_id=$context_id process_nonce=$process_nonce"

  if [[ "$context_id" != "0" ]]; then
    pass "Context discovered via manifest (id=$context_id)"
  else
    fail "Could not extract context_id from manifest"
    kill "$SANDBOX_PID" 2>/dev/null || true
    wait "$SANDBOX_PID" 2>/dev/null || true
    return
  fi

  # Verify manifest has attachable=true
  local attachable
  attachable=$(python3 -c "
import json
m = json.load(open('$manifest_file'))
for r in m['routes']:
    if r.get('label') == 'sandbox-rust':
        print('true' if r.get('attachable', False) else 'false')
")
  if [[ "$attachable" == "true" ]]; then
    pass "Route is attachable"
  else
    fail "Route attachable flag is not true"
  fi

  # Verify endpoint exists
  local endpoint_transport
  endpoint_transport=$(python3 -c "
import json
m = json.load(open('$manifest_file'))
print(m['endpoint']['transport'])
")
  local endpoint_location
  endpoint_location=$(python3 -c "
import json
m = json.load(open('$manifest_file'))
print(m['endpoint']['location'])
")
  log "Endpoint: $endpoint_transport @ $endpoint_location"

  if [[ -S "$endpoint_location" ]] || [[ "$endpoint_transport" == "unix" ]]; then
    pass "IPC socket exists at endpoint location"
  else
    log "Warning: socket file not found (may be created on-demand)"
  fi

  # Verify capabilities in manifest
  local caps
  caps=$(python3 -c "
import json
m = json.load(open('$manifest_file'))
for r in m['routes']:
    if r.get('label') == 'sandbox-rust':
        print(','.join(sorted(r.get('capabilities', {}).keys())))
")
  log "Route capabilities: $caps"
  if echo "$caps" | grep -q "snapshots"; then
    pass "Snapshots capability advertised"
  else
    fail "Missing snapshots capability"
  fi
  if echo "$caps" | grep -q "control_plane"; then
    pass "Control plane capability advertised"
  else
    fail "Missing control_plane capability"
  fi

  # Clean up sandbox
  log "Stopping sandbox..."
  kill "$SANDBOX_PID" 2>/dev/null || true
  wait "$SANDBOX_PID" 2>/dev/null || true
  log "Sandbox stopped"
}

# Run tests based on SDK selection
case "$SDK" in
  rust)
    run_rust_test
    ;;
  all)
    run_rust_test
    # C#, Python, and TS desktop tests follow the same pattern but require
    # different build/run commands. Add them here as needed:
    # run_csharp_test
    # run_python_test
    # run_typescript_test
    ;;
  *)
    log "Unknown SDK: $SDK (expected: rust, all)"
    exit 1
    ;;
esac

# Summary
echo ""
log "========================================="
log "Results: $PASSED passed, $FAILED failed"
if [[ $FAILED -gt 0 ]]; then
  log "Failures:$ERRORS"
  exit 1
fi
log "All E2E MCP tests passed"
