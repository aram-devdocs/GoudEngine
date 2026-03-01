#!/bin/bash
# Regenerate all SDKs from the universal schema.
# Run this after modifying codegen/goud_sdk.schema.json or codegen/ffi_mapping.json.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
cd "$SCRIPT_DIR"

echo "╔══════════════════════════════════════════════════════════╗"
echo "║         GoudEngine SDK Code Generation                   ║"
echo "╠══════════════════════════════════════════════════════════╣"

echo "║ [1/8] Building Rust engine (extracts FFI manifest)..."
cargo build 2>&1 | grep -v "^$" | head -5 || true

echo "║ [2/8] Checking layer dependencies..."
cargo run -p lint-layers || { echo "║ ✗ Layer violation — fix imports"; exit 1; }

echo "║ [3/8] Validating FFI coverage (manifest vs mapping)..."
python3 codegen/validate_coverage.py || { echo "║ ✗ FFI coverage gap — fix ffi_mapping.json"; exit 1; }

echo "║ [4/8] Generating C# SDK..."
python3 codegen/gen_csharp.py

echo "║ [5/8] Generating Python SDK..."
python3 codegen/gen_python.py

echo "║ [6/8] Generating TypeScript Node SDK..."
python3 codegen/gen_ts_node.py

echo "║ [7/8] Generating TypeScript Web SDK..."
python3 codegen/gen_ts_web.py

echo "║ [8/8] Validating schema consistency..."
python3 codegen/validate.py || { echo "║ ✗ Schema mismatch — fix goud_sdk.schema.json"; exit 1; }

echo "╠══════════════════════════════════════════════════════════╣"
echo "║ ✓ All SDKs generated from goud_sdk.schema.json          ║"
echo "╚══════════════════════════════════════════════════════════╝"
