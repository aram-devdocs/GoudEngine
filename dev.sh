#!/bin/bash
set -e

echo "Setting up GoudEngine for development..."

# Check if the build directory exists
if [ ! -d "build" ]; then
    echo "Build directory does not exist. Please run ./init.sh first."
    exit 1
fi

# Configure and build in Debug mode
cd build
cmake -DCMAKE_BUILD_TYPE=Debug ..
cmake --build . --config Debug

echo "Development build complete. Running BasicSample..."

# Check if a run URL is provided
if [ -z "$1" ]; then
    # No URL provided, run the BasicSample executable
    ./samples/BasicSample/BasicSample
else
    # URL provided, run the executable with the URL
    ./samples/BasicSample/BasicSample "$1"
fi
