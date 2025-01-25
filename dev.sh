#!/bin/bash

# Default values
GAME="flappy_goud"
LOCAL=false

# Parse command line arguments
while [[ "$#" -gt 0 ]]; do
    case $1 in
    --game)
        GAME="$2"
        shift
        ;;
    --local) LOCAL=true ;;
    *)
        echo "Unknown parameter: $1"
        exit 1
        ;;
    esac
    shift
done

# Validate game selection
case $GAME in
"flappy_goud" | "3d_cube" | "goud_jumper")
    echo "Building and running $GAME..."
    ;;
*)
    echo "Error: Invalid game selection. Choose from: flappy_goud, 3d_cube, goud_jumper"
    exit 1
    ;;
esac

# Build the project
if [ "$LOCAL" = false ]; then
    sh package.sh --prod
else
    sh package.sh --local
fi

# cd into selected game directory and restore packages from the local feed
cd examples/$GAME
dotnet restore --source $HOME/nuget-local
dotnet build
dotnet run
