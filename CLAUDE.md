# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Essential Commands

### Building and Testing
```bash
# Quick development with automatic build and run
./dev.sh --game flappy_goud       # Run 2D game (default)
./dev.sh --game 3d_cube          # Run 3D demo
./dev.sh --game goud_jumper      # Run platform game
./dev.sh --game <game> --local   # Use local NuGet feed

# Core build commands
cargo build                      # Debug build
cargo build --release           # Release build
./build.sh --release            # Full release build with SDK

# Testing
cargo test                       # Run all tests
cargo test -- --nocapture       # Show test output
cargo test graphics             # Test specific module
cargo test test_texture_manager # Run specific test

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

### Core Structure
GoudEngine is a Rust game engine with C# bindings via FFI:
- **Rust Core** (`goud_engine/`): Performance-critical engine code
- **C# SDK** (`sdks/GoudEngine/`): User-facing .NET API
- **FFI Layer** (`goud_engine/src/sdk/`): csbindgen-generated bindings

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
After modifying `.husky/hooks/pre-commit`:
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
- All public functions in `sdk/` must be `#[no_mangle] extern "C"`
- Structs shared with C# need `#[repr(C)]`
- Memory management crosses FFI boundary - be careful with ownership

### Testing Graphics Components
When testing graphics code:
1. Many tests require OpenGL context (may fail in CI)
2. Use `test_helpers::init_test_context()` for tests needing GL
3. Texture tests may need valid image files in `assets/`