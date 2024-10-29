# GoudEngine

GoudEngine is a modular, cross-platform 2D game engine written in C++ with future C# interoperability. This engine provides foundational systems like rendering (via SDL2 and OpenGL), with a design that anticipates future Vulkan support.

## Building the Engine

### Prerequisites

- C++17 compiler
- CMake 3.15 or higher
- SDL2 (with development headers)
  - macOS: Install via Homebrew with `brew install sdl2`
  - VSCode
- OpenGL
  - OpenGL is included as a framework on macOS but may need separate installation on other platforms.

### Build Steps

1. **Run Initialization Script:**
   This script sets up dependencies, configures the build, and prepares the project.
   ```bash
   ./init.sh
   ```
2. **Build the Project:**
   This command builds the engine, including the core and all modules.
   ```bash
   ./dev.sh
   ```

### Running the Sample

After building, you can run the sample application, which demonstrates an SDL2 window with an OpenGL context:

```bash
./build/samples/BasicSample/BasicSample
```

## Project Structure

```
GoudEngine/
├── engine/                  # Core engine code
│   ├── include/             # Core engine headers
│   └── src/                 # Core engine source files
├── modules/                 # Modular components
│   ├── graphics/            # Graphics module (SDL2 + OpenGL)
│   ├── audio/               # Audio module
│   ├── input/               # Input handling module
│   ├── physics/             # Physics engine (future expansion)
│   └── networking/          # Networking module (future expansion)
├── samples/                 # Sample applications to test engine features
│   └── BasicSample/         # Basic SDL2 window + OpenGL sample
├── tests/                   # Test suites
├── docs/                    # Documentation
└── tools/                   # Utility tools and scripts
```

## Additional Details

- **SDL2 and OpenGL Abstraction:**
  GoudEngine currently supports SDL2 for window management and input handling and uses OpenGL for rendering.
  The graphics module is designed to accommodate Vulkan in the future if more low-level control is required.
- **Cross-Platform Setup:**
  On macOS, OpenGL is linked as a framework, so the CMake configuration uses a conditional check for APPLE to ensure compatibility.
- **Future Expansion:**
  The engine architecture anticipates future expansion with additional modules and potential support for different graphics APIs, including Vulkan.

## Commands Recap

1. **Setup:** `./init.sh`
2. **Development Build:** `./dev.sh`

### Running the Sample with Optional URL

You can run the sample application with an optional URL parameter. If no URL is provided, it will run the BasicSample executable by default.
