# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Essential Commands

### Building and Testing
```bash
# Quick development with automatic build and run (C# SDK - default)
./dev.sh --game flappy_goud       # Run 2D game (default)
./dev.sh --game 3d_cube          # Run 3D demo
./dev.sh --game goud_jumper      # Run platform game
./dev.sh --game <game> --local   # Use local NuGet feed

# Python SDK demos
./dev.sh --sdk python --game python_demo  # Run Python demo
./dev.sh --sdk python --game flappy_bird  # Run Python Flappy Bird

# Rust SDK (runs tests)
./dev.sh --sdk rust              # Run Rust SDK tests

# Core build commands
cargo build                      # Debug build
cargo build --release           # Release build
./build.sh --release            # Full release build with SDK

# Testing
cargo test                       # Run all tests
cargo test -- --nocapture       # Show test output
cargo test --lib sdk            # Test Rust SDK specifically
cargo test graphics             # Test specific module

# Python SDK tests
python3 sdks/python/test_bindings.py  # Run Python SDK tests

# Pre-commit checks (must pass)
cargo check
cargo fmt --all -- --check
cargo clippy -- -D warnings
cargo deny check
```

### Version Management
After making changes, ALWAYS increment version before packaging:
```bash
./increment_version.sh         # Patch (0.0.X)
./increment_version.sh --minor # Minor (0.X.0)
./increment_version.sh --major # Major (X.0.0)
```

### Local Development Cycle
```bash
./increment_version.sh          # 1. Increment version
./build.sh                      # 2. Build everything
./package.sh --local           # 3. Deploy to local NuGet
./dev.sh --game <game> --local # 4. Test with example
```

## Architecture Overview

### Design Principle: Rust-First
**All logic lives in Rust.** SDKs are thin wrappers that marshal data and call FFI functions.

This means:
- Component methods (e.g., `Transform2D.translate()`) are implemented in Rust
- SDKs call FFI functions, they don't implement logic
- If you need a new feature, add it to Rust first, then expose via FFI

### Core Structure
GoudEngine is a Rust game engine with multi-language SDK support:
- **Rust Core** (`goud_engine/`): Performance-critical engine code
- **Rust SDK** (`goud_engine/src/sdk/`): Native Rust API (zero FFI overhead)
- **C# SDK** (`sdks/GoudEngine/`): User-facing .NET API via FFI
- **Python SDK** (`sdks/python/`): Python bindings via FFI (ctypes)
- **FFI Layer** (`goud_engine/src/ffi/`): csbindgen-generated bindings

### Module Organization
```
libs/
├── graphics/           # Rendering subsystem
│   ├── renderer/      # Base renderer trait
│   ├── renderer2d/    # 2D rendering (sprites, 2D camera)
│   ├── renderer3d/    # 3D rendering (primitives, lighting)
│   └── components/    # Shared (shaders, textures, buffers)
├── platform/          # Platform layer
│   └── window/       # GLFW window management
├── ecs/              # Entity Component System
└── logger/           # Logging infrastructure
```

### Renderer Selection
The engine supports runtime renderer selection:
- **2D Renderer**: Sprites, 2D camera, Tiled maps
- **3D Renderer**: Primitives, dynamic lighting, 3D camera

Selected at `GoudGame` initialization:
```csharp
new GoudGame(800, 600, "Title", RendererType.Renderer2D)  // 2D
new GoudGame(800, 600, "Title", RendererType.Renderer3D)  // 3D
```

### Graphics Testing Focus
Currently improving test coverage for graphics components:
- Texture system (`texture.rs`, `texture_manager.rs`)
- Cameras (`camera2d.rs`, `camera3d.rs`)
- Shader programs (`shader_program.rs`)
- Tiled map support (`tiled.rs`)

## Key Development Notes

### Git Hooks
Two hooks are configured:
- **pre-commit**: Fast checks (format, clippy, basic tests, Python SDK)
- **pre-push**: Comprehensive checks (full test suite, doctests, security)

After modifying `.husky/hooks/pre-commit` or `.husky/hooks/pre-push`:
```bash
cargo clean && cargo test  # Required for husky-rs to reload
```

### Module Dependencies
Generate visual dependency graph:
```bash
./graph.sh  # Creates module_graph.png and .pdf
```

### Local NuGet Feed
Location: `$HOME/nuget-local`

### FFI Considerations
- All public functions in `ffi/` must be `#[no_mangle] extern "C"`
- Structs shared with C#/Python need `#[repr(C)]`
- Memory management crosses FFI boundary - be careful with ownership
- Component FFI exports are in `ffi/component_*.rs` files

### SDK Development Guidelines
When adding new features:
1. **Implement in Rust first** (`goud_engine/src/`)
2. **Add FFI exports** (`goud_engine/src/ffi/`)
3. **Run csbindgen** to update C# bindings (`cargo build` triggers this)
4. **Update Python bindings** (`sdks/python/goud_engine/bindings.py`)
5. **Update SDK wrappers** if needed (C# in `sdks/GoudEngine/`, Python classes)

DRY Validation:
- Search for method implementations in both Rust and SDK code
- If logic exists in SDK, it should be moved to Rust

### Testing Graphics Components
When testing graphics code:
1. Many tests require OpenGL context (may fail in CI)
2. Use `test_helpers::init_test_context()` for tests needing GL
3. Texture tests may need valid image files in `assets/`

### Example Games
Examples are organized by SDK language:

**C# Examples** (`examples/csharp/`):
- `flappy_goud/` - Flappy Bird clone
- `3d_cube/` - 3D rendering demo
- `goud_jumper/` - Platformer game
- `isometric_rpg/` - Isometric RPG demo
- `hello_ecs/` - ECS basics

**Python Examples** (`examples/python/`):
- `main.py` - Python SDK demo
- `flappy_bird.py` - Python Flappy Bird clone

**Rust Examples** (`examples/rust/`):
- (Future Rust SDK examples)

The Python Flappy Bird mirrors the C# version, demonstrating SDK parity.