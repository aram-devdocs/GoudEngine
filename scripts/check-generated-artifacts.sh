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

base_required_paths=(
  "codegen/ffi_manifest.json"
  "codegen/generated/goud_engine.h"
  "sdks/csharp/generated/NativeMethods.g.cs"
  "sdks/csharp/GoudEngine.csproj"
  "sdks/csharp/build/GoudEngine.targets"
  "sdks/csharp/generated/GoudContext.g.cs"
  "sdks/csharp/NetworkManager.cs"
  "sdks/csharp/NetworkEndpoint.cs"
  "sdks/python/pyproject.toml"
  "sdks/python/MANIFEST.in"
  "sdks/python/goudengine/__init__.py"
  "sdks/python/goudengine/generated/_ffi.py"
  "sdks/python/goudengine/generated/_game.py"
  "sdks/python/goudengine/networking.py"
  "sdks/typescript/package.json"
  "sdks/typescript/tsconfig.json"
  "sdks/typescript/tsconfig.web.json"
  "sdks/typescript/tsconfig.typedoc.json"
  "sdks/typescript/native/Cargo.toml"
  "sdks/typescript/native/build.rs"
  "sdks/typescript/wasm/package.json"
  "sdks/typescript/src/generated/node/index.g.ts"
  "sdks/typescript/src/generated/web/index.g.ts"
  "sdks/typescript/src/generated/types/input.g.ts"
  "sdks/typescript/src/index.ts"
  "sdks/typescript/src/node/index.ts"
  "sdks/typescript/src/web/index.ts"
  "sdks/typescript/src/shared/network.ts"
  "sdks/typescript/native/src/audio.g.rs"
  "sdks/typescript/native/src/components.g.rs"
  "sdks/typescript/native/src/entity.g.rs"
  "sdks/typescript/native/src/game.g.rs"
  "sdks/typescript/native/src/lib.rs"
  "sdks/typescript/native/src/types.g.rs"
)

docs_required_paths=(
  "docs/src/generated/showcase/flappy-goud-csharp-desktop.png"
  "docs/src/generated/showcase/flappy-bird-python-desktop.png"
  "docs/src/generated/showcase/flappy-bird-typescript-desktop.png"
  "docs/src/generated/showcase/flappy-bird-typescript-web.png"
  "docs/src/generated/showcase/flappy-bird-rust-desktop.png"
  "docs/src/generated/showcase/sandbox-csharp-desktop.png"
  "docs/src/generated/showcase/sandbox-python-desktop.png"
  "docs/src/generated/showcase/sandbox-typescript-desktop.png"
  "docs/src/generated/showcase/sandbox-typescript-web.png"
  "docs/src/generated/showcase/sandbox-rust-desktop.png"
  "docs/src/generated/showcase/feature-lab-csharp-headless.png"
  "docs/src/generated/showcase/feature-lab-python-headless.png"
  "docs/src/generated/showcase/feature-lab-typescript-desktop.png"
  "docs/src/generated/showcase/feature-lab-typescript-web.png"
  "docs/src/generated/showcase/feature-lab-rust-headless.png"
  "docs/src/generated/showcase/3d-cube-csharp-desktop.png"
  "docs/src/generated/showcase/goud-jumper-csharp-desktop.png"
  "docs/src/generated/showcase/isometric-rpg-csharp-desktop.png"
  "docs/src/generated/showcase/hello-ecs-csharp-desktop.png"
  "docs/src/generated/showcase/python-sdk-demo-python-desktop.png"
  "docs/src/generated/downloads/flappy-csharp.zip"
  "docs/src/generated/downloads/flappy-python.zip"
  "docs/src/generated/downloads/flappy-typescript.zip"
  "docs/src/generated/downloads/flappy-rust.zip"
  "docs/src/generated/media/getting-started-typescript-web.webm"
  "docs/src/generated/media/getting-started-typescript-web.vtt"
  "docs/src/generated/snippets/csharp/first-project.md"
  "docs/src/generated/snippets/python/first-project.md"
  "docs/src/generated/snippets/rust/first-project.md"
  "docs/src/generated/snippets/typescript/first-project-desktop.md"
  "docs/src/guides/showcase.md"
  "examples/README.md"
)

required_paths=("${base_required_paths[@]}")
if [[ "$INCLUDE_DOCS" -eq 1 ]]; then
  required_paths+=("${docs_required_paths[@]}")
fi

missing=0
for path in "${required_paths[@]}"; do
  if [[ ! -e "$path" ]]; then
    echo "Missing generated artifact: $path"
    missing=1
  fi
done

if [[ "$missing" -ne 0 ]]; then
  exit 1
fi

echo "Validating generated C header..."
python3 scripts/validate_c_header.py

base_tracked_diff_paths=(
  "codegen/ffi_manifest.json"
  "codegen/generated/goud_engine.h"
  "sdks/csharp/generated"
  "sdks/csharp/GoudEngine.csproj"
  "sdks/csharp/build/GoudEngine.targets"
  "sdks/csharp/NetworkManager.cs"
  "sdks/csharp/NetworkEndpoint.cs"
  "sdks/python/pyproject.toml"
  "sdks/python/MANIFEST.in"
  "sdks/python/goudengine/__init__.py"
  "sdks/python/goudengine/generated"
  "sdks/python/goudengine/networking.py"
  "sdks/typescript/package.json"
  "sdks/typescript/tsconfig.json"
  "sdks/typescript/tsconfig.web.json"
  "sdks/typescript/tsconfig.typedoc.json"
  "sdks/typescript/native/Cargo.toml"
  "sdks/typescript/native/build.rs"
  "sdks/typescript/wasm/package.json"
  "sdks/typescript/src/generated"
  "sdks/typescript/src/index.ts"
  "sdks/typescript/src/node/index.ts"
  "sdks/typescript/src/web/index.ts"
  "sdks/typescript/src/shared/network.ts"
  "sdks/typescript/native/src/audio.g.rs"
  "sdks/typescript/native/src/components.g.rs"
  "sdks/typescript/native/src/entity.g.rs"
  "sdks/typescript/native/src/game.g.rs"
  "sdks/typescript/native/src/lib.rs"
  "sdks/typescript/native/src/types.g.rs"
)

docs_tracked_diff_paths=(
  "docs/src/generated/snippets"
  "docs/src/guides/showcase.md"
  "examples/README.md"
)

tracked_diff_paths=("${base_tracked_diff_paths[@]}")
if [[ "$INCLUDE_DOCS" -eq 1 ]]; then
  tracked_diff_paths+=("${docs_tracked_diff_paths[@]}")
fi

generated_scan_paths=(
  "codegen/generated"
  "docs/src/generated/showcase"
  "docs/src/generated/snippets"
  "sdks/csharp/generated"
  "sdks/python/goudengine/generated"
  "sdks/typescript/src/generated"
  "sdks/typescript/native/src"
)

echo "Checking tracked generated drift..."
git diff --exit-code -- "${tracked_diff_paths[@]}"

if [[ "$INCLUDE_DOCS" -eq 1 ]]; then
  echo "Skipping byte-for-byte drift on generated downloads/media..."
  echo "Download bundles and captured media are still required above, but their"
  echo "archive/video containers are not stable enough to diff across clean-room runs."
  echo "Those binaries are also ignored in Git so clean-room docs runs do not dirty the tree."
  echo "Skipping byte-for-byte drift on generated showcase screenshots..."
  echo "Showcase PNGs are still required above, but renderer/font/runtime differences"
  echo "make them unsuitable for cross-environment binary drift enforcement."

  echo "Checking generated showcase docs drift..."
  python3 scripts/generate-showcase-docs.py --check
fi

echo "Checking for ignored or untracked generated artifacts..."
untracked_generated=()
while IFS= read -r path; do
  untracked_generated+=("$path")
done < <(
  git ls-files --others --ignored --exclude-standard -- "${generated_scan_paths[@]}" |
    grep -Ev '(__pycache__/|\.pyc$|\.pyo$)' || true
)

if [[ "${#untracked_generated[@]}" -ne 0 ]]; then
  printf 'Untracked generated artifacts detected:\n'
  printf '  %s\n' "${untracked_generated[@]}"
  exit 1
fi

echo "Generated artifact check passed."
