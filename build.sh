#!/bin/bash
# build.sh

echo "Building the project in release mode..."
cargo build --release --workspace
echo "Build complete."
echo "Copying sdk_bindings/bindings.h and target/release/libsdk_bindings.dylib to sample_net_app. TODO: This should be OS specific."
cp sdk_bindings/bindings.h sample_net_app/
cp target/release/libsdk_bindings.dylib sample_net_app/
echo "Copied files to sample_net_app."
