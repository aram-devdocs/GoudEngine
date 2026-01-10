#!/bin/bash
# Run the Python demo from the correct directory

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

# Check if argument provided
if [ "$1" = "flappy" ] || [ "$1" = "flappy_bird" ]; then
    echo "Running Flappy Bird demo..."
    python3 flappy_bird.py
else
    echo "Running basic demo..."
    python3 main.py
fi
