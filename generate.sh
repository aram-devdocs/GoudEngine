#!/bin/bash
# generate.sh

echo "Warning: Deprecated in favor of csbindgen, which runs at build time and is configured in game/Cargo.toml and game/build.rs."

echo "Generating C bindings for SDK..."
cbindgen --config game/cbindgen.toml --crate game --output game/bindings.h
echo "Bindings generated in game/bindings.h."
