#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

PY_SANDBOX="$ROOT_DIR/examples/python/sandbox.py"
if [[ ! -f "$PY_SANDBOX" ]]; then
  echo "Missing required Python sandbox: $PY_SANDBOX"
  echo "Peer smoke expects a Rust host plus Python client."
  exit 2
fi

PORT="${GOUD_SANDBOX_NETWORK_PORT:-39591}"
SMOKE_SECONDS="${GOUD_SANDBOX_SMOKE_SECONDS:-6}"
LOG_DIR="$(mktemp -d "${TMPDIR:-/tmp}/goud-sandbox-peer-XXXXXX")"
RUST_LOG="$LOG_DIR/rust-host.log"
PY_LOG="$LOG_DIR/python-client.log"
case "$(uname -s)" in
  Linux*)  PY_NATIVE_LIB="$ROOT_DIR/target/debug/libgoud_engine.so" ;;
  Darwin*) PY_NATIVE_LIB="$ROOT_DIR/target/debug/libgoud_engine.dylib" ;;
  MINGW*|MSYS*|CYGWIN*) PY_NATIVE_LIB="$ROOT_DIR/target/debug/goud_engine.dll" ;;
  *)       PY_NATIVE_LIB="$ROOT_DIR/target/debug/libgoud_engine.so" ;;
esac

RUNNER=()
if command -v xvfb-run >/dev/null 2>&1; then
  RUNNER=(xvfb-run -a)
fi

cleanup() {
  if [[ -n "${RUST_PID:-}" ]] && kill -0 "$RUST_PID" >/dev/null 2>&1; then
    kill "$RUST_PID" >/dev/null 2>&1 || true
  fi
  if [[ -n "${PY_PID:-}" ]] && kill -0 "$PY_PID" >/dev/null 2>&1; then
    kill "$PY_PID" >/dev/null 2>&1 || true
  fi
}
trap cleanup EXIT

echo "Running sandbox peer smoke on port $PORT (logs: $LOG_DIR)"

echo "Prebuilding Python native library and Rust sandbox binary for deterministic startup..."
cargo build -p goud-engine-core -p sandbox >/dev/null
if [[ ! -f "$PY_NATIVE_LIB" ]]; then
  echo "Missing Python native library after build: $PY_NATIVE_LIB"
  exit 1
fi

(
  export GOUD_SANDBOX_NETWORK_PORT="$PORT"
  export GOUD_SANDBOX_NETWORK_ROLE=host
  export GOUD_SANDBOX_EXIT_ON_PEER=1
  export GOUD_SANDBOX_EXPECT_PEER=1
  export GOUD_SANDBOX_SMOKE_SECONDS="$SMOKE_SECONDS"
  if [[ "${#RUNNER[@]}" -gt 0 ]]; then
    "${RUNNER[@]}" cargo run -p sandbox
  else
    cargo run -p sandbox
  fi
) >"$RUST_LOG" 2>&1 &
RUST_PID=$!

HOST_READY=0
for _ in $(seq 1 120); do
  if ! kill -0 "$RUST_PID" >/dev/null 2>&1; then
    echo "Rust host exited before readiness."
    echo "--- Rust host log ---"
    sed -n '1,200p' "$RUST_LOG"
    exit 1
  fi
  if grep -q "Network host:$PORT" "$RUST_LOG"; then
    HOST_READY=1
    break
  fi
  sleep 0.25
done

if [[ "$HOST_READY" -ne 1 ]]; then
  echo "Timed out waiting for Rust host readiness on port $PORT."
  echo "--- Rust host log ---"
  sed -n '1,200p' "$RUST_LOG"
  exit 1
fi

echo "Rust host is ready; starting Python client."

(
  export GOUD_SANDBOX_NETWORK_PORT="$PORT"
  export GOUD_SANDBOX_NETWORK_ROLE=client
  export GOUD_SANDBOX_EXIT_ON_PEER=1
  export GOUD_SANDBOX_EXPECT_PEER=1
  export GOUD_SANDBOX_SMOKE_SECONDS="$SMOKE_SECONDS"
  export GOUD_ENGINE_LIB="$PY_NATIVE_LIB"
  export PYTHONPATH="$ROOT_DIR/sdks/python"
  if [[ "${#RUNNER[@]}" -gt 0 ]]; then
    "${RUNNER[@]}" python3 "$PY_SANDBOX"
  else
    python3 "$PY_SANDBOX"
  fi
) >"$PY_LOG" 2>&1 &
PY_PID=$!

HOST_STATUS=0
CLIENT_STATUS=0
wait "$RUST_PID" || HOST_STATUS=$?
wait "$PY_PID" || CLIENT_STATUS=$?

if [[ "$HOST_STATUS" -ne 0 || "$CLIENT_STATUS" -ne 0 ]]; then
  echo "Sandbox peer smoke failed (rust=$HOST_STATUS python=$CLIENT_STATUS)"
  echo "--- Rust host log ---"
  sed -n '1,200p' "$RUST_LOG"
  echo "--- Python client log ---"
  sed -n '1,200p' "$PY_LOG"
  exit 1
fi

echo "Sandbox peer smoke passed."
