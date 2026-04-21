#!/usr/bin/env bash
# Codex Cloud environment setup for GoudEngine.
#
# Runs once during the Codex "setup" phase (internet is available) on the
# codex-universal base image (Ubuntu 24.04, root user). Installs everything
# the CI lanes need so the agent phase can build, test, and run codegen
# without network access.
#
# This script is the single source of truth: edit it here, not in the Codex
# UI. The Codex environment just invokes `bash scripts/codex-setup.sh`.

set -euo pipefail

log() { printf '\n\033[1;34m▶ %s\033[0m\n' "$*"; }

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$REPO_ROOT"

# --- System packages (graphics, audio, build tooling) -----------------------
# Pulled from .github/workflows/ci.yml — keep in sync with Ubuntu CI lanes.
log "Installing apt packages"
export DEBIAN_FRONTEND=noninteractive
apt-get update -y
apt-get install -y --no-install-recommends \
  cmake \
  pkg-config \
  libgl1-mesa-dev libglu1-mesa-dev \
  libxrandr-dev libxinerama-dev libxcursor-dev libxi-dev libxxf86vm-dev \
  libxkbcommon-x11-dev \
  libasound2-dev libudev-dev \
  libvulkan-dev mesa-vulkan-drivers \
  lua5.4 liblua5.4-dev \
  xvfb \
  ca-certificates wget gnupg

# --- .NET 8 SDK (not pre-installed in codex-universal) ----------------------
if ! command -v dotnet >/dev/null 2>&1; then
  log "Installing .NET 8 SDK"
  # shellcheck disable=SC1091
  . /etc/os-release
  wget -qO /tmp/packages-microsoft-prod.deb \
    "https://packages.microsoft.com/config/ubuntu/${VERSION_ID}/packages-microsoft-prod.deb"
  dpkg -i /tmp/packages-microsoft-prod.deb
  apt-get update -y
  apt-get install -y dotnet-sdk-8.0
  rm -f /tmp/packages-microsoft-prod.deb
fi

# --- Rust toolchain ---------------------------------------------------------
# codex-universal ships rustup + multiple pinned versions. Pin the working
# toolchain via CODEX_ENV_RUST_VERSION in the environment settings; this
# block just ensures components/targets we need are present.
if ! command -v rustup >/dev/null 2>&1; then
  log "Installing rustup"
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \
    | sh -s -- -y --default-toolchain stable --profile minimal
  # shellcheck disable=SC1091
  . "$HOME/.cargo/env"
fi
export PATH="$HOME/.cargo/bin:$PATH"
rustup component add rustfmt clippy
rustup target add wasm32-unknown-unknown

# --- wasm-pack (pinned to CI version) ---------------------------------------
WASM_PACK_VERSION="0.13.1"
if ! command -v wasm-pack >/dev/null 2>&1 \
   || [[ "$(wasm-pack --version 2>/dev/null || true)" != *"${WASM_PACK_VERSION}"* ]]; then
  log "Installing wasm-pack ${WASM_PACK_VERSION}"
  curl -sSfL \
    "https://github.com/rustwasm/wasm-pack/releases/download/v${WASM_PACK_VERSION}/wasm-pack-v${WASM_PACK_VERSION}-x86_64-unknown-linux-musl.tar.gz" \
    -o /tmp/wasm-pack.tar.gz
  tar -xzf /tmp/wasm-pack.tar.gz -C /tmp
  install "/tmp/wasm-pack-v${WASM_PACK_VERSION}-x86_64-unknown-linux-musl/wasm-pack" \
    /usr/local/bin/wasm-pack
  rm -rf /tmp/wasm-pack*
fi

# --- Python tooling ---------------------------------------------------------
log "Priming Python tooling"
python3 -m pip install --quiet --upgrade pip

# --- Node dependencies for TypeScript SDK -----------------------------------
if [[ -f sdks/typescript/package-lock.json ]]; then
  log "Installing TypeScript SDK npm deps"
  (cd sdks/typescript && npm ci --no-audit --no-fund)
fi

# --- Environment variables that must persist into the agent phase -----------
# Setup-phase `export`s do NOT carry into the agent shell — write them to
# ~/.bashrc so they're available when the agent runs.
BASHRC_MARKER="# >>> goudengine codex env >>>"
BASHRC_END_MARKER="# <<< goudengine codex env <<<"
if ! grep -qF "$BASHRC_MARKER" "$HOME/.bashrc" 2>/dev/null; then
  log "Persisting env vars into ~/.bashrc"
  cat >> "$HOME/.bashrc" <<EOF

${BASHRC_MARKER}
# sdl2-sys bundled build uses cmake_minimum_required(VERSION 3.1.3); CMake 4+
# needs this shim. Keep in lockstep with .github/workflows/ci.yml.
export CMAKE_POLICY_VERSION_MINIMUM=3.5
export PATH="\$HOME/.cargo/bin:\$PATH"
${BASHRC_END_MARKER}
EOF
fi
export CMAKE_POLICY_VERSION_MINIMUM=3.5

# --- Pre-fetch + pre-build so the agent phase works without internet --------
# Setup phase has internet, so let cargo refresh Cargo.lock if it's stale.
# Workspace version bumps sometimes land before the lockfile is regenerated;
# --locked would fail hard in that window, so we omit it here.
log "Pre-fetching cargo registry"
cargo fetch

log "Priming target/ (builds ffi_manifest.json required by codegen)"
cargo build -p goud-engine-core -p goud-engine

log "Codex environment setup complete."
