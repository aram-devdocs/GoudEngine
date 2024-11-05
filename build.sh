#!/bin/bash
# build.sh

echo "Building the project in release mode..."
cargo build --release --workspace

# NOTE: This uses cbindgen to generate the bindings.h file. This is not needed for now as we switched to cs_bindgen.
# echo "Build complete."
echo "Copying and target/release/libgame.dylib to sample_net_app. TODO: This should be OS specific."
# cp game/bindings.h sample_net_app/
cp target/release/libgame.dylib sample_net_app/
# echo "Copied files to sample_net_app."
