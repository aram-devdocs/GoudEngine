#!/bin/bash
# build.sh
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BUILD_MODE="debug"
TARGET_DIR="target/debug"
CARGO_MODE_ARGS=()
BUILD_SCOPE="workspace"
HOST_RUNTIME_ONLY=false
SKIP_CSHARP_SDK_BUILD=false
HEADER_SOURCE="codegen/generated/goud_engine.h"

while [[ "$#" -gt 0 ]]; do
    case "$1" in
        --release|--prod)
            BUILD_MODE="release"
            TARGET_DIR="target/release"
            CARGO_MODE_ARGS=(--release)
            ;;
        --local)
            BUILD_MODE="debug"
            TARGET_DIR="target/debug"
            CARGO_MODE_ARGS=()
            ;;
        --core-only)
            BUILD_SCOPE="core"
            ;;
        --host-runtime-only)
            HOST_RUNTIME_ONLY=true
            ;;
        --skip-csharp-sdk-build)
            SKIP_CSHARP_SDK_BUILD=true
            ;;
        *)
            echo "Unknown parameter: $1"
            echo "Usage: ./build.sh [--release|--prod|--local] [--core-only] [--host-runtime-only] [--skip-csharp-sdk-build]"
            exit 1
            ;;
    esac
    shift
done

stage_header_copy() {
    local destination_dir="$1"
    if [ ! -f "$HEADER_SOURCE" ]; then
        echo "Warning: generated header not found at $HEADER_SOURCE"
        return 1
    fi

    mkdir -p "$destination_dir"
    cp "$HEADER_SOURCE" "$destination_dir/goud_engine.h"
    echo "  staged header: $destination_dir/goud_engine.h"
}

stage_file_copy() {
    local source_path="$1"
    local destination_path="$2"
    if [ ! -f "$source_path" ]; then
        echo "Warning: expected file not found at $source_path"
        return 1
    fi

    mkdir -p "$(dirname "$destination_path")"
    cp "$source_path" "$destination_path"
    echo "  staged file: $destination_path"
}

if [[ "$BUILD_MODE" == "release" ]]; then
    echo "Building the project in release mode..."
    if [[ "$BUILD_SCOPE" == "core" ]]; then
        cargo build --release -p goud-engine-core
    else
        cargo build --release --workspace
    fi

    # Package assets into a single archive for distribution.
    # Games can also call goud_engine::assets::packager::package_directory() directly.
    if [ -d "assets" ] && [[ "$BUILD_SCOPE" != "core" ]]; then
        echo "Packaging assets..."
        cargo run --release --example package_assets 2>/dev/null || echo "Note: Asset packager example not found, skipping asset bundling"
    fi
elif [[ "$BUILD_SCOPE" == "core" ]]; then
    echo "Building the project in local core-only mode..."
    cargo build -p goud-engine-core
elif [[ "$BUILD_MODE" == "debug" ]]; then
    echo "Building the project in debug mode..."
    cargo build --workspace
fi

echo "Build complete."

echo "Staging generated C header..."
stage_header_copy "$TARGET_DIR/include"
stage_header_copy "sdks/c/include"
stage_header_copy "sdks/cpp/include"
stage_header_copy "sdks/csharp/include"
stage_header_copy "sdks/python/goud_engine/include"
stage_header_copy "sdks/go/include"
stage_file_copy "sdks/c/include/goud/goud.h" "sdks/cpp/include/goud/goud.h"

# Copy native library based on platform
if [[ "$OSTYPE" == "darwin"* ]]; then
    echo "Copying libgoud_engine.dylib to sdks/csharp runtimes..."
    mkdir -p sdks/csharp/runtimes/osx-x64/native/ sdks/csharp/runtimes/osx-arm64/native/

    HOST_ARCH="$(uname -m)"
    TARGET_MODE_DIR="$(dirname "$TARGET_DIR")"
    SECONDARY_TARGET=""

    if [ "$HOST_ARCH" = "arm64" ]; then
      SECONDARY_TARGET="x86_64-apple-darwin"
    elif [ "$HOST_ARCH" = "x86_64" ]; then
      SECONDARY_TARGET="aarch64-apple-darwin"
    fi

    if [ "$HOST_RUNTIME_ONLY" = false ] && [ -n "$SECONDARY_TARGET" ] && rustup target list --installed | grep -qx "$SECONDARY_TARGET"; then
      echo "Building secondary macOS target for C# runtime packaging: $SECONDARY_TARGET"
      if [ ${#CARGO_MODE_ARGS[@]} -gt 0 ]; then
        cargo build "${CARGO_MODE_ARGS[@]}" -p goud-engine-core --target "$SECONDARY_TARGET"
      else
        cargo build -p goud-engine-core --target "$SECONDARY_TARGET"
      fi
    elif [ "$HOST_RUNTIME_ONLY" = false ] && [ -n "$SECONDARY_TARGET" ]; then
      echo "  warning: Rust target '$SECONDARY_TARGET' is not installed; only the host macOS runtime will be refreshed."
    fi

    copy_if_exists() {
      local source_path="$1"
      local runtime_dir="$2"
      local label="$3"
      if [ -f "$source_path" ]; then
        cp "$source_path" "$runtime_dir/libgoud_engine.dylib"
        echo "  copied $label: $source_path -> $runtime_dir/"
        return 0
      fi
      return 1
    }

    COPIED=0
    if [ "$HOST_RUNTIME_ONLY" = true ]; then
      if [ "$HOST_ARCH" = "x86_64" ]; then
        copy_if_exists "$TARGET_DIR/libgoud_engine.dylib" "sdks/csharp/runtimes/osx-x64/native" "host-only (x86_64)" && COPIED=1 || true
      elif [ "$HOST_ARCH" = "arm64" ]; then
        copy_if_exists "$TARGET_DIR/libgoud_engine.dylib" "sdks/csharp/runtimes/osx-arm64/native" "host-only (arm64)" && COPIED=1 || true
      fi
    elif [ "$HOST_ARCH" = "x86_64" ]; then
      copy_if_exists "$TARGET_DIR/libgoud_engine.dylib" "sdks/csharp/runtimes/osx-x64/native" "host (x86_64)" && COPIED=1 || true
      copy_if_exists "$TARGET_MODE_DIR/x86_64-apple-darwin/$BUILD_MODE/libgoud_engine.dylib" "sdks/csharp/runtimes/osx-x64/native" "cross x86_64 target" && COPIED=1 || true
      copy_if_exists "$TARGET_MODE_DIR/aarch64-apple-darwin/$BUILD_MODE/libgoud_engine.dylib" "sdks/csharp/runtimes/osx-arm64/native" "cross arm64 target" && COPIED=1 || true
    elif [ "$HOST_ARCH" = "arm64" ]; then
      copy_if_exists "$TARGET_DIR/libgoud_engine.dylib" "sdks/csharp/runtimes/osx-arm64/native" "host (arm64)" && COPIED=1 || true
      copy_if_exists "$TARGET_MODE_DIR/aarch64-apple-darwin/$BUILD_MODE/libgoud_engine.dylib" "sdks/csharp/runtimes/osx-arm64/native" "cross arm64 target" && COPIED=1 || true
      copy_if_exists "$TARGET_MODE_DIR/x86_64-apple-darwin/$BUILD_MODE/libgoud_engine.dylib" "sdks/csharp/runtimes/osx-x64/native" "cross x86_64 target" && COPIED=1 || true
    else
      # Unknown host architecture; preserve previous behavior but keep explicit copies.
      echo "  warning: unknown macOS architecture '$HOST_ARCH'; attempting default output copy"
      copy_if_exists "$TARGET_DIR/libgoud_engine.dylib" "sdks/csharp/runtimes/osx-x64/native" "host fallback" && COPIED=1 || true
      copy_if_exists "$TARGET_DIR/libgoud_engine.dylib" "sdks/csharp/runtimes/osx-arm64/native" "host fallback" && COPIED=1 || true
    fi

    if [ "$COPIED" -eq 0 ]; then
      echo "Warning: libgoud_engine.dylib not found in any expected macOS output path."
    fi
elif [[ "$OSTYPE" == "linux-gnu"* ]]; then
    echo "Copying libgoud_engine.so to sdks/csharp..."
    mkdir -p sdks/csharp/runtimes/linux-x64/native/
    cp "$TARGET_DIR/libgoud_engine.so" sdks/csharp/runtimes/linux-x64/native/ 2>/dev/null || echo "Warning: .so not found"
elif [[ "$OSTYPE" == "msys"* || "$OSTYPE" == "cygwin"* ]]; then
    echo "Copying goud_engine.dll to sdks/csharp..."
    mkdir -p sdks/csharp/runtimes/win-x64/native/
    cp "$TARGET_DIR/goud_engine.dll" sdks/csharp/runtimes/win-x64/native/ 2>/dev/null || echo "Warning: .dll not found"
fi

if [ "$SKIP_CSHARP_SDK_BUILD" = false ]; then
    cd sdks/csharp

    if [[ "$BUILD_MODE" == "release" ]]; then
        echo "Building the project in release mode..."
        dotnet build -c Release
    else
        echo "Building the project in debug mode..."
        dotnet build -c Debug
    fi

    echo "Build complete."
else
    echo "Skipping sdks/csharp build because the caller is using a direct project-reference fast path."
fi

# Clean old .nupkg files from package output (keep only latest)
NUPKG_DIR="$SCRIPT_DIR/sdks/nuget_package_output"
if [ -d "$NUPKG_DIR" ]; then
    shopt -s nullglob
    nupkgs=("$NUPKG_DIR"/*.nupkg)
    shopt -u nullglob

    if [ "${#nupkgs[@]}" -gt 1 ]; then
        LATEST=$(ls -1t "${nupkgs[@]}" | head -1)
        for pkg in "${nupkgs[@]}"; do
            if [ "$pkg" != "$LATEST" ]; then
                echo "Removing old package: $(basename "$pkg")"
                rm -f "$pkg"
            fi
        done
    fi
fi
