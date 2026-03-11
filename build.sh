#!/bin/bash
# build.sh

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BUILD_MODE="debug"
TARGET_DIR="target/debug"
CARGO_MODE_ARGS=()

if [[ "$1" == "--release" || "$1" == "--prod" ]]; then
    echo "Building the project in release mode..."
    cargo build --release --workspace
    BUILD_MODE="release"
    TARGET_DIR="target/release"
    CARGO_MODE_ARGS=(--release)

    # Package assets into a single archive for distribution.
    # Games can also call goud_engine::assets::packager::package_directory() directly.
    if [ -d "assets" ]; then
        echo "Packaging assets..."
        cargo run --release --example package_assets 2>/dev/null || echo "Note: Asset packager example not found, skipping asset bundling"
    fi
elif [[ "$1" == "--local" ]]; then
    echo "Building the project in local mode..."
    cargo build --workspace
else
    echo "Building the project in debug mode..."
    cargo build --workspace
fi

echo "Build complete."

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

    if [ -n "$SECONDARY_TARGET" ] && rustup target list --installed | grep -qx "$SECONDARY_TARGET"; then
      echo "Building secondary macOS target for C# runtime packaging: $SECONDARY_TARGET"
      cargo build "${CARGO_MODE_ARGS[@]}" -p goud-engine-core --target "$SECONDARY_TARGET"
    elif [ -n "$SECONDARY_TARGET" ]; then
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
    if [ "$HOST_ARCH" = "x86_64" ]; then
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

cd sdks/csharp

if [[ "$1" == "--release" || "$1" == "--prod" ]]; then
    echo "Building the project in release mode..."
    dotnet build -c Release
elif [[ "$1" == "--local" ]]; then
    echo "Building the project in local mode..."
    dotnet build -c Debug
else
    echo "Building the project in debug mode..."
    dotnet build -c Debug
fi

echo "Build complete."

# Clean old .nupkg files from package output (keep only latest)
NUPKG_DIR="$SCRIPT_DIR/sdks/nuget_package_output"
if [ -d "$NUPKG_DIR" ]; then
    NUPKG_COUNT=$(ls -1 "$NUPKG_DIR"/*.nupkg 2>/dev/null | wc -l)
    if [ "$NUPKG_COUNT" -gt 1 ]; then
        LATEST=$(ls -1t "$NUPKG_DIR"/*.nupkg 2>/dev/null | head -1)
        for pkg in "$NUPKG_DIR"/*.nupkg; do
            if [ "$pkg" != "$LATEST" ]; then
                echo "Removing old package: $(basename "$pkg")"
                rm -f "$pkg"
            fi
        done
    fi
fi
