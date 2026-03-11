#!/bin/bash
# Regenerate all SDKs from the universal schema.
# Run this after modifying codegen/goud_sdk.schema.json or codegen/ffi_mapping.json.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
cd "$SCRIPT_DIR"

echo "╔══════════════════════════════════════════════════════════╗"
echo "║         GoudEngine SDK Code Generation                   ║"
echo "╠══════════════════════════════════════════════════════════╣"

echo "║ [1/12] Generating SDK package/build scaffolding..."
python3 codegen/gen_sdk_scaffolding.py

echo "║ [2/12] Building Rust engine (extracts FFI manifest)..."
build_log="$(mktemp)"
if ! cargo build -p goud-engine-core -p goud-engine >"$build_log" 2>&1; then
  grep -v "^$" "$build_log" | head -20 || true
  rm -f "$build_log"
  echo "║ ✗ Rust build failed — codegen requires a fresh ffi_manifest.json"
  exit 1
fi
grep -v "^$" "$build_log" | head -5 || true
rm -f "$build_log"

echo "║ [3/12] Bootstrapping TypeScript Node SDK sources..."
python3 codegen/gen_ts_node.py

echo "║ [4/12] Checking layer dependencies..."
cargo run -p lint-layers || { echo "║ ✗ Layer violation — fix imports"; exit 1; }

echo "║ [5/12] Validating FFI coverage (manifest vs mapping)..."
python3 codegen/validate_coverage.py || { echo "║ ✗ FFI coverage gap — fix ffi_mapping.json"; exit 1; }

echo "║ [6/12] Generating C# SDK..."
python3 codegen/gen_csharp.py

echo "║ [7/12] Generating Python SDK..."
python3 codegen/gen_python.py

echo "║ [8/12] Regenerating TypeScript Node SDK..."
python3 codegen/gen_ts_node.py

echo "║ [9/12] Generating TypeScript Web SDK..."
python3 codegen/gen_ts_web.py

echo "║ [10/12] Formatting generated Rust sources..."
cargo fmt -p goud-engine-node

echo "║ [11/12] Validating schema consistency..."
python3 codegen/validate.py || { echo "║ ✗ Schema mismatch — fix goud_sdk.schema.json"; exit 1; }

echo "║ [12/12] Generating docs snippets from validated sources..."
python3 scripts/generate-doc-snippets.py

echo "╠══════════════════════════════════════════════════════════╣"
echo "║ ✓ All SDKs generated from goud_sdk.schema.json          ║"
echo "╚══════════════════════════════════════════════════════════╝"
