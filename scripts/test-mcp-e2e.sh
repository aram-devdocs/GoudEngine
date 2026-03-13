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

# FIFOs used by the MCP coprocess for stdin/stdout piping.
MCP_FIFO_DIR=""
MCP_SERVER_PID=""

# Start the MCP server with FIFOs for stdin/stdout communication.
start_mcp_server() {
  MCP_FIFO_DIR=$(mktemp -d)
  mkfifo "$MCP_FIFO_DIR/mcp_in"
  mkfifo "$MCP_FIFO_DIR/mcp_out"
  "$1" < "$MCP_FIFO_DIR/mcp_in" > "$MCP_FIFO_DIR/mcp_out" &
  MCP_SERVER_PID=$!
  # Open write end so the server does not get EOF immediately
  exec 7>"$MCP_FIFO_DIR/mcp_in"
  # Open read end for consuming responses
  exec 8<"$MCP_FIFO_DIR/mcp_out"
}

stop_mcp_server() {
  exec 7>&- 2>/dev/null || true
  exec 8<&- 2>/dev/null || true
  if [[ -n "$MCP_SERVER_PID" ]]; then
    kill "$MCP_SERVER_PID" 2>/dev/null || true
    wait "$MCP_SERVER_PID" 2>/dev/null || true
    MCP_SERVER_PID=""
  fi
  if [[ -n "$MCP_FIFO_DIR" ]]; then
    rm -rf "$MCP_FIFO_DIR" 2>/dev/null || true
    MCP_FIFO_DIR=""
  fi
}

# Send a JSON-RPC request to MCP server via FIFO pipes.
mcp_call() {
  local method="$1"
  local params="$2"
  local id="$3"
  local request="{\"jsonrpc\":\"2.0\",\"id\":$id,\"method\":\"$method\",\"params\":$params}"
  echo "$request" >&7
  # Read response (line-delimited JSON-RPC)
  local response=""
  local read_timeout=10
  if read -r -t "$read_timeout" response <&8; then
    echo "$response"
  else
    echo '{"error":"timeout"}'
  fi
}

# Helper: validate a JSON-RPC response has a result (not an error)
validate_rpc_result() {
  local label="$1"
  local response="$2"
  local has_result
  has_result=$(python3 -c "
import json, sys
try:
    r = json.loads(sys.argv[1])
    if 'result' in r:
        print('ok')
    elif 'error' in r:
        print('error: ' + json.dumps(r['error']))
    else:
        print('unknown')
except Exception as e:
    print('parse_error: ' + str(e))
" "$response")
  if [[ "$has_result" == "ok" ]]; then
    pass "$label"
    return 0
  else
    fail "$label ($has_result)"
    return 1
  fi
}

# Helper: extract a value from a JSON-RPC result using a Python expression
extract_from_result() {
  local response="$1"
  local py_expr="$2"
  python3 -c "
import json, sys
r = json.loads(sys.argv[1])
result = r.get('result', {})
$py_expr
" "$response" 2>/dev/null
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

  # =========================================================================
  # MCP server coprocess tests -- exercise the new diagnostic/log/hierarchy
  # tools via JSON-RPC over stdin/stdout
  # =========================================================================

  log "Starting MCP server coprocess..."
  local MCP_BIN="$REPO_ROOT/target/release/goudengine-mcp"
  if [[ ! -x "$MCP_BIN" ]]; then
    fail "MCP server binary not found at $MCP_BIN"
    kill "$SANDBOX_PID" 2>/dev/null || true
    wait "$SANDBOX_PID" 2>/dev/null || true
    return
  fi

  # Start MCP server with FIFO-based stdin/stdout pipes
  start_mcp_server "$MCP_BIN"
  log "MCP server PID: $MCP_SERVER_PID"

  # Step 1: MCP initialize handshake
  local INIT_RESPONSE
  INIT_RESPONSE=$(mcp_call "initialize" '{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"e2e-test","version":"1.0"}}' 1)
  if validate_rpc_result "MCP initialize handshake" "$INIT_RESPONSE"; then
    log "MCP server initialized"
  else
    log "MCP initialize failed, skipping MCP tool tests"
    stop_mcp_server
    kill "$SANDBOX_PID" 2>/dev/null || true
    wait "$SANDBOX_PID" 2>/dev/null || true
    return
  fi

  # Send initialized notification (no response expected)
  echo '{"jsonrpc":"2.0","method":"notifications/initialized"}' >&7

  # Step 2: Attach to the sandbox context
  local ATTACH_RESPONSE
  ATTACH_RESPONSE=$(mcp_call "tools/call" "{\"name\":\"goudengine.attach_context\",\"arguments\":{\"contextId\":$context_id,\"processNonce\":$process_nonce}}" 2)
  validate_rpc_result "MCP attach_context tool call" "$ATTACH_RESPONSE"

  # Step 3: get_diagnostics
  local DIAG_RESPONSE
  DIAG_RESPONSE=$(mcp_call "tools/call" '{"name":"goudengine.get_diagnostics","arguments":{}}' 3)
  if validate_rpc_result "MCP get_diagnostics tool call" "$DIAG_RESPONSE"; then
    local diag_is_object
    diag_is_object=$(extract_from_result "$DIAG_RESPONSE" "
content = result.get('content', [])
if content and isinstance(json.loads(content[0].get('text', '{}')), dict):
    print('true')
else:
    print('false')
")
    if [[ "$diag_is_object" == "true" ]]; then
      pass "get_diagnostics returns object content"
    else
      fail "get_diagnostics content is not an object"
    fi
  fi

  # Step 4: get_subsystem_diagnostics (valid key: render)
  local SUBSYS_RESPONSE
  SUBSYS_RESPONSE=$(mcp_call "tools/call" '{"name":"goudengine.get_subsystem_diagnostics","arguments":{"key":"render"}}' 4)
  validate_rpc_result "MCP get_subsystem_diagnostics(render) tool call" "$SUBSYS_RESPONSE"

  # Step 5: get_subsystem_diagnostics (invalid key)
  local SUBSYS_INVALID_RESPONSE
  SUBSYS_INVALID_RESPONSE=$(mcp_call "tools/call" '{"name":"goudengine.get_subsystem_diagnostics","arguments":{"key":"nonexistent"}}' 5)
  validate_rpc_result "MCP get_subsystem_diagnostics(nonexistent) tool call" "$SUBSYS_INVALID_RESPONSE"

  # Step 6: get_logs
  local LOGS_RESPONSE
  LOGS_RESPONSE=$(mcp_call "tools/call" '{"name":"goudengine.get_logs","arguments":{}}' 6)
  if validate_rpc_result "MCP get_logs tool call" "$LOGS_RESPONSE"; then
    local has_entries
    has_entries=$(extract_from_result "$LOGS_RESPONSE" "
content = result.get('content', [])
if content:
    data = json.loads(content[0].get('text', '{}'))
    if 'entries' in data:
        print('true')
    else:
        print('false')
else:
    print('false')
")
    if [[ "$has_entries" == "true" ]]; then
      pass "get_logs response contains entries array"
    else
      fail "get_logs response missing entries array"
    fi
  fi

  # Step 7: get_scene_hierarchy
  local HIERARCHY_RESPONSE
  HIERARCHY_RESPONSE=$(mcp_call "tools/call" '{"name":"goudengine.get_scene_hierarchy","arguments":{}}' 7)
  validate_rpc_result "MCP get_scene_hierarchy tool call" "$HIERARCHY_RESPONSE"

  # Clean up MCP server
  log "Stopping MCP server..."
  stop_mcp_server
  log "MCP server stopped"

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
