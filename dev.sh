#!/bin/bash

# Default values
GAME="flappy_goud"
LOCAL=false
SKIP_BUILD=false
NEXT=false
SDK_TYPE="csharp"  # csharp, python, rust

# Script directory for absolute paths
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Parse command line arguments
while [[ "$#" -gt 0 ]]; do
    case $1 in
    --game)
        GAME="$2"
        shift
        ;;
    --sdk)
        SDK_TYPE="$2"
        shift
        ;;
    --local) LOCAL=true ;;
    --skipBuild) SKIP_BUILD=true ;;
    --next) NEXT=true ;;
    -h|--help)
        echo "Usage: ./dev.sh [OPTIONS]"
        echo ""
        echo "Options:"
        echo "  --game <name>    Game to run (default: flappy_goud)"
        echo "  --sdk <type>     SDK type: csharp, python, rust (default: csharp)"
        echo "  --local          Use local NuGet feed"
        echo "  --skipBuild      Skip build step"
        echo "  --next           Run version increment and rebuild"
        echo "  -h, --help       Show this help message"
        echo ""
        echo "C# Games:     flappy_goud, 3d_cube, goud_jumper, isometric_rpg, hello_ecs"
        echo "Python Demos: python_demo, flappy_bird (use --sdk python)"
        echo "Rust SDK:     rust_demo (use --sdk rust)"
        echo ""
        echo "Examples:"
        echo "  ./dev.sh --game flappy_goud            # Run C# Flappy Goud"
        echo "  ./dev.sh --sdk python --game python_demo  # Run Python demo"
        echo "  ./dev.sh --sdk python --game flappy_bird  # Run Python Flappy Bird"
        echo "  ./dev.sh --sdk rust                    # Run Rust SDK tests"
        exit 0
        ;;
    *)
        echo "Unknown parameter: $1"
        echo "Use --help for usage information"
        exit 1
        ;;
    esac
    shift
done

# Validate SDK type
case $SDK_TYPE in
"csharp" | "python" | "rust")
    ;;
*)
    echo "Error: Invalid SDK type. Choose from: csharp, python, rust"
    exit 1
    ;;
esac

# Validate game selection based on SDK type
case $SDK_TYPE in
"csharp")
    case $GAME in
    "flappy_goud" | "3d_cube" | "goud_jumper" | "isometric_rpg" | "hello_ecs")
        echo "Building and running C# game: $GAME..."
        ;;
    *)
        echo "Error: Invalid C# game selection."
        echo "Choose from: flappy_goud, 3d_cube, goud_jumper, isometric_rpg, hello_ecs"
        exit 1
        ;;
    esac
    ;;
"python")
    case $GAME in
    "python_demo" | "flappy_bird")
        echo "Running Python demo: $GAME..."
        ;;
    *)
        echo "Error: Invalid Python demo selection."
        echo "Choose from: python_demo, flappy_bird"
        exit 1
        ;;
    esac
    ;;
"rust")
    echo "Running Rust SDK..."
    ;;
esac

# Build the project if not skipped
if [ "$SKIP_BUILD" = false ]; then
    # Run cargo check first to catch compilation errors early
    if ! cargo check; then
        echo "Cargo check failed. Fixing errors before proceeding with full build."
        exit 1
    fi

    if [ "$SDK_TYPE" = "csharp" ]; then
        if [ "$LOCAL" = false ]; then
            sh "$SCRIPT_DIR/package.sh" --prod
        else
            sh "$SCRIPT_DIR/package.sh" --local
        fi
    else
        # For Python and Rust, just build the native library
        echo "Building native library..."
        cargo build --release
    fi
fi

# If --next flag is set, run additional scripts
if [ "$NEXT" = true ]; then
    echo "Running next script..."
    "$SCRIPT_DIR/increment_version.sh"
    "$SCRIPT_DIR/build.sh"
    "$SCRIPT_DIR/package.sh" --local
fi

# Run the appropriate SDK demo
case $SDK_TYPE in
"csharp")
    # cd into selected game directory and restore packages from the local feed
    cd "$SCRIPT_DIR/examples/csharp/$GAME"

    # Optimize dotnet commands
    dotnet clean --nologo
    dotnet restore --source $HOME/nuget-local --nologo
    dotnet build --no-restore --nologo
    dotnet run --no-build --nologo
    ;;

"python")
    # Ensure Python SDK path is accessible
    export PYTHONPATH="$SCRIPT_DIR/sdks/python:$PYTHONPATH"
    
    # Set library path for native bindings
    if [[ "$OSTYPE" == "darwin"* ]]; then
        export DYLD_LIBRARY_PATH="$SCRIPT_DIR/target/release:$DYLD_LIBRARY_PATH"
    elif [[ "$OSTYPE" == "linux-gnu"* ]]; then
        export LD_LIBRARY_PATH="$SCRIPT_DIR/target/release:$LD_LIBRARY_PATH"
    fi
    
    cd "$SCRIPT_DIR/examples/python"
    
    case $GAME in
    "python_demo")
        echo "Running Python SDK demo..."
        python3 main.py
        ;;
    "flappy_bird")
        echo "Running Python Flappy Bird..."
        python3 flappy_bird.py
        ;;
    esac
    ;;

"rust")
    # Run Rust SDK tests and examples
    echo "Running Rust SDK tests..."
    cargo test --lib sdk -- --nocapture
    
    echo ""
    echo "Running Rust SDK doctests..."
    cargo test --doc sdk -- --nocapture
    
    echo ""
    echo "=== Rust SDK Demo Complete ==="
    echo "All Rust SDK tests passed!"
    ;;
esac
