#!/bin/bash
# build.sh

echo "Building the project in release mode..."
cargo build --release --workspace

# NOTE: This uses cbindgen to generate the bindings.h file. This is not needed for now as we switched to cs_bindgen.
# echo "Build complete."
echo "Copying and target/release/libgoud_engine.dylib to flappy_goud. TODO: This should be OS specific."
# cp goud_enginebindings.h flappy_goud/
cp target/release/libgoud_engine.dylib GoudEngine/runtimes/native/osx-x64/
# cp target/release/libgoud_engine.dylib GoudEngine/

# echo "Copied files to flappy_goud."

cd GoudEngine

if [[ "$1" == "--release" ]]; then
    echo "Building the project in release mode..."
    dotnet build -c Release
    dotnet pack -c Release
else
    echo "Building the project in debug mode..."
    dotnet build -c Debug
    dotnet pack -c Debug
fi

echo "Build complete."
