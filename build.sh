#!/bin/bash
# build.sh

BUILD_MODE="debug"
TARGET_DIR="target/debug"

if [[ "$1" == "--release" || "$1" == "--prod" ]]; then
    echo "Building the project in release mode..."
    cargo build --release --workspace
    BUILD_MODE="release"
    TARGET_DIR="target/release"
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
    echo "Copying libgoud_engine.dylib to sdks/GoudEngine..."
    mkdir -p sdks/GoudEngine/runtimes/osx-x64/native/
    cp "$TARGET_DIR/libgoud_engine.dylib" sdks/GoudEngine/runtimes/osx-x64/native/ 2>/dev/null || echo "Warning: dylib not found"
elif [[ "$OSTYPE" == "linux-gnu"* ]]; then
    echo "Copying libgoud_engine.so to sdks/GoudEngine..."
    mkdir -p sdks/GoudEngine/runtimes/linux-x64/native/
    cp "$TARGET_DIR/libgoud_engine.so" sdks/GoudEngine/runtimes/linux-x64/native/ 2>/dev/null || echo "Warning: .so not found"
elif [[ "$OSTYPE" == "msys"* || "$OSTYPE" == "cygwin"* ]]; then
    echo "Copying goud_engine.dll to sdks/GoudEngine..."
    mkdir -p sdks/GoudEngine/runtimes/win-x64/native/
    cp "$TARGET_DIR/goud_engine.dll" sdks/GoudEngine/runtimes/win-x64/native/ 2>/dev/null || echo "Warning: .dll not found"
fi

cd sdks/GoudEngine

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
