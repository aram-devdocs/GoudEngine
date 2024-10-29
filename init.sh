#!/bin/bash
set -e

echo "Initializing GoudEngine project..."

# Initialize and update git submodules (for third-party libraries, if using submodules)
if [ -f .gitmodules ]; then
  echo "Initializing git submodules..."
  git submodule update --init --recursive
fi

# Create the build directory if it doesn't exist
if [ ! -d "build" ]; then
  echo "Creating build directory..."
  mkdir build
fi

echo "Running CMake configuration..."
cd build
cmake ..

echo "Initialization complete."