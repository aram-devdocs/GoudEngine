#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

INCLUDE_DOCS=0
if [[ "${1:-}" == "--docs" ]]; then
  INCLUDE_DOCS=1
elif [[ "${1:-}" != "" ]]; then
  echo "Usage: $0 [--docs]"
  exit 1
fi

delete_path() {
  local path="$1"
  if [[ -e "$path" ]]; then
    rm -rf "$path"
    echo "Removed: $path"
  fi
}

generated_outputs=(
  "codegen/ffi_manifest.json"
  "codegen/generated/goud_engine.h"
  "docs/src/generated/snippets"
  "docs/src/guides/showcase.md"
  "examples/README.md"
  "sdks/csharp/generated"
  "sdks/csharp/GoudEngine.csproj"
  "sdks/csharp/build/GoudEngine.targets"
  "sdks/csharp/NetworkManager.cs"
  "sdks/csharp/NetworkEndpoint.cs"
  "sdks/python/pyproject.toml"
  "sdks/python/MANIFEST.in"
  "sdks/python/goud_engine/generated"
  "sdks/python/goud_engine/__init__.py"
  "sdks/python/goud_engine/networking.py"
  "sdks/typescript/package.json"
  "sdks/typescript/tsconfig.json"
  "sdks/typescript/tsconfig.web.json"
  "sdks/typescript/tsconfig.typedoc.json"
  "sdks/typescript/index.js"
  "sdks/typescript/index.d.ts"
  "sdks/typescript/native/Cargo.toml"
  "sdks/typescript/native/build.rs"
  "sdks/typescript/wasm/package.json"
  "sdks/typescript/src/generated"
  "sdks/typescript/src/index.ts"
  "sdks/typescript/src/node/index.ts"
  "sdks/typescript/src/web/index.ts"
  "sdks/typescript/src/shared/network.ts"
  "sdks/typescript/native/src/lib.rs"
)

for path in "${generated_outputs[@]}"; do
  delete_path "$path"
done

for path in sdks/typescript/native/src/*.g.rs; do
  [[ -e "$path" ]] || continue
  delete_path "$path"
done

for path in sdks/typescript/*.node; do
  [[ -e "$path" ]] || continue
  delete_path "$path"
done

if [[ "$INCLUDE_DOCS" -eq 1 ]]; then
  docs_outputs=(
    "docs/book"
    "docs/src/generated/downloads"
    "docs/src/generated/media"
    "docs/src/generated/showcase"
    "target/doc"
    "sdks/csharp/_site"
  )
  for path in "${docs_outputs[@]}"; do
    delete_path "$path"
  done
fi

./codegen.sh

if [[ "$INCLUDE_DOCS" -eq 1 ]]; then
  (
    cd sdks/typescript
    npm ci
    npx playwright install --with-deps chromium
    node scripts/generate-doc-media.mjs
  )
  python3 scripts/generate-showcase-docs.py
else
  python3 scripts/generate-showcase-docs.py
fi

if [[ "$INCLUDE_DOCS" -eq 1 ]]; then
  mkdir -p docs/src/generated/downloads
  rm -f docs/src/generated/downloads/*.zip
  zip -qr docs/src/generated/downloads/flappy-csharp.zip examples/csharp/flappy_goud
  zip -qr docs/src/generated/downloads/flappy-python.zip examples/python/flappy_bird.py
  zip -qr docs/src/generated/downloads/flappy-typescript.zip examples/typescript/flappy_bird
  zip -qr docs/src/generated/downloads/flappy-rust.zip examples/rust/flappy_bird
fi

if [[ "$INCLUDE_DOCS" -eq 1 ]]; then
  bash scripts/check-generated-artifacts.sh --docs
else
  bash scripts/check-generated-artifacts.sh
fi

if [[ "$INCLUDE_DOCS" -eq 1 ]]; then
  python3 scripts/generate-doc-snippets.py
  cargo build --release
  cargo doc --no-deps -p goud-engine-core -p goud-engine
  mdbook build

  mkdir -p docs/book/api
  rm -rf docs/book/api/rust docs/book/api/python docs/book/api/typescript docs/book/api/csharp
  cp -r target/doc docs/book/api/rust

  GOUD_ENGINE_LIB="$ROOT_DIR/target/release" \
  PDOC_ALLOW_EXEC=1 \
  PYTHONPATH="$ROOT_DIR/sdks/python" \
  pdoc --output-dir docs/book/api/python sdks/python/goud_engine

  (
    cd sdks/typescript
    npm run build:native:debug
    npm run build:ts
    npx typedoc \
      --tsconfig tsconfig.typedoc.json \
      --entryPoints src/generated/node/index.g.ts src/generated/web/index.g.ts src/generated/types/engine.g.ts src/generated/types/math.g.ts \
      --out ../../docs/book/api/typescript \
      --name "GoudEngine TypeScript SDK"
  )

  (
    cd sdks/csharp
    dotnet restore GoudEngine.csproj
    dotnet build -c Release --no-restore
    docfx build docfx.json
    cp -r _site ../../docs/book/api/csharp
  )
  rm -rf sdks/csharp/_site
fi

echo "Clean-room regeneration completed successfully."
