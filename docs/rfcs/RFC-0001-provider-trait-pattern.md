---
rfc: "0001"
title: Provider Trait Pattern
status: accepted
created: 2026-03-06
authors: ["aram-devdocs"]
tracking-issue: "#217"
---

# RFC-0001: Provider Trait Pattern

## 1. Summary

This RFC defines a universal provider abstraction for all GoudEngine subsystems. It replaces the hardcoded `OpenGLBackend` in `GoudGame` with configurable, swappable providers selected at engine initialization. The pattern applies uniformly to rendering, physics, audio, windowing, and input. SDK users select built-in providers via enums; Rust SDK users may supply custom implementations.

---

## 2. Motivation

`GoudGame` in `goud_engine/src/sdk/game/instance.rs` currently holds:

```rust
#[cfg(feature = "native")]
render_backend: Option<OpenGLBackend>,
#[cfg(feature = "native")]
sprite_batch: Option<SpriteBatch<OpenGLBackend>>,
```

This hardcoding creates several problems:

- Adding a `wgpu` backend requires duplicating every code path that touches `backend` and `sprite_batch`.
- Physics, audio, and windowing have the same problem — there is no swap point.
- Cross-platform targets (consoles, mobile) need NDA-bound backends that cannot ship in the public repo. There is no way to inject them without forking engine internals.
- Runtime renderer selection (e.g., falling back from Vulkan to OpenGL) is not possible.

`PlatformBackend` in `goud_engine/src/libs/platform/mod.rs` already solves this for the platform layer: `GoudGame` stores `Option<Box<dyn PlatformBackend>>`. This RFC extends that pattern to all subsystems.

---

## 3. Design

### 3.1 Provider Supertrait

All providers implement a common base trait:

```rust
pub trait Provider: Send + Sync + 'static {
    fn name(&self) -> &str;
    fn version(&self) -> &str;
    fn capabilities(&self) -> Box<dyn std::any::Any>;
}
```

`Send + Sync + 'static` is required because providers are stored in `ProviderRegistry`, which may be accessed from worker threads during asset streaming. The exception is `WindowProvider` (see §3.7). Subsystem traits extend `Provider` with domain-specific methods.

---

### 3.2 Subsystem Provider Traits

#### RenderProvider

```rust
pub trait RenderProvider: Provider {
    // Lifecycle — FrameContext token enforces begin/end pairing
    fn begin_frame(&mut self) -> GoudResult<FrameContext>;
    fn end_frame(&mut self, frame: FrameContext) -> GoudResult<()>;
    fn resize(&mut self, width: u32, height: u32) -> GoudResult<()>;
    // Resources
    fn create_texture(&mut self, desc: &TextureDesc) -> GoudResult<TextureHandle>;
    fn destroy_texture(&mut self, handle: TextureHandle);
    fn create_buffer(&mut self, desc: &BufferDesc) -> GoudResult<BufferHandle>;
    fn destroy_buffer(&mut self, handle: BufferHandle);
    fn create_shader(&mut self, desc: &ShaderDesc) -> GoudResult<ShaderHandle>;
    fn destroy_shader(&mut self, handle: ShaderHandle);
    fn create_pipeline(&mut self, desc: &PipelineDesc) -> GoudResult<PipelineHandle>;
    fn destroy_pipeline(&mut self, handle: PipelineHandle);
    fn create_render_target(&mut self, desc: &RenderTargetDesc) -> GoudResult<RenderTargetHandle>;
    fn destroy_render_target(&mut self, handle: RenderTargetHandle);
    // Drawing
    fn draw(&mut self, cmd: &DrawCommand) -> GoudResult<()>;
    fn draw_batch(&mut self, cmds: &[DrawCommand]) -> GoudResult<()>;
    fn draw_mesh(&mut self, cmd: &MeshDrawCommand) -> GoudResult<()>;
    fn draw_text(&mut self, cmd: &TextDrawCommand) -> GoudResult<()>;
    fn draw_particles(&mut self, cmd: &ParticleDrawCommand) -> GoudResult<()>;
    // State
    fn set_viewport(&mut self, x: i32, y: i32, width: u32, height: u32);
    fn set_camera(&mut self, camera: &CameraData);
    fn set_render_target(&mut self, handle: Option<RenderTargetHandle>);
    fn clear(&mut self, color: [f32; 4]);
}
```

`FrameContext` is an opaque token returned by `begin_frame` and consumed by `end_frame`, ensuring the caller cannot skip frame finalization. `PipelineDesc` abstracts render pipeline state (shader + vertex layout + blend mode) required by wgpu; OpenGL providers can map this to their internal state tracking.

| Built-in | Feature Flag | Notes |
|----------|-------------|-------|
| `OpenGLRenderProvider` | `native` | Wraps existing `OpenGLBackend` |
| `WgpuRenderProvider` | `wgpu` | Future (F02-03) |
| `NullRenderProvider` | always | No-op, for headless tests |

The existing `RenderBackend` trait (`goud_engine/src/libs/graphics/backend/render_backend.rs`) is NOT object-safe by design. It becomes an internal detail of `OpenGLRenderProvider` and `WgpuRenderProvider` and does not appear in the public provider API.

#### PhysicsProvider

```rust
pub trait PhysicsProvider: Provider {
    fn step(&mut self, delta: f32) -> GoudResult<()>;
    fn set_gravity(&mut self, gravity: Vec2);
    fn gravity(&self) -> Vec2;
    fn create_body(&mut self, desc: &BodyDesc) -> GoudResult<BodyHandle>;
    fn destroy_body(&mut self, handle: BodyHandle);
    fn body_position(&self, handle: BodyHandle) -> GoudResult<Vec2>;
    fn set_body_position(&mut self, handle: BodyHandle, pos: Vec2) -> GoudResult<()>;
    fn body_velocity(&self, handle: BodyHandle) -> GoudResult<Vec2>;
    fn set_body_velocity(&mut self, handle: BodyHandle, vel: Vec2) -> GoudResult<()>;
    fn apply_force(&mut self, handle: BodyHandle, force: Vec2) -> GoudResult<()>;
    fn apply_impulse(&mut self, handle: BodyHandle, impulse: Vec2) -> GoudResult<()>;
    fn create_collider(&mut self, body: BodyHandle, desc: &ColliderDesc) -> GoudResult<ColliderHandle>;
    fn destroy_collider(&mut self, handle: ColliderHandle);
    fn raycast(&self, origin: Vec2, dir: Vec2, max_dist: f32) -> Option<RaycastHit>;
    fn overlap_circle(&self, center: Vec2, radius: f32) -> Vec<BodyHandle>;
    fn drain_collision_events(&mut self) -> Vec<CollisionEvent>;
    fn contact_pairs(&self) -> Vec<ContactPair>;
    fn create_joint(&mut self, desc: &JointDesc) -> GoudResult<JointHandle>;
    fn destroy_joint(&mut self, handle: JointHandle);
    fn debug_shapes(&self) -> Vec<DebugShape>;
}
```

`drain_collision_events` returns owned `Vec` rather than a slice reference to avoid lifetime coupling between the event buffer and the provider borrow — callers process events after the physics step without holding a borrow on the provider.

| Built-in | Feature Flag | Notes |
|----------|-------------|-------|
| `Rapier2DPhysicsProvider` | `rapier2d` | 2D rigid-body physics |
| `Rapier3DPhysicsProvider` | `rapier3d` | 3D rigid-body physics |
| `SimplePhysicsProvider` | always | AABB collision + gravity only, no rapier dependency |
| `NullPhysicsProvider` | always | No-op passthrough |

#### AudioProvider

```rust
pub trait AudioProvider: Provider {
    fn update(&mut self) -> GoudResult<()>;
    fn play(&mut self, handle: SoundHandle, config: &PlayConfig) -> GoudResult<PlaybackId>;
    fn stop(&mut self, id: PlaybackId) -> GoudResult<()>;
    fn pause(&mut self, id: PlaybackId) -> GoudResult<()>;
    fn resume(&mut self, id: PlaybackId) -> GoudResult<()>;
    fn is_playing(&self, id: PlaybackId) -> bool;
    fn set_volume(&mut self, id: PlaybackId, volume: f32) -> GoudResult<()>;
    fn set_master_volume(&mut self, volume: f32);
    fn set_channel_volume(&mut self, channel: AudioChannel, volume: f32);
    fn set_listener_position(&mut self, pos: Vec3);
    fn set_source_position(&mut self, id: PlaybackId, pos: Vec3) -> GoudResult<()>;
}
```

| Built-in | Feature Flag | Notes |
|----------|-------------|-------|
| `RodioAudioProvider` | `audio` | Uses existing rodio integration |
| `WebAudioProvider` | `web` | Browser/WASM via web-sys |
| `NullAudioProvider` | always | No-op, for CI / headless |

#### WindowProvider

`WindowProvider` extracts the surface-management part of `PlatformBackend`. It is NOT `Send + Sync` because GLFW requires all window calls on the main thread (see §3.7).

```rust
pub trait WindowProvider: 'static {  // NOT Send + Sync
    fn should_close(&self) -> bool;
    fn set_should_close(&mut self, value: bool);
    fn poll_events(&mut self);
    fn swap_buffers(&mut self);
    fn get_size(&self) -> (u32, u32);
    fn get_framebuffer_size(&self) -> (u32, u32);
}
```

| Built-in | Feature Flag | Notes |
|----------|-------------|-------|
| `GlfwWindowProvider` | `native` | Wraps existing GLFW platform layer |
| `WinitWindowProvider` | `winit` | Future — needed for mobile/web targets |
| `NullWindowProvider` | always | No-op for headless contexts |

GLFW is the current platform layer (`goud_engine/src/libs/platform/glfw_platform.rs`). The roadmap targets winit for broader platform support (mobile, web); `WinitWindowProvider` will be added when that migration begins.

#### InputProvider

Extracted from `PlatformBackend` separately because input has a different update cadence and can be mocked without a real window (e.g., test harnesses injecting synthetic events).

```rust
pub trait InputProvider: Provider {
    fn key_pressed(&self, key: KeyCode) -> bool;
    fn key_just_pressed(&self, key: KeyCode) -> bool;
    fn key_just_released(&self, key: KeyCode) -> bool;
    fn mouse_position(&self) -> Vec2;
    fn mouse_button_pressed(&self, button: MouseButton) -> bool;
}
```

| Built-in | Feature Flag | Notes |
|----------|-------------|-------|
| `GlfwInputProvider` | `native` | Reads from GLFW event queue |
| `NullInputProvider` | always | All buttons unpressed |

---

### 3.3 Lifecycle Protocol

Every provider follows a five-phase lifecycle:

1. **Create** — constructed with a config struct (`RenderConfig`, `PhysicsConfig`, etc.)
2. **Init** — `init()` called during `GoudGame::new()`. Failure is fatal unless a fallback is configured (§3.9).
3. **Update** — per-frame `update(delta)` for providers that need it (physics, audio).
4. **Shutdown** — `shutdown()` called during `GoudGame::drop()`. Must not fail.
5. **Drop** — provider dropped after shutdown. All GPU/OS resources must be released before this point.

```rust
pub trait ProviderLifecycle {
    fn init(&mut self) -> GoudResult<()>;
    fn update(&mut self, delta: f32) -> GoudResult<()>;
    fn shutdown(&mut self);
}
```

All subsystem provider traits (`RenderProvider`, `PhysicsProvider`, etc.) extend both `Provider` and `ProviderLifecycle`. For example: `pub trait RenderProvider: Provider + ProviderLifecycle { ... }`.

---

### 3.4 Capability Query Pattern

Each subsystem defines a typed capability struct, building on the existing `BackendCapabilities` pattern in `goud_engine/src/libs/graphics/backend/capabilities.rs`:

```rust
pub struct RenderCapabilities {
    pub max_texture_units: u32,
    pub max_texture_size: u32,
    pub supports_instancing: bool,
    pub supports_compute: bool,
    pub supports_msaa: bool,
}
```

The engine downcasts and checks capabilities before using optional features:

```rust
let caps = render_provider.capabilities()
    .downcast::<RenderCapabilities>()
    .map_err(|_| GoudError::ProviderError {
        subsystem: "render",
        message: "capabilities type mismatch".into(),
    })?;
if caps.supports_instancing {
    renderer.use_instanced_path();
} else {
    renderer.use_fallback_path();
}
```

---

### 3.5 Registration and Selection

`ProviderRegistry` lives at Layer 2 (`goud_engine/src/core/providers/registry.rs`):

```rust
pub struct ProviderRegistry {
    pub render: Box<dyn RenderProvider>,
    pub physics: Box<dyn PhysicsProvider>,
    pub audio: Box<dyn AudioProvider>,
    pub input: Box<dyn InputProvider>,
    // WindowProvider is !Send+Sync, stored separately in GoudGame.
}
```

**Rust SDK** — builder pattern with `Null*Provider` defaults for unconfigured slots:

```rust
let game = GoudEngine::builder()
    .with_renderer(OpenGLRenderProvider::new(RenderConfig::default()))
    .with_physics(Rapier2DPhysicsProvider::new(PhysicsConfig {
        gravity: Vec2::new(0.0, -9.81), ..Default::default()
    }))
    .with_audio(RodioAudioProvider::new(AudioConfig::default()))
    .with_window(GlfwWindowProvider::new(WindowConfig {
        width: 1280, height: 720, title: "My Game".into(),
    }))
    .build()?;
```

**FFI SDKs** — enum-based selection (custom providers require the Rust SDK):

```rust
#[repr(C)]
pub enum GoudRendererType {
    WgpuAuto = 0,     // Auto-select best wgpu backend
    WgpuVulkan = 1,
    WgpuMetal = 2,
    WgpuDx12 = 3,
    WgpuWebGpu = 4,
    OpenGL = 10,
    Null = 99,
}

#[repr(C)]
pub enum GoudPhysicsType { Rapier2D = 0, Rapier3D = 1, Simple = 2, Null = 99 }

#[repr(C)]
pub enum GoudAudioType { Rodio = 0, WebAudio = 1, Null = 99 }
```

The renderer enum exposes wgpu backend sub-selection to allow SDK users to force a specific GPU API when needed (e.g., Vulkan for Linux, Metal for macOS).

---

### 3.6 Object Safety and Dispatch Strategy

All provider traits are object-safe: no associated types, no generic methods. Stored as `Box<dyn RenderProvider>` etc. Dynamic dispatch overhead is acceptable here because calls are coarse-grained (per-frame or per-batch), not per-vertex.

The existing `RenderBackend` trait (`goud_engine/src/libs/graphics/backend/render_backend.rs`) is intentionally NOT object-safe and remains an internal detail inside concrete providers. This mirrors `AssetLoader`/`ErasedAssetLoader` in `goud_engine/src/assets/loader/traits.rs` (same crate): typed generics for hot paths, erased trait objects for storage.

For performance-critical inner loops needing direct backend access, providers expose typed accessors:

```rust
impl OpenGLRenderProvider {
    pub(crate) fn backend(&mut self) -> &mut OpenGLBackend { ... }
}
```

---

### 3.7 Thread Safety

Default bound: `Send + Sync + 'static` on all provider traits.

**Exception: `WindowProvider`.** GLFW requires main-thread access, so `WindowProvider` is `!Send + !Sync`. The engine enforces this by storing it outside `ProviderRegistry` directly in `GoudGame`, making `GoudGame` itself `!Send` when a native window is present. This matches `PlatformBackend` in `goud_engine/src/libs/platform/mod.rs`.

---

### 3.8 Hot-Swap (Dev Mode)

Dev-mode only, gated behind `#[cfg(debug_assertions)]` or `dev-tools` feature. Protocol:

1. `shutdown()` on active provider → drop it
2. Create and `init()` replacement provider
3. Invalidate all resource handles (textures, buffers, shaders) from old provider
4. Trigger one-frame resource re-upload

Providers declare support with `fn supports_hot_swap(&self) -> bool` (default `false`). Implementation tracked in F02-08; this RFC defines constraints only.

**Constraints:** handle invalidation must produce errors (not silent UB), hot-swap must not be called from multiple threads, and replacement must pass `init()` before the old provider is dropped.

---

### 3.9 Error Handling

All fallible provider methods return `GoudResult<T>`. A new variant is added to `GoudError` (`goud_engine/src/core/error/types.rs`):

```rust
ProviderError { subsystem: &'static str, message: String }
```

This uses a struct variant (unlike the existing tuple variants like `InitializationFailed(String)`) because provider errors need the `subsystem` discriminator for FFI error code routing — error codes 600–609 for render, 610–619 for physics, etc. A single `String` would require parsing to extract the subsystem.

If `init()` fails and no fallback is configured, `GoudGame::new()` returns `Err(GoudError::ProviderError { ... })`. Fallback providers can be configured via `.with_fallback_renderer(NullRenderProvider::new())`.

---

### 3.10 Layer Placement

`libs/` is a module within `goud_engine/src/libs/`, not a standalone workspace crate. Per CLAUDE.md, `libs/` is Layer 1 (lowest) and must not import from Layer 2 (`core/`, `assets/`, `sdk/`) or higher.

| Component | Layer | Path |
|-----------|-------|------|
| Provider trait definitions | Layer 1 | `goud_engine/src/libs/providers/` (new module) |
| Concrete implementations | Layer 1 | `goud_engine/src/libs/providers/impls/` |
| `ProviderRegistry` | Layer 2 | `goud_engine/src/core/providers/registry.rs` |
| Builder (`GoudEngine::builder()`) | Layer 2 | `goud_engine/src/core/providers/builder.rs` |
| FFI enum selection | Layer 3 | `goud_engine/src/ffi/providers.rs` |
| SDK enum wrappers | Layer 4 | generated via codegen from `goud_sdk.schema.json` |

Provider traits in `goud_engine/src/libs/providers/` may import from sibling Layer 1 modules (`libs/graphics/`, `libs/ecs/`) but must not import from `core/`, `sdk/`, or `ffi/` — those are Layer 2+ and importing them would violate the downward-only rule.

**Prerequisite: error type placement.** Provider trait methods return `GoudResult<T>`, but `GoudResult` and `GoudError` currently live in `goud_engine/src/core/error/` (Layer 2). Existing `libs/` modules already import from `core/error/` (e.g., `libs/graphics/backend/` uses `GoudResult`), which is an existing Layer 1→2 violation. Before implementing this RFC, error types must be moved to a Layer 1 location (e.g., `libs/error/`) so that provider traits can reference them without upward imports. This is tracked as a prerequisite for F02-02.

---

### 3.11 FFI Boundary

SDK users never interact with provider traits. The FFI exposes enum parameters on init, capability query functions returning `#[repr(C)]` structs, and no provider handles. The high-level API (`draw_sprite`, `play_sound`) is unchanged.

```rust
#[no_mangle]
pub unsafe extern "C" fn goud_game_create(
    width: u32, height: u32, title: *const c_char,
    renderer_type: GoudRendererType,
    physics_type: GoudPhysicsType,
    audio_type: GoudAudioType,
) -> *mut GoudGame { ... }
```

---

## 4. Alternatives Considered

### Feature-flag-only selection (compile-time)

Selecting the backend at compile time with `#[cfg(feature = "opengl")]` / `#[cfg(feature = "wgpu")]` avoids dynamic dispatch. It does not support runtime fallback, cannot support NDA backends that cannot be checked in, and requires separate binaries for each backend configuration. Rejected.

### Dynamic plugin system (.so/.dll loading)

Loading provider implementations from shared libraries at runtime would allow third-party backends without engine recompilation. It introduces significant complexity: platform differences in library loading, symbol resolution, versioning, and safety. The marginal benefit does not justify the cost for an engine at this stage. Rejected; revisit post-1.0.

### Enum dispatch instead of trait objects

Wrapping all built-in providers in an enum and dispatching with `match` avoids dynamic dispatch costs. It prevents custom providers entirely and grows the match arms with every new backend. It also does not solve the NDA backend problem. Rejected.

### Merge Window+Input into a single PlatformProvider

`PlatformBackend` currently handles both windowing and input together. Keeping them merged is simpler. However, input and windowing have different threading and testing requirements: input can be mocked without a real window, but a window cannot exist without being on the main thread. Splitting them enables cleaner headless testing. The split is the recommended design (see §3.2); merging is noted as an open question (§6) if implementation complexity proves too high.

---

## 5. Impact

### Breaking Changes

- `GoudGame` struct changes: `Option<OpenGLBackend>` and `SpriteBatch<OpenGLBackend>` fields are replaced by `ProviderRegistry` and `Option<Box<dyn WindowProvider>>`.
- `SpriteBatch<B: RenderBackend>` generic parameter changes: `SpriteBatch` will receive a `&mut dyn RenderProvider` or a concrete backend reference via downcast. The public API surface of `SpriteBatch` may change.
- `GoudGame::new(width, height, title, renderer_type)` signature expands to accept physics and audio provider selections.

### FFI Changes

- `goud_game_create` gains `GoudPhysicsType` and `GoudAudioType` parameters.
- New capability query functions added to the FFI surface.
- C# bindings regenerated via csbindgen after `cargo build`.

### SDK Changes

- All three SDK wrappers (C#, Python, TypeScript) updated via codegen from the schema.
- Init functions gain physics and audio type parameters.
- Existing game code that uses the default `OpenGL` + `Null` physics + `Null` audio continues to work with updated init calls.

### Examples

- All C# examples in `examples/csharp/` updated to pass the new init parameters.
- Python and TypeScript examples updated in parallel.

### Migration Path

Implementation proceeds in phases F02-02 through F02-09 as defined in `ALPHA_ROADMAP.md`:

- F02-02: Define `libs/providers/` module with all trait definitions.
- F02-03: Implement `OpenGLRenderProvider` wrapping the existing backend.
- F02-04: Implement `Rapier2DPhysicsProvider`.
- F02-05: Implement `RodioAudioProvider`.
- F02-06: Split `PlatformBackend` into `WindowProvider` + `GlfwInputProvider`.
- F02-07: Wire `ProviderRegistry` into `GoudGame`, remove hardcoded fields.
- F02-08: Hot-swap mechanism (dev-tools feature only).
- F02-09: FFI enum selection + SDK codegen updates.

---

## 6. Open Questions

1. **Window+Input split vs. merge**: Should `WindowProvider` and `InputProvider` remain separate traits, or is a single `PlatformProvider` with both responsibilities simpler? The split enables better headless testing but adds an interface boundary. Decision needed before F02-06.

2. **Shared resource pool vs. provider-owned resources**: Should textures and buffers be owned by the render provider, or by a separate `ResourcePool` that the provider borrows from? Provider ownership is simpler; a shared pool enables multi-backend scenarios (e.g., a compute provider sharing buffers with a render provider). Deferred to post-provider-trait stabilization.

3. **NullProvider as trait default methods or separate struct**: Default no-op behavior could live as provided methods on the trait (e.g., `fn update(&mut self, _delta: f32) -> GoudResult<()> { Ok(()) }`), or in an explicit `Null*Provider` struct. Default methods reduce boilerplate for providers that do not need every lifecycle call; explicit structs are more visible and testable. Current leaning: explicit `Null*Provider` structs, no default method implementations.

4. **Async context and main-thread window access**: If GoudEngine adds async systems in the future, `WindowProvider`'s main-thread requirement becomes a scheduling constraint. The current design pins `GoudGame` as `!Send` when a native window is present. This may conflict with an async executor that migrates tasks across threads. No resolution proposed here; flagged for the async systems design.
