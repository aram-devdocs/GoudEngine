# GoudEngine

> **Alpha Release** — GoudEngine is under active development. APIs and SDKs change frequently. Use with caution in production. Questions? [aram.devdocs@gmail.com](mailto:aram.devdocs@gmail.com). Found a bug? [Open an issue](https://github.com/aram-devdocs/GoudEngine/issues).

[![CI](https://github.com/aram-devdocs/GoudEngine/actions/workflows/ci.yml/badge.svg)](https://github.com/aram-devdocs/GoudEngine/actions/workflows/ci.yml)
[![codecov](https://codecov.io/gh/aram-devdocs/GoudEngine/branch/main/graph/badge.svg)](https://codecov.io/gh/aram-devdocs/GoudEngine)
[![Security Audit](https://github.com/aram-devdocs/GoudEngine/actions/workflows/security.yml/badge.svg)](https://github.com/aram-devdocs/GoudEngine/actions/workflows/security.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

[![crates.io](https://img.shields.io/crates/v/goud-engine.svg)](https://crates.io/crates/goud-engine)
[![npm](https://img.shields.io/npm/v/goudengine.svg)](https://www.npmjs.com/package/goudengine)
[![NuGet](https://img.shields.io/nuget/v/GoudEngine.svg)](https://www.nuget.org/packages/GoudEngine/)
[![PyPI](https://img.shields.io/pypi/v/goudengine.svg)](https://pypi.org/project/goudengine/)

Game engine written in Rust. Build 2D and 3D games from Rust, C#, Python, or TypeScript.

| | |
|---|---|
| **SDKs** | [Rust](sdks/rust/) · [C#](sdks/csharp/) · [Python](sdks/python/) · [TypeScript](sdks/typescript/) |
| **Examples** | [All Examples](examples/) · [Flappy Bird (Rust)](examples/rust/flappy_bird/) |
| **Docs** | [Architecture](docs/architecture/) · [Development](docs/DEVELOPMENT.md) · [Building](docs/BUILDING.md) · [AI Setup](docs/AI_SETUP.md) |

## Get Started

| Language | Install | Examples |
|----------|---------|----------|
| C# (.NET) | `dotnet add package GoudEngine` | [C# examples](examples/csharp/) |
| Python | `pip install goudengine` | [Python examples](examples/python/) |
| TypeScript | `npm install goudengine` | [TypeScript examples](examples/typescript/) |
| Rust | `cargo add goud-engine` | [Rust examples](examples/rust/) |

## Design Philosophy

**All logic lives in Rust.** Language SDKs (C#, Python, TypeScript) are thin wrappers that marshal data and call FFI functions, ensuring consistent behavior across all bindings.

## Features

- Multi-language SDK support: Rust (native), C# (.NET), Python, TypeScript (Node.js + Web/WASM)
- Rust-first architecture: all logic in Rust, SDKs are thin FFI wrappers
- WASM support: TypeScript SDK runs in browsers via WebAssembly
- Flexible renderer selection: 2D or 3D at runtime
- 2D rendering: sprites, 2D camera, Tiled map support
- 3D rendering: primitives (cubes, spheres, planes, cylinders)
- Dynamic lighting: point, directional, and spot lights
- Entity Component System (ECS): high-performance game object management
- Input handling: keyboard and mouse with frame-based state tracking
- Cross-platform window management via GLFW
- OpenGL rendering backend

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    Your Game Code                            │
│         (Rust / C# / Python / TypeScript)                    │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                     Language SDKs                            │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────────┐   │
│  │ Rust SDK │ │ C# SDK   │ │Python SDK│ │TypeScript SDK│   │
│  │(zero FFI)│ │(csbindgen│ │ (ctypes) │ │(napi + WASM) │   │
│  └──────────┘ └──────────┘ └──────────┘ └──────────────┘   │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                    Rust Engine Core                          │
│  ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐          │
│  │Graphics │ │   ECS   │ │Platform │ │  Audio  │          │
│  │(OpenGL) │ │ (World) │ │ (GLFW)  │ │         │          │
│  └─────────┘ └─────────┘ └─────────┘ └─────────┘          │
└─────────────────────────────────────────────────────────────┘
```

### Codegen Pipeline

SDK bindings are generated from a unified schema:

```
codegen/goud_sdk.schema.json   (source of truth)
         │
         ├── gen_csharp.py     → sdks/csharp/
         ├── gen_python.py     → sdks/python/
         ├── gen_ts_node.py    → sdks/typescript/ (Node.js/napi)
         └── gen_ts_web.py     → sdks/typescript/wasm/ (WebAssembly)
```

See `codegen/CLAUDE.md` for details.

### Project Directory

| Directory | Purpose |
|-----------|---------|
| `libs/` | Core libraries (graphics, platform, ECS, logger) |
| `goud_engine/` | Engine crate (core, assets, SDK, FFI) |
| `goud_engine_macros/` | Procedural macros |
| `sdks/` | Language SDKs (C#, Python, TypeScript, Rust re-export) |
| `codegen/` | Unified SDK code generation pipeline |
| `tools/` | Development utilities (lint-layers) |
| `examples/` | Example games organized by SDK language |
| `docs/architecture/` | Architecture documentation |

## Examples

See [examples/README.md](examples/README.md) for the full list with run commands. All examples use `./dev.sh`:

```sh
./dev.sh --game flappy_goud                          # C# Flappy Bird
./dev.sh --game 3d_cube                              # C# 3D demo
./dev.sh --sdk python --game flappy_bird             # Python Flappy Bird
./dev.sh --sdk typescript --game flappy_bird         # TypeScript desktop
./dev.sh --sdk typescript --game flappy_bird_web     # TypeScript web
cargo run -p flappy-bird                             # Rust Flappy Bird
```

## Documentation

- [SDK-First Architecture](docs/architecture/sdk-first-architecture.md)
- [Adding a New Language](docs/architecture/adding-a-new-language.md)
- [Development Guide](docs/DEVELOPMENT.md) — dev.sh, Git hooks, version management
- [Building](docs/BUILDING.md) — build.sh, package.sh, NuGet feed
- [AI Setup](docs/AI_SETUP.md) — Claude Code, Cursor, Gemini configuration
- [Rust SDK](sdks/rust/README.md)
- [C# SDK](sdks/csharp/README.md)
- [Python SDK](sdks/python/README.md)
- [TypeScript SDK](sdks/typescript/README.md)
- [csbindgen](https://github.com/Cysharp/csbindgen) — C# bindings generator
- [cbindgen](https://github.com/mozilla/cbindgen) — C header generator

## License

MIT — see [LICENSE](LICENSE) for details.
