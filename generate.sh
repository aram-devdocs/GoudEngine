#!/bin/bash
# generate.sh

echo "Generating C bindings for SDK..."
cbindgen --config sdk_bindings/cbindgen.toml --crate sdk_bindings --output sdk_bindings/bindings.h
echo "Bindings generated in sdk_bindings/bindings.h."