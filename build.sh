#!/bin/bash
# build.sh

# echo "Building the project in release mode..."
# cargo build --release --workspace

if [[ "$1" == "--release" || "$1" == "--prod" ]]; then
    echo "Building the project in release mode..."
    cargo build --release --workspace
elif [[ "$1" == "--local" ]]; then
    echo "Building the project in local mode..."
    cargo build --workspace
else
    echo "Building the project in debug mode..."
    cargo build --workspace
fi

# NOTE: This uses cbindgen to generate the bindings.h file. This is not needed for now as we switched to cs_bindgen.
# echo "Build complete."
echo "Build complete."
echo "Copying libgoud_engine.dylib to sdks/GoudEngine..."
cp target/release/libgoud_engine.dylib sdks/GoudEngine/runtimes/osx-x64/native/

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
