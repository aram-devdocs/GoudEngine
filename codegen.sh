#!/bin/bash
# Regenerate all SDKs from the universal schema.
# Run this after modifying codegen/goud_sdk.schema.json or codegen/ffi_mapping.json.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
cd "$SCRIPT_DIR"

echo "╔══════════════════════════════════════════════════════════╗"
echo "║         GoudEngine SDK Code Generation                   ║"
echo "╠══════════════════════════════════════════════════════════╣"

echo "║ [1/5] Building Rust engine (extracts FFI surface)..."
cargo build 2>&1 | grep -v "^$" | head -5 || true

echo "║ [2/5] Generating C# SDK..."
python3 codegen/gen_csharp.py

echo "║ [3/5] Generating Python SDK..."
python3 codegen/gen_python.py

echo "║ [4/5] Generating TypeScript Node SDK..."
python3 codegen/gen_ts_node.py

echo "║ [5/5] Generating TypeScript Web SDK..."
python3 codegen/gen_ts_web.py

echo "╠══════════════════════════════════════════════════════════╣"
echo "║ ✓ All SDKs generated from goud_sdk.schema.json          ║"
echo "╚══════════════════════════════════════════════════════════╝"
