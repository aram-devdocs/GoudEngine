#!/bin/bash
# Regenerate all SDKs from the universal schema.
# Run this after modifying codegen/goud_sdk.schema.json or codegen/ffi_mapping.json.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
cd "$SCRIPT_DIR"
HEADER_SOURCE="codegen/generated/goud_engine.h"

stage_header_copy() {
  local destination_dir="$1"
  if [ ! -f "$HEADER_SOURCE" ]; then
    echo "Missing generated header at $HEADER_SOURCE"
    return 1
  fi

  mkdir -p "$destination_dir"
  cp "$HEADER_SOURCE" "$destination_dir/goud_engine.h"
}

stage_file_copy() {
  local source_path="$1"
  local destination_path="$2"
  if [ ! -f "$source_path" ]; then
    echo "Missing staged source file at $source_path"
    return 1
  fi

  mkdir -p "$(dirname "$destination_path")"
  cp "$source_path" "$destination_path"
}

echo "╔══════════════════════════════════════════════════════════╗"
echo "║         GoudEngine SDK Code Generation                   ║"
echo "╠══════════════════════════════════════════════════════════╣"

echo "║ [1/15] Generating SDK package/build scaffolding..."
python3 codegen/gen_sdk_scaffolding.py

echo "║ [2/15] Building Rust engine (extracts FFI manifest and C header)..."
build_log="$(mktemp)"
if ! cargo build -p goud-engine-core -p goud-engine >"$build_log" 2>&1; then
  grep -v "^$" "$build_log" | head -20 || true
  rm -f "$build_log"
  echo "║ ✗ Rust build failed — codegen requires a fresh ffi_manifest.json"
  exit 1
fi
grep -v "^$" "$build_log" | head -5 || true
rm -f "$build_log"

echo "║ [3/15] Validating generated C header..."
stage_header_copy "sdks/c/include"
stage_header_copy "sdks/cpp/include"
stage_header_copy "sdks/csharp/include"
stage_header_copy "sdks/python/goud_engine/include"
stage_header_copy "sdks/go/include"
stage_header_copy "sdks/swift/Sources/CGoudEngine/include"
stage_file_copy "sdks/c/include/goud/goud.h" "sdks/cpp/include/goud/goud.h"
python3 scripts/validate_c_header.py

echo "║ [4/15] Bootstrapping TypeScript Node SDK sources..."
python3 codegen/gen_ts_node.py

echo "║ [5/15] Checking layer dependencies..."
cargo run -p lint-layers || { echo "║ ✗ Layer violation — fix imports"; exit 1; }

echo "║ [6/15] Validating FFI coverage (manifest vs mapping)..."
python3 codegen/validate_coverage.py || { echo "║ ✗ FFI coverage gap — fix ffi_mapping.json"; exit 1; }

echo "║ [7/15] Generating C# SDK..."
python3 codegen/gen_csharp.py

echo "║ [8/15] Generating Python SDK..."
python3 codegen/gen_python.py

echo "║ [9/15] Generating Go SDK cgo bindings..."
python3 codegen/gen_go.py

echo "║ [9b/15] Generating Go SDK wrapper package..."
python3 codegen/gen_go_sdk.py

echo "║ [10/15] Regenerating TypeScript Node SDK..."
python3 codegen/gen_ts_node.py

echo "║ [11/15] Generating TypeScript Web SDK..."
python3 codegen/gen_ts_web.py

echo "║ [11b/15] Generating Swift SDK..."
stage_header_copy "sdks/swift/Sources/CGoudEngine/include"
python3 codegen/gen_swift.py

echo "║ [12/15] Generating Kotlin SDK..."
python3 codegen/gen_kotlin.py

echo "║ [13/15] Formatting generated Rust sources..."
cargo fmt -p goud-engine-node

echo "║ [14/15] Validating schema consistency..."
python3 codegen/validate.py || { echo "║ ✗ Schema mismatch — fix goud_sdk.schema.json"; exit 1; }

echo "║ [15/15] Generating docs snippets from validated sources..."
python3 scripts/generate-doc-snippets.py

echo "╠══════════════════════════════════════════════════════════╣"
echo "║ ✓ All SDKs generated from goud_sdk.schema.json          ║"
echo "╚══════════════════════════════════════════════════════════╝"
