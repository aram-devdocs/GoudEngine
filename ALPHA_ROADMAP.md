# GoudEngine Alpha Roadmap

## Vision

GoudEngine is a Rust-first game engine library that replaces Unity, Unreal, Godot, MonoGame, and Pygame. Build games in any language, deploy everywhere, with Rust performance. AI-native, unopinionated, code-first.

**Alpha Definition**: A developer can build a complete game (2D or 3D) using any supported SDK, and deploy it to desktop, web, and mobile from the same codebase.

---

## Architecture Principles

### Provider Abstraction Pattern

Every major subsystem uses a provider trait that can be swapped at configuration time. Users can choose built-in providers or inject their own.

```
Engine Config
├── Renderer Provider  → OpenGL | wgpu/Vulkan | wgpu/Metal | wgpu/D3D12 | wgpu/WebGPU | Custom
├── Physics Provider   → Rapier2D | Rapier3D | Built-in Simple | Custom
├── Audio Provider     → Rodio | Web Audio | Custom
├── Windowing Provider → winit (primary) | SDL (console fallback) | Custom
└── Input Provider     → winit | SDL | Web DOM | Custom
```

### Platform Strategy

**Rendering**: wgpu is the primary backend (covers Vulkan, Metal, D3D12, WebGPU). OpenGL retained as legacy/debug fallback. Provider trait allows console porting companies to plug in proprietary backends (GNM, NVN).

**Windowing**: Migrate from GLFW to winit as primary. winit covers desktop + iOS + Android + web. SDL kept as optional for console portkit access.

### SDK Strategy

All SDKs are thin wrappers generated from a single schema. No logic in SDKs. Rust-first, FFI boundary, codegen.

**Current SDKs**: Rust, C#, Python, TypeScript (Node + Web)
**Alpha additions**: C/C++, Lua, Swift, Kotlin/Java, Go

### Deployment Matrix (Alpha Target)

| Language | Desktop (Win/Mac/Linux) | Web (Browser) | iOS | Android | Console |
|----------|------------------------|---------------|-----|---------|---------|
| Rust | Yes | Yes (WASM) | Yes | Yes | Via C header |
| C# | Yes (P/Invoke) | Blazor WASM | .NET MAUI | .NET MAUI | No |
| Python | Yes (ctypes) | No | No | No | No |
| TypeScript | Yes (napi) | Yes (WASM) | React Native | React Native | No |
| C/C++ | Yes | Emscripten | Yes (native) | Yes (NDK) | Yes (NDA) |
| Lua | Yes (embedded) | Yes (embedded) | Yes (embedded) | Yes (embedded) | Yes (embedded) |
| Swift | macOS | No | Yes (native) | No | No |
| Kotlin/Java | Yes (JNI) | No | No | Yes (JNI) | No |
| Go | Yes (cgo) | No | No | No | No |

---

## Phase Structure

Six sequential phases. Within each phase, feature tracks can be worked in parallel by different developers. Each phase has a milestone. A phase gate review happens before moving to the next.

```
Phase 0: Foundation          ← DX, templates, docs, project restructure (no code)
Phase 1: Core Stabilization  ← Fix broken things, refactor, architecture cleanup
Phase 2: Core Features       ← Physics, audio, text, animation, scenes, UI
Phase 3: Rendering Pipeline  ← wgpu consolidation, batching, shaders, 3D, particles
Phase 4: SDK Expansion       ← New languages, fix existing SDKs, error handling
Phase 5: Platform Expansion  ← Mobile, console strategy, cross-compilation
Phase 6: Quality & Polish    ← Tests, benchmarks, docs site, tutorials, examples
```

---

## Master Issue: `ALPHA-001` — GoudEngine Alpha Release

Tracks overall alpha progress. All sub-issues link here.

### Sub-Issue Structure

```
ALPHA-001: GoudEngine Alpha Release (master)
│
├── ALPHA-F00: Foundation & Developer Experience
│   ├── ALPHA-F00-01: Create issue templates (bug, feature, task, RFC)
│   ├── ALPHA-F00-02: Create CONTRIBUTING.md
│   ├── ALPHA-F00-03: Create ARCHITECTURE.md with diagrams
│   ├── ALPHA-F00-04: Restructure GitHub project board
│   ├── ALPHA-F00-05: Add issue label taxonomy
│   ├── ALPHA-F00-06: Create milestone structure in GitHub
│   ├── ALPHA-F00-07: Create RFC template for design decisions
│   ├── ALPHA-F00-08: Update README with alpha roadmap link
│   ├── ALPHA-F00-09: Create docs/ getting-started guides per SDK
│   ├── ALPHA-F00-10: Set up mdBook or similar for docs site
│   ├── ALPHA-F00-11: Add CODE_OF_CONDUCT.md
│   ├── ALPHA-F00-12: Triage and update all existing open issues
│   └── ALPHA-F00-13: Create development environment setup guide (install.sh docs)
│
├── ALPHA-F01: Core Stabilization
│   ├── ALPHA-F01-01: Refactor schedule.rs (8301 lines → multiple files)
│   ├── ALPHA-F01-02: Refactor world.rs (4374 lines → multiple files)
│   ├── ALPHA-F01-03: Refactor handle.rs (3574 lines → multiple files)
│   ├── ALPHA-F01-04: Refactor remaining 20 oversized files (>500 lines)
│   ├── ALPHA-F01-05: Fix layer hierarchy violations (sprite_batch, platform imports)
│   ├── ALPHA-F01-06: Add SAFETY comments to all unsafe blocks
│   ├── ALPHA-F01-07: Fix uniform location caching in OpenGL backend
│   ├── ALPHA-F01-08: Add GL error checking to all state operations
│   ├── ALPHA-F01-09: Fix hierarchy cascade deletion
│   ├── ALPHA-F01-10: Resolve duplicate physics data (Collider vs RigidBody)
│   ├── ALPHA-F01-11: Fix lint-layers tool to catch current violations
│   ├── ALPHA-F01-12: Add tests for parallel system execution safety
│   └── ALPHA-F01-13: Remove all #[allow(dead_code)] from production code
│
├── ALPHA-F02: Provider Abstraction Layer
│   ├── ALPHA-F02-01: Design provider trait pattern (RFC)
│   ├── ALPHA-F02-02: Implement RenderProvider trait
│   ├── ALPHA-F02-03: Implement PhysicsProvider trait
│   ├── ALPHA-F02-04: Implement AudioProvider trait
│   ├── ALPHA-F02-05: Implement WindowProvider trait
│   ├── ALPHA-F02-06: Implement InputProvider trait
│   ├── ALPHA-F02-07: Engine configuration builder with provider selection
│   ├── ALPHA-F02-08: Provider hot-swap support (dev mode only)
│   └── ALPHA-F02-09: Provider capability query API (what does this provider support?)
│
├── ALPHA-F03: Physics System
│   ├── ALPHA-F03-01: Integrate rapier2d as default 2D physics provider
│   ├── ALPHA-F03-02: Integrate rapier3d as default 3D physics provider
│   ├── ALPHA-F03-03: Physics step system (fixed timestep, accumulator)
│   ├── ALPHA-F03-04: Gravity system
│   ├── ALPHA-F03-05: Collision response system (impulse resolution)
│   ├── ALPHA-F03-06: Collision event system (on_enter, on_stay, on_exit)
│   ├── ALPHA-F03-07: Trigger/sensor zones
│   ├── ALPHA-F03-08: Raycasting API
│   ├── ALPHA-F03-09: Collision layers and masks (filtering)
│   ├── ALPHA-F03-10: Constraints and joints (distance, revolute, prismatic)
│   ├── ALPHA-F03-11: Continuous collision detection (CCD)
│   ├── ALPHA-F03-12: Physics debug visualization (wireframe shapes)
│   ├── ALPHA-F03-13: Expose physics via FFI
│   └── ALPHA-F03-14: Built-in simple physics provider (fallback without rapier)
│
├── ALPHA-F04: Audio System
│   ├── ALPHA-F04-01: Implement audio asset loader (WAV, OGG, FLAC)
│   ├── ALPHA-F04-02: Complete AudioManager integration with asset system
│   ├── ALPHA-F04-03: Per-channel volume control (Music, SFX, Voice, Ambience, UI)
│   ├── ALPHA-F04-04: Audio streaming for large files
│   ├── ALPHA-F04-05: 3D spatial audio (position, attenuation, panning)
│   ├── ALPHA-F04-06: Web Audio provider (for WASM target)
│   ├── ALPHA-F04-07: Expose audio via FFI
│   ├── ALPHA-F04-08: Audio in all SDKs (C#, Python, TypeScript)
│   └── ALPHA-F04-09: Audio crossfade and mixing
│
├── ALPHA-F05: Text & Font Rendering
│   ├── ALPHA-F05-01: Font asset loader (TTF, OTF via fontdue or ab_glyph)
│   ├── ALPHA-F05-02: Glyph atlas generation and caching
│   ├── ALPHA-F05-03: Text rendering API (drawText with position, size, color)
│   ├── ALPHA-F05-04: Text layout (alignment, wrapping, line spacing)
│   ├── ALPHA-F05-05: Bitmap font support (for pixel art games)
│   ├── ALPHA-F05-06: Expose text rendering via FFI
│   ├── ALPHA-F05-07: Text in all SDKs
│   └── ALPHA-F05-08: Unicode and multi-language text support
│
├── ALPHA-F06: Animation System
│   ├── ALPHA-F06-01: Sprite sheet animation (frame sequences, timing)
│   ├── ALPHA-F06-02: Animation controller (states, transitions)
│   ├── ALPHA-F06-03: Tween/easing library (lerp, ease-in, ease-out, bezier)
│   ├── ALPHA-F06-04: Skeletal animation (2D bones)
│   ├── ALPHA-F06-05: Animation blending and layering
│   ├── ALPHA-F06-06: Animation events (callbacks at keyframes)
│   ├── ALPHA-F06-07: Expose animation via FFI
│   └── ALPHA-F06-08: Animation in all SDKs
│
├── ALPHA-F07: Scene Management
│   ├── ALPHA-F07-01: Multiple worlds/scenes support
│   ├── ALPHA-F07-02: Scene loading and unloading
│   ├── ALPHA-F07-03: Scene serialization (save/load to JSON/binary)
│   ├── ALPHA-F07-04: Prefab system (entity templates)
│   ├── ALPHA-F07-05: Scene transitions (fade, wipe, custom)
│   ├── ALPHA-F07-06: Expose scene management via FFI
│   └── ALPHA-F07-07: Scene management in all SDKs
│
├── ALPHA-F08: UI Framework
│   ├── ALPHA-F08-01: UI component system (separate from game ECS)
│   ├── ALPHA-F08-02: Layout engine (flex-like, anchors, margins)
│   ├── ALPHA-F08-03: Basic widgets (Button, Label, Image, Panel, Slider)
│   ├── ALPHA-F08-04: Input handling for UI (click, hover, focus)
│   ├── ALPHA-F08-05: UI theming/styling system
│   ├── ALPHA-F08-06: UI rendering integration with main renderer
│   ├── ALPHA-F08-07: Expose UI via FFI
│   └── ALPHA-F08-08: UI in all SDKs
│
├── ALPHA-F09: Rendering Pipeline
│   ├── ALPHA-F09-01: Consolidate on wgpu as primary renderer
│   ├── ALPHA-F09-02: Migrate native windowing from GLFW to winit
│   ├── ALPHA-F09-03: Fix sprite batch (implement shader loading, texture upload)
│   ├── ALPHA-F09-04: Framebuffer / render target support
│   ├── ALPHA-F09-05: Custom shader system (load, compile, hot-reload)
│   ├── ALPHA-F09-06: 3D mesh loading (GLTF, OBJ)
│   ├── ALPHA-F09-07: Particle system
│   ├── ALPHA-F09-08: Post-processing pipeline (bloom, blur, color grading)
│   ├── ALPHA-F09-09: Shadow mapping
│   ├── ALPHA-F09-10: Anti-aliasing (MSAA, FXAA)
│   ├── ALPHA-F09-11: Texture atlasing and sprite sheets
│   ├── ALPHA-F09-12: Camera resize handling and viewport management
│   ├── ALPHA-F09-13: Z-layer system (configurable draw order, not Y-position hack)
│   ├── ALPHA-F09-14: Instanced rendering
│   ├── ALPHA-F09-15: Frustum culling and occlusion
│   ├── ALPHA-F09-16: Skeletal mesh rendering (3D)
│   ├── ALPHA-F09-17: Material system (shader + textures + params)
│   ├── ALPHA-F09-18: PBR lighting model (optional, alongside Phong)
│   ├── ALPHA-F09-19: OpenGL backend retained as legacy/debug provider
│   └── ALPHA-F09-20: Window resize, fullscreen toggle, aspect ratio management
│
├── ALPHA-F10: ECS Improvements
│   ├── ALPHA-F10-01: Change detection (Changed<T>, Added<T> query filters)
│   ├── ALPHA-F10-02: EventReader<T> / EventWriter<T> system parameters
│   ├── ALPHA-F10-03: Optional component queries (Option<&T>)
│   ├── ALPHA-F10-04: 3D hierarchy transform propagation
│   ├── ALPHA-F10-05: Entity cloning / prefab instantiation
│   ├── ALPHA-F10-06: Plugin system (register systems, resources, components)
│   ├── ALPHA-F10-07: Default systems (auto-register transform propagation, etc.)
│   ├── ALPHA-F10-08: Non-send resources (window handles, GL contexts)
│   ├── ALPHA-F10-09: Query caching between frames
│   └── ALPHA-F10-10: System sets and ordering groups
│
├── ALPHA-F11: Asset System Completion
│   ├── ALPHA-F11-01: Async asset loading (background thread pool)
│   ├── ALPHA-F11-02: Asset dependency tracking and cascade reload
│   ├── ALPHA-F11-03: Tiled map loader and renderer
│   ├── ALPHA-F11-04: Mesh asset loader (GLTF, OBJ)
│   ├── ALPHA-F11-05: Material asset type
│   ├── ALPHA-F11-06: Animation asset type
│   ├── ALPHA-F11-07: Config asset type (JSON, TOML)
│   ├── ALPHA-F11-08: Asset packaging/bundling for distribution
│   ├── ALPHA-F11-09: Compressed texture support (DDS, BC)
│   ├── ALPHA-F11-10: Reference counting for asset handles
│   ├── ALPHA-F11-11: Fallback/default assets on load failure
│   └── ALPHA-F11-12: Virtual filesystem abstraction
│
├── ALPHA-F12: Error Handling Overhaul
│   ├── ALPHA-F12-01: Structured error types across FFI (not just i32 codes)
│   ├── ALPHA-F12-02: Error context propagation (what failed, where, why)
│   ├── ALPHA-F12-03: Error recovery strategies documentation
│   ├── ALPHA-F12-04: SDK error types (C# exceptions, Python exceptions, TS errors)
│   ├── ALPHA-F12-05: Error logging integration
│   └── ALPHA-F12-06: Debug/diagnostic mode with verbose error output
│
├── ALPHA-F13: Existing SDK Fixes
│   ├── ALPHA-F13-01: Fix WASM isKeyJustPressed (#111)
│   ├── ALPHA-F13-02: Fix WASM recursive borrow crash (#112)
│   ├── ALPHA-F13-03: Address IRON FIST feedback (#113)
│   ├── ALPHA-F13-04: Restore C# test suite
│   ├── ALPHA-F13-05: Expand Python test coverage to 80%+
│   ├── ALPHA-F13-06: Expand TypeScript test coverage to 80%+
│   ├── ALPHA-F13-07: Web platform gotchas documentation
│   ├── ALPHA-F13-08: Codegen drift CI validation (validate_coverage.py)
│   └── ALPHA-F13-09: Asset preloader API for web
│
├── ALPHA-F14: C/C++ SDK
│   ├── ALPHA-F14-01: cbindgen header cleanup and packaging
│   ├── ALPHA-F14-02: C SDK wrapper (idiomatic C API over raw FFI)
│   ├── ALPHA-F14-03: C++ SDK wrapper (RAII, classes, namespaces)
│   ├── ALPHA-F14-04: CMake/Meson build integration
│   ├── ALPHA-F14-05: C/C++ example game (Flappy Bird parity)
│   ├── ALPHA-F14-06: C/C++ test suite
│   ├── ALPHA-F14-07: vcpkg/Conan package
│   └── ALPHA-F14-08: C/C++ SDK documentation
│
├── ALPHA-F15: Lua Scripting SDK
│   ├── ALPHA-F15-01: mlua integration (embedded Lua VM)
│   ├── ALPHA-F15-02: Lua API binding generation from schema
│   ├── ALPHA-F15-03: Lua scripting runtime (load/execute .lua files)
│   ├── ALPHA-F15-04: Hot-reload for Lua scripts
│   ├── ALPHA-F15-05: Lua example game (Flappy Bird parity)
│   ├── ALPHA-F15-06: Lua test suite
│   ├── ALPHA-F15-07: LuaRocks package
│   └── ALPHA-F15-08: Lua SDK documentation
│
├── ALPHA-F16: Swift SDK
│   ├── ALPHA-F16-01: Swift bridging header from cbindgen
│   ├── ALPHA-F16-02: Swift wrapper classes (idiomatic Swift API)
│   ├── ALPHA-F16-03: Swift Package Manager integration
│   ├── ALPHA-F16-04: Swift example game (Flappy Bird parity)
│   ├── ALPHA-F16-05: Swift test suite
│   └── ALPHA-F16-06: Swift SDK documentation
│
├── ALPHA-F17: Kotlin/Java SDK
│   ├── ALPHA-F17-01: JNI bindings via jni crate
│   ├── ALPHA-F17-02: Kotlin wrapper classes (idiomatic Kotlin API)
│   ├── ALPHA-F17-03: Gradle build integration
│   ├── ALPHA-F17-04: Kotlin example game (Flappy Bird parity)
│   ├── ALPHA-F17-05: Kotlin test suite
│   ├── ALPHA-F17-06: Maven Central package
│   └── ALPHA-F17-07: Kotlin SDK documentation
│
├── ALPHA-F18: Go SDK
│   ├── ALPHA-F18-01: cgo bindings from C header
│   ├── ALPHA-F18-02: Go wrapper package (idiomatic Go API)
│   ├── ALPHA-F18-03: Go module setup
│   ├── ALPHA-F18-04: Go example game (Flappy Bird parity)
│   ├── ALPHA-F18-05: Go test suite
│   └── ALPHA-F18-06: Go SDK documentation
│
├── ALPHA-F19: Mobile Platform Support
│   ├── ALPHA-F19-01: iOS build pipeline (cargo-lipo, Xcode project template)
│   ├── ALPHA-F19-02: iOS touch input handling
│   ├── ALPHA-F19-03: iOS app lifecycle (background, foreground, terminate)
│   ├── ALPHA-F19-04: iOS example app
│   ├── ALPHA-F19-05: Android build pipeline (cargo-ndk, Gradle template)
│   ├── ALPHA-F19-06: Android touch input handling
│   ├── ALPHA-F19-07: Android app lifecycle (Activity, NativeActivity)
│   ├── ALPHA-F19-08: Android example app
│   ├── ALPHA-F19-09: Mobile CI (iOS simulator, Android emulator)
│   ├── ALPHA-F19-10: Responsive scaling for mobile screen sizes
│   └── ALPHA-F19-11: Gamepad/controller input on mobile
│
├── ALPHA-F20: Console Strategy
│   ├── ALPHA-F20-01: Document rendering backend trait for console porters
│   ├── ALPHA-F20-02: Static C library packaging for console consumption
│   ├── ALPHA-F20-03: Console porting guide (for NDA partners)
│   ├── ALPHA-F20-04: Xbox GDK feasibility proof-of-concept
│   ├── ALPHA-F20-05: SDL windowing provider (for console portkit access)
│   └── ALPHA-F20-06: Nintendo Switch Vulkan feasibility assessment
│
├── ALPHA-F21: Debugging & Profiling Tools
│   ├── ALPHA-F21-01: FPS counter / debug overlay
│   ├── ALPHA-F21-02: Frame profiler (CPU time per system)
│   ├── ALPHA-F21-03: Debug draw API (wireframe shapes, lines, points)
│   ├── ALPHA-F21-04: Render statistics (draw calls, triangles, state changes)
│   ├── ALPHA-F21-05: Memory usage tracking
│   ├── ALPHA-F21-06: Entity/component inspector (runtime)
│   ├── ALPHA-F21-07: Expose debug tools via FFI
│   └── ALPHA-F21-08: Performance benchmark suite (criterion)
│
├── ALPHA-F22: Testing & Quality
│   ├── ALPHA-F22-01: Headless OpenGL/wgpu for CI (osmesa or software renderer)
│   ├── ALPHA-F22-02: FFI safety test suite (null pointers, double-free, invalid handles)
│   ├── ALPHA-F22-03: Integration test suite (cross-layer)
│   ├── ALPHA-F22-04: Code coverage reporting (80%+ target)
│   ├── ALPHA-F22-05: Performance regression detection in CI
│   ├── ALPHA-F22-06: Fuzz testing for FFI boundary
│   ├── ALPHA-F22-07: Security audit for unsafe code
│   └── ALPHA-F22-08: Memory leak detection (valgrind/miri)
│
├── ALPHA-F23: Documentation & Guides
│   ├── ALPHA-F23-01: Hosted docs site (mdBook + cargo doc)
│   ├── ALPHA-F23-02: API reference generation and hosting
│   ├── ALPHA-F23-03: Getting started tutorial per SDK language
│   ├── ALPHA-F23-04: Architecture deep-dive guide
│   ├── ALPHA-F23-05: "Build Your First Game" tutorial
│   ├── ALPHA-F23-06: Provider system documentation
│   ├── ALPHA-F23-07: Cross-platform deployment guide
│   ├── ALPHA-F23-08: FAQ and troubleshooting
│   ├── ALPHA-F23-09: Video tutorials (optional)
│   └── ALPHA-F23-10: Example game showcase
│
├── ALPHA-F24: Example Games
│   ├── ALPHA-F24-01: Flappy Bird in ALL SDK languages (parity test)
│   ├── ALPHA-F24-02: Platformer game (physics, tiles, animation)
│   ├── ALPHA-F24-03: Top-down RPG (UI, text, scenes, save/load)
│   ├── ALPHA-F24-04: 3D showcase (mesh loading, lighting, camera)
│   ├── ALPHA-F24-05: Multiplayer game example (networking showcase)
│   └── ALPHA-F24-06: Mobile-optimized game example
│
└── ALPHA-F25: Networking System
    ├── ALPHA-F25-01: NetworkProvider trait design (RFC)
    ├── ALPHA-F25-02: UDP transport layer (raw sockets, send/recv, channels)
    ├── ALPHA-F25-03: TCP transport layer (reliable ordered delivery)
    ├── ALPHA-F25-04: WebSocket transport (web-compatible, ws/wss)
    ├── ALPHA-F25-05: WebRTC data channels (peer-to-peer web support)
    ├── ALPHA-F25-06: Serialization framework (compact binary, delta compression)
    ├── ALPHA-F25-07: Client-server architecture (host/join, authority model)
    ├── ALPHA-F25-08: Peer-to-peer architecture (mesh topology, relay fallback)
    ├── ALPHA-F25-09: State synchronization (snapshot interpolation, entity sync)
    ├── ALPHA-F25-10: Rollback netcode (input prediction, state rollback, resimulation)
    ├── ALPHA-F25-11: Lobby and matchmaking (create/join/list rooms, ready state)
    ├── ALPHA-F25-12: RPC framework (remote procedure calls across network)
    ├── ALPHA-F25-13: Network simulation tools (artificial latency, packet loss, jitter)
    ├── ALPHA-F25-14: Network debug overlay (ping, bandwidth, packet stats)
    ├── ALPHA-F25-15: Expose networking via FFI
    └── ALPHA-F25-16: Networking in all SDKs
```

---

## Phase Assignments

### Phase 0: Foundation (No Code Changes)
**Goal**: Get the project ready for parallel development by multiple contributors.
**Milestone**: `alpha-phase-0`
**Tracks** (all parallel):

| Track | Issues | Dependencies |
|-------|--------|-------------|
| Project setup | F00-01 through F00-06 | None |
| Documentation | F00-07 through F00-13 | None |
| Issue triage | F00-12 | None |

**Gate**: All templates, guides, and project structure in place.

---

### Phase 1: Core Stabilization
**Goal**: Fix all broken code, refactor oversized files, resolve architecture violations. No new features.
**Milestone**: `alpha-phase-1`
**Depends on**: Phase 0
**Tracks** (parallel where files don't overlap):

| Track | Issues | Dependencies |
|-------|--------|-------------|
| File refactoring | F01-01 through F01-04 | None (different files) |
| Architecture fixes | F01-05, F01-06, F01-11 | None |
| OpenGL fixes | F01-07, F01-08 | None |
| ECS fixes | F01-09, F01-10, F01-12, F01-13 | None |

**Gate**: All files under 500 lines. Zero architecture violations. All unsafe blocks documented. CI green.

---

### Phase 2: Core Systems
**Goal**: Implement the provider abstraction and all missing core features. This is the largest phase.
**Milestone**: `alpha-phase-2`
**Depends on**: Phase 1
**Tracks** (parallel — each feature is independent):

| Track | Issues | Dependencies |
|-------|--------|-------------|
| Provider abstraction | F02-01 through F02-09 | Phase 1 complete |
| Physics | F03-01 through F03-14 | F02-03 (PhysicsProvider trait) |
| Audio | F04-01 through F04-09 | F02-04 (AudioProvider trait) |
| Text/Fonts | F05-01 through F05-08 | Rendering works |
| Animation | F06-01 through F06-08 | ECS works |
| Scene management | F07-01 through F07-07 | ECS works |
| UI framework | F08-01 through F08-08 | Text rendering (F05), Input |
| ECS improvements | F10-01 through F10-10 | Phase 1 complete |
| Asset completion | F11-01 through F11-12 | Phase 1 complete |
| Error handling | F12-01 through F12-06 | Phase 1 complete |

**Gate**: Physics simulation runs. Audio plays. Text renders. Animations play. Scenes load/unload. Basic UI works. All through Rust SDK.

---

### Phase 3: Rendering Pipeline
**Goal**: Consolidate rendering on wgpu, implement advanced rendering features.
**Milestone**: `alpha-phase-3`
**Depends on**: Phase 2 (provider abstraction done)
**Tracks**:

| Track | Issues | Dependencies |
|-------|--------|-------------|
| wgpu consolidation | F09-01, F09-02, F09-19 | F02-02 (RenderProvider) |
| 2D rendering | F09-03, F09-11, F09-13 | wgpu consolidation |
| 3D rendering | F09-06, F09-16, F09-17, F09-18 | wgpu consolidation |
| Advanced rendering | F09-04, F09-05, F09-07, F09-08, F09-09, F09-10 | wgpu consolidation |
| Window management | F09-12, F09-14, F09-15, F09-20 | F09-02 (winit migration) |

**Gate**: wgpu is primary renderer. Sprite batching works. 3D meshes load. Custom shaders compile. Post-processing pipeline exists.

---

### Phase 4: SDK Expansion
**Goal**: Fix existing SDKs, add all new language SDKs, ensure feature parity.
**Milestone**: `alpha-phase-4`
**Depends on**: Phase 2 (core features done — FFI needs something to expose)
**Tracks** (all parallel — each SDK is independent):

| Track | Issues | Dependencies |
|-------|--------|-------------|
| Existing SDK fixes | F13-01 through F13-09 | Phase 2 features |
| C/C++ SDK | F14-01 through F14-08 | cbindgen header packaging lands in F14-01 |
| Lua SDK | F15-01 through F15-08 | Engine API stable |
| Swift SDK | F16-01 through F16-06 | C header (F14-01) |
| Kotlin SDK | F17-01 through F17-07 | JNI boundary |
| Go SDK | F18-01 through F18-06 | C header (F14-01) |

**Gate**: All SDKs can run Flappy Bird. Tests pass for every SDK. Feature parity verified.

---

### Phase 5: Platform Expansion
**Goal**: Ship on mobile, prepare for console.
**Milestone**: `alpha-phase-5`
**Depends on**: Phase 3 (wgpu + winit), Phase 4 (SDKs done)
**Tracks**:

| Track | Issues | Dependencies |
|-------|--------|-------------|
| iOS | F19-01 through F19-04, F19-10, F19-11 | winit + wgpu |
| Android | F19-05 through F19-08, F19-10, F19-11 | winit + wgpu |
| Mobile CI | F19-09 | iOS + Android builds |
| Console strategy | F20-01 through F20-06 | C/C++ SDK (F14) |

**Gate**: Flappy Bird runs on iOS simulator and Android emulator. Console porting guide published.

---

### Phase 6: Quality & Polish
**Goal**: Production-quality testing, documentation, examples, performance.
**Milestone**: `alpha-phase-6`
**Depends on**: All previous phases
**Tracks** (all parallel):

| Track | Issues | Dependencies |
|-------|--------|-------------|
| Testing | F22-01 through F22-08 | All features implemented |
| Debugging tools | F21-01 through F21-08 | Rendering + ECS |
| Networking | F25-01 through F25-16 | Core systems + SDKs |
| Documentation | F23-01 through F23-10 | All features implemented |
| Example games | F24-01 through F24-06 | All SDKs + features + networking |

**Gate**: 80%+ test coverage. Docs site live. 3+ example games. Benchmark baselines set. Alpha release tagged.

---

## Issue Label Taxonomy

```
Type:
  type:bug           — Something broken
  type:feature       — New capability
  type:refactor      — Code improvement, no behavior change
  type:docs          — Documentation only
  type:test          — Test coverage
  type:ci            — CI/CD pipeline
  type:dx            — Developer experience improvement

Area:
  area:ecs           — Entity Component System
  area:rendering     — Graphics pipeline
  area:physics       — Physics simulation
  area:audio         — Audio system
  area:input         — Input handling
  area:assets        — Asset loading and management
  area:ffi           — FFI boundary
  area:sdk-csharp    — C# SDK
  area:sdk-python    — Python SDK
  area:sdk-typescript — TypeScript SDK
  area:sdk-rust      — Rust SDK
  area:sdk-cpp       — C/C++ SDK
  area:sdk-lua       — Lua SDK
  area:sdk-swift     — Swift SDK
  area:sdk-kotlin    — Kotlin SDK
  area:sdk-go        — Go SDK
  area:platform      — Platform-specific (mobile, console, web)
  area:codegen       — Code generation pipeline
  area:ui            — UI framework
  area:animation     — Animation system
  area:scene         — Scene management
  area:text          — Text/font rendering
  area:networking    — Networking and multiplayer
  area:tools         — Development tools

Priority:
  priority:critical  — Blocks other work
  priority:high      — Must have for alpha
  priority:medium    — Should have for alpha
  priority:low       — Nice to have

Phase:
  phase:0-foundation
  phase:1-stabilization
  phase:2-core
  phase:3-rendering
  phase:4-sdks
  phase:5-platforms
  phase:6-polish

Status:
  status:needs-design    — Needs RFC/design before implementation
  status:ready           — Designed, ready to implement
  status:in-progress     — Being worked on
  status:needs-review    — PR open, needs review
  status:blocked         — Waiting on dependency
```

---

## GitHub Project Board Structure

### Views

1. **Alpha Overview** (Board) — Columns: Backlog | Phase 0 | Phase 1 | Phase 2 | Phase 3 | Phase 4 | Phase 5 | Phase 6 | Done
2. **Current Sprint** (Board) — Columns: To Do | In Progress | In Review | Done
3. **By Feature** (Table) — Group by feature sub-issue (F01, F02, etc.)
4. **By Area** (Table) — Group by area label
5. **Blockers** (Table) — Filter: status:blocked

### Fields

- **Phase**: Single select (Phase 0-6)
- **Feature**: Single select (F00-F25)
- **Effort**: Single select (XS/S/M/L/XL)
- **Priority**: Single select (Critical/High/Medium/Low)
- **Blocked By**: Text (issue numbers)

---

## Issue Templates

### Bug Report
```markdown
---
name: Bug Report
about: Report a bug in GoudEngine
labels: type:bug
---

## Description
<!-- Clear description of the bug -->

## Steps to Reproduce
1.
2.
3.

## Expected Behavior
<!-- What should happen -->

## Actual Behavior
<!-- What actually happens -->

## Environment
- OS:
- SDK Language:
- SDK Version:
- Renderer:

## Minimal Reproduction
<!-- Code snippet or link to minimal repo -->
```

### Feature Request
```markdown
---
name: Feature Request
about: Propose a new feature
labels: type:feature
---

## Problem Statement
<!-- What problem does this solve? -->

## Proposed Solution
<!-- How should it work? -->

## API Design
<!-- What would the API look like from the SDK user's perspective? -->

## Alternatives Considered
<!-- Other approaches and why they were rejected -->

## Impact
- [ ] Requires FFI changes
- [ ] Requires codegen changes
- [ ] Requires SDK updates (which ones?)
- [ ] Breaking change
```

### Task
```markdown
---
name: Task
about: Implementation task (usually created from a feature sub-issue)
labels: type:feature
---

## Parent Issue
<!-- Link to feature sub-issue -->

## Description
<!-- What needs to be done -->

## Acceptance Criteria
- [ ] Criterion 1
- [ ] Criterion 2

## Technical Notes
<!-- Implementation guidance, relevant files, patterns to follow -->

## Testing
<!-- How to verify this is done correctly -->
```

### RFC (Request for Comments)
```markdown
---
name: RFC
about: Propose a significant design decision
labels: status:needs-design
---

## Summary
<!-- One paragraph explanation -->

## Motivation
<!-- Why are we doing this? -->

## Design
<!-- Detailed design. Include code examples. -->

## Alternatives
<!-- What else was considered? -->

## Impact
<!-- What does this change? What breaks? -->

## Open Questions
<!-- What isn't decided yet? -->
```

---

## Existing Issue Triage

Map existing open issues to new structure:

| Existing # | Title | Map To | Action |
|-----------|-------|--------|--------|
| #7 | Interop Sprite Batching | F09-03 | Close, superseded |
| #32 | Font support | F05-01 | Update with design spec |
| #55 | FFI Memory Safety | F01-06, F12-01 | Update, split across features |
| #56 | Panic Prevention | F12-01 | Update, merge into error handling |
| #57 | Sprite Batching | F09-03 | Update with current findings |
| #58 | ECS Optimization | F10-01 through F10-10 | Update, split |
| #59 | Separate Engine/Game Logic | F02-01 | Update, superseded by provider pattern |
| #60 | Window Management | F09-20 | Update with resize/fullscreen spec |
| #61 | Testing Strategy | F22-01 through F22-08 | Update, split |
| #62 | Debug/Profiling Tools | F21-01 through F21-08 | Update, split |
| #63 | Error Messages | F12-01 through F12-06 | Update, split |
| #65 | Advanced Rendering | F09-04 through F09-18 | Update, split |
| #66 | Audio System | F04-01 through F04-09 | Update, split |
| #111 | WASM isKeyJustPressed | F13-01 | Keep, add to Phase 4 |
| #112 | WASM recursive borrow | F13-02 | Keep, add to Phase 4 |
| #113 | TS SDK feedback | F13-03 | Keep, add to Phase 4 |

---

## Execution Plan

When this plan is approved:

1. **Create label taxonomy** in GitHub (all labels above)
2. **Create milestones** (alpha-phase-0 through alpha-phase-6)
3. **Create issue templates** (.github/ISSUE_TEMPLATE/)
4. **Create CONTRIBUTING.md**
5. **Create master issue** ALPHA-001
6. **Create feature sub-issues** (F00 through F25) linking to master
7. **Create task issues** under each feature sub-issue
8. **Triage existing issues** — update, close, or link to new structure
9. **Restructure project board** with new views and fields
10. **Update README** with link to alpha roadmap

All issue creation done via `gh` CLI with agent teams working in parallel batches.

---

## Success Criteria (Alpha Complete)

- [ ] A complete 2D game (platformer with physics, audio, text, animation, UI, scenes) can be built in Rust, C#, Python, TypeScript, C++, Lua, Swift, Kotlin, and Go
- [ ] The same game deploys to Windows, macOS, Linux, Web, iOS, and Android
- [ ] Console porting guide exists with C/C++ SDK ready for NDA partners
- [ ] 80%+ test coverage across engine and all SDKs
- [ ] Docs site is live with getting-started guides for every language
- [ ] Performance benchmarks show competitive frame rates with Bevy/Godot for equivalent workloads
- [ ] Zero known critical bugs
- [ ] At least 3 example games demonstrating different genres
- [ ] Provider system allows swapping renderer, physics, audio at config time
- [ ] All SDKs pass parity tests (same Flappy Bird in every language)
- [ ] Multiplayer networking works (client-server and peer-to-peer, including web via WebSocket/WebRTC)
- [ ] At least one multiplayer example game demonstrating networking

---

## Technical Design: Provider Abstraction Layer

### Core Philosophy

Every major engine subsystem is accessed through a provider trait. The engine ships with default providers but any can be replaced. Providers are registered at engine initialization and cannot be changed at runtime (except in dev mode with hot-swap). This is the "bring your own tools" philosophy applied to engine internals.

### Provider Registry

```rust
// Engine initialization with provider selection
let engine = GoudEngine::builder()
    .with_renderer(WgpuRenderer::new(WgpuConfig {
        backend: wgpu::Backends::PRIMARY,  // Auto-select best
        power_preference: wgpu::PowerPreference::HighPerformance,
        present_mode: wgpu::PresentMode::AutoVsync,
    }))
    .with_physics(RapierPhysics2D::new(PhysicsConfig {
        gravity: Vec2::new(0.0, -980.0),
        timestep: 1.0 / 60.0,
        solver_iterations: 8,
    }))
    .with_audio(RodioAudio::new(AudioConfig {
        sample_rate: 44100,
        channels: 2,
    }))
    .with_window(WinitWindow::new(WindowConfig {
        title: "My Game",
        width: 1280,
        height: 720,
        resizable: true,
        fullscreen: false,
    }))
    .build()?;
```

### RenderProvider Trait

```rust
/// Core rendering abstraction. Implementations wrap a GPU backend.
/// The engine never calls gl::, wgpu::, or any graphics API directly —
/// everything goes through this trait.
pub trait RenderProvider: Send + Sync + 'static {
    /// Capabilities this provider supports (queried by engine to adapt behavior)
    fn capabilities(&self) -> RenderCapabilities;

    // --- Lifecycle ---
    fn begin_frame(&mut self) -> Result<FrameContext, RenderError>;
    fn end_frame(&mut self, frame: FrameContext) -> Result<(), RenderError>;
    fn resize(&mut self, width: u32, height: u32) -> Result<(), RenderError>;

    // --- Resources ---
    fn create_buffer(&mut self, desc: &BufferDescriptor) -> Result<BufferHandle, RenderError>;
    fn create_texture(&mut self, desc: &TextureDescriptor) -> Result<TextureHandle, RenderError>;
    fn create_shader(&mut self, desc: &ShaderDescriptor) -> Result<ShaderHandle, RenderError>;
    fn create_render_target(&mut self, desc: &RenderTargetDescriptor) -> Result<RenderTargetHandle, RenderError>;
    fn create_pipeline(&mut self, desc: &PipelineDescriptor) -> Result<PipelineHandle, RenderError>;
    fn destroy_buffer(&mut self, handle: BufferHandle);
    fn destroy_texture(&mut self, handle: TextureHandle);
    fn destroy_shader(&mut self, handle: ShaderHandle);

    // --- Drawing ---
    fn draw(&mut self, cmd: &DrawCommand) -> Result<(), RenderError>;
    fn draw_batch(&mut self, batch: &SpriteBatch) -> Result<(), RenderError>;
    fn draw_mesh(&mut self, mesh: MeshHandle, transform: &Mat4, material: &MaterialHandle) -> Result<(), RenderError>;
    fn draw_text(&mut self, text: &TextDrawCommand) -> Result<(), RenderError>;
    fn draw_particles(&mut self, emitter: &ParticleEmitter) -> Result<(), RenderError>;

    // --- State ---
    fn set_viewport(&mut self, x: u32, y: u32, w: u32, h: u32);
    fn set_camera(&mut self, view: &Mat4, projection: &Mat4);
    fn set_render_target(&mut self, target: Option<RenderTargetHandle>);
    fn clear(&mut self, color: Color, depth: bool);
}

/// What this renderer can do — engine adapts behavior based on this
pub struct RenderCapabilities {
    pub max_texture_size: u32,
    pub max_textures_per_batch: u32,
    pub supports_compute: bool,
    pub supports_msaa: bool,
    pub max_msaa_samples: u32,
    pub supports_render_targets: bool,
    pub supports_instancing: bool,
    pub shader_model: ShaderModel,
    pub backend_name: String,  // "wgpu/Vulkan", "wgpu/Metal", "OpenGL 3.3", etc.
}
```

### Built-in Render Providers

| Provider | Crate | Platforms | Use Case |
|----------|-------|-----------|----------|
| `WgpuRenderer` | wgpu 28 | Desktop, Mobile, Web | Primary — use this |
| `OpenGLRenderer` | gl 0.14 | Desktop only | Legacy/debug fallback |
| `NullRenderer` | none | All | Headless testing, server-side |
| `ConsoleRenderer` | (user-provided) | Console | NDA partners implement this |

### PhysicsProvider Trait

```rust
/// Physics simulation abstraction. Wraps a physics engine.
pub trait PhysicsProvider: Send + Sync + 'static {
    fn capabilities(&self) -> PhysicsCapabilities;

    // --- World ---
    fn step(&mut self, dt: f32);
    fn set_gravity(&mut self, gravity: Vec2);
    fn gravity(&self) -> Vec2;

    // --- Bodies ---
    fn create_body(&mut self, desc: &RigidBodyDescriptor) -> Result<PhysicsBodyHandle, PhysicsError>;
    fn destroy_body(&mut self, handle: PhysicsBodyHandle);
    fn set_position(&mut self, handle: PhysicsBodyHandle, pos: Vec2);
    fn get_position(&self, handle: PhysicsBodyHandle) -> Vec2;
    fn set_velocity(&mut self, handle: PhysicsBodyHandle, vel: Vec2);
    fn get_velocity(&self, handle: PhysicsBodyHandle) -> Vec2;
    fn apply_force(&mut self, handle: PhysicsBodyHandle, force: Vec2);
    fn apply_impulse(&mut self, handle: PhysicsBodyHandle, impulse: Vec2);

    // --- Colliders ---
    fn create_collider(&mut self, body: PhysicsBodyHandle, desc: &ColliderDescriptor) -> Result<ColliderHandle, PhysicsError>;
    fn destroy_collider(&mut self, handle: ColliderHandle);

    // --- Queries ---
    fn raycast(&self, origin: Vec2, direction: Vec2, max_dist: f32, filter: CollisionFilter) -> Option<RaycastHit>;
    fn overlap_circle(&self, center: Vec2, radius: f32, filter: CollisionFilter) -> Vec<PhysicsBodyHandle>;

    // --- Events ---
    fn collision_events(&self) -> &[CollisionEvent];  // Drain after reading
    fn contact_pairs(&self) -> &[ContactPair];

    // --- Constraints ---
    fn create_joint(&mut self, desc: &JointDescriptor) -> Result<JointHandle, PhysicsError>;
    fn destroy_joint(&mut self, handle: JointHandle);

    // --- Debug ---
    fn debug_shapes(&self) -> Vec<DebugShape>;  // For debug draw overlay
}

pub struct PhysicsCapabilities {
    pub supports_3d: bool,
    pub supports_ccd: bool,
    pub supports_joints: bool,
    pub supports_sleeping: bool,
    pub max_bodies: usize,
    pub backend_name: String,  // "rapier2d", "rapier3d", "built-in", etc.
}
```

### Built-in Physics Providers

| Provider | Crate | Features | Use Case |
|----------|-------|----------|----------|
| `RapierPhysics2D` | rapier2d | Full 2D physics | Default for 2D games |
| `RapierPhysics3D` | rapier3d | Full 3D physics | Default for 3D games |
| `SimplePhysics` | built-in | AABB collision + gravity only | Lightweight games, no rapier dependency |
| `NullPhysics` | none | No physics | Games that handle physics manually |

### AudioProvider Trait

```rust
/// Audio playback abstraction.
pub trait AudioProvider: Send + Sync + 'static {
    fn capabilities(&self) -> AudioCapabilities;

    // --- Playback ---
    fn play(&mut self, data: &AudioData, settings: PlaybackSettings) -> Result<AudioHandle, AudioError>;
    fn stop(&mut self, handle: AudioHandle);
    fn pause(&mut self, handle: AudioHandle);
    fn resume(&mut self, handle: AudioHandle);
    fn is_playing(&self, handle: AudioHandle) -> bool;

    // --- Volume ---
    fn set_volume(&mut self, handle: AudioHandle, volume: f32);
    fn set_global_volume(&mut self, volume: f32);
    fn set_channel_volume(&mut self, channel: AudioChannel, volume: f32);

    // --- Spatial ---
    fn set_listener_position(&mut self, pos: Vec3);
    fn set_source_position(&mut self, handle: AudioHandle, pos: Vec3);

    // --- Lifecycle ---
    fn update(&mut self);  // Called each frame to clean up finished sounds
}

pub struct AudioCapabilities {
    pub supports_spatial: bool,
    pub supports_streaming: bool,
    pub max_concurrent_sounds: usize,
    pub supported_formats: Vec<AudioFormat>,
    pub backend_name: String,
}
```

### Built-in Audio Providers

| Provider | Crate | Platforms | Use Case |
|----------|-------|-----------|----------|
| `RodioAudio` | rodio 0.17 | Desktop, Mobile | Primary native audio |
| `WebAudio` | web-sys | Browser/WASM | Web games |
| `NullAudio` | none | All | Headless testing, no audio needed |

### Provider Injection via FFI

SDKs configure providers through the engine config at init time. The FFI exposes provider selection as enum values:

```rust
#[repr(C)]
pub enum GoudRendererType {
    WgpuAuto = 0,       // Auto-select best wgpu backend
    WgpuVulkan = 1,
    WgpuMetal = 2,
    WgpuDx12 = 3,
    WgpuWebGpu = 4,
    OpenGL = 10,
    Null = 99,
}

#[repr(C)]
pub enum GoudPhysicsType {
    Rapier2D = 0,
    Rapier3D = 1,
    Simple = 2,
    None = 99,
}

#[no_mangle]
pub extern "C" fn goud_engine_create(config: *const GoudEngineConfig) -> *mut GoudEngine {
    // config includes renderer_type, physics_type, audio_type
    // Engine builder selects providers based on config
}
```

Custom providers (user-injected) are only available through the Rust SDK, where users can implement the traits directly. FFI SDKs get the built-in providers only.

---

## Technical Design: SDK Binding Approaches

### Binding Architecture Overview

```
                    ┌──────────────────────────────┐
                    │   goud_sdk.schema.json        │  ← Single source of truth
                    │   (types, methods, enums)      │
                    └──────────┬───────────────────┘
                               │
              ┌────────────────┼────────────────────┐
              ▼                ▼                     ▼
    ┌─────────────┐  ┌──────────────┐     ┌──────────────┐
    │ ffi_mapping  │  │ codegen/     │     │ C header     │
    │    .json     │  │ gen_*.py     │     │ (cbindgen)   │
    └──────┬──────┘  └──────┬───────┘     └──────┬───────┘
           │                │                     │
    ┌──────┴──────┐  ┌──────┴───────┐     ┌──────┴───────┐
    │ Managed SDKs │  │ Codegen SDKs │     │ C-header SDKs│
    │ (runtime)    │  │ (generated)  │     │ (native FFI) │
    └──────────────┘  └──────────────┘     └──────────────┘

    C# (P/Invoke)     TypeScript (napi)     C/C++ (direct)
    Python (ctypes)    TypeScript (WASM)     Swift (bridging)
                       Kotlin (JNI)          Go (cgo)
                       Lua (mlua embed)
```

### SDK-Specific Technical Approaches

#### C/C++ SDK (F14) — Native Header

**FFI Mechanism**: Direct C ABI — no wrapper layer. F14-01 makes `cargo build` generate and package `goud_engine.h`.

**Approach**:
1. Clean up cbindgen output (group functions, add doc comments, sort by subsystem)
2. Create `goud_engine.hpp` C++ wrapper with RAII classes:
   - `goud::Game` wraps `GoudEngine*` with constructor/destructor
   - `goud::Entity` wraps entity handle
   - `goud::Texture` wraps texture handle
   - Smart pointers for automatic cleanup
3. Ship as static library (`.a` / `.lib`) + header
4. CMakeLists.txt for find_package() integration
5. vcpkg port and Conan recipe

**Build**: `cargo build --release` produces the native library for the target plus `codegen/generated/goud_engine.h`

**Codegen**: Extend `codegen/gen_cpp.py` to generate C++ wrapper from schema (similar to gen_csharp.py but emitting C++ classes).

**Key design**: C++ SDK is the console escape hatch. It must be the cleanest, most portable SDK.

```cpp
// User-facing C++ API
#include <goud_engine.hpp>

int main() {
    auto game = goud::Game({
        .title = "My Game",
        .width = 1280,
        .height = 720,
        .renderer = goud::Renderer::WgpuAuto,
        .physics = goud::Physics::Rapier2D,
    });

    auto player = game.spawn_entity();
    player.add<goud::Transform2D>({.x = 100, .y = 200});
    player.add<goud::Sprite>({.texture = game.load_texture("player.png")});

    game.run([&](float dt) {
        // game loop
    });
}
```

#### Lua SDK (F15) — Embedded VM

**FFI Mechanism**: No FFI. Lua VM runs inside the Rust process via `mlua` crate.

**Approach**:
1. `mlua` (v0.10+) with `luajit` or `lua54` feature
2. Register engine API as Lua global table: `goud.create_game()`, `goud.spawn_entity()`, etc.
3. Lua scripts loaded from asset system (hot-reloadable)
4. Type conversion handled by mlua (Lua table ↔ Rust struct)
5. Error handling: Lua errors captured and surfaced to engine error system

**Codegen**: New `codegen/gen_lua.py` generates Lua type stubs (for IDE autocomplete) and Rust-side mlua registration code from schema.

**Key design**: Lua is unique because it runs everywhere the engine runs — no platform restrictions. It's the scripting layer for rapid iteration.

```lua
-- User-facing Lua API
local game = goud.create_game({
    title = "My Game",
    width = 1280,
    height = 720,
})

local player = game:spawn_entity()
player:add_transform2d({ x = 100, y = 200 })
player:add_sprite({ texture = game:load_texture("player.png") })

game:run(function(dt)
    -- game loop
end)
```

**Hot-reload**: File watcher detects `.lua` changes → Lua VM reloads script → game state preserved. No engine restart needed. This is a killer DX feature.

#### Swift SDK (F16) — Bridging Header

**FFI Mechanism**: Swift C interop via bridging header.

**Approach**:
1. Use cbindgen output (`goud_engine.h`) as Swift bridging header
2. Create Swift wrapper package with idiomatic Swift API
3. Swift Package Manager (SPM) distribution
4. Compile Rust as static lib (`.a`) for iOS/macOS, link via Xcode

**Codegen**: New `codegen/gen_swift.py` generates Swift classes from schema. Alternatively, explore UniFFI (Mozilla) which generates Swift bindings from Rust proc-macros.

**Key design**: Swift SDK exists primarily for iOS-native development. macOS is secondary.

```swift
// User-facing Swift API
import GoudEngine

let game = GoudGame(config: GameConfig(
    title: "My Game",
    width: 1280,
    height: 720
))

let player = game.spawnEntity()
player.addTransform2D(Transform2D(x: 100, y: 200))
player.addSprite(Sprite(texture: game.loadTexture("player.png")))

game.run { dt in
    // game loop
}
```

#### Kotlin/Java SDK (F17) — JNI

**FFI Mechanism**: JNI via the `jni` crate. Alternative: UniFFI Kotlin generation.

**Approach**:
1. Rust-side JNI glue using `jni` crate (maps Java method calls to Rust FFI functions)
2. Kotlin wrapper classes with idiomatic API (suspending functions for async, data classes for types)
3. Gradle build plugin with cargo-ndk for Android cross-compilation
4. AAR package for Android, JAR for desktop JVM

**Codegen**: New `codegen/gen_kotlin.py` generates Kotlin classes and JNI Rust glue from schema.

**Key design**: Kotlin SDK exists primarily for Android-native development. JVM desktop is secondary.

```kotlin
// User-facing Kotlin API
import com.goudengine.GoudGame

fun main() {
    val game = GoudGame(GameConfig(
        title = "My Game",
        width = 1280,
        height = 720
    ))

    val player = game.spawnEntity()
    player.addTransform2D(Transform2D(x = 100f, y = 200f))
    player.addSprite(Sprite(texture = game.loadTexture("player.png")))

    game.run { dt ->
        // game loop
    }
}
```

#### Go SDK (F18) — cgo

**FFI Mechanism**: cgo calling C functions from the cbindgen header.

**Approach**:
1. Use cbindgen header directly via cgo `// #include "goud_engine.h"` directives
2. Go wrapper package with idiomatic Go API (error returns, interfaces)
3. Go module distribution
4. Static linking of Rust library

**Codegen**: New `codegen/gen_go.py` generates Go wrapper from schema.

**Key design**: Go SDK is lowest priority for game development but useful for game server tooling and procedural generation.

```go
// User-facing Go API
package main

import "github.com/aram-devdocs/goudengine-go"

func main() {
    game, err := goud.NewGame(goud.GameConfig{
        Title:  "My Game",
        Width:  1280,
        Height: 720,
    })
    if err != nil { panic(err) }
    defer game.Close()

    player := game.SpawnEntity()
    player.AddTransform2D(goud.Transform2D{X: 100, Y: 200})
    player.AddSprite(goud.Sprite{Texture: game.LoadTexture("player.png")})

    game.Run(func(dt float32) {
        // game loop
    })
}
```

### UniFFI Consideration

Mozilla's UniFFI generates bindings for Swift, Kotlin, Python, Ruby, and (third-party) C# from a single Rust interface definition. This could replace per-language codegen for some SDKs. However:

**Pros**: Less codegen maintenance, proven by Firefox mobile
**Cons**: Another dependency, less control over API shape, doesn't cover TypeScript/Go/Lua/C++

**Recommendation**: Evaluate UniFFI for Swift and Kotlin SDKs specifically. Keep custom codegen for C#, Python, TypeScript (already mature) and C++ (needs raw C header). Lua is embedded (no FFI needed).

---

## Technical Design: Rendering Architecture

### Current State (Problems)

The engine currently has two disconnected rendering paths:
1. **Native (OpenGL + GLFW)**: Used by C#, Python, Node.js SDKs. Raw `gl::` calls leak into FFI layer. Sprite batch is a non-functional stub. 3D shaders hardcoded.
2. **Web (wgpu + winit)**: Used by TypeScript WASM SDK. Feature-gated. `wgpu_backend.rs` is incomplete.

Neither path is complete. The FFI renderer (`ffi/renderer.rs`) calls `gl::Viewport()` directly, bypassing the backend abstraction.

### Target Architecture

```
                        ┌─────────────────────────────────┐
                        │         Game / SDK Layer          │
                        │  (drawSprite, drawText, drawMesh) │
                        └──────────────┬──────────────────┘
                                       │
                        ┌──────────────┴──────────────────┐
                        │      Render Graph / Command Queue │
                        │  (records draw commands per frame) │
                        └──────────────┬──────────────────┘
                                       │
                    ┌──────────────────┼──────────────────┐
                    │                  │                   │
            ┌───────┴──────┐  ┌───────┴──────┐   ┌───────┴──────┐
            │ 2D Renderer   │  │ 3D Renderer   │   │ UI Renderer   │
            │ (SpriteBatch) │  │ (MeshBatch)   │   │ (Immediate)   │
            └───────┬──────┘  └───────┬──────┘   └───────┬──────┘
                    │                  │                   │
                    └──────────────────┼──────────────────┘
                                       │
                        ┌──────────────┴──────────────────┐
                        │      RenderProvider Trait         │
                        │  (buffers, textures, shaders,    │
                        │   pipelines, draw calls)          │
                        └──────────────┬──────────────────┘
                                       │
              ┌────────────────────────┼────────────────────────┐
              │                        │                         │
     ┌────────┴────────┐    ┌─────────┴─────────┐    ┌─────────┴─────────┐
     │  WgpuRenderer    │    │  OpenGLRenderer    │    │  NullRenderer      │
     │  (Vulkan/Metal/  │    │  (GL 3.3, desktop  │    │  (headless test)   │
     │   D3D12/WebGPU)  │    │   debug only)      │    │                    │
     └──────────────────┘    └────────────────────┘    └────────────────────┘
```

### Render Command Queue

Instead of immediate-mode draw calls hitting the GPU, all rendering goes through a command queue that is sorted and batched before submission:

```rust
pub enum RenderCommand {
    SetCamera { view: Mat4, projection: Mat4 },
    SetRenderTarget(Option<RenderTargetHandle>),
    Clear { color: Color, depth: bool },
    DrawSprite {
        texture: TextureHandle,
        src_rect: Rect,
        dst_rect: Rect,
        color: Color,
        z_layer: f32,
        flip_x: bool,
        flip_y: bool,
    },
    DrawMesh {
        mesh: MeshHandle,
        material: MaterialHandle,
        transform: Mat4,
    },
    DrawText {
        text: String,
        font: FontHandle,
        position: Vec2,
        size: f32,
        color: Color,
    },
    DrawParticles {
        emitter: ParticleEmitterHandle,
    },
    DrawDebug {
        shape: DebugShape,
        color: Color,
    },
}
```

**Frame flow**:
1. Game logic emits `RenderCommand`s into a queue
2. Queue is sorted: by render target → by z-layer → by texture (for batching)
3. Sprites with same texture + z-layer are batched into a single draw call
4. Batched commands submitted to `RenderProvider`
5. Provider translates to GPU API calls

### Sprite Batching (Fixed)

The current sprite batch is a stub. The fix:

```rust
pub struct SpriteBatcher {
    vertices: Vec<SpriteVertex>,    // Pre-allocated, reused each frame
    indices: Vec<u32>,              // Pre-allocated, reused each frame
    batches: Vec<BatchEntry>,       // Groups by texture + shader
    vertex_buffer: BufferHandle,    // GPU buffer, resized as needed
    index_buffer: BufferHandle,     // GPU buffer, resized as needed
    default_shader: ShaderHandle,   // Compiled at init, not per-frame
    max_sprites: usize,             // Configurable, default 10000
}

impl SpriteBatcher {
    /// Called each frame after command sorting
    pub fn flush(&mut self, provider: &mut dyn RenderProvider) -> Result<(), RenderError> {
        if self.batches.is_empty() { return Ok(()); }

        // Upload vertices to GPU (single buffer upload)
        provider.update_buffer(self.vertex_buffer, &self.vertices)?;
        provider.update_buffer(self.index_buffer, &self.indices)?;

        // Draw each batch (one draw call per unique texture)
        for batch in &self.batches {
            provider.bind_texture(batch.texture, 0)?;
            provider.draw_indexed(self.vertex_buffer, self.index_buffer,
                                  batch.index_offset, batch.index_count)?;
        }

        self.vertices.clear();
        self.indices.clear();
        self.batches.clear();
        Ok(())
    }
}
```

**Key fixes over current code**:
- Shader compiled once at init (not `NotImplemented`)
- Texture upload works (not `NotImplemented`)
- Pre-allocated buffers reused (no per-frame allocation)
- Z-layer is explicit field (not Y-position hack)
- Draw calls minimized (one per unique texture)

### Material System

```rust
pub struct Material {
    pub shader: ShaderHandle,
    pub textures: HashMap<String, TextureHandle>,  // "albedo", "normal", etc.
    pub uniforms: HashMap<String, UniformValue>,    // "color", "roughness", etc.
    pub blend_mode: BlendMode,
    pub cull_mode: CullMode,
    pub depth_test: bool,
    pub depth_write: bool,
}

pub enum UniformValue {
    Float(f32),
    Vec2(Vec2),
    Vec3(Vec3),
    Vec4(Vec4),
    Mat4(Mat4),
    Color(Color),
    Int(i32),
}
```

### Shader System

Replace hardcoded shader strings with a loadable, hot-reloadable system:

```rust
pub struct ShaderDescriptor {
    pub vertex_source: ShaderSource,
    pub fragment_source: ShaderSource,
    pub vertex_layout: VertexLayout,
    pub defines: HashMap<String, String>,  // Preprocessor defines
}

pub enum ShaderSource {
    Wgsl(String),           // wgpu native shader language
    Glsl { source: String, stage: ShaderStage },  // For OpenGL fallback
    SpirV(Vec<u8>),         // Pre-compiled
    File(PathBuf),          // Load from asset system
}
```

**Shader compilation flow**: WGSL source → wgpu compiles to native (SPIR-V, MSL, HLSL, DXIL) at runtime. For OpenGL fallback, GLSL source is compiled by the GL driver. For shipping, SPIR-V pre-compilation is optional.

### 3D Mesh Pipeline

```rust
pub struct Mesh {
    pub vertices: Vec<Vertex3D>,
    pub indices: Vec<u32>,
    pub submeshes: Vec<SubMesh>,  // Material per submesh
    pub bounds: BoundingBox,
}

pub struct Vertex3D {
    pub position: Vec3,
    pub normal: Vec3,
    pub uv: Vec2,
    pub tangent: Vec4,    // For normal mapping
    pub color: Color,     // Vertex color
    pub bone_indices: [u8; 4],  // For skeletal animation
    pub bone_weights: [f32; 4],
}
```

**Loader**: Use `gltf` crate (v0.16+) for GLTF 2.0 (industry standard). OBJ via `tobj` crate as fallback. GLTF supports materials, animations, and skeletons in one file.

### Post-Processing Pipeline

```
Screen                     RenderTarget A           RenderTarget B
   │                            │                        │
   │  ┌─────────┐              │  ┌─────────┐          │
   │  │ Scene    │──render──▶  │  │ Bloom    │──pass──▶ │
   │  │ Render   │              │  │ Extract  │          │
   │  └─────────┘              │  └─────────┘          │
   │                            │                        │
   │  ┌─────────┐              │  ┌─────────┐          │
   │  │ Final   │◀──compose──  │  │ Bloom    │◀──blur── │
   │  │ Output  │              │  │ Combine  │          │
   │  └─────────┘              │  └─────────┘          │
   ▼                            ▼                        ▼
```

Post-processing as a chain of passes, each reading from one render target and writing to another:

```rust
pub trait PostProcessPass: Send + Sync {
    fn name(&self) -> &str;
    fn execute(&mut self, input: RenderTargetHandle, output: RenderTargetHandle,
               provider: &mut dyn RenderProvider) -> Result<(), RenderError>;
}

// Built-in passes
pub struct BloomPass { pub threshold: f32, pub intensity: f32 }
pub struct ToneMappingPass { pub exposure: f32 }
pub struct FxaaPass;
pub struct ColorGradingPass { pub lut: TextureHandle }
```

### GLFW to winit Migration Plan

**Why**: GLFW doesn't work on iOS, Android, or web. winit covers all platforms.

**Migration steps**:
1. Abstract window operations behind `WindowProvider` trait
2. Implement `WinitWindowProvider` alongside existing `GlfwWindowProvider`
3. Move input handling to `InputProvider` trait (winit delivers events differently than GLFW)
4. Update `dev.sh` and build system to default to winit
5. Feature-gate GLFW behind `legacy-glfw` feature flag
6. Remove GLFW from default features
7. Test all platforms with winit
8. Eventually deprecate GLFW provider

**Risk**: winit's event loop model (run_return vs run) differs from GLFW's poll model. The game loop architecture may need adjustment. winit 0.30 supports `pump_events()` which is closer to GLFW's model.

### Particle System Design

```rust
pub struct ParticleEmitter {
    pub config: ParticleConfig,
    pub particles: Vec<Particle>,  // Object pool, pre-allocated
    pub alive_count: usize,
    pub elapsed: f32,
}

pub struct ParticleConfig {
    pub max_particles: usize,
    pub emission_rate: f32,       // Particles per second
    pub lifetime: RangeF32,       // Min/max lifetime
    pub initial_velocity: RangeVec2,
    pub acceleration: Vec2,       // Gravity, wind
    pub initial_size: RangeF32,
    pub end_size: RangeF32,
    pub initial_color: Color,
    pub end_color: Color,
    pub texture: Option<TextureHandle>,
    pub blend_mode: BlendMode,
    pub world_space: bool,        // Particles stay in world or follow emitter
}

pub struct Particle {
    pub position: Vec2,
    pub velocity: Vec2,
    pub size: f32,
    pub color: Color,
    pub lifetime: f32,
    pub age: f32,
    pub alive: bool,
}
```

Particles rendered as instanced quads — single draw call for all particles in an emitter. GPU particle simulation is a stretch goal (compute shader).

---

## Technical Design: Networking System

### Core Philosophy

Networking follows the same provider abstraction pattern as rendering, physics, and audio. A `NetworkProvider` trait abstracts transport, and the engine ships with built-in providers for common topologies. Games choose their networking model at config time.

### NetworkProvider Trait

```rust
/// Core networking abstraction. Implementations wrap a transport layer.
pub trait NetworkProvider: Send + Sync + 'static {
    fn capabilities(&self) -> NetworkCapabilities;

    // --- Connection ---
    fn host(&mut self, config: &HostConfig) -> Result<(), NetworkError>;
    fn connect(&mut self, addr: &str) -> Result<PeerId, NetworkError>;
    fn disconnect(&mut self, peer: PeerId) -> Result<(), NetworkError>;
    fn disconnect_all(&mut self) -> Result<(), NetworkError>;

    // --- Messaging ---
    fn send(&mut self, peer: PeerId, channel: Channel, data: &[u8]) -> Result<(), NetworkError>;
    fn broadcast(&mut self, channel: Channel, data: &[u8]) -> Result<(), NetworkError>;
    fn recv(&mut self) -> Vec<NetworkEvent>;

    // --- State ---
    fn connected_peers(&self) -> &[PeerId];
    fn local_peer_id(&self) -> Option<PeerId>;
    fn connection_state(&self) -> ConnectionState;
    fn stats(&self) -> NetworkStats;

    // --- Lifecycle ---
    fn update(&mut self);  // Called each frame to process packets
}

pub struct NetworkCapabilities {
    pub supports_reliable: bool,
    pub supports_unreliable: bool,
    pub supports_ordered: bool,
    pub supports_p2p: bool,
    pub supports_web: bool,       // Works in browser/WASM
    pub max_peers: usize,
    pub max_message_size: usize,
    pub backend_name: String,
}

pub enum NetworkEvent {
    PeerConnected(PeerId),
    PeerDisconnected(PeerId, DisconnectReason),
    Message { peer: PeerId, channel: Channel, data: Vec<u8> },
    Error(NetworkError),
}

pub enum Channel {
    Reliable,           // TCP-like: ordered, guaranteed delivery
    UnreliableOrdered,  // Ordered but may drop packets
    Unreliable,         // UDP-like: fire and forget
}
```

### Built-in Network Providers

| Provider | Crate | Transport | Platforms | Use Case |
|----------|-------|-----------|-----------|----------|
| `UdpNetProvider` | std::net / tokio | UDP + reliability layer | Desktop, Mobile | Low-latency multiplayer |
| `WebSocketNetProvider` | tungstenite / tokio-tungstenite | WebSocket (ws/wss) | All (inc. Web) | Web-compatible multiplayer |
| `WebRtcNetProvider` | webrtc-rs / web-sys | WebRTC data channels | All (inc. Web) | P2P web multiplayer |
| `SteamNetProvider` | steamworks-rs | Steam Networking Sockets | Desktop | Steam relay + matchmaking |
| `NullNetProvider` | none | No network | All | Single-player, testing |

### Architecture Layers

```
┌─────────────────────────────────────────┐
│           Game Networking Layer           │
│  (lobby, matchmaking, game-specific RPC) │
└──────────────────┬──────────────────────┘
                   │
┌──────────────────┴──────────────────────┐
│         State Synchronization            │
│  (snapshot, delta, interpolation, rollback)│
└──────────────────┬──────────────────────┘
                   │
┌──────────────────┴──────────────────────┐
│        Serialization / Protocol          │
│  (binary encoding, delta compression)    │
└──────────────────┬──────────────────────┘
                   │
┌──────────────────┴──────────────────────┐
│         NetworkProvider Trait             │
│  (connect, send, recv, disconnect)       │
└──────────────────┬──────────────────────┘
                   │
    ┌──────────────┼──────────────┐
    │              │              │
┌───┴────┐  ┌─────┴─────┐  ┌────┴─────┐
│  UDP   │  │ WebSocket  │  │  WebRTC  │
└────────┘  └───────────┘  └──────────┘
```

### State Synchronization

Two built-in strategies (game chooses at config):

**Snapshot Interpolation** (default for most games):
- Server sends full world snapshots at fixed rate (e.g., 20/sec)
- Client interpolates between two received snapshots
- Simple, robust, works for most game types
- Higher bandwidth but lower complexity

**Rollback Netcode** (for fighting/action games):
- Each client runs simulation locally with predicted inputs
- When authoritative input arrives, rewind and resimulate if prediction was wrong
- Low-latency feel, complex implementation
- Uses `bevy_ggrs` or custom rollback on top of ECS snapshots

```rust
pub enum SyncStrategy {
    SnapshotInterpolation {
        snapshot_rate: u32,       // Snapshots per second
        interpolation_delay: f32, // Seconds of buffering
    },
    Rollback {
        max_rollback_frames: u32,
        input_delay: u32,         // Frames of input delay
    },
    Custom(Box<dyn SyncProvider>),
}
```

### Lobby and Matchmaking

Built-in lobby system that works locally (LAN discovery) or via a relay server:

```rust
pub struct Lobby {
    pub id: LobbyId,
    pub host: PeerId,
    pub players: Vec<PlayerInfo>,
    pub max_players: usize,
    pub state: LobbyState,       // Waiting, Starting, InGame
    pub metadata: HashMap<String, String>,
}

pub trait MatchmakingProvider: Send + Sync + 'static {
    fn create_lobby(&mut self, config: LobbyConfig) -> Result<LobbyId, NetworkError>;
    fn join_lobby(&mut self, id: LobbyId) -> Result<(), NetworkError>;
    fn leave_lobby(&mut self) -> Result<(), NetworkError>;
    fn list_lobbies(&self, filter: LobbyFilter) -> Result<Vec<LobbyInfo>, NetworkError>;
    fn set_ready(&mut self, ready: bool) -> Result<(), NetworkError>;
    fn start_game(&mut self) -> Result<(), NetworkError>;  // Host only
}
```

### Network Simulation (Dev Tools)

For testing multiplayer without deploying:

```rust
pub struct NetworkSimConfig {
    pub latency_ms: u32,          // Added round-trip delay
    pub jitter_ms: u32,           // Random latency variance
    pub packet_loss_percent: f32, // 0.0 to 1.0
    pub duplicate_percent: f32,   // Packet duplication rate
    pub bandwidth_limit_kbps: u32,// Throttle bandwidth
}
```

Applied as a wrapper around any NetworkProvider — invisible to game code.

### FFI Exposure

Networking exposed via FFI following existing patterns:

```rust
#[no_mangle]
pub extern "C" fn goud_network_host(
    engine: *mut GoudEngine,
    port: u16,
    max_peers: u32,
) -> i32 { /* ... */ }

#[no_mangle]
pub extern "C" fn goud_network_connect(
    engine: *mut GoudEngine,
    address: *const c_char,
    port: u16,
) -> i32 { /* ... */ }

#[no_mangle]
pub extern "C" fn goud_network_send(
    engine: *mut GoudEngine,
    peer_id: u64,
    channel: u8,
    data: *const u8,
    data_len: u32,
) -> i32 { /* ... */ }

#[no_mangle]
pub extern "C" fn goud_network_poll_events(
    engine: *mut GoudEngine,
    events: *mut GoudNetworkEvent,
    max_events: u32,
) -> i32 { /* returns number of events written */ }
```

### Crate Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| `tokio` | 1.x | Async runtime for networking |
| `tungstenite` | 0.24+ | WebSocket client/server |
| `webrtc-rs` | 0.11+ | WebRTC for P2P |
| `bincode` | 2.x | Fast binary serialization |
| `lz4_flex` | 0.11+ | Fast compression for snapshots |
| `steamworks` | 0.11+ | Steam networking (optional feature) |
