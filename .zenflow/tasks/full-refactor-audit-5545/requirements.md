# GoudEngine Full Refactor Audit - Product Requirements Document

## Executive Summary

This document details the comprehensive audit findings for GoudEngine, a Rust-based game engine with C# FFI bindings. The audit covers architecture design, implementation quality, developer experience, and scalability concerns. The goal is to transform GoudEngine from a prototype-level engine into a professional, hardened game engine that supports multi-language bindings and can scale to support complex game development.

---

## 1. Current State Assessment

### 1.1 Architecture Overview

**What Exists:**
- Rust core engine (`goud_engine/`) compiled as a C dynamic library (cdylib)
- C# SDK (`sdks/GoudEngine/`) wrapping native functions via P/Invoke
- csbindgen-based automatic binding generation
- 2D and 3D renderer implementations (Renderer2D, Renderer3D)
- Basic ECS-style sprite management system
- GLFW-based window/input handling
- OpenGL 3.3 Core Profile rendering
- 4 example games demonstrating capabilities

**Technology Stack:**
- Rust 2021 Edition with cdylib output
- .NET 8.0 for C# SDK
- OpenGL 3.3 via `gl` crate
- GLFW via `glfw` crate
- csbindgen 1.2.0 for FFI generation
- cgmath for vector/matrix math

### 1.2 Critical Issues Identified

#### 1.2.1 Architecture Issues

| Issue | Severity | Description |
|-------|----------|-------------|
| **Misleading ECS Name** | High | The "ECS" is not a proper Entity-Component-System. It's a specialized sprite container with no components, no systems, no queries. |
| **Single-Threaded Design** | High | Uses `Rc<Texture>` throughout, preventing any multi-threaded access. No thread safety considerations. |
| **Hardcoded Limits** | Medium | 8 lights maximum (hardcoded in shader), no batching, no instancing |
| **No Abstraction Layer** | High | Graphics code directly calls OpenGL. No render backend abstraction for Vulkan/Metal/DX support. |
| **Tight Coupling** | Medium | `GameSdk` struct owns everything directly; no service locator, no dependency injection pattern |
| **No Scene Graph** | Medium | Flat sprite list with z-layers only; no parent-child relationships, no transform hierarchies |

#### 1.2.2 FFI/SDK Issues

| Issue | Severity | Description |
|-------|----------|-------------|
| **Unsafe Lifetime Extension** | Critical | `game_ref_mut()` extends pointer lifetime to `'static`, trusting caller entirely |
| **No Thread Safety** | High | No locks, mutexes, or thread-safe patterns for concurrent access |
| **Primitive Error Handling** | High | Returns `bool` or `null` for errors; no error codes, no error messages across FFI |
| **String Encoding Issues** | Medium | Manual UTF-8 encoding in C#; no helper utilities |
| **No Batch Operations** | High | N sprites = N FFI calls; massive overhead for large scenes |
| **Zero-Value Semantics** | Medium | Uses 0.0 to mean "keep current value" in sprite updates; ambiguous and error-prone |

#### 1.2.3 Graphics Issues

| Issue | Severity | Description |
|-------|----------|-------------|
| **No Draw Call Batching** | High | Each sprite is a separate draw call; will not scale |
| **No Texture Atlas Support** | High | Individual textures only; no sprite sheets with automatic slicing |
| **No Render Graph** | Medium | Fixed rendering pipeline; no customization |
| **Shader Hardcoding** | Medium | Shaders embedded as strings; no external shader loading |
| **No Post-Processing** | Medium | No support for bloom, blur, tone mapping, etc. |
| **Limited Lighting** | Medium | Only 8 lights; no shadow mapping; no global illumination |

#### 1.2.4 Missing Core Features

| Feature | Priority | Description |
|---------|----------|-------------|
| **Audio System** | Critical | No audio whatsoever |
| **Physics System** | Critical | No physics; only basic AABB collision in ECS |
| **Animation System** | High | Only frame-based animation controller in C#; no skeletal, no tweening |
| **UI System** | High | No UI framework; examples build ad-hoc UI |
| **Asset Pipeline** | High | No asset cooking, compression, or caching |
| **Serialization** | High | No save/load; no scene serialization |
| **Scene Management** | Medium | No scene loading/unloading; single scene only |
| **Input Mapping** | Medium | Direct key checks only; no action mapping |
| **Networking** | Low | No multiplayer support |
| **Editor** | Low | No visual editor |

#### 1.2.5 Developer Experience Issues

| Issue | Severity | Description |
|-------|----------|-------------|
| **Inconsistent API** | Medium | Mix of raw uint IDs and type-safe wrappers; no uniform pattern |
| **Too Many Parameters** | Medium | `ConfigureGrid` has 27 parameters; `AddLight` has 14 parameters |
| **No Builder Pattern** | Medium | No fluent API for complex object creation |
| **Poor Documentation** | High | Minimal XML docs; no API reference; no tutorials |
| **Version Churn** | Low | Version 0.0.808 suggests rapid iteration without stability |

---

## 2. Requirements for Professional-Grade Engine

### 2.1 Core Engine Architecture (MUST HAVE)

#### R1: Render Backend Abstraction
**Requirement:** Create an abstract render backend interface that can support multiple graphics APIs.

**Acceptance Criteria:**
- [ ] Define `RenderBackend` trait with methods for buffers, textures, shaders, draw calls
- [ ] Implement OpenGL backend as reference implementation
- [ ] All rendering code uses backend abstraction, never raw GL calls
- [ ] Backend selection at runtime via configuration
- [ ] Design supports future Vulkan/Metal/WebGPU backends

**Rationale:** Current OpenGL-only implementation limits platform support and prevents optimization. A proper abstraction enables future modernization without API breaking changes.

---

#### R2: Proper Entity-Component-System
**Requirement:** Replace the current "ECS" with a true archetype-based ECS or integrate an established Rust ECS library.

**Acceptance Criteria:**
- [ ] Components are pure data structs with no behavior
- [ ] Systems are functions that query and process component data
- [ ] Support for entity hierarchies (parent-child relationships)
- [ ] Component queries with filters (e.g., "all entities with Transform AND Sprite")
- [ ] Parallel system execution capability
- [ ] Entity archetypes for efficient memory layout

**Options:**
1. **Adopt `bevy_ecs`** - Industry-standard, well-maintained, extensive features
2. **Adopt `hecs`** - Lightweight, minimal, good for learning
3. **Custom Implementation** - Full control, more effort

**Recommended:** Adopt `bevy_ecs` standalone (without full Bevy) for proven architecture.

---

#### R3: Scene Graph System
**Requirement:** Implement a hierarchical scene graph with transform inheritance.

**Acceptance Criteria:**
- [ ] Nodes can have parent-child relationships
- [ ] Transform (position, rotation, scale) inherits from parent
- [ ] Efficient world transform calculation with dirty flags
- [ ] Support for node enable/disable (affects children)
- [ ] Serialization/deserialization of scene graphs

---

#### R4: Resource Management System
**Requirement:** Create a centralized resource management system with reference counting, caching, and async loading.

**Acceptance Criteria:**
- [ ] All resources (textures, shaders, audio, meshes) managed centrally
- [ ] Automatic reference counting and garbage collection
- [ ] Support for hot-reloading during development
- [ ] Async loading with progress callbacks
- [ ] Resource handles (not raw pointers) for safe access
- [ ] Resource groups for batch loading/unloading

---

#### R5: Thread-Safe Architecture
**Requirement:** Design the engine for safe multi-threaded access.

**Acceptance Criteria:**
- [ ] Replace `Rc` with `Arc` where needed
- [ ] Define clear thread ownership boundaries
- [ ] Use channels for cross-thread communication
- [ ] Job system for parallel task execution
- [ ] Thread-safe resource handles
- [ ] Render thread separation from game logic thread

---

### 2.2 FFI Layer Hardening (MUST HAVE)

#### R6: Type-Safe C ABI Layer
**Requirement:** Create a robust, safe FFI boundary with proper error handling.

**Acceptance Criteria:**
- [ ] Define clear C ABI contract with versioning
- [ ] Opaque handle types for all engine objects (no raw pointers in API)
- [ ] Error codes enum with human-readable error messages
- [ ] Thread-safety documentation for all FFI functions
- [ ] Validation on all inputs (null checks, range checks)
- [ ] Batch operation support to reduce FFI call overhead

**Example API:**
```rust
// Current (bad)
pub extern "C" fn game_add_sprite(game: *mut GameSdk, dto: SpriteCreateDto) -> u32;

// Proposed (good)
pub extern "C" fn goud_sprite_create(
    ctx: GoudContext,           // Opaque handle
    desc: *const GoudSpriteDesc, // Validated descriptor
    out_handle: *mut GoudSprite, // Output handle
) -> GoudResult;                // Error code
```

---

#### R7: Language-Agnostic Binding Generator
**Requirement:** Design FFI layer to support multiple language bindings beyond C#.

**Acceptance Criteria:**
- [ ] C header generation via `cbindgen` (already partial)
- [ ] Document binding conventions for each language
- [ ] Provide binding generator configuration for: C#, Python, TypeScript/Node, Lua, Go
- [ ] Example bindings in at least 3 languages
- [ ] Versioned API with backward compatibility policy

---

#### R8: Batch Operations API
**Requirement:** Add batch operations to reduce FFI overhead.

**Acceptance Criteria:**
- [ ] `goud_sprites_create_batch(ctx, descs[], count, out_handles[])`
- [ ] `goud_sprites_update_batch(ctx, updates[], count)`
- [ ] `goud_sprites_destroy_batch(ctx, handles[], count)`
- [ ] Measure and document performance improvement vs single calls

---

### 2.3 Graphics System (MUST HAVE)

#### R9: Draw Call Batching
**Requirement:** Implement automatic draw call batching for sprites.

**Acceptance Criteria:**
- [ ] Sprites with same texture batched into single draw call
- [ ] Texture atlas support for combining multiple textures
- [ ] Instance rendering for identical sprites
- [ ] Measurable: <100 draw calls for 10,000 sprites with same texture

---

#### R10: Shader System Overhaul
**Requirement:** Create a flexible shader management system.

**Acceptance Criteria:**
- [ ] External shader file loading (not embedded strings)
- [ ] Shader hot-reloading in development mode
- [ ] Shader reflection for automatic uniform binding
- [ ] Material system (shader + parameters)
- [ ] Shader preprocessor with #include support

---

#### R11: Camera System
**Requirement:** Unified camera system supporting 2D and 3D modes.

**Acceptance Criteria:**
- [ ] Base Camera class with common functionality
- [ ] Orthographic camera for 2D
- [ ] Perspective camera for 3D
- [ ] Camera effects (shake, follow, smooth follow)
- [ ] Multiple viewports/cameras rendering

---

### 2.4 Missing Core Systems (HIGH PRIORITY)

#### R12: Audio System
**Requirement:** Implement a complete audio system.

**Acceptance Criteria:**
- [ ] Support for WAV, OGG, MP3 formats
- [ ] Sound effects (one-shot sounds)
- [ ] Background music (looping, crossfade)
- [ ] 3D spatial audio positioning
- [ ] Volume control, muting, audio groups
- [ ] Low latency playback (<50ms)

**Recommended Backend:** `rodio` or `kira` for Rust audio

---

#### R13: Physics System
**Requirement:** Integrate a physics engine for 2D and 3D.

**Acceptance Criteria:**
- [ ] Rigid body dynamics
- [ ] Collision detection and response
- [ ] Multiple collider shapes (box, circle/sphere, polygon/mesh)
- [ ] Triggers (non-physical collision events)
- [ ] Physics materials (friction, bounciness)
- [ ] Ray casting

**Recommended Backend:** `rapier` (Rust-native, 2D and 3D)

---

#### R14: Animation System
**Requirement:** Create a comprehensive animation system.

**Acceptance Criteria:**
- [ ] Sprite frame animation (current, enhanced)
- [ ] Skeletal animation for 2D and 3D
- [ ] Tweening/interpolation system
- [ ] Animation state machines
- [ ] Animation blending
- [ ] Event callbacks at animation frames

---

#### R15: Input System
**Requirement:** Create an abstracted input system with action mapping.

**Acceptance Criteria:**
- [ ] Input actions (abstract: "Jump", "Fire") mapped to keys
- [ ] Multiple bindings per action
- [ ] Gamepad support (already stubbed)
- [ ] Input buffering for frame-perfect inputs
- [ ] Input recording/playback for testing
- [ ] Touch input support

---

#### R16: Asset Pipeline
**Requirement:** Create a build-time asset processing pipeline.

**Acceptance Criteria:**
- [ ] Asset cooking (convert source formats to runtime formats)
- [ ] Texture compression (DXT, ASTC, etc.)
- [ ] Asset bundling (pak files)
- [ ] Dependency tracking and incremental builds
- [ ] Asset metadata (.meta files)
- [ ] Command-line asset processor tool

---

### 2.5 Developer Experience (MEDIUM PRIORITY)

#### R17: Builder Pattern APIs
**Requirement:** Replace long parameter lists with builder patterns.

**Acceptance Criteria:**
- [ ] `SpriteBuilder::new().position(x, y).texture(id).build()`
- [ ] `LightBuilder::new().point().position(x, y, z).color(r, g, b).build()`
- [ ] `GridConfig::builder().size(20.0).divisions(20).build()`
- [ ] All builders validate parameters and return `Result`

---

#### R18: Comprehensive Documentation
**Requirement:** Create professional-level documentation.

**Acceptance Criteria:**
- [ ] API reference generated from doc comments
- [ ] Getting started tutorial
- [ ] Architecture overview document
- [ ] Migration guide for breaking changes
- [ ] Code examples for all major features
- [ ] Inline examples in doc comments

---

#### R19: Testing Infrastructure
**Requirement:** Comprehensive TDD testing strategy.

**Acceptance Criteria:**
- [ ] Unit tests for all core modules (>80% coverage)
- [ ] Integration tests for FFI boundary
- [ ] Rendering tests with headless context or mocking
- [ ] Performance benchmarks with regression tracking
- [ ] Fuzz testing for FFI inputs
- [ ] CI/CD pipeline runs all tests on every PR

---

#### R20: Debug and Profiling Tools
**Requirement:** Built-in debugging and profiling capabilities.

**Acceptance Criteria:**
- [ ] Frame profiler (CPU time per system)
- [ ] GPU profiler (draw call timing)
- [ ] Memory usage tracking
- [ ] Debug overlays (FPS, entity count, draw calls)
- [ ] Logging with categories and log levels
- [ ] Integration with external profilers (Tracy, RenderDoc)

---

### 2.6 Scalability and Extensibility (MEDIUM PRIORITY)

#### R21: Plugin System
**Requirement:** Create a plugin architecture for extensibility.

**Acceptance Criteria:**
- [ ] Plugin interface for loading custom systems
- [ ] Plugin lifecycle (init, update, shutdown)
- [ ] Plugin dependencies and load order
- [ ] Safe plugin sandboxing

---

#### R22: Modular Subsystems
**Requirement:** Allow subsystems to be replaced or disabled.

**Acceptance Criteria:**
- [ ] Physics, audio, rendering are swappable modules
- [ ] Feature flags for compile-time module inclusion
- [ ] Dependency injection for subsystem access
- [ ] Clean interfaces between subsystems

---

## 3. Implementation Phases

### Phase 1: Foundation Hardening (Weeks 1-4)
**Goal:** Fix critical architectural issues without breaking existing functionality.

**Tasks:**
1. Create render backend abstraction
2. Design and implement new FFI layer with error handling
3. Replace Rc with Arc for thread safety
4. Add comprehensive unit tests for existing code
5. Implement resource handle system

**Deliverables:**
- Render backend trait + OpenGL implementation
- New FFI contract with error codes
- Thread-safe resource management
- 80%+ test coverage on core modules

---

### Phase 2: Core Systems (Weeks 5-10)
**Goal:** Implement missing critical systems.

**Tasks:**
1. Integrate `bevy_ecs` or implement proper ECS
2. Implement scene graph system
3. Integrate `rapier` for physics
4. Integrate `rodio` or `kira` for audio
5. Create asset pipeline tooling

**Deliverables:**
- Full ECS with component queries
- Hierarchical scene graph
- 2D/3D physics with collisions
- Audio playback (effects + music)
- Asset cooking CLI tool

---

### Phase 3: Graphics Enhancement (Weeks 11-14)
**Goal:** Bring graphics system to production quality.

**Tasks:**
1. Implement draw call batching
2. Add texture atlas support
3. Overhaul shader system
4. Add basic post-processing
5. Implement camera system enhancements

**Deliverables:**
- Batched sprite rendering (<100 draws for 10K sprites)
- Automatic texture atlasing
- External shader loading
- Bloom, blur post-processing
- Camera effects (shake, follow)

---

### Phase 4: Developer Experience (Weeks 15-18)
**Goal:** Make the engine pleasant to use.

**Tasks:**
1. Implement builder pattern APIs
2. Write comprehensive documentation
3. Create example games showcasing features
4. Add debug overlays and profiling
5. Multi-language binding examples

**Deliverables:**
- Fluent API for all major types
- Full API documentation site
- 3+ polished example games
- Built-in profiler
- Python and Lua binding examples

---

### Phase 5: Polish and Optimization (Weeks 19-22)
**Goal:** Production-ready release.

**Tasks:**
1. Performance optimization pass
2. Memory optimization pass
3. API stability review
4. Backward compatibility testing
5. Release documentation

**Deliverables:**
- Performance benchmarks passing targets
- Memory leak testing clean
- API frozen for 1.0
- Migration guide
- Release candidate

---

## 4. Success Metrics

### Performance Targets
| Metric | Target |
|--------|--------|
| Sprites rendered | 100,000 @ 60fps |
| Draw calls for 10K sprites (same texture) | <100 |
| Physics bodies simulated | 10,000 @ 60fps |
| Audio latency | <50ms |
| Asset load time (100MB pak) | <2 seconds |
| FFI batch operation speedup | 10x vs individual calls |

### Quality Targets
| Metric | Target |
|--------|--------|
| Test coverage | >80% |
| Documentation coverage | 100% public API |
| CI build time | <10 minutes |
| Zero known memory leaks | Pass |
| Zero known thread safety issues | Pass |

### Developer Experience Targets
| Metric | Target |
|--------|--------|
| Time to hello world | <5 minutes |
| API breaking changes per release | 0 for minor versions |
| Examples compile and run | 100% |
| Binding generation success | 100% for all target languages |

---

## 5. Risks and Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| bevy_ecs API changes | High | Pin to specific version, abstract usage |
| OpenGL deprecation on macOS | High | Render backend abstraction allows Metal backend |
| FFI complexity | Medium | Comprehensive testing, binding generator automation |
| Scope creep | High | Strict phase boundaries, MVP per phase |
| Performance regression | Medium | Automated benchmarks in CI |

---

## 6. Non-Requirements (Out of Scope for Initial Refactor)

The following are explicitly NOT in scope for this refactor:
- Visual editor/IDE integration
- Networking/multiplayer
- VR/AR support
- Console platform support
- Mobile platform support (iOS/Android)
- Scripting language runtime (Lua VM, etc.)

These may be considered for future phases after the foundation is solid.

---

## 7. Appendix: Current File Structure Reference

```
goud_engine/
├── src/
│   ├── lib.rs                    # Crate root
│   ├── game.rs                   # GameSdk main struct
│   ├── types.rs                  # FFI types
│   ├── sdk.rs                    # FFI exports
│   ├── ffi_privates.rs           # Opaque types
│   └── libs/
│       ├── ecs/mod.rs            # Sprite manager (misnamed)
│       ├── graphics/
│       │   ├── renderer.rs       # Renderer enum
│       │   ├── renderer2d.rs     # 2D renderer
│       │   ├── renderer3d.rs     # 3D renderer
│       │   └── components/       # Shared graphics primitives
│       ├── platform/
│       │   └── window/           # GLFW window
│       └── logger/               # Logging
└── Cargo.toml

sdks/GoudEngine/
├── GoudGame.cs                   # Main C# wrapper
├── NativeMethods.g.cs            # Auto-generated P/Invoke
├── Core/                         # ID types
├── Entities/                     # Sprite, Light, Object3D wrappers
├── Math/                         # Vector, Color, Rectangle
└── runtimes/                     # Platform-specific native libs
```

---

## 8. Decision Log

| Decision | Rationale | Date |
|----------|-----------|------|
| Use bevy_ecs standalone | Proven, well-maintained, features we need | 2026-01-04 |
| Use rapier for physics | Rust-native, 2D+3D support, active development | 2026-01-04 |
| Render backend abstraction | Future-proofs for Vulkan/Metal | 2026-01-04 |
| FFI handle-based API | Safer than raw pointers, more debuggable | 2026-01-04 |
| Batch operations | Critical for performance at scale | 2026-01-04 |

---

*Document Version: 1.0*
*Last Updated: 2026-01-04*
*Author: Claude (Audit Agent)*
