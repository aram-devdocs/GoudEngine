# GoudEngine

> **Alpha Release** — GoudEngine is under active development. APIs and SDKs change frequently. Use with caution in production. Questions? [aram.devdocs@gmail.com](mailto:aram.devdocs@gmail.com). Found a bug? [Open an issue](https://github.com/aram-devdocs/GoudEngine/issues).

[![CI](https://github.com/aram-devdocs/GoudEngine/actions/workflows/ci.yml/badge.svg)](https://github.com/aram-devdocs/GoudEngine/actions/workflows/ci.yml)
[![codecov](https://codecov.io/gh/aram-devdocs/GoudEngine/branch/main/graph/badge.svg)](https://codecov.io/gh/aram-devdocs/GoudEngine)
[![Security Audit](https://github.com/aram-devdocs/GoudEngine/actions/workflows/security.yml/badge.svg)](https://github.com/aram-devdocs/GoudEngine/actions/workflows/security.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Docs](https://img.shields.io/badge/docs-mdBook-blue)](https://goudengine.aramhammoudeh.com/)

[![crates.io](https://img.shields.io/crates/v/goud-engine.svg)](https://crates.io/crates/goud-engine)
[![npm](https://img.shields.io/npm/v/goudengine.svg)](https://www.npmjs.com/package/goudengine)
[![NuGet](https://img.shields.io/nuget/v/GoudEngine.svg)](https://www.nuget.org/packages/GoudEngine/)
[![PyPI](https://img.shields.io/pypi/v/goudengine.svg)](https://pypi.org/project/goudengine/)
[![Maven Central](https://img.shields.io/maven-central/v/io.github.aram-devdocs/goud-engine-kotlin.svg)](https://central.sonatype.com/artifact/io.github.aram-devdocs/goud-engine-kotlin)
[![LuaRocks](https://img.shields.io/luarocks/v/aram-devdocs/goudengine.svg)](https://luarocks.org/modules/aram-devdocs/goudengine)
[![Go Reference](https://pkg.go.dev/badge/github.com/aram-devdocs/goud-engine-go/goud.svg)](https://pkg.go.dev/github.com/aram-devdocs/goud-engine-go/goud)

[![total downloads](https://img.shields.io/badge/total_downloads-6%2C637-brightgreen)](#downloads)

Game engine written in Rust. Build 2D and 3D games from Rust, C#, Python, TypeScript, C, C++, Go, Kotlin, Swift, or Lua.

| | |
|---|---|
| **SDKs** | [Rust](sdks/rust/) · [C#](sdks/csharp/) · [Python](sdks/python/) · [TypeScript](sdks/typescript/) · [C](sdks/c/) · [C++](sdks/cpp/) · [Go](sdks/go/) · [Kotlin](sdks/kotlin/) · [Swift](sdks/swift/) · [Lua](luarocks/) |
| **Examples** | [All Examples](examples/) · [Flappy Bird (Rust)](examples/rust/) |
| **Docs** | [Documentation Site](https://goudengine.aramhammoudeh.com/) · [Getting Started](docs/src/getting-started/) · [Architecture](docs/src/architecture/) · [Development Guide](docs/src/development/guide.md) · [Building](docs/src/development/building.md) · [AI Setup](docs/src/development/ai-setup.md) |

## Alpha Roadmap

GoudEngine is working toward an alpha release. The full plan covers physics, audio, text rendering, animation, scenes, UI, 10 SDK languages, mobile/console support, and a networking system.

- **[ALPHA_ROADMAP.md](ALPHA_ROADMAP.md)** — Full technical roadmap
- **[Master tracking issue](https://github.com/aram-devdocs/GoudEngine/issues/114)** — ALPHA-001: GoudEngine Alpha Release
- **[Contributing](CONTRIBUTING.md)** — How to contribute

## Get Started

| Language | Install | Guide | Examples |
|----------|---------|-------|----------|
| Rust | `cargo add goud-engine` | [Getting Started](docs/src/getting-started/rust.md) | [Rust examples](examples/rust/) |
| C# (.NET) | `dotnet add package GoudEngine` | [Getting Started](docs/src/getting-started/csharp.md) | [C# examples](examples/csharp/) |
| Python | `pip install goudengine` | [Getting Started](docs/src/getting-started/python.md) | [Python examples](examples/python/) |
| TypeScript | `npm install goudengine` | [Getting Started](docs/src/getting-started/typescript.md) | [TypeScript examples](examples/typescript/) |
| C | Header-only | [Getting Started](docs/src/getting-started/c-cpp.md) | [C examples](examples/c/) |
| C++ | CMake / Meson | [Getting Started](docs/src/getting-started/c-cpp.md) | [C++ examples](examples/cpp/) |
| Go | `go get github.com/aram-devdocs/goud-engine-go` | [Getting Started](docs/src/getting-started/go.md) | [Go examples](examples/go/) |
| Kotlin | Gradle / Maven Central | [Getting Started](docs/src/getting-started/kotlin.md) | [Kotlin examples](examples/kotlin/) |
| Swift | Swift Package Manager | [Getting Started](docs/src/getting-started/swift.md) | [Swift examples](examples/swift/) |
| Lua | `luarocks install goudengine` | [Getting Started](docs/src/getting-started/lua.md) | [Lua examples](examples/lua/) |

## Design Philosophy

**All logic lives in Rust.** Language SDKs are thin wrappers that marshal data and call FFI functions, ensuring consistent behavior across all 10 bindings.

## Features

- Multi-language SDK support: Rust (native), C#, Python, TypeScript, C, C++, Go, Kotlin, Swift, Lua
- Rust-first architecture: all logic in Rust, SDKs are thin FFI wrappers
- WASM support: TypeScript SDK runs in browsers via WebAssembly
- Flexible renderer selection: 2D or 3D at runtime
- 2D rendering: sprites, 2D camera, Tiled map support
- 3D rendering: primitives (cubes, spheres, planes, cylinders)
- wgpu rendering backend (Vulkan/Metal/DX12/WebGPU) with OpenGL 3.3 legacy option
- winit windowing with GLFW legacy option
- Dynamic lighting: point, directional, and spot lights
- Physics simulation: Rapier 2D/3D rigid bodies, colliders, raycasting, collision events
- Audio playback: rodio-powered per-channel volume (Music, SFX, Ambience, UI, Voice) and spatial audio
- Text rendering: TrueType and bitmap fonts with alignment and word-wrapping
- Animation system: state machine controller, multi-layer blending, tweening
- Scene management: transitions (instant, fade, custom)
- UI component system: hierarchical node tree
- Networking: UDP, TCP, WebSocket, and WebRTC protocols
- Entity Component System (ECS): high-performance game object management
- Input handling: keyboard and mouse with frame-based state tracking
- Structured error diagnostics with error codes and recovery hints

## Debugger Runtime

Desktop/native SDKs now expose the shared debugger runtime surface for snapshots, control, debug draw, capture, replay, metrics export, and MCP attach.

- Enable it through config before creating the game or headless context.
- Use the raw JSON accessors for the full snapshot, capture, replay, and metrics payloads.
- Use the thin helpers to pause, step, change time scale, inject input, and toggle debug draw.
- TypeScript browser/WASM builds do not support the debugger runtime in this batch.
- The runtime is local-only. `goudengine-mcp` attaches over local IPC instead of running inside the game process.

See [Debugger Runtime Guide](docs/src/guides/debugger-runtime.md) for scope, determinism limits, artifact formats, and the MCP workflow.

## Architecture

```
┌─────────────────────────────────────────────────────────────────────┐
│                          Your Game Code                             │
│   (Rust / C# / Python / TypeScript / C / C++ / Go / Kotlin /       │
│    Swift / Lua)                                                     │
└─────────────────────────────────────────────────────────────────────┘
                                │
                                ▼
┌─────────────────────────────────────────────────────────────────────┐
│                         Language SDKs                               │
│  ┌────────┐ ┌────────┐ ┌────────┐ ┌──────────────┐ ┌─────┐ ┌─────┐│
│  │  Rust  │ │  C#    │ │ Python │ │  TypeScript   │ │  C  │ │ C++ ││
│  │(native)│ │(csbin- │ │(ctypes)│ │(napi + WASM)  │ │(hdr)│ │(RAII││
│  │        │ │ dgen)  │ │        │ │               │ │     │ │)    ││
│  └────────┘ └────────┘ └────────┘ └──────────────┘ └─────┘ └─────┘│
│  ┌────────┐ ┌────────┐ ┌────────┐ ┌──────────────┐                │
│  │   Go   │ │ Kotlin │ │ Swift  │ │     Lua      │                │
│  │  (cgo) │ │ (JNI)  │ │(C FFI) │ │   (mlua)     │                │
│  └────────┘ └────────┘ └────────┘ └──────────────┘                │
└─────────────────────────────────────────────────────────────────────┘
                                │
                                ▼
┌─────────────────────────────────────────────────────────────────────┐
│                       Rust Engine Core                              │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐              │
│  │ Graphics │ │   ECS    │ │ Platform │ │  Audio   │              │
│  │(wgpu ·   │ │ (World)  │ │(winit ·  │ │ (rodio)  │              │
│  │ OpenGL)  │ │          │ │ GLFW)    │ │          │              │
│  └──────────┘ └──────────┘ └──────────┘ └──────────┘              │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐              │
│  │ Physics  │ │   Text   │ │Animation │ │Networking│              │
│  │(Rapier)  │ │Rendering │ │ System   │ │(UDP/TCP/ │              │
│  │          │ │          │ │          │ │ WS/WebRTC│              │
│  └──────────┘ └──────────┘ └──────────┘ └──────────┘              │
└─────────────────────────────────────────────────────────────────────┘
```

### Codegen Pipeline

SDK bindings are generated from a unified schema:

```
codegen/goud_sdk.schema.json   (source of truth)
         │
         ├── gen_csharp.py     → sdks/csharp/
         ├── gen_python.py     → sdks/python/
         ├── gen_ts_node.py    → sdks/typescript/ (Node.js/napi)
         ├── gen_ts_web.py     → sdks/typescript/wasm/ (WebAssembly)
         ├── gen_swift.py      → sdks/swift/
         ├── gen_kotlin.py     → sdks/kotlin/
         ├── gen_go.py         → sdks/go/internal/ffi/
         ├── gen_go_sdk.py     → sdks/go/goud/
         └── gen_lua.py        → luarocks/goudengine/
```

See `codegen/AGENTS.md` for details.

### Project Directory

| Directory | Purpose |
|-----------|---------|
| `libs/` | Core libraries (graphics, platform, ECS, logger) |
| `goud_engine/` | Engine crate (core, assets, SDK, FFI) |
| `goud_engine_macros/` | Procedural macros |
| `sdks/` | Language SDKs (Rust, C#, Python, TypeScript, C, C++, Go, Kotlin, Swift) |
| `luarocks/` | Lua SDK (LuaRocks distribution package) |
| `codegen/` | Unified SDK code generation pipeline |
| `tools/` | Development utilities (lint-layers) |
| `examples/` | Example games organized by SDK language |
| `docs/` | mdBook documentation site source |
| `scripts/` | Build, codegen, and CI helper scripts |
| `ports/` | Package manager ports (Conan, vcpkg) |

## Examples

See [examples/README.md](examples/README.md) for the full list with run commands. All examples use `./dev.sh`:

```sh
./dev.sh --game flappy_goud                          # C# Flappy Bird
./dev.sh --game 3d_cube                              # C# 3D demo
./dev.sh --game feature_lab                          # C# headless feature lab
./dev.sh --sdk python --game flappy_bird             # Python Flappy Bird
./dev.sh --sdk typescript --game flappy_bird         # TypeScript desktop
./dev.sh --sdk typescript --game flappy_bird_web     # TypeScript web
./dev.sh --sdk go --game flappy_bird                 # Go Flappy Bird
./dev.sh --sdk kotlin --game flappy_bird             # Kotlin Flappy Bird
./dev.sh --sdk swift --game flappy_bird              # Swift Flappy Bird
./dev.sh --sdk lua --game flappy_bird                # Lua Flappy Bird
cargo run -p flappy-bird                             # Rust Flappy Bird
```

## Documentation

- Getting Started: [Rust](docs/src/getting-started/rust.md) · [C#](docs/src/getting-started/csharp.md) · [Python](docs/src/getting-started/python.md) · [TypeScript](docs/src/getting-started/typescript.md) · [C/C++](docs/src/getting-started/c-cpp.md) · [Go](docs/src/getting-started/go.md) · [Kotlin](docs/src/getting-started/kotlin.md) · [Swift](docs/src/getting-started/swift.md) · [Lua](docs/src/getting-started/lua.md)
- [SDK-First Architecture](docs/src/architecture/sdk-first.md)
- [Adding a New Language](docs/src/architecture/adding-a-language.md)
- [Development Guide](docs/src/development/guide.md) — dev.sh, Git hooks, version management
- [Building](docs/src/development/building.md) — build.sh, package.sh, NuGet feed
- [AI Setup](docs/src/development/ai-setup.md) — Claude Code, Cursor, Gemini configuration
- [Rust SDK](sdks/rust/README.md)
- [C# SDK](sdks/csharp/README.md)
- [Python SDK](sdks/python/README.md)
- [TypeScript SDK](sdks/typescript/README.md)
- [Go SDK](sdks/go/README.md)
- [Kotlin SDK](sdks/kotlin/README.md)
- [Swift SDK](sdks/swift/README.md)
- [C SDK](sdks/c/README.md)
- [C++ SDK](sdks/cpp/README.md)
- [Lua SDK](luarocks/README.md)
- [csbindgen](https://github.com/Cysharp/csbindgen) — C# bindings generator
- [cbindgen](https://github.com/mozilla/cbindgen) — C header generator

## Community

<!-- COMMUNITY-STATS:START -->
| | Stars | Forks | Contributors |
|--|-------|-------|--------------|
| **GitHub** | ![stars](https://img.shields.io/github/stars/aram-devdocs/GoudEngine) | ![forks](https://img.shields.io/github/forks/aram-devdocs/GoudEngine) | ![contributors](https://img.shields.io/github/contributors/aram-devdocs/GoudEngine) |

### Downloads

| Registry | Total Downloads |
|----------|-----------------|
| crates.io | [39](https://crates.io/crates/goud-engine) |
| NuGet | [1,647](https://www.nuget.org/packages/GoudEngine/) |
| PyPI | [3,514](https://pypi.org/project/goudengine/) |
| npm | [1,437](https://www.npmjs.com/package/goudengine) |
| Maven Central | [0](https://central.sonatype.com/artifact/io.github.aram-devdocs/goud-engine-kotlin) |
| LuaRocks | [0](https://luarocks.org/modules/aram-devdocs/goudengine) |
| Go | [0 versions](https://pkg.go.dev/github.com/aram-devdocs/goud-engine-go/goud) |

<sub>PyPI totals exclude mirrors.</sub>

![Total Downloads Over Time](.github/stats/downloads.svg)

[![Star History Chart](https://api.star-history.com/svg?repos=aram-devdocs/GoudEngine&type=Date)](https://star-history.com/#aram-devdocs/GoudEngine&Date)

<sub>Last updated: 2026-03-25 via [GitHub Action](.github/workflows/community-stats.yml)</sub>
<!-- COMMUNITY-STATS:END -->

## License

MIT
