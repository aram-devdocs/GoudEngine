# GoudEngine Full Refactor - Technical Specification

## Document Information
- **Version:** 1.0
- **Date:** 2026-01-04
- **Based on:** requirements.md PRD v1.0
- **Status:** Draft

---

## 1. Technical Context

### 1.1 Current Technology Stack

| Layer | Technology | Version | Purpose |
|-------|------------|---------|---------|
| **Core Engine** | Rust | 2021 Edition | Performance-critical game logic |
| **Output Format** | cdylib | - | C-compatible dynamic library |
| **FFI Generation** | csbindgen | 1.2.0 | C# binding auto-generation |
| **C Header Gen** | cbindgen | 0.27 | C header for multi-language support |
| **Graphics** | OpenGL | 3.3 Core | Hardware-accelerated rendering |
| **Window/Input** | GLFW | 0.59 | Cross-platform window management |
| **Math** | cgmath | 0.18 | Linear algebra |
| **Image Loading** | image | 0.24 | Texture file parsing |
| **Map Format** | tiled | 0.13 | Tiled map support |
| **C# SDK** | .NET | 8.0 | Managed game development |

### 1.2 Supported Platforms

| Platform | Architecture | Binary Format | Status |
|----------|--------------|---------------|--------|
| macOS | x86_64 | .dylib | ✓ Tested |
| macOS | arm64 | .dylib | Planned |
| Linux | x86_64 | .so | ✓ Tested |
| Windows | x86_64 | .dll | ✓ Basic |

### 1.3 Build & Quality Infrastructure

| Tool | Purpose | Configuration |
|------|---------|---------------|
| cargo fmt | Code formatting | rustfmt.toml |
| cargo clippy | Linting | clippy.toml (threshold=20) |
| cargo deny | License/security audit | deny.toml |
| cargo test | Unit testing | - |
| cargo tarpaulin | Coverage | CI only |
| GitHub Actions | CI/CD | .github/workflows/ |

---

## 2. Architectural Analysis

### 2.1 Current Architecture (AS-IS)

```
┌─────────────────────────────────────────────────────────────┐
│                      C# SDK Layer                            │
│  ┌─────────────────────────────────────────────────────────┐│
│  │ GoudGame.cs       │ Entities/     │ Math/              ││
│  │ AnimationController│ Sprite.cs    │ Vector2/3.cs       ││
│  │ AssetManager      │ Object3D.cs   │ Color.cs           ││
│  │                   │ Light.cs      │ Rectangle.cs       ││
│  └─────────────────────────────────────────────────────────┘│
│                           ↓ P/Invoke                         │
├─────────────────────────────────────────────────────────────┤
│                      FFI Boundary                            │
│  ┌─────────────────────────────────────────────────────────┐│
│  │ sdk.rs (1956 LOC) - 40+ #[no_mangle] extern "C" funcs   ││
│  │ NativeMethods.g.cs - Auto-generated bindings            ││
│  └─────────────────────────────────────────────────────────┘│
│                           ↓                                  │
├─────────────────────────────────────────────────────────────┤
│                    Rust Engine Core                          │
│  ┌───────────────┐  ┌───────────────┐  ┌─────────────────┐ │
│  │ game.rs       │  │ libs/ecs/     │  │ libs/platform/  │ │
│  │ GameSdk       │→ │ SpriteManager │  │ Window          │ │
│  │ Lifecycle     │  │ (not ECS)     │  │ InputHandler    │ │
│  └───────────────┘  └───────────────┘  └─────────────────┘ │
│           ↓                                                  │
│  ┌─────────────────────────────────────────────────────────┐│
│  │                 libs/graphics/                           ││
│  │  ┌───────────┐  ┌───────────────┐  ┌─────────────────┐ ││
│  │  │ renderer  │  │ renderer2d    │  │ renderer3d      │ ││
│  │  │ (enum)    │→ │ Camera2D      │  │ Camera3D        │ ││
│  │  │           │  │ Sprites       │  │ Objects         │ ││
│  │  │           │  │               │  │ Lights (8 max)  │ ││
│  │  └───────────┘  └───────────────┘  └─────────────────┘ ││
│  │                        ↓                                 ││
│  │  ┌─────────────────────────────────────────────────────┐││
│  │  │              components/                             │││
│  │  │  shader.rs │ textures/ │ buffer.rs │ vao.rs        │││
│  │  │  light.rs  │ camera/   │ sprite.rs │ skybox.rs     │││
│  │  └─────────────────────────────────────────────────────┘││
│  └─────────────────────────────────────────────────────────┘│
│                           ↓                                  │
├─────────────────────────────────────────────────────────────┤
│                  External Dependencies                       │
│      gl (OpenGL)  │  glfw  │  image  │  tiled  │  cgmath   │
└─────────────────────────────────────────────────────────────┘
```

### 2.2 Target Architecture (TO-BE)

```
┌─────────────────────────────────────────────────────────────────────┐
│                    Language Binding Layer                            │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌───────┐│
│  │ C# SDK   │  │ Python   │  │ Lua      │  │ TypeScript│ │ Rust  ││
│  │ (rich)   │  │ (thin)   │  │ (thin)   │  │ (thin)   │  │ native││
│  └──────────┘  └──────────┘  └──────────┘  └──────────┘  └───────┘│
│        ↓            ↓            ↓             ↓             ↓      │
├─────────────────────────────────────────────────────────────────────┤
│                    C ABI Layer (Stable API)                          │
│  ┌─────────────────────────────────────────────────────────────────┐│
│  │ goud_api.h - Versioned C header (generated by cbindgen)         ││
│  │ goud_ffi/ module - Organized FFI exports                         ││
│  │ ┌───────────┐ ┌───────────┐ ┌───────────┐ ┌───────────┐        ││
│  │ │ lifecycle │ │ entities  │ │ resources │ │ rendering │        ││
│  │ │ context   │ │ sprites   │ │ textures  │ │ cameras   │        ││
│  │ │ errors    │ │ objects   │ │ audio     │ │ lights    │        ││
│  │ └───────────┘ └───────────┘ └───────────┘ └───────────┘        ││
│  └─────────────────────────────────────────────────────────────────┘│
│        ↓                                                             │
├─────────────────────────────────────────────────────────────────────┤
│                    Core Engine Layer                                 │
│  ┌─────────────────────────────────────────────────────────────────┐│
│  │                     Engine Context                               ││
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────────┐ ││
│  │  │ ResourceMgr │  │ SystemSched │  │ EventBus                │ ││
│  │  │ (handles)   │  │ (parallel)  │  │ (pub/sub)               │ ││
│  │  └─────────────┘  └─────────────┘  └─────────────────────────┘ ││
│  └─────────────────────────────────────────────────────────────────┘│
│        ↓                                                             │
├─────────────────────────────────────────────────────────────────────┤
│                    Subsystem Layer (Modular)                         │
│  ┌───────────┐ ┌───────────┐ ┌───────────┐ ┌───────────┐ ┌───────┐│
│  │ ECS       │ │ Graphics  │ │ Audio     │ │ Physics   │ │ Input ││
│  │ (bevy_ecs)│ │ (abstract)│ │ (rodio)   │ │ (rapier)  │ │(mapped)││
│  └───────────┘ └───────────┘ └───────────┘ └───────────┘ └───────┘│
│        ↓             ↓             ↓             ↓            ↓     │
├─────────────────────────────────────────────────────────────────────┤
│                    Backend Implementation Layer                      │
│  ┌─────────────────────────────────────────────────────────────────┐│
│  │ Graphics Backends       │ Audio Backends  │ Physics Backends    ││
│  │ ┌────────┐ ┌──────────┐│ ┌─────────────┐│ ┌─────────────────┐ ││
│  │ │OpenGL  │ │Vulkan    ││ │rodio        ││ │rapier2d/3d      │ ││
│  │ │(3.3)   │ │(future)  ││ │             ││ │                 │ ││
│  │ └────────┘ └──────────┘│ └─────────────┘│ └─────────────────┘ ││
│  └─────────────────────────────────────────────────────────────────┘│
│        ↓                                                             │
├─────────────────────────────────────────────────────────────────────┤
│                    Platform Layer                                    │
│  ┌───────────┐ ┌───────────┐ ┌───────────┐ ┌───────────┐           │
│  │ Window    │ │ Input     │ │ Filesystem│ │ Threading │           │
│  │ (GLFW)    │ │ (Raw)     │ │ (async)   │ │ (rayon)   │           │
│  └───────────┘ └───────────┘ └───────────┘ └───────────┘           │
└─────────────────────────────────────────────────────────────────────┘
```

### 2.3 Key Architectural Changes

| Area | Current | Target | Rationale |
|------|---------|--------|-----------|
| **ECS** | Sprite-only container | bevy_ecs integration | Industry standard, parallel systems |
| **FFI** | Monolithic sdk.rs | Modular goud_ffi/ | Maintainability, organization |
| **Error Handling** | Panic/bool/String | GoudResult enum | Type-safe error propagation |
| **Memory Safety** | Raw pointers | Handle-based API | Prevent use-after-free |
| **Threading** | Rc everywhere | Arc + Send/Sync | Enable parallelism |
| **Graphics** | Direct OpenGL | Backend abstraction | Future Vulkan/Metal support |
| **Audio** | None | rodio integration | Critical missing feature |
| **Physics** | Basic AABB | rapier integration | Full physics simulation |
| **Resource Mgmt** | HashMap per type | Unified ResourceManager | Centralized lifecycle |
| **Game Loop** | Callback-based | Event-driven | Flexibility, modern pattern |

---

## 3. Implementation Approach

### 3.1 Phase 1: Foundation Hardening

**Objective:** Fix critical architectural issues while maintaining backward compatibility.

#### 3.1.1 Error Handling System

**New Error Types:**

```rust
// goud_engine/src/error.rs
use thiserror::Error;

#[derive(Error, Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub enum GoudError {
    #[error("Success")]
    Ok = 0,

    // Context errors (1-99)
    #[error("Invalid context handle")]
    InvalidContext = 1,
    #[error("Context not initialized")]
    ContextNotInitialized = 2,
    #[error("Context already initialized")]
    ContextAlreadyInitialized = 3,

    // Resource errors (100-199)
    #[error("Resource not found")]
    ResourceNotFound = 100,
    #[error("Resource loading failed")]
    ResourceLoadFailed = 101,
    #[error("Invalid resource handle")]
    InvalidResourceHandle = 102,
    #[error("Resource already exists")]
    ResourceAlreadyExists = 103,

    // Graphics errors (200-299)
    #[error("Shader compilation failed")]
    ShaderCompileFailed = 200,
    #[error("Shader linking failed")]
    ShaderLinkFailed = 201,
    #[error("Texture creation failed")]
    TextureCreationFailed = 202,
    #[error("Invalid renderer state")]
    InvalidRendererState = 203,

    // Entity errors (300-399)
    #[error("Entity not found")]
    EntityNotFound = 300,
    #[error("Invalid entity handle")]
    InvalidEntityHandle = 301,
    #[error("Component not found")]
    ComponentNotFound = 302,

    // Input errors (400-499)
    #[error("Invalid input parameter")]
    InvalidParameter = 400,
    #[error("Null pointer")]
    NullPointer = 401,
    #[error("Buffer too small")]
    BufferTooSmall = 402,

    // System errors (500-599)
    #[error("Window creation failed")]
    WindowCreationFailed = 500,
    #[error("OpenGL initialization failed")]
    OpenGLInitFailed = 501,
    #[error("Audio initialization failed")]
    AudioInitFailed = 502,
    #[error("Physics initialization failed")]
    PhysicsInitFailed = 503,

    // Internal errors (900-999)
    #[error("Internal error")]
    InternalError = 900,
    #[error("Not implemented")]
    NotImplemented = 901,
    #[error("Unknown error")]
    Unknown = 999,
}

pub type GoudResult<T> = Result<T, GoudError>;

// FFI-safe result wrapper
#[repr(C)]
pub struct GoudResultCode {
    pub code: GoudError,
    pub message_ptr: *const c_char,  // Optional error message
}
```

#### 3.1.2 Handle-Based Resource System

**Handle Types:**

```rust
// goud_engine/src/handles.rs
use std::marker::PhantomData;

/// Type-safe handle with generation counter for use-after-free detection
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Handle<T> {
    index: u32,
    generation: u32,
    _marker: PhantomData<T>,
}

impl<T> Handle<T> {
    pub const INVALID: Self = Handle {
        index: u32::MAX,
        generation: u32::MAX,
        _marker: PhantomData,
    };

    pub fn is_valid(&self) -> bool {
        self.index != u32::MAX
    }
}

// Concrete handle types for FFI
#[repr(C)]
pub struct ContextHandle(pub Handle<Context>);

#[repr(C)]
pub struct SpriteHandle(pub Handle<Sprite>);

#[repr(C)]
pub struct TextureHandle(pub Handle<Texture>);

#[repr(C)]
pub struct ObjectHandle(pub Handle<Object3D>);

#[repr(C)]
pub struct LightHandle(pub Handle<Light>);

#[repr(C)]
pub struct AudioHandle(pub Handle<AudioSource>);

// Handle storage with generation tracking
pub struct HandleAllocator<T> {
    items: Vec<Option<T>>,
    generations: Vec<u32>,
    free_list: Vec<u32>,
}

impl<T> HandleAllocator<T> {
    pub fn allocate(&mut self, item: T) -> Handle<T> {
        if let Some(index) = self.free_list.pop() {
            let gen = self.generations[index as usize] + 1;
            self.generations[index as usize] = gen;
            self.items[index as usize] = Some(item);
            Handle { index, generation: gen, _marker: PhantomData }
        } else {
            let index = self.items.len() as u32;
            self.items.push(Some(item));
            self.generations.push(0);
            Handle { index, generation: 0, _marker: PhantomData }
        }
    }

    pub fn get(&self, handle: Handle<T>) -> Option<&T> {
        if handle.index as usize >= self.items.len() {
            return None;
        }
        if self.generations[handle.index as usize] != handle.generation {
            return None;  // Stale handle
        }
        self.items[handle.index as usize].as_ref()
    }

    pub fn get_mut(&mut self, handle: Handle<T>) -> Option<&mut T> {
        if handle.index as usize >= self.items.len() {
            return None;
        }
        if self.generations[handle.index as usize] != handle.generation {
            return None;
        }
        self.items[handle.index as usize].as_mut()
    }

    pub fn free(&mut self, handle: Handle<T>) -> Option<T> {
        if handle.index as usize >= self.items.len() {
            return None;
        }
        if self.generations[handle.index as usize] != handle.generation {
            return None;
        }
        self.free_list.push(handle.index);
        self.items[handle.index as usize].take()
    }
}
```

#### 3.1.3 Thread-Safe Context

```rust
// goud_engine/src/context.rs
use std::sync::{Arc, RwLock, Mutex};
use parking_lot::RwLock as FastRwLock;

/// Thread-safe engine context
pub struct Context {
    // Core state (read-heavy)
    pub resources: FastRwLock<ResourceManager>,
    pub ecs: FastRwLock<World>,  // bevy_ecs World

    // Rendering state (single-thread access)
    pub renderer: Mutex<Option<Box<dyn Renderer>>>,

    // Window state (main thread only)
    pub window: Window,

    // Audio state (thread-safe)
    pub audio: FastRwLock<Option<AudioManager>>,

    // Physics state (can be stepped from any thread)
    pub physics: FastRwLock<Option<PhysicsWorld>>,

    // Event queue (lock-free)
    pub events: crossbeam::queue::SegQueue<EngineEvent>,

    // State flags
    initialized: AtomicBool,
    running: AtomicBool,
}

impl Context {
    pub fn new(config: ContextConfig) -> GoudResult<Arc<Self>> {
        // Validation and initialization
    }
}

/// Global context registry for FFI
static CONTEXTS: OnceLock<Mutex<HandleAllocator<Arc<Context>>>> = OnceLock::new();

fn contexts() -> &'static Mutex<HandleAllocator<Arc<Context>>> {
    CONTEXTS.get_or_init(|| Mutex::new(HandleAllocator::new()))
}

/// FFI-safe context creation
#[no_mangle]
pub extern "C" fn goud_context_create(
    config: *const GoudContextConfig,
    out_handle: *mut ContextHandle,
) -> GoudError {
    // Null checks
    if config.is_null() || out_handle.is_null() {
        return GoudError::NullPointer;
    }

    let config = unsafe { &*config };

    match Context::new(config.into()) {
        Ok(ctx) => {
            let handle = contexts().lock().unwrap().allocate(ctx);
            unsafe { *out_handle = ContextHandle(handle) };
            GoudError::Ok
        }
        Err(e) => e,
    }
}
```

#### 3.1.4 FFI Module Organization

**New Directory Structure:**

```
goud_engine/src/
├── lib.rs                    # Crate root, re-exports
├── error.rs                  # Error types
├── handles.rs                # Handle system
├── context.rs                # Engine context
├── ffi/                      # FFI layer (NEW - replaces sdk.rs)
│   ├── mod.rs               # FFI root
│   ├── context.rs           # Context creation/destruction
│   ├── entities.rs          # Entity/sprite operations
│   ├── resources.rs         # Texture/audio loading
│   ├── rendering.rs         # Camera/light control
│   ├── input.rs             # Input queries
│   ├── physics.rs           # Physics operations
│   ├── batch.rs             # Batch operations
│   └── utils.rs             # String conversion, helpers
├── subsystems/              # Core subsystems (NEW)
│   ├── mod.rs
│   ├── ecs/                 # bevy_ecs integration
│   │   ├── mod.rs
│   │   ├── components.rs    # Transform, Sprite, etc.
│   │   └── systems.rs       # Built-in systems
│   ├── graphics/            # Rendering subsystem
│   │   ├── mod.rs
│   │   ├── backend.rs       # Backend trait
│   │   ├── opengl/          # OpenGL implementation
│   │   │   ├── mod.rs
│   │   │   ├── renderer2d.rs
│   │   │   ├── renderer3d.rs
│   │   │   ├── shader.rs
│   │   │   ├── texture.rs
│   │   │   └── buffer.rs
│   │   ├── camera.rs        # Unified camera
│   │   ├── light.rs         # Light types
│   │   └── material.rs      # Material system
│   ├── audio/               # Audio subsystem
│   │   ├── mod.rs
│   │   ├── source.rs
│   │   └── listener.rs
│   ├── physics/             # Physics subsystem
│   │   ├── mod.rs
│   │   ├── bodies.rs
│   │   ├── colliders.rs
│   │   └── queries.rs
│   └── input/               # Input subsystem
│       ├── mod.rs
│       ├── action_map.rs
│       └── devices.rs
├── resources/               # Resource management (NEW)
│   ├── mod.rs
│   ├── manager.rs           # Unified resource manager
│   ├── loader.rs            # Async loading
│   └── cache.rs             # Caching layer
└── platform/                # Platform abstraction (existing)
    ├── window/
    └── custom_errors.rs     # (deprecated, use error.rs)
```

### 3.2 Phase 2: Core Systems Integration

#### 3.2.1 bevy_ecs Integration

```rust
// goud_engine/src/subsystems/ecs/mod.rs
use bevy_ecs::prelude::*;

// Core components
#[derive(Component, Default, Clone, Copy)]
pub struct Transform {
    pub position: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}

#[derive(Component, Default, Clone, Copy)]
pub struct Transform2D {
    pub position: Vec2,
    pub rotation: f32,
    pub scale: Vec2,
    pub z_layer: i32,
}

#[derive(Component)]
pub struct Sprite {
    pub texture: TextureHandle,
    pub source_rect: Option<Rect>,
    pub color: Color,
    pub flip_x: bool,
    pub flip_y: bool,
}

#[derive(Component)]
pub struct Mesh {
    pub mesh_handle: MeshHandle,
    pub material: MaterialHandle,
}

#[derive(Component)]
pub struct RigidBody {
    pub body_handle: rapier::RigidBodyHandle,
}

#[derive(Component)]
pub struct Collider {
    pub collider_handle: rapier::ColliderHandle,
}

#[derive(Component)]
pub struct AudioSource {
    pub source_handle: AudioHandle,
    pub spatial: bool,
}

// Parent-child relationships (built into bevy_ecs)
// Use Parent and Children components

// Systems
pub fn transform_propagation_system(
    mut root_query: Query<(&Transform, &Children), Without<Parent>>,
    mut child_query: Query<(&mut Transform, Option<&Children>), With<Parent>>,
) {
    // Propagate transforms down hierarchy
}

pub fn sprite_render_system(
    sprites: Query<(&Transform2D, &Sprite)>,
    mut renderer: ResMut<Renderer2D>,
) {
    // Batch and render sprites
}
```

#### 3.2.2 Render Backend Abstraction

```rust
// goud_engine/src/subsystems/graphics/backend.rs

/// Abstract render backend trait
pub trait RenderBackend: Send + Sync {
    // Resource creation
    fn create_texture(&self, desc: &TextureDesc) -> GoudResult<TextureHandle>;
    fn create_shader(&self, desc: &ShaderDesc) -> GoudResult<ShaderHandle>;
    fn create_buffer(&self, desc: &BufferDesc) -> GoudResult<BufferHandle>;
    fn create_render_target(&self, desc: &RenderTargetDesc) -> GoudResult<RenderTargetHandle>;

    // Resource destruction
    fn destroy_texture(&self, handle: TextureHandle);
    fn destroy_shader(&self, handle: ShaderHandle);
    fn destroy_buffer(&self, handle: BufferHandle);
    fn destroy_render_target(&self, handle: RenderTargetHandle);

    // Rendering commands
    fn begin_frame(&mut self);
    fn end_frame(&mut self);
    fn set_render_target(&mut self, target: Option<RenderTargetHandle>);
    fn set_viewport(&mut self, viewport: Viewport);
    fn set_scissor(&mut self, scissor: Option<Rect>);
    fn clear(&mut self, color: Color, depth: Option<f32>, stencil: Option<u32>);

    // Draw calls
    fn draw(&mut self, cmd: &DrawCommand);
    fn draw_indexed(&mut self, cmd: &DrawIndexedCommand);
    fn draw_instanced(&mut self, cmd: &DrawInstancedCommand);

    // State management
    fn set_blend_state(&mut self, state: &BlendState);
    fn set_depth_state(&mut self, state: &DepthState);
    fn set_rasterizer_state(&mut self, state: &RasterizerState);

    // Queries
    fn get_capabilities(&self) -> &BackendCapabilities;
}

/// OpenGL 3.3 implementation
pub struct OpenGLBackend {
    // OpenGL state
}

impl RenderBackend for OpenGLBackend {
    // Implementation
}

/// Draw call batching system
pub struct DrawBatcher {
    batches: Vec<SpriteBatch>,
    current_batch: Option<SpriteBatch>,
}

pub struct SpriteBatch {
    texture: TextureHandle,
    vertices: Vec<SpriteVertex>,
    indices: Vec<u32>,
}

impl DrawBatcher {
    pub fn add_sprite(&mut self, sprite: &SpriteRenderData) {
        // Check if can batch with current
        // If not, flush and start new batch
    }

    pub fn flush(&mut self, backend: &mut dyn RenderBackend) {
        for batch in self.batches.drain(..) {
            // Single draw call per batch
            backend.draw_indexed(&DrawIndexedCommand {
                vertex_buffer: batch.vertex_buffer,
                index_buffer: batch.index_buffer,
                index_count: batch.indices.len() as u32,
                texture: batch.texture,
                // ...
            });
        }
    }
}
```

#### 3.2.3 Audio System (rodio)

```rust
// goud_engine/src/subsystems/audio/mod.rs
use rodio::{OutputStream, Sink, Decoder, Source};

pub struct AudioManager {
    _stream: OutputStream,
    stream_handle: OutputStreamHandle,
    sources: HandleAllocator<AudioSourceData>,
    listener_position: Vec3,
}

pub struct AudioSourceData {
    sink: Sink,
    position: Option<Vec3>,  // None = non-spatial
    volume: f32,
    looping: bool,
}

impl AudioManager {
    pub fn new() -> GoudResult<Self> {
        let (stream, handle) = OutputStream::try_default()
            .map_err(|_| GoudError::AudioInitFailed)?;

        Ok(Self {
            _stream: stream,
            stream_handle: handle,
            sources: HandleAllocator::new(),
            listener_position: Vec3::ZERO,
        })
    }

    pub fn play(&mut self, path: &str, config: AudioPlayConfig) -> GoudResult<AudioHandle> {
        let file = std::fs::File::open(path)
            .map_err(|_| GoudError::ResourceNotFound)?;
        let source = Decoder::new(BufReader::new(file))
            .map_err(|_| GoudError::ResourceLoadFailed)?;

        let sink = Sink::try_new(&self.stream_handle)
            .map_err(|_| GoudError::AudioInitFailed)?;

        if config.looping {
            sink.append(source.repeat_infinite());
        } else {
            sink.append(source);
        }

        sink.set_volume(config.volume);

        let data = AudioSourceData {
            sink,
            position: config.position,
            volume: config.volume,
            looping: config.looping,
        };

        Ok(self.sources.allocate(data))
    }

    pub fn stop(&mut self, handle: AudioHandle) -> GoudResult<()> {
        let source = self.sources.get_mut(handle.0)
            .ok_or(GoudError::InvalidResourceHandle)?;
        source.sink.stop();
        self.sources.free(handle.0);
        Ok(())
    }

    pub fn set_volume(&mut self, handle: AudioHandle, volume: f32) -> GoudResult<()> {
        let source = self.sources.get_mut(handle.0)
            .ok_or(GoudError::InvalidResourceHandle)?;
        source.volume = volume;
        source.sink.set_volume(volume);
        Ok(())
    }

    pub fn update_spatial(&mut self) {
        // Update volume based on distance for spatial sources
        for (_, source) in self.sources.iter_mut() {
            if let Some(pos) = source.position {
                let distance = (pos - self.listener_position).length();
                let falloff = 1.0 / (1.0 + distance * 0.1);
                source.sink.set_volume(source.volume * falloff);
            }
        }
    }
}
```

#### 3.2.4 Physics System (rapier)

```rust
// goud_engine/src/subsystems/physics/mod.rs
use rapier2d::prelude::*;  // or rapier3d for 3D

pub struct PhysicsWorld2D {
    gravity: Vector<Real>,
    integration_params: IntegrationParameters,
    physics_pipeline: PhysicsPipeline,
    island_manager: IslandManager,
    broad_phase: BroadPhase,
    narrow_phase: NarrowPhase,
    rigid_body_set: RigidBodySet,
    collider_set: ColliderSet,
    impulse_joint_set: ImpulseJointSet,
    multibody_joint_set: MultibodyJointSet,
    ccd_solver: CCDSolver,
    query_pipeline: QueryPipeline,
    event_handler: ChannelEventCollector,
    collision_recv: Receiver<CollisionEvent>,
    contact_recv: Receiver<ContactForceEvent>,
}

impl PhysicsWorld2D {
    pub fn new(config: PhysicsConfig) -> Self {
        let (collision_send, collision_recv) = crossbeam::channel::unbounded();
        let (contact_send, contact_recv) = crossbeam::channel::unbounded();

        Self {
            gravity: vector![config.gravity.x, config.gravity.y],
            integration_params: IntegrationParameters::default(),
            physics_pipeline: PhysicsPipeline::new(),
            island_manager: IslandManager::new(),
            broad_phase: BroadPhase::new(),
            narrow_phase: NarrowPhase::new(),
            rigid_body_set: RigidBodySet::new(),
            collider_set: ColliderSet::new(),
            impulse_joint_set: ImpulseJointSet::new(),
            multibody_joint_set: MultibodyJointSet::new(),
            ccd_solver: CCDSolver::new(),
            query_pipeline: QueryPipeline::new(),
            event_handler: ChannelEventCollector::new(collision_send, contact_send),
            collision_recv,
            contact_recv,
        }
    }

    pub fn create_rigid_body(&mut self, desc: RigidBodyDesc) -> RigidBodyHandle {
        let body = match desc.body_type {
            BodyType::Dynamic => RigidBodyBuilder::dynamic(),
            BodyType::Static => RigidBodyBuilder::fixed(),
            BodyType::Kinematic => RigidBodyBuilder::kinematic_position_based(),
        }
        .translation(vector![desc.position.x, desc.position.y])
        .rotation(desc.rotation)
        .build();

        self.rigid_body_set.insert(body)
    }

    pub fn create_collider(
        &mut self,
        body: RigidBodyHandle,
        desc: ColliderDesc,
    ) -> ColliderHandle {
        let shape = match desc.shape {
            ColliderShape::Box { width, height } =>
                SharedShape::cuboid(width / 2.0, height / 2.0),
            ColliderShape::Circle { radius } =>
                SharedShape::ball(radius),
            ColliderShape::Capsule { half_height, radius } =>
                SharedShape::capsule_y(half_height, radius),
        };

        let collider = ColliderBuilder::new(shape)
            .friction(desc.friction)
            .restitution(desc.restitution)
            .sensor(desc.is_trigger)
            .build();

        self.collider_set.insert_with_parent(collider, body, &mut self.rigid_body_set)
    }

    pub fn step(&mut self, dt: f32) {
        self.integration_params.dt = dt;

        self.physics_pipeline.step(
            &self.gravity,
            &self.integration_params,
            &mut self.island_manager,
            &mut self.broad_phase,
            &mut self.narrow_phase,
            &mut self.rigid_body_set,
            &mut self.collider_set,
            &mut self.impulse_joint_set,
            &mut self.multibody_joint_set,
            &mut self.ccd_solver,
            Some(&mut self.query_pipeline),
            &(),
            &self.event_handler,
        );
    }

    pub fn poll_collision_events(&self) -> Vec<CollisionEvent> {
        self.collision_recv.try_iter().collect()
    }

    pub fn raycast(&self, origin: Vec2, direction: Vec2, max_dist: f32) -> Option<RaycastHit> {
        let ray = Ray::new(point![origin.x, origin.y], vector![direction.x, direction.y]);

        self.query_pipeline.cast_ray(
            &self.rigid_body_set,
            &self.collider_set,
            &ray,
            max_dist,
            true,
            QueryFilter::default(),
        ).map(|(handle, toi)| RaycastHit {
            collider: handle,
            distance: toi,
            point: origin + direction * toi,
        })
    }
}
```

### 3.3 Phase 3: Graphics Enhancement

#### 3.3.1 Texture Atlas System

```rust
// goud_engine/src/subsystems/graphics/atlas.rs

pub struct TextureAtlas {
    texture: TextureHandle,
    regions: HashMap<String, AtlasRegion>,
    width: u32,
    height: u32,
}

pub struct AtlasRegion {
    pub rect: Rect,
    pub uv: UVRect,
    pub name: String,
}

pub struct AtlasPacker {
    bins: Vec<AtlasBin>,
    max_size: u32,
}

impl AtlasPacker {
    pub fn new(max_size: u32) -> Self {
        Self {
            bins: vec![AtlasBin::new(max_size)],
            max_size,
        }
    }

    pub fn pack(&mut self, images: &[(&str, &DynamicImage)]) -> GoudResult<Vec<TextureAtlas>> {
        // Rectangle bin packing algorithm (MaxRects)
        let mut rects: Vec<_> = images.iter()
            .map(|(name, img)| PackRect {
                name: name.to_string(),
                width: img.width(),
                height: img.height(),
                image: img,
            })
            .collect();

        // Sort by height (descending) for better packing
        rects.sort_by(|a, b| b.height.cmp(&a.height));

        let mut atlases = Vec::new();
        let mut current_bin = AtlasBin::new(self.max_size);

        for rect in rects {
            if let Some(placement) = current_bin.insert(&rect) {
                // Placed successfully
            } else {
                // Bin full, start new one
                atlases.push(current_bin.finalize()?);
                current_bin = AtlasBin::new(self.max_size);
                current_bin.insert(&rect).ok_or(GoudError::TextureCreationFailed)?;
            }
        }

        if !current_bin.is_empty() {
            atlases.push(current_bin.finalize()?);
        }

        Ok(atlases)
    }
}

// Automatic sprite sheet slicing
impl TextureAtlas {
    pub fn from_grid(
        texture: TextureHandle,
        cell_width: u32,
        cell_height: u32,
        columns: u32,
        rows: u32,
    ) -> Self {
        let mut regions = HashMap::new();

        for row in 0..rows {
            for col in 0..columns {
                let name = format!("{}_{}", row, col);
                let x = col * cell_width;
                let y = row * cell_height;

                regions.insert(name.clone(), AtlasRegion {
                    rect: Rect { x: x as f32, y: y as f32, width: cell_width as f32, height: cell_height as f32 },
                    uv: UVRect::from_pixel_coords(x, y, cell_width, cell_height, texture.width, texture.height),
                    name,
                });
            }
        }

        Self { texture, regions, width: columns * cell_width, height: rows * cell_height }
    }
}
```

#### 3.3.2 Material System

```rust
// goud_engine/src/subsystems/graphics/material.rs

pub struct Material {
    pub shader: ShaderHandle,
    pub properties: MaterialProperties,
    pub textures: HashMap<String, TextureHandle>,
}

pub struct MaterialProperties {
    pub albedo: Color,
    pub metallic: f32,
    pub roughness: f32,
    pub emission: Color,
    pub emission_strength: f32,
}

impl Default for MaterialProperties {
    fn default() -> Self {
        Self {
            albedo: Color::WHITE,
            metallic: 0.0,
            roughness: 0.5,
            emission: Color::BLACK,
            emission_strength: 0.0,
        }
    }
}

pub struct MaterialBuilder {
    shader: ShaderHandle,
    properties: MaterialProperties,
    textures: HashMap<String, TextureHandle>,
}

impl MaterialBuilder {
    pub fn new(shader: ShaderHandle) -> Self {
        Self {
            shader,
            properties: MaterialProperties::default(),
            textures: HashMap::new(),
        }
    }

    pub fn albedo(mut self, color: Color) -> Self {
        self.properties.albedo = color;
        self
    }

    pub fn texture(mut self, slot: &str, texture: TextureHandle) -> Self {
        self.textures.insert(slot.to_string(), texture);
        self
    }

    pub fn metallic(mut self, value: f32) -> Self {
        self.properties.metallic = value.clamp(0.0, 1.0);
        self
    }

    pub fn roughness(mut self, value: f32) -> Self {
        self.properties.roughness = value.clamp(0.0, 1.0);
        self
    }

    pub fn build(self) -> Material {
        Material {
            shader: self.shader,
            properties: self.properties,
            textures: self.textures,
        }
    }
}
```

### 3.4 Phase 4: Developer Experience

#### 3.4.1 Builder Pattern for FFI Types

```rust
// goud_engine/src/ffi/builders.rs

#[repr(C)]
pub struct GoudSpriteDesc {
    pub texture: TextureHandle,
    pub x: f32,
    pub y: f32,
    pub z_layer: i32,
    pub width: f32,
    pub height: f32,
    pub scale_x: f32,
    pub scale_y: f32,
    pub rotation: f32,
    pub color_r: f32,
    pub color_g: f32,
    pub color_b: f32,
    pub color_a: f32,
    pub source_rect: *const GoudRect,  // Optional
}

impl Default for GoudSpriteDesc {
    fn default() -> Self {
        Self {
            texture: TextureHandle::INVALID,
            x: 0.0,
            y: 0.0,
            z_layer: 0,
            width: 0.0,  // 0 = use texture size
            height: 0.0,
            scale_x: 1.0,
            scale_y: 1.0,
            rotation: 0.0,
            color_r: 1.0,
            color_g: 1.0,
            color_b: 1.0,
            color_a: 1.0,
            source_rect: std::ptr::null(),
        }
    }
}

// C# Builder (in SDK)
// public class SpriteBuilder
// {
//     private GoudSpriteDesc _desc = new();
//
//     public SpriteBuilder Texture(TextureHandle tex) { _desc.texture = tex; return this; }
//     public SpriteBuilder Position(float x, float y) { _desc.x = x; _desc.y = y; return this; }
//     public SpriteBuilder Layer(int z) { _desc.z_layer = z; return this; }
//     public SpriteBuilder Scale(float x, float y) { _desc.scale_x = x; _desc.scale_y = y; return this; }
//     public SpriteBuilder Rotation(float rad) { _desc.rotation = rad; return this; }
//     public SpriteBuilder Color(Color c) { /* set color fields */ return this; }
//
//     public SpriteHandle Build(ContextHandle ctx) => NativeMethods.goud_sprite_create(ctx, ref _desc);
// }
```

#### 3.4.2 Batch Operations

```rust
// goud_engine/src/ffi/batch.rs

#[no_mangle]
pub extern "C" fn goud_sprites_create_batch(
    ctx: ContextHandle,
    descs: *const GoudSpriteDesc,
    count: u32,
    out_handles: *mut SpriteHandle,
) -> GoudError {
    if descs.is_null() || out_handles.is_null() {
        return GoudError::NullPointer;
    }

    let context = match get_context(ctx) {
        Some(c) => c,
        None => return GoudError::InvalidContext,
    };

    let descs = unsafe { std::slice::from_raw_parts(descs, count as usize) };
    let out = unsafe { std::slice::from_raw_parts_mut(out_handles, count as usize) };

    let mut world = context.ecs.write();

    for (i, desc) in descs.iter().enumerate() {
        let entity = world.spawn((
            Transform2D {
                position: Vec2::new(desc.x, desc.y),
                rotation: desc.rotation,
                scale: Vec2::new(desc.scale_x, desc.scale_y),
                z_layer: desc.z_layer,
            },
            Sprite {
                texture: desc.texture,
                color: Color::new(desc.color_r, desc.color_g, desc.color_b, desc.color_a),
                // ...
            },
        )).id();

        out[i] = SpriteHandle(/* entity to handle conversion */);
    }

    GoudError::Ok
}

#[no_mangle]
pub extern "C" fn goud_sprites_update_batch(
    ctx: ContextHandle,
    updates: *const GoudSpriteUpdate,
    count: u32,
) -> GoudError {
    // Similar batch update logic
}

#[no_mangle]
pub extern "C" fn goud_sprites_destroy_batch(
    ctx: ContextHandle,
    handles: *const SpriteHandle,
    count: u32,
) -> GoudError {
    // Batch destruction
}
```

---

## 4. Source Code Structure Changes

### 4.1 Files to Add

| Path | Purpose |
|------|---------|
| `src/error.rs` | Comprehensive error types |
| `src/handles.rs` | Type-safe handle system |
| `src/context.rs` | Thread-safe engine context |
| `src/ffi/mod.rs` | FFI module root |
| `src/ffi/context.rs` | Context FFI functions |
| `src/ffi/entities.rs` | Entity FFI functions |
| `src/ffi/resources.rs` | Resource FFI functions |
| `src/ffi/rendering.rs` | Rendering FFI functions |
| `src/ffi/input.rs` | Input FFI functions |
| `src/ffi/physics.rs` | Physics FFI functions |
| `src/ffi/batch.rs` | Batch operations |
| `src/ffi/utils.rs` | FFI utilities |
| `src/subsystems/mod.rs` | Subsystem root |
| `src/subsystems/ecs/mod.rs` | ECS module |
| `src/subsystems/ecs/components.rs` | Built-in components |
| `src/subsystems/ecs/systems.rs` | Built-in systems |
| `src/subsystems/graphics/mod.rs` | Graphics module |
| `src/subsystems/graphics/backend.rs` | Render backend trait |
| `src/subsystems/graphics/opengl/mod.rs` | OpenGL backend |
| `src/subsystems/graphics/camera.rs` | Unified camera |
| `src/subsystems/graphics/material.rs` | Material system |
| `src/subsystems/graphics/atlas.rs` | Texture atlasing |
| `src/subsystems/graphics/batch.rs` | Draw batching |
| `src/subsystems/audio/mod.rs` | Audio module |
| `src/subsystems/audio/source.rs` | Audio sources |
| `src/subsystems/physics/mod.rs` | Physics module |
| `src/subsystems/physics/world.rs` | Physics world |
| `src/subsystems/input/mod.rs` | Input module |
| `src/subsystems/input/action_map.rs` | Input mapping |
| `src/resources/mod.rs` | Resource management |
| `src/resources/manager.rs` | Resource manager |
| `src/resources/loader.rs` | Async loading |

### 4.2 Files to Modify

| Path | Changes |
|------|---------|
| `src/lib.rs` | Re-export new modules, deprecate old |
| `src/game.rs` | Refactor to use Context, deprecate callbacks |
| `src/types.rs` | Move to appropriate modules, deprecate |
| `src/sdk.rs` | Split into ffi/ modules, deprecate |
| `Cargo.toml` | Add bevy_ecs, rapier, rodio dependencies |

### 4.3 Files to Deprecate/Remove

| Path | Reason |
|------|--------|
| `src/sdk.rs` | Replaced by `src/ffi/` modules |
| `src/ffi_privates.rs` | Replaced by `src/handles.rs` |
| `src/libs/ecs/mod.rs` | Replaced by bevy_ecs integration |
| `src/platform/custom_errors.rs` | Replaced by `src/error.rs` |

---

## 5. Data Model / API / Interface Changes

### 5.1 New FFI API Contract

```c
// goud_api.h (generated by cbindgen)

#ifndef GOUD_API_H
#define GOUD_API_H

#include <stdint.h>
#include <stdbool.h>

#define GOUD_API_VERSION 2

// Error codes
typedef enum {
    GOUD_OK = 0,
    GOUD_ERROR_INVALID_CONTEXT = 1,
    GOUD_ERROR_CONTEXT_NOT_INITIALIZED = 2,
    // ... (full enum from error.rs)
} GoudError;

// Handle types (opaque)
typedef struct { uint32_t index; uint32_t generation; } GoudContextHandle;
typedef struct { uint32_t index; uint32_t generation; } GoudSpriteHandle;
typedef struct { uint32_t index; uint32_t generation; } GoudTextureHandle;
typedef struct { uint32_t index; uint32_t generation; } GoudObjectHandle;
typedef struct { uint32_t index; uint32_t generation; } GoudLightHandle;
typedef struct { uint32_t index; uint32_t generation; } GoudAudioHandle;
typedef struct { uint32_t index; uint32_t generation; } GoudBodyHandle;
typedef struct { uint32_t index; uint32_t generation; } GoudColliderHandle;

// Configuration structures
typedef struct {
    uint32_t width;
    uint32_t height;
    const char* title;
    uint32_t target_fps;
    int32_t renderer_type;  // 0=2D, 1=3D
    bool enable_audio;
    bool enable_physics;
} GoudContextConfig;

typedef struct {
    GoudTextureHandle texture;
    float x, y;
    int32_t z_layer;
    float width, height;
    float scale_x, scale_y;
    float rotation;
    float color_r, color_g, color_b, color_a;
    const GoudRect* source_rect;  // Optional
} GoudSpriteDesc;

// Lifecycle functions
GoudError goud_context_create(const GoudContextConfig* config, GoudContextHandle* out_handle);
GoudError goud_context_destroy(GoudContextHandle handle);
GoudError goud_context_initialize(GoudContextHandle handle);
bool goud_context_should_close(GoudContextHandle handle);
GoudError goud_context_begin_frame(GoudContextHandle handle);
GoudError goud_context_end_frame(GoudContextHandle handle, float* out_delta_time);

// Resource functions
GoudError goud_texture_load(GoudContextHandle ctx, const char* path, GoudTextureHandle* out_handle);
GoudError goud_texture_unload(GoudContextHandle ctx, GoudTextureHandle handle);

// Entity functions (single)
GoudError goud_sprite_create(GoudContextHandle ctx, const GoudSpriteDesc* desc, GoudSpriteHandle* out_handle);
GoudError goud_sprite_destroy(GoudContextHandle ctx, GoudSpriteHandle handle);
GoudError goud_sprite_set_position(GoudContextHandle ctx, GoudSpriteHandle handle, float x, float y);
GoudError goud_sprite_get_position(GoudContextHandle ctx, GoudSpriteHandle handle, float* out_x, float* out_y);

// Entity functions (batch)
GoudError goud_sprites_create_batch(GoudContextHandle ctx, const GoudSpriteDesc* descs, uint32_t count, GoudSpriteHandle* out_handles);
GoudError goud_sprites_destroy_batch(GoudContextHandle ctx, const GoudSpriteHandle* handles, uint32_t count);

// 3D object functions
GoudError goud_object_create_cube(GoudContextHandle ctx, GoudTextureHandle tex, float w, float h, float d, GoudObjectHandle* out_handle);
GoudError goud_object_create_sphere(GoudContextHandle ctx, GoudTextureHandle tex, float radius, uint32_t segments, GoudObjectHandle* out_handle);

// Light functions
GoudError goud_light_create_point(GoudContextHandle ctx, const GoudPointLightDesc* desc, GoudLightHandle* out_handle);
GoudError goud_light_create_directional(GoudContextHandle ctx, const GoudDirLightDesc* desc, GoudLightHandle* out_handle);
GoudError goud_light_destroy(GoudContextHandle ctx, GoudLightHandle handle);

// Camera functions
GoudError goud_camera_set_position(GoudContextHandle ctx, float x, float y, float z);
GoudError goud_camera_set_rotation(GoudContextHandle ctx, float pitch, float yaw, float roll);
GoudError goud_camera_set_zoom(GoudContextHandle ctx, float zoom);

// Input functions
bool goud_input_is_key_pressed(GoudContextHandle ctx, int32_t key);
bool goud_input_is_key_just_pressed(GoudContextHandle ctx, int32_t key);
bool goud_input_is_mouse_button_pressed(GoudContextHandle ctx, int32_t button);
GoudError goud_input_get_mouse_position(GoudContextHandle ctx, float* out_x, float* out_y);

// Audio functions
GoudError goud_audio_play(GoudContextHandle ctx, const char* path, const GoudAudioConfig* config, GoudAudioHandle* out_handle);
GoudError goud_audio_stop(GoudContextHandle ctx, GoudAudioHandle handle);
GoudError goud_audio_set_volume(GoudContextHandle ctx, GoudAudioHandle handle, float volume);

// Physics functions
GoudError goud_physics_create_body(GoudContextHandle ctx, const GoudBodyDesc* desc, GoudBodyHandle* out_handle);
GoudError goud_physics_create_collider(GoudContextHandle ctx, GoudBodyHandle body, const GoudColliderDesc* desc, GoudColliderHandle* out_handle);
GoudError goud_physics_step(GoudContextHandle ctx, float dt);
GoudError goud_physics_raycast(GoudContextHandle ctx, const GoudRay* ray, GoudRaycastHit* out_hit);

// Utility functions
const char* goud_error_message(GoudError error);
uint32_t goud_api_version(void);

#endif // GOUD_API_H
```

### 5.2 C# SDK Changes

**New Structure:**

```csharp
// sdks/GoudEngine/
├── GoudEngine.csproj
├── GoudContext.cs          // Main entry point (replaces GoudGame.cs)
├── GoudError.cs            // Error enum + helpers
├── Handles/
│   ├── SpriteHandle.cs
│   ├── TextureHandle.cs
│   ├── ObjectHandle.cs
│   ├── LightHandle.cs
│   ├── AudioHandle.cs
│   └── PhysicsHandles.cs
├── Builders/
│   ├── SpriteBuilder.cs
│   ├── LightBuilder.cs
│   ├── BodyBuilder.cs
│   └── MaterialBuilder.cs
├── Components/
│   ├── Transform.cs
│   ├── Sprite.cs
│   └── RigidBody.cs
├── Math/
│   └── (existing)
├── Config/
│   ├── ContextConfig.cs
│   ├── AudioConfig.cs
│   └── PhysicsConfig.cs
├── Native/
│   ├── NativeMethods.g.cs  // Auto-generated
│   └── NativeUtils.cs      // String marshaling, etc.
└── runtimes/
    └── (existing)
```

**Example New API:**

```csharp
// High-level C# API
public class GoudContext : IDisposable
{
    private ContextHandle _handle;

    public GoudContext(ContextConfig config)
    {
        var nativeConfig = config.ToNative();
        var result = NativeMethods.goud_context_create(ref nativeConfig, out _handle);
        GoudError.ThrowIfFailed(result);
    }

    public void Initialize()
    {
        GoudError.ThrowIfFailed(NativeMethods.goud_context_initialize(_handle));
    }

    public bool ShouldClose => NativeMethods.goud_context_should_close(_handle);

    public float BeginFrame()
    {
        GoudError.ThrowIfFailed(NativeMethods.goud_context_begin_frame(_handle));
    }

    public float EndFrame()
    {
        float deltaTime;
        GoudError.ThrowIfFailed(NativeMethods.goud_context_end_frame(_handle, out deltaTime));
        return deltaTime;
    }

    // Resource loading
    public TextureHandle LoadTexture(string path)
    {
        TextureHandle handle;
        GoudError.ThrowIfFailed(NativeMethods.goud_texture_load(_handle, path, out handle));
        return handle;
    }

    // Sprite creation via builder
    public SpriteBuilder CreateSprite() => new SpriteBuilder(this);

    // Batch operations
    public SpriteHandle[] CreateSprites(SpriteDesc[] descs)
    {
        var handles = new SpriteHandle[descs.Length];
        var nativeDescs = descs.Select(d => d.ToNative()).ToArray();
        GoudError.ThrowIfFailed(NativeMethods.goud_sprites_create_batch(
            _handle, nativeDescs, (uint)descs.Length, handles));
        return handles;
    }

    // Audio
    public AudioHandle PlaySound(string path, AudioConfig config = null)
    {
        config ??= AudioConfig.Default;
        AudioHandle handle;
        GoudError.ThrowIfFailed(NativeMethods.goud_audio_play(
            _handle, path, config.ToNative(), out handle));
        return handle;
    }

    // Physics
    public BodyHandle CreateBody(BodyDesc desc)
    {
        BodyHandle handle;
        GoudError.ThrowIfFailed(NativeMethods.goud_physics_create_body(
            _handle, ref desc.ToNative(), out handle));
        return handle;
    }

    public void Dispose()
    {
        if (_handle.IsValid)
        {
            NativeMethods.goud_context_destroy(_handle);
            _handle = default;
        }
    }
}

// Builder pattern
public class SpriteBuilder
{
    private readonly GoudContext _ctx;
    private SpriteDesc _desc = new();

    internal SpriteBuilder(GoudContext ctx) => _ctx = ctx;

    public SpriteBuilder Texture(TextureHandle tex) { _desc.Texture = tex; return this; }
    public SpriteBuilder Position(float x, float y) { _desc.X = x; _desc.Y = y; return this; }
    public SpriteBuilder Position(Vector2 pos) { _desc.X = pos.X; _desc.Y = pos.Y; return this; }
    public SpriteBuilder Layer(int z) { _desc.ZLayer = z; return this; }
    public SpriteBuilder Scale(float x, float y) { _desc.ScaleX = x; _desc.ScaleY = y; return this; }
    public SpriteBuilder Scale(float s) { _desc.ScaleX = s; _desc.ScaleY = s; return this; }
    public SpriteBuilder Scale(Vector2 s) { _desc.ScaleX = s.X; _desc.ScaleY = s.Y; return this; }
    public SpriteBuilder Rotation(float radians) { _desc.Rotation = radians; return this; }
    public SpriteBuilder Color(Color c) { _desc.Color = c; return this; }

    public SpriteHandle Build()
    {
        SpriteHandle handle;
        var native = _desc.ToNative();
        GoudError.ThrowIfFailed(NativeMethods.goud_sprite_create(
            _ctx._handle, ref native, out handle));
        return handle;
    }
}

// Error handling
public static class GoudError
{
    public static void ThrowIfFailed(GoudErrorCode code)
    {
        if (code != GoudErrorCode.Ok)
            throw new GoudException(code);
    }
}

public class GoudException : Exception
{
    public GoudErrorCode ErrorCode { get; }

    public GoudException(GoudErrorCode code)
        : base(NativeMethods.goud_error_message(code))
    {
        ErrorCode = code;
    }
}
```

---

## 6. Delivery Phases

### Phase 1: Foundation Hardening (Weeks 1-4)

| Week | Milestone | Deliverables |
|------|-----------|--------------|
| 1 | Error System | `error.rs`, `handles.rs`, unit tests |
| 2 | Context System | `context.rs`, thread-safe resource storage |
| 3 | FFI Reorganization | `ffi/` modules, deprecation of `sdk.rs` |
| 4 | Integration | C# SDK updates, backward compat shim |

**Verification:**
- `cargo test` - All existing tests pass
- `cargo clippy -- -D warnings` - No warnings
- New tests for error propagation
- C# SDK compiles and runs example games

### Phase 2: Core Systems (Weeks 5-10)

| Week | Milestone | Deliverables |
|------|-----------|--------------|
| 5-6 | bevy_ecs Integration | ECS module, component definitions |
| 7 | Scene Graph | Parent-child, transform propagation |
| 8 | Physics (rapier) | Physics world, bodies, colliders |
| 9 | Audio (rodio) | Audio manager, spatial audio |
| 10 | Integration | FFI for new systems, C# wrappers |

**Verification:**
- ECS unit tests (spawning, querying, systems)
- Physics simulation tests
- Audio playback tests
- Integration tests with example games

### Phase 3: Graphics Enhancement (Weeks 11-14)

| Week | Milestone | Deliverables |
|------|-----------|--------------|
| 11 | Backend Abstraction | `RenderBackend` trait, OpenGL impl |
| 12 | Draw Batching | SpriteBatcher, instancing |
| 13 | Texture Atlas | Atlas packing, sprite sheets |
| 14 | Material System | Materials, shader hot-reload |

**Verification:**
- Benchmark: 10,000 sprites < 100 draw calls
- Visual regression tests
- Shader compilation tests

### Phase 4: Developer Experience (Weeks 15-18)

| Week | Milestone | Deliverables |
|------|-----------|--------------|
| 15 | Builder APIs | All builder patterns in Rust & C# |
| 16 | Batch Operations | Batch create/update/destroy FFI |
| 17 | Documentation | API docs, architecture guide |
| 18 | Examples | Updated examples, new showcases |

**Verification:**
- API documentation 100% coverage
- All examples compile and run
- Benchmark batch vs single operations (10x improvement)

### Phase 5: Polish & Optimization (Weeks 19-22)

| Week | Milestone | Deliverables |
|------|-----------|--------------|
| 19 | Performance | Profiling, optimization pass |
| 20 | Memory | Leak detection, memory optimization |
| 21 | API Stability | Deprecation cleanup, API freeze |
| 22 | Release | RC build, migration guide |

**Verification:**
- All performance targets met
- Zero memory leaks (Valgrind/ASAN)
- CI passes on all platforms
- Migration guide tested

---

## 7. Verification Approach

### 7.1 Testing Strategy

| Level | Scope | Tools | Target Coverage |
|-------|-------|-------|-----------------|
| **Unit** | Individual modules | `cargo test` | >80% |
| **Integration** | Cross-module | `cargo test --test integration` | Key paths |
| **FFI** | Rust↔C# boundary | C# test project | All FFI functions |
| **Rendering** | Visual output | Snapshot testing | Core rendering |
| **Performance** | Benchmarks | `criterion` | Regression tracking |
| **Fuzz** | FFI inputs | `cargo-fuzz` | Error handling |

### 7.2 Test Commands

```bash
# Run all Rust tests
cargo test

# Run with output
cargo test -- --nocapture

# Run specific module tests
cargo test error::
cargo test handles::
cargo test ffi::

# Run integration tests
cargo test --test integration

# Run benchmarks
cargo bench

# Run fuzzing (requires nightly)
cargo +nightly fuzz run ffi_sprite_create

# Run C# SDK tests
cd sdks/GoudEngine.Tests && dotnet test

# Coverage report
cargo tarpaulin --out Html
```

### 7.3 CI Pipeline Updates

```yaml
# .github/workflows/ci.yml additions
jobs:
  test:
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
        rust: [stable, nightly]
    steps:
      - uses: actions/checkout@v4
      - name: Install Rust
        uses: dtolnay/rust-action@stable
      - name: Run tests
        run: cargo test --all-features
      - name: Run clippy
        run: cargo clippy -- -D warnings
      - name: Check formatting
        run: cargo fmt -- --check

  benchmarks:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Run benchmarks
        run: cargo bench --no-run  # Compile only in CI

  fuzz:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Install nightly
        run: rustup install nightly
      - name: Run fuzz tests
        run: cargo +nightly fuzz run ffi_inputs -- -max_total_time=60

  integration:
    needs: test
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Build Rust
        run: cargo build --release
      - name: Build C# SDK
        run: cd sdks/GoudEngine && dotnet build
      - name: Run C# tests
        run: cd sdks/GoudEngine.Tests && dotnet test
      - name: Run example games (headless)
        run: |
          for example in examples/*/; do
            cd $example && dotnet run --headless || true
            cd ../..
          done
```

### 7.4 Performance Targets Verification

```rust
// benches/sprite_benchmark.rs
use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};

fn benchmark_sprite_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("sprite_creation");

    for count in [100, 1000, 10000].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(count),
            count,
            |b, &count| {
                b.iter(|| {
                    // Create sprites
                });
            },
        );
    }
    group.finish();
}

fn benchmark_batch_vs_single(c: &mut Criterion) {
    let mut group = c.benchmark_group("batch_vs_single");

    group.bench_function("single_1000", |b| {
        b.iter(|| {
            for _ in 0..1000 {
                // Single sprite creation
            }
        });
    });

    group.bench_function("batch_1000", |b| {
        b.iter(|| {
            // Batch creation of 1000 sprites
        });
    });

    group.finish();
}

criterion_group!(benches, benchmark_sprite_creation, benchmark_batch_vs_single);
criterion_main!(benches);
```

---

## 8. Dependencies

### 8.1 New Cargo Dependencies

```toml
[dependencies]
# Existing
gl = "0.14"
glfw = "0.59"
cgmath = "0.18"
image = "0.24"
tiled = "0.13"
log = "0.4"
env_logger = "0.11"
thiserror = "1.0"

# New - Core
bevy_ecs = "0.15"              # ECS framework
parking_lot = "0.12"           # Fast RwLock
crossbeam = "0.8"              # Lock-free queues
rayon = "1.10"                 # Parallel iteration

# New - Audio
rodio = "0.19"                 # Audio playback

# New - Physics
rapier2d = "0.22"              # 2D physics
rapier3d = "0.22"              # 3D physics

# New - Utilities
bitflags = "2.6"               # Bit flags
smallvec = "1.13"              # Small vector optimization
ahash = "0.8"                  # Fast hashing

[build-dependencies]
csbindgen = "1.2"
cbindgen = "0.27"

[dev-dependencies]
criterion = "0.5"              # Benchmarking
proptest = "1.5"               # Property testing
```

### 8.2 Version Compatibility Matrix

| Dependency | Min Version | Tested Version | Notes |
|------------|-------------|----------------|-------|
| Rust | 1.75.0 | 1.83.0 | MSRV for bevy_ecs |
| bevy_ecs | 0.15.0 | 0.15.0 | Latest stable |
| rapier2d/3d | 0.22.0 | 0.22.0 | Latest stable |
| rodio | 0.19.0 | 0.19.0 | Latest stable |
| .NET | 8.0 | 8.0 | LTS |

---

## 9. Risk Mitigation

| Risk | Mitigation |
|------|------------|
| bevy_ecs breaking changes | Pin to specific version, wrap in abstraction layer |
| rapier API changes | Use rapier's stable API only, version lock |
| OpenGL deprecation (macOS) | Backend abstraction enables Metal port |
| FFI complexity | Comprehensive testing, binding generation automation |
| Performance regression | Automated benchmarks in CI with alerts |
| Memory leaks | ASAN/Valgrind testing, handle tracking |

---

## 10. Backward Compatibility

### 10.1 Deprecation Strategy

```rust
// Old API (deprecated but functional)
#[deprecated(since = "2.0.0", note = "Use goud_context_create instead")]
#[no_mangle]
pub extern "C" fn game_create(
    width: u32,
    height: u32,
    title: *const c_char,
    target_fps: u32,
    renderer_type: c_int,
) -> *mut GameSdk {
    // Internally delegates to new API
    let config = GoudContextConfig {
        width, height,
        title: unsafe { CStr::from_ptr(title).to_str().unwrap_or("") }.to_string(),
        target_fps,
        renderer_type,
        enable_audio: false,
        enable_physics: false,
    };

    let mut handle = ContextHandle::default();
    if goud_context_create(&config, &mut handle) == GoudError::Ok {
        // Return raw pointer for compatibility
        Box::into_raw(Box::new(LegacyWrapper { handle }))
    } else {
        std::ptr::null_mut()
    }
}
```

### 10.2 Migration Path

1. **Phase 1:** New API available alongside old
2. **Phase 2:** Old API marked deprecated, emits warnings
3. **Phase 3:** Old API removed in next major version

---

## Appendix A: File-by-File Implementation Notes

### A.1 Priority 1 Files (Week 1-2)

| File | LOC Est. | Dependencies | Notes |
|------|----------|--------------|-------|
| `src/error.rs` | ~150 | thiserror | Error enum, GoudResult |
| `src/handles.rs` | ~200 | - | Generic handle system |
| `src/context.rs` | ~400 | parking_lot, crossbeam | Thread-safe context |
| `src/ffi/mod.rs` | ~50 | - | Module organization |
| `src/ffi/utils.rs` | ~100 | - | String conversion |

### A.2 Priority 2 Files (Week 3-4)

| File | LOC Est. | Dependencies | Notes |
|------|----------|--------------|-------|
| `src/ffi/context.rs` | ~200 | context.rs | Context FFI |
| `src/ffi/entities.rs` | ~400 | ecs | Entity FFI |
| `src/ffi/resources.rs` | ~200 | resources | Resource FFI |
| `src/ffi/rendering.rs` | ~300 | graphics | Camera/light FFI |
| `src/ffi/input.rs` | ~100 | platform | Input FFI |

### A.3 Priority 3 Files (Week 5-10)

| File | LOC Est. | Dependencies | Notes |
|------|----------|--------------|-------|
| `src/subsystems/ecs/mod.rs` | ~300 | bevy_ecs | ECS integration |
| `src/subsystems/audio/mod.rs` | ~400 | rodio | Audio system |
| `src/subsystems/physics/mod.rs` | ~500 | rapier | Physics system |
| `src/subsystems/graphics/backend.rs` | ~300 | - | Backend trait |

---

*Document Version: 1.0*
*Last Updated: 2026-01-04*
