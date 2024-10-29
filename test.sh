#!/bin/bash
set -e

echo "Running tests for GoudEngine..."

# Ensure the build directory exists and tests are built
if [ ! -d "build" ]; then
  echo "Build directory does not exist. Please run ./init.sh first."
  exit 1
fi

# Navigate to the build directory and run CMake tests
cd build
ctest --output-on-failure

echo "Testing complete."