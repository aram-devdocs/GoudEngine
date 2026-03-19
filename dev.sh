#!/bin/bash

# Default values
GAME="flappy_goud"
LOCAL=false
SKIP_BUILD=false
NEXT=false
SDK_TYPE="csharp"  # csharp, python, rust, typescript

# Script directory for absolute paths
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Returns success when a localhost TCP port is available to bind.
is_port_available() {
    local port="$1"
    python3 - "$port" <<'PY'
import socket
import sys

port = int(sys.argv[1])
sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
try:
    sock.bind(("127.0.0.1", port))
except OSError:
    sys.exit(1)
finally:
    sock.close()
sys.exit(0)
PY
}

# Picks a web server port for TS web demo. Defaults to 8765, falls back if occupied.
pick_web_port() {
    local preferred="${TS_WEB_PORT:-8765}"
    local port
    if is_port_available "$preferred"; then
        echo "$preferred"
        return 0
    fi
    for port in $(seq 8766 8799); do
        if is_port_available "$port"; then
            echo "$port"
            return 0
        fi
    done
    return 1
}

csharp_example_uses_project_reference() {
    local project_file="$SCRIPT_DIR/examples/csharp/$1/$1.csproj"
    [ -f "$project_file" ] && rg -q "<ProjectReference " "$project_file"
}

native_lib_name() {
    case "$(uname -s)" in
    Darwin) echo "libgoud_engine.dylib" ;;
    Linux) echo "libgoud_engine.so" ;;
    MINGW*|MSYS*|CYGWIN*) echo "goud_engine.dll" ;;
    *) echo "libgoud_engine.so" ;;
    esac
}

rust_sources_newer_than() {
    local artifact="$1"
    [ -f "$artifact" ] || return 0
    find \
        "$SCRIPT_DIR/Cargo.toml" \
        "$SCRIPT_DIR/Cargo.lock" \
        "$SCRIPT_DIR/goud_engine" \
        "$SCRIPT_DIR/goud_engine_macros" \
        -type f \
        \( -name '*.rs' -o -name '*.toml' \) \
        -newer "$artifact" \
        -print -quit | grep -q .
}

typescript_sources_newer_than() {
    local artifact="$1"
    [ -f "$artifact" ] || return 0
    find \
        "$SCRIPT_DIR/codegen" \
        "$SCRIPT_DIR/sdks/typescript" \
        "$SCRIPT_DIR/goud_engine" \
        -type f \
        \( -name '*.py' -o -name '*.json' -o -name '*.ts' -o -name '*.rs' -o -name '*.toml' \) \
        -not -path '*/node_modules/*' \
        -not -path '*/dist/*' \
        -not -path '*/wasm/*' \
        -newer "$artifact" \
        -print -quit | grep -q .
}

python_release_artifact_fresh() {
    local artifact="$SCRIPT_DIR/target/release/$(native_lib_name)"
    [ -f "$artifact" ] && ! rust_sources_newer_than "$artifact"
}

typescript_native_artifacts_fresh() {
    local node_artifact
    node_artifact="$(find "$SCRIPT_DIR/sdks/typescript" -maxdepth 2 -type f -name '*.node' | head -n 1)"
    [ -n "$node_artifact" ] || return 1
    [ -f "$SCRIPT_DIR/sdks/typescript/dist/node/index.js" ] || return 1
    ! typescript_sources_newer_than "$node_artifact"
}

typescript_web_artifacts_fresh() {
    [ -f "$SCRIPT_DIR/sdks/typescript/wasm/goud_engine_bg.wasm" ] || return 1
    [ -f "$SCRIPT_DIR/sdks/typescript/dist/web/web/index.js" ] || return 1
    ! typescript_sources_newer_than "$SCRIPT_DIR/sdks/typescript/wasm/goud_engine_bg.wasm"
}

ensure_example_node_modules() {
    local example_dir="$1"
    if [ -d "$example_dir/node_modules" ]; then
        echo "Skipping npm install; example dependencies already present in $example_dir/node_modules"
        return 0
    fi
    echo "Installing example dependencies..."
    cd "$example_dir" && npm install
}

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
        echo "  --sdk <type>     SDK type: csharp, cpp, go, python, rust, typescript (default: csharp)"
        echo "  --local          Use local feed when needed; direct-project C# examples use a fast local path"
        echo "  --skipBuild      Skip build step"
        echo "  --next           Run version increment and rebuild"
        echo "  -h, --help       Show this help message"
        echo ""
        echo "C# Games:       flappy_goud, 3d_cube, goud_jumper, isometric_rpg, hello_ecs, feature_lab, sandbox"
        echo "Python Demos:   python_demo, flappy_bird, sandbox (use --sdk python)"
        echo "Go Games:       flappy_bird (use --sdk go)"
        echo "Rust SDK:       rust_demo (use --sdk rust)"
        echo "TypeScript:     flappy_bird (desktop), flappy_bird_web (web), feature_lab (desktop), feature_lab_web (web), sandbox (desktop), sandbox_web (web) (use --sdk typescript)"
        echo ""
        echo "Examples:"
        echo "  ./dev.sh --game flappy_goud            # Run C# Flappy Goud"
        echo "  ./dev.sh --sdk python --game python_demo  # Run Python demo"
        echo "  ./dev.sh --sdk python --game flappy_bird  # Run Python Flappy Bird"
        echo "  ./dev.sh --sdk cpp --game flappy_bird      # Run C++ Flappy Bird"
        echo "  ./dev.sh --sdk cpp --game cmake_example    # Run C++ CMake example"
        echo "  ./dev.sh --sdk go --game flappy_bird       # Run Go Flappy Bird"
        echo "  ./dev.sh --sdk rust                    # Run Rust SDK tests"
        echo "  ./dev.sh --sdk typescript --game flappy_bird      # TS desktop"
        echo "  ./dev.sh --sdk typescript --game flappy_bird_web  # TS web (browser)"
        echo "  ./dev.sh --sdk typescript --game feature_lab      # TS Feature Lab desktop"
        echo "  ./dev.sh --sdk typescript --game feature_lab_web  # TS Feature Lab web"
        echo "  ./dev.sh --game sandbox                           # C# Sandbox desktop"
        echo "  ./dev.sh --game sandbox --local                   # C# Sandbox desktop with fast local project-reference path"
        echo "  ./dev.sh --sdk python --game sandbox             # Python Sandbox desktop"
        echo "  ./dev.sh --sdk typescript --game sandbox         # TS Sandbox desktop"
        echo "  ./dev.sh --sdk typescript --game sandbox_web     # TS Sandbox web"
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
"csharp" | "cpp" | "go" | "python" | "rust" | "typescript")
    ;;
*)
    echo "Error: Invalid SDK type. Choose from: csharp, cpp, go, python, rust, typescript"
    exit 1
    ;;
esac

# Validate game selection based on SDK type
case $SDK_TYPE in
"csharp")
    case $GAME in
    "flappy_goud" | "3d_cube" | "goud_jumper" | "isometric_rpg" | "hello_ecs" | "feature_lab" | "sandbox")
        echo "Building and running C# game: $GAME..."
        ;;
    *)
        echo "Error: Invalid C# game selection."
        echo "Choose from: flappy_goud, 3d_cube, goud_jumper, isometric_rpg, hello_ecs, feature_lab, sandbox"
        exit 1
        ;;
    esac
    ;;
"cpp")
    case $GAME in
    "smoke" | "cmake_example" | "flappy_bird")
        echo "Building and running C++ example: $GAME..."
        ;;
    *)
        echo "Error: Invalid C++ example selection."
        echo "Choose from: smoke, cmake_example, flappy_bird"
        exit 1
        ;;
    esac
    ;;
"go")
    case $GAME in
    "flappy_bird")
        echo "Building and running Go example: $GAME..."
        ;;
    *)
        echo "Error: Invalid Go example selection."
        echo "Choose from: flappy_bird"
        exit 1
        ;;
    esac
    ;;
"python")
    case $GAME in
    "python_demo" | "flappy_bird" | "sandbox")
        echo "Running Python demo: $GAME..."
        ;;
    *)
        echo "Error: Invalid Python demo selection."
        echo "Choose from: python_demo, flappy_bird, sandbox"
        exit 1
        ;;
    esac
    ;;
"rust")
    echo "Running Rust SDK..."
    ;;
"typescript")
    case $GAME in
    "flappy_bird" | "flappy_bird_web" | "feature_lab" | "feature_lab_web" | "sandbox" | "sandbox_web")
        echo "Building and running TypeScript example: $GAME..."
        ;;
    *)
        echo "Error: Invalid TypeScript example selection."
        echo "Choose from: flappy_bird (desktop), flappy_bird_web (web), feature_lab (desktop), feature_lab_web (web), sandbox (desktop), sandbox_web (web)"
        exit 1
        ;;
    esac
    ;;
esac

# Build the project if not skipped
if [ "$SKIP_BUILD" = false ]; then
    # Prune stale incremental compilation cache (>7 days old) to prevent unbounded growth
    if [ -d "target/debug/incremental" ]; then
        find "$SCRIPT_DIR/target/debug/incremental" -maxdepth 1 -type d -mtime +7 -exec rm -rf {} + 2>/dev/null
    fi

    FAST_LOCAL_CSHARP_PATH=false
    if [ "$SDK_TYPE" = "csharp" ] && [ "$LOCAL" = true ] && csharp_example_uses_project_reference "$GAME"; then
        FAST_LOCAL_CSHARP_PATH=true
    fi

    # Skip the preflight check on the direct project-reference fast path.
    # build.sh will compile the core once, so running cargo check here just duplicates the work.
    if [ "$FAST_LOCAL_CSHARP_PATH" = true ]; then
        echo "Skipping preflight cargo check for fast local C# path; the core build below will validate and compile once."
    elif ! cargo check; then
        echo "Cargo check failed. Fixing errors before proceeding with full build."
        exit 1
    fi

    if [ "$SDK_TYPE" = "csharp" ]; then
        # Ensure local NuGet feed directory exists
        mkdir -p "$HOME/nuget-local"

        if [ "$FAST_LOCAL_CSHARP_PATH" = true ]; then
            echo "Using fast local C# path for project-reference example: $GAME"
            bash "$SCRIPT_DIR/build.sh" --local --core-only --host-runtime-only --skip-csharp-sdk-build
        elif [ "$LOCAL" = false ]; then
            bash "$SCRIPT_DIR/package.sh" --prod
        else
            bash "$SCRIPT_DIR/package.sh" --local
        fi
    elif [ "$SDK_TYPE" = "cpp" ]; then
        # Build native library for C++ examples
        if python_release_artifact_fresh; then
            echo "Skipping native rebuild; release artifact is fresh."
        else
            echo "Building native library..."
            cargo build --release
        fi
    elif [ "$SDK_TYPE" = "typescript" ]; then
        if [ "$GAME" = "flappy_bird_web" ] || [ "$GAME" = "feature_lab_web" ] || [ "$GAME" = "sandbox_web" ]; then
            if typescript_web_artifacts_fresh; then
                echo "Skipping TypeScript web SDK rebuild; wasm artifacts are fresh."
            else
                echo "Running codegen..."
                python3 "$SCRIPT_DIR/codegen/gen_ts_node.py"
                python3 "$SCRIPT_DIR/codegen/gen_ts_web.py"
                echo "Building TypeScript web SDK (wasm)..."
                cd "$SCRIPT_DIR/sdks/typescript" && npm run build:web
                cd "$SCRIPT_DIR"
            fi
        else
            if typescript_native_artifacts_fresh; then
                echo "Skipping TypeScript native SDK rebuild; native artifacts are fresh."
            else
                echo "Running codegen..."
                python3 "$SCRIPT_DIR/codegen/gen_ts_node.py"
                python3 "$SCRIPT_DIR/codegen/gen_ts_web.py"
                echo "Building TypeScript native SDK..."
                cd "$SCRIPT_DIR/sdks/typescript" && npm run build:native && npm run build:ts
                cd "$SCRIPT_DIR"
            fi
        fi

        TS_EXAMPLE_DIR="flappy_bird"
        case $GAME in
        "feature_lab" | "feature_lab_web")
            TS_EXAMPLE_DIR="feature_lab"
            ;;
        "sandbox" | "sandbox_web")
            TS_EXAMPLE_DIR="sandbox"
            ;;
        esac

        ensure_example_node_modules "$SCRIPT_DIR/examples/typescript/$TS_EXAMPLE_DIR"
        cd "$SCRIPT_DIR"
    elif [ "$SDK_TYPE" = "go" ]; then
        echo "Building native library for Go SDK..."
        cargo build --release
    else
        # For Python and Rust, just build the native library
        if [ "$SDK_TYPE" = "python" ] && python_release_artifact_fresh; then
            echo "Skipping native rebuild; release Python artifact is fresh."
        else
            echo "Building native library..."
            cargo build --release
        fi
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

    # Prefer host dotnet. If .NET 8 runtime is missing but a newer runtime exists,
    # allow major roll-forward so net8 apps still launch.
    DOTNET_CMD="dotnet"
    DOTNET_RUNNER=()
    if ! dotnet --list-runtimes | grep -q "Microsoft.NETCore.App 8\\."; then
        if dotnet --list-runtimes | grep -q "Microsoft.NETCore.App "; then
            DOTNET_RUNNER=(env DOTNET_ROLL_FORWARD=Major)
        fi
    fi

    # Direct-project examples do not need local package publish or feed restore churn.
    if [ "$LOCAL" = true ] && csharp_example_uses_project_reference "$GAME"; then
        "${DOTNET_RUNNER[@]}" "$DOTNET_CMD" restore --nologo
        "${DOTNET_RUNNER[@]}" "$DOTNET_CMD" build --no-restore --nologo -p:GeneratePackageOnBuild=false
    else
        "${DOTNET_RUNNER[@]}" "$DOTNET_CMD" clean --nologo
        "${DOTNET_RUNNER[@]}" "$DOTNET_CMD" restore --source "$HOME/nuget-local" --source https://api.nuget.org/v3/index.json --nologo
        "${DOTNET_RUNNER[@]}" "$DOTNET_CMD" build --no-restore --nologo
    fi
    "${DOTNET_RUNNER[@]}" "$DOTNET_CMD" run --no-build --nologo
    ;;

"cpp")
    CPP_EXAMPLE_DIR="$SCRIPT_DIR/examples/cpp/$GAME"
    CPP_BUILD_DIR="$CPP_EXAMPLE_DIR/build"

    echo "Configuring C++ example: $GAME..."
    cmake -B "$CPP_BUILD_DIR" \
        -DGOUD_ENGINE_ROOT="$SCRIPT_DIR" \
        -DCMAKE_BUILD_TYPE=Release \
        "$CPP_EXAMPLE_DIR"

    echo "Building C++ example: $GAME..."
    cmake --build "$CPP_BUILD_DIR" --config Release

    echo "Running C++ example: $GAME..."
    cd "$CPP_EXAMPLE_DIR"
    "$CPP_BUILD_DIR/$GAME"
    ;;

"python")
    # Ensure Python SDK path is accessible
    export PYTHONPATH="$SCRIPT_DIR/sdks/python:$PYTHONPATH"
    
    # Set library path for native bindings
    if [[ "$OSTYPE" == "darwin"* ]]; then
        export DYLD_LIBRARY_PATH="$SCRIPT_DIR/target/release:$DYLD_LIBRARY_PATH"
        export GOUD_ENGINE_LIB="${GOUD_ENGINE_LIB:-$SCRIPT_DIR/target/release/$(native_lib_name)}"
    elif [[ "$OSTYPE" == "linux-gnu"* ]]; then
        export LD_LIBRARY_PATH="$SCRIPT_DIR/target/release:$LD_LIBRARY_PATH"
        export GOUD_ENGINE_LIB="${GOUD_ENGINE_LIB:-$SCRIPT_DIR/target/release/$(native_lib_name)}"
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
    "sandbox")
        echo "Running Python Sandbox..."
        python3 sandbox.py
        ;;
    esac
    ;;

"go")
    GO_EXAMPLE_DIR="$SCRIPT_DIR/examples/go/$GAME"
    echo "Building and running Go example: $GAME..."

    # Set library path for native bindings
    if [[ "$OSTYPE" == "darwin"* ]]; then
        export DYLD_LIBRARY_PATH="$SCRIPT_DIR/target/release:$DYLD_LIBRARY_PATH"
    elif [[ "$OSTYPE" == "linux-gnu"* ]]; then
        export LD_LIBRARY_PATH="$SCRIPT_DIR/target/release:$LD_LIBRARY_PATH"
    fi

    cd "$GO_EXAMPLE_DIR"
    CGO_ENABLED=1 go run .
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

"typescript")
    case $GAME in
    "flappy_bird")
        echo "Running TypeScript Flappy Bird (desktop)..."
        cd "$SCRIPT_DIR/examples/typescript/flappy_bird"
        npx tsx desktop.ts
        ;;
    "flappy_bird_web")
        echo "Compiling game.ts for web..."
        cd "$SCRIPT_DIR/examples/typescript/flappy_bird"
        npx tsc

        WEB_PORT="$(pick_web_port)" || {
            echo "Error: No available localhost port found for web server (8765-8799)."
            exit 1
        }

        echo ""
        if [ "$WEB_PORT" != "${TS_WEB_PORT:-8765}" ]; then
            echo "Port ${TS_WEB_PORT:-8765} is in use; using $WEB_PORT instead."
        fi
        echo "Starting web server on http://localhost:$WEB_PORT"
        echo "Open: http://localhost:$WEB_PORT/examples/typescript/flappy_bird/web/index.html"
        echo "Press Ctrl+C to stop."
        echo ""
        cd "$SCRIPT_DIR"
        python3 -m http.server "$WEB_PORT" --bind 127.0.0.1
        ;;
    "feature_lab")
        echo "Running TypeScript Feature Lab (desktop)..."
        cd "$SCRIPT_DIR/examples/typescript/feature_lab"
        npx tsx desktop.ts
        ;;
    "feature_lab_web")
        echo "Compiling Feature Lab for web..."
        cd "$SCRIPT_DIR/examples/typescript/feature_lab"
        npx tsc

        WEB_PORT="$(pick_web_port)" || {
            echo "Error: No available localhost port found for web server (8765-8799)."
            exit 1
        }

        echo ""
        if [ "$WEB_PORT" != "${TS_WEB_PORT:-8765}" ]; then
            echo "Port ${TS_WEB_PORT:-8765} is in use; using $WEB_PORT instead."
        fi
        echo "Starting web server on http://localhost:$WEB_PORT"
        echo "Open: http://localhost:$WEB_PORT/examples/typescript/feature_lab/web/index.html"
        echo "Press Ctrl+C to stop."
        echo ""
        cd "$SCRIPT_DIR"
        python3 -m http.server "$WEB_PORT" --bind 127.0.0.1
        ;;
    "sandbox")
        echo "Running TypeScript Sandbox (desktop)..."
        cd "$SCRIPT_DIR/examples/typescript/sandbox"
        npx tsx desktop.ts
        ;;
    "sandbox_web")
        echo "Compiling Sandbox for web..."
        cd "$SCRIPT_DIR/examples/typescript/sandbox"
        npx tsc

        WEB_PORT="$(pick_web_port)" || {
            echo "Error: No available localhost port found for web server (8765-8799)."
            exit 1
        }

        echo ""
        if [ "$WEB_PORT" != "${TS_WEB_PORT:-8765}" ]; then
            echo "Port ${TS_WEB_PORT:-8765} is in use; using $WEB_PORT instead."
        fi
        echo "Starting web server on http://localhost:$WEB_PORT"
        echo "Open: http://localhost:$WEB_PORT/examples/typescript/sandbox/web/index.html"
        echo "Press Ctrl+C to stop."
        echo ""
        cd "$SCRIPT_DIR"
        python3 -m http.server "$WEB_PORT" --bind 127.0.0.1
        ;;
    esac
    ;;
esac
