#!/bin/bash
# generate.sh

echo "Warning: Deprecated in favor of csbindgen, which runs at build time and is configured in sdk_bindings/Cargo.toml and sdk_bindings/build.rs."

# echo "Generating C bindings for SDK..."
# cbindgen --config sdk_bindings/cbindgen.toml --crate sdk_bindings --output sdk_bindings/bindings.h
# echo "Bindings generated in sdk_bindings/bindings.h."
