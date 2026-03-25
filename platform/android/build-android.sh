#!/bin/bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
OUTPUT_DIR="${SCRIPT_DIR}/build"
INCLUDE_DIR="${OUTPUT_DIR}/include"
JNI_LIBS_DIR="${SCRIPT_DIR}/template/app/src/main/jniLibs"
CRATE_MANIFEST="${REPO_ROOT}/goud_engine/Cargo.toml"
HEADER_SOURCE="${REPO_ROOT}/codegen/generated/goud_engine.h"

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

resolve_ndk() {
    if [ -n "${ANDROID_NDK_HOME:-}" ]; then
        echo "${ANDROID_NDK_HOME}"
        return 0
    fi
    if [ -n "${ANDROID_NDK_ROOT:-}" ]; then
        echo "${ANDROID_NDK_ROOT}"
        return 0
    fi
    if [ -n "${ANDROID_HOME:-}" ] && [ -d "${ANDROID_HOME}/ndk" ]; then
        find "${ANDROID_HOME}/ndk" -mindepth 1 -maxdepth 1 -type d | sort | tail -n 1
        return 0
    fi
    if [ -d "${HOME}/Library/Android/sdk/ndk" ]; then
        find "${HOME}/Library/Android/sdk/ndk" -mindepth 1 -maxdepth 1 -type d | sort | tail -n 1
        return 0
    fi
    return 1
}

ensure_command cargo-ndk "cargo install cargo-ndk --locked"
ensure_target aarch64-linux-android
ensure_target x86_64-linux-android

ANDROID_NDK="$(resolve_ndk || true)"
if [ -z "${ANDROID_NDK}" ]; then
    echo "Android NDK not found. Set ANDROID_NDK_HOME or install an NDK under ANDROID_HOME." >&2
    exit 1
fi

export ANDROID_NDK_HOME="${ANDROID_NDK}"
export ANDROID_NDK_ROOT="${ANDROID_NDK}"

mkdir -p "${OUTPUT_DIR}" "${INCLUDE_DIR}" "${JNI_LIBS_DIR}/arm64-v8a" "${JNI_LIBS_DIR}/x86_64"
rm -f "${JNI_LIBS_DIR}/arm64-v8a/"*.so "${JNI_LIBS_DIR}/x86_64/"*.so

cargo ndk \
    --manifest-path "${CRATE_MANIFEST}" \
    --platform 26 \
    --target arm64-v8a \
    --target x86_64 \
    -o "${JNI_LIBS_DIR}" \
    build \
    --release \
    -p goud-engine-core \
    --lib \
    --no-default-features \
    --features native,jni-bridge

if [ ! -f "${HEADER_SOURCE}" ]; then
    echo "Expected generated header at ${HEADER_SOURCE}" >&2
    exit 1
fi
cp "${HEADER_SOURCE}" "${INCLUDE_DIR}/goud_engine.h"

echo "Android build artifacts:"
echo "  arm64-v8a ${JNI_LIBS_DIR}/arm64-v8a/libgoud_engine.so"
echo "  x86_64    ${JNI_LIBS_DIR}/x86_64/libgoud_engine.so"
echo "  header    ${INCLUDE_DIR}/goud_engine.h"
