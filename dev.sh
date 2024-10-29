#!/bin/bash
set -e

echo "Setting up GoudEngine for development..."

# Check if build directory exists
if [ ! -d "build" ]; then
  echo "Build directory does not exist. Please run ./init.sh first."
  exit 1
fi

# Configure and build in Debug mode
cd build
cmake -DCMAKE_BUILD_TYPE=Debug ..
cmake --build . --config Debug

echo "Development build complete. You can now start coding with live reload (if supported)."