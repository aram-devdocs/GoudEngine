#!/bin/bash
# build.sh

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BUILD_MODE="debug"
TARGET_DIR="target/debug"

if [[ "$1" == "--release" || "$1" == "--prod" ]]; then
    echo "Building the project in release mode..."
    cargo build --release --workspace
    BUILD_MODE="release"
    TARGET_DIR="target/release"

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
    echo "Copying libgoud_engine.dylib to sdks/csharp..."
    mkdir -p sdks/csharp/runtimes/osx-x64/native/
    cp "$TARGET_DIR/libgoud_engine.dylib" sdks/csharp/runtimes/osx-x64/native/ 2>/dev/null || echo "Warning: dylib not found"
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
