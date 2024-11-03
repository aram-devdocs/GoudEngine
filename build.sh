#!/bin/bash
# build.sh

echo "Building the project in release mode..."
cargo build --release --workspace
echo "Build complete."