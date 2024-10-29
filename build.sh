#!/bin/bash
set -e

echo "Building GoudEngine..."

# Go to the build directory and build the project
if [ ! -d "build" ]; then
  echo "Build directory does not exist. Please run ./init.sh first."
  exit 1
fi

cd build
cmake --build . --config Release

echo "Build complete."