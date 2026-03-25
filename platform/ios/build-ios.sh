#!/bin/bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
OUTPUT_DIR="${SCRIPT_DIR}/build"
DEVICE_DIR="${OUTPUT_DIR}/device"
SIMULATOR_DIR="${OUTPUT_DIR}/simulator"
INCLUDE_DIR="${OUTPUT_DIR}/include"
HEADER_SOURCE="${REPO_ROOT}/codegen/generated/goud_engine.h"
CRATE_MANIFEST="${REPO_ROOT}/goud_engine/Cargo.toml"

ensure_command() {
    local name="$1"
    local install_cmd="$2"
    if command -v "$name" >/dev/null 2>&1; then
        return 0
    fi
    echo "Installing ${name}..."
    eval "$install_cmd"
}

ensure_target() {
    local target="$1"
    if rustup target list --installed | grep -qx "$target"; then
        return 0
    fi
    echo "Installing Rust target ${target}..."
    rustup target add "$target"
}

mkdir -p "${DEVICE_DIR}" "${SIMULATOR_DIR}" "${INCLUDE_DIR}"

ensure_target aarch64-apple-ios
ensure_target aarch64-apple-ios-sim
ensure_target x86_64-apple-ios

COMMON_ARGS=(
    --manifest-path "${CRATE_MANIFEST}"
    --release
    -p goud-engine-core
    --lib
    --no-default-features
    --features native
)

echo "Building iOS device archive..."
cargo build "${COMMON_ARGS[@]}" --target aarch64-apple-ios

echo "Building iOS simulator archives..."
cargo build "${COMMON_ARGS[@]}" --target aarch64-apple-ios-sim
cargo build "${COMMON_ARGS[@]}" --target x86_64-apple-ios

cp "${REPO_ROOT}/target/aarch64-apple-ios/release/libgoud_engine.a" "${DEVICE_DIR}/libgoud_engine.a"
lipo -create \
    "${REPO_ROOT}/target/aarch64-apple-ios-sim/release/libgoud_engine.a" \
    "${REPO_ROOT}/target/x86_64-apple-ios/release/libgoud_engine.a" \
    -output "${SIMULATOR_DIR}/libgoud_engine.a"

if [ ! -f "${HEADER_SOURCE}" ]; then
    echo "Expected generated header at ${HEADER_SOURCE}" >&2
    exit 1
fi
cp "${HEADER_SOURCE}" "${INCLUDE_DIR}/goud_engine.h"

echo "iOS build artifacts:"
echo "  device    ${DEVICE_DIR}/libgoud_engine.a"
echo "  simulator ${SIMULATOR_DIR}/libgoud_engine.a"
echo "  header    ${INCLUDE_DIR}/goud_engine.h"
