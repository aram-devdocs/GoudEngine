#!/bin/bash

# Default values
GAME="flappy_goud"
LOCAL=false
SKIP_BUILD=false
NEXT=false

# Parse command line arguments
while [[ "$#" -gt 0 ]]; do
    case $1 in
    --game)
        GAME="$2"
        shift
        ;;
    --local) LOCAL=true ;;
    --skipBuild) SKIP_BUILD=true ;;
    --next) NEXT=true ;;
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

# Build the project if not skipped
if [ "$SKIP_BUILD" = false ]; then
    # Run cargo check first to catch compilation errors early
    if ! cargo check; then
        echo "Cargo check failed. Fixing errors before proceeding with full build."
        exit 1
    fi

    if [ "$LOCAL" = false ]; then
        sh package.sh --prod
    else
        sh package.sh --local
    fi
fi

# If --next flag is set, run additional scripts
if [ "$NEXT" = true ]; then
    echo "Running next script..."
    ./increment_version.sh
    ./build.sh
    ./package.sh --local
fi

# cd into selected game directory and restore packages from the local feed
cd examples/$GAME

# Optimize dotnet commands
dotnet clean --nologo
dotnet restore --source $HOME/nuget-local --nologo
dotnet build --no-restore --nologo
dotnet run --no-build --nologo
