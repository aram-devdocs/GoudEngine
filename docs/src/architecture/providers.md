# Provider System

## Introduction

Providers are swappable backend implementations for engine subsystems. Each subsystem — rendering, physics, audio, input, and windowing — is represented by a trait. At engine initialization, you supply concrete implementations. The rest of the engine uses those implementations through the trait interface without knowing which backend is active.

When to use the provider system:

- **Adding a new backend.** Implement the relevant provider trait and pass it to the builder. No engine internals need to change.
- **Headless testing.** Null providers ship for every subsystem. Tests that do not need real audio or a real window use null providers and avoid any platform dependency.
- **Platform-specific implementations.** NDA-bound or platform-restricted backends can be kept out of the public repository and injected via the Rust SDK builder.

The engine selects providers at startup and does not swap them at runtime in v1 (see [Design Decisions](#design-decisions)).

---

## Provider Trait Hierarchy

Every provider implements two supertraits: `Provider` and `ProviderLifecycle`. Subsystem traits extend both.

```
Provider + ProviderLifecycle
         |
         +-- RenderProvider
         +-- PhysicsProvider
         +-- AudioProvider
         +-- InputProvider

WindowProvider  (standalone — no Provider supertrait, see below)
```

### Provider

```rust
pub trait Provider: Send + Sync + 'static {
    fn name(&self) -> &str;
    fn version(&self) -> &str;
    fn capabilities(&self) -> Box<dyn std::any::Any>;
}
```

`name` and `version` identify the implementation (e.g., `"opengl"`, `"1.0.0"`). `capabilities` returns a type-erased value; prefer the typed accessor on the subsystem trait (e.g., `RenderProvider::render_capabilities()`) to avoid downcasting.

`Send + Sync + 'static` is required because `ProviderRegistry` may be accessed from worker threads during asset streaming.

### ProviderLifecycle

```rust
pub trait ProviderLifecycle {
    fn init(&mut self) -> GoudResult<()>;
    fn update(&mut self, delta: f32) -> GoudResult<()>;
    fn shutdown(&mut self);
}
```

The five lifecycle phases:

```
Create -> Init -> Update (per frame) -> Shutdown -> Drop
```

- `init` is called once during `GoudGame::new()`. Failure is fatal unless a fallback is configured.
- `update` is called once per frame. Providers that do not need per-frame work implement it as a no-op.
- `shutdown` is called during `GoudGame::drop()`. Must not fail; all GPU and OS resources must be released before `Drop`.

### Subsystem Traits

Each subsystem trait extends `Provider + ProviderLifecycle` and adds domain methods:

```rust
pub trait AudioProvider: Provider + ProviderLifecycle { ... }
pub trait PhysicsProvider: Provider + ProviderLifecycle { ... }
pub trait RenderProvider: Provider + ProviderLifecycle { ... }
pub trait InputProvider: Provider { ... }
```

`InputProvider` extends only `Provider` (not `ProviderLifecycle`) because input polling is handled through `update_input()`, a method on the trait itself, rather than through the generic lifecycle.

### WindowProvider Exception

`WindowProvider` does not extend `Provider` and has no `Send + Sync` bounds:

```rust
pub trait WindowProvider: 'static {
    fn name(&self) -> &str;
    fn init(&mut self) -> GoudResult<()>;
    fn shutdown(&mut self);
    fn should_close(&self) -> bool;
    fn set_should_close(&mut self, value: bool);
    fn poll_events(&mut self);
    fn swap_buffers(&mut self);
    fn get_size(&self) -> (u32, u32);
    fn get_framebuffer_size(&self) -> (u32, u32);
}
```

GLFW requires all window calls on the main thread. Making `WindowProvider` `!Send + !Sync` enforces this at the type level. As a consequence, `WindowProvider` is stored directly in `GoudGame` rather than in `ProviderRegistry`. See [Thread Safety](#thread-safety).

---

## Built-in Provider Reference

### RenderProvider

| Implementation | Feature | Notes |
|---|---|---|
| `OpenGLRenderProvider` | `native` | Wraps `OpenGLBackend` |
| `NullRenderProvider` | always | No-op, returns zero handles |

Key methods:

- `begin_frame() -> GoudResult<FrameContext>` — starts a frame, returns an opaque token
- `end_frame(frame: FrameContext) -> GoudResult<()>` — finalizes and presents; consumes the token
- `create_texture(desc) / destroy_texture(handle)` — GPU texture management
- `create_buffer / create_shader / create_pipeline / create_render_target` — resource creation with paired destroy methods
- `draw(cmd) / draw_batch(cmds)` — 2D sprite draws
- `draw_mesh / draw_text / draw_particles` — specialized draw paths
- `set_viewport / set_camera / set_render_target / clear` — render state

The `FrameContext` token returned by `begin_frame` must be passed to `end_frame`. This enforces correct frame pairing at compile time — you cannot present without first beginning, and you cannot call `begin_frame` twice without calling `end_frame` between.

Console partners should read [Console Render Backend Contract](console-render-provider.md) before writing a proprietary renderer. That guide maps each method to the engine's frame loop, resource lifetime rules, and the public integration points for windowing and command submission.

### AudioProvider

| Implementation | Feature | Notes |
|---|---|---|
| `RodioAudioProvider` | `audio` | Uses the rodio crate directly |
| `NullAudioProvider` | always | Silent no-op |

Key methods:

- `play(handle, config) -> GoudResult<PlaybackId>` — starts playback, returns a handle for the active instance
- `stop / pause / resume(id)` — control a specific instance
- `is_playing(id) -> bool`
- `set_volume(id, volume) / set_master_volume(volume) / set_channel_volume(channel, volume)`
- `set_listener_position(pos) / set_source_position(id, pos)` — spatial audio (3D positions as `[f32; 3]`)
- `audio_update()` — per-frame stream refill and listener sync

`RodioAudioProvider` wraps rodio directly (a Layer 1 dependency) rather than going through the `AudioManager` asset system. Layer 2 code bridges between the asset system and this provider as needed.

### InputProvider

| Implementation | Feature | Notes |
|---|---|---|
| `GlfwInputProvider` | `native` | State synced from `InputManager` each frame |
| `NullInputProvider` | always | All buttons unpressed, all axes zero |

Key methods:

- `update_input()` — process queued events and update state
- `key_pressed / key_just_pressed / key_just_released(key: KeyCode) -> bool`
- `mouse_position() -> [f32; 2]` — window coordinates
- `mouse_delta() -> [f32; 2]` — movement since last frame
- `mouse_button_pressed(button: MouseButton) -> bool`
- `scroll_delta() -> [f32; 2]`
- `gamepad_connected(id) / gamepad_axis(id, axis) / gamepad_button_pressed(id, button)`

`GlfwInputProvider` does not read GLFW state directly. It exposes a `sync_from_input_manager` method that Layer 2 code calls each frame to copy state from `InputManager`. This avoids a Layer 1 import of a Layer 2 type.

### PhysicsProvider

| Implementation | Feature | Notes |
|---|---|---|
| `Rapier2DPhysicsProvider` | `physics` | Wraps rapier2d for 2D rigid body simulation |
| `Rapier3DPhysicsProvider` | `physics` | Wraps rapier3d for 3D rigid body simulation |
| `NullPhysicsProvider` | always | No simulation; all queries return defaults |

Key methods:

- `step(delta)` — advance the simulation
- `set_gravity / gravity` — global gravity as `[f32; 2]`
- `create_body(desc) / destroy_body(handle)`
- `body_position / set_body_position / body_velocity / set_body_velocity`
- `apply_force / apply_impulse`
- `create_collider / destroy_collider`
- `raycast(origin, dir, max_dist) -> Option<RaycastHit>`
- `overlap_circle(center, radius) -> Vec<BodyHandle>`
- `drain_collision_events() -> Vec<CollisionEvent>` — returns owned `Vec` to avoid lifetime coupling with the provider borrow
- `create_joint / destroy_joint`
- `debug_shapes() -> Vec<DebugShape>`

All `Vec2`-like parameters use `[f32; 2]` arrays to avoid depending on external math types in the trait definition.

### WindowProvider

| Implementation | Feature | Notes |
|---|---|---|
| `GlfwWindowProvider` | `native` | Wraps `GlfwPlatform` |
| `NullWindowProvider` | always | No-op, `should_close` always false |

Key methods:

- `init() / shutdown()`
- `should_close() -> bool` / `set_should_close(value)`
- `poll_events()` — pump the OS event queue
- `swap_buffers()` — present the frame
- `get_size() -> (u32, u32)` — screen coordinates
- `get_framebuffer_size() -> (u32, u32)` — pixel coordinates (differs from `get_size` on high-DPI)

`GlfwWindowProvider::poll_events()` only calls GLFW event polling without dispatching to an input manager. Layer 2 code (`GoudGame`) calls `PlatformBackend::poll_events()` with an `InputManager` for full input dispatch.

---

## Implementing a Custom Provider

This example implements a custom `AudioProvider`. The same pattern applies to all other subsystem traits.

```rust
use goud_engine::core::error::GoudResult;
use goud_engine::core::providers::audio::AudioProvider;
use goud_engine::core::providers::types::{
    AudioCapabilities, AudioChannel, PlayConfig, PlaybackId, SoundHandle,
};
use goud_engine::core::providers::{Provider, ProviderLifecycle};

pub struct MyAudioProvider {
    capabilities: AudioCapabilities,
    master_volume: f32,
}

impl MyAudioProvider {
    pub fn new() -> Self {
        Self {
            capabilities: AudioCapabilities {
                supports_spatial: true,
                max_channels: 32,
            },
            master_volume: 1.0,
        }
    }
}

// Step 1: Implement the base Provider supertrait.
impl Provider for MyAudioProvider {
    fn name(&self) -> &str { "my-audio" }
    fn version(&self) -> &str { "1.0.0" }
    fn capabilities(&self) -> Box<dyn std::any::Any> {
        Box::new(self.capabilities.clone())
    }
}

// Step 2: Implement ProviderLifecycle.
impl ProviderLifecycle for MyAudioProvider {
    fn init(&mut self) -> GoudResult<()> {
        // Open audio device, allocate buffers, etc.
        Ok(())
    }

    fn update(&mut self, _delta: f32) -> GoudResult<()> {
        // Refill stream buffers, sync listener position, etc.
        Ok(())
    }

    fn shutdown(&mut self) {
        // Close device, free buffers. Must not fail.
    }
}

// Step 3: Implement the subsystem trait.
impl AudioProvider for MyAudioProvider {
    fn audio_capabilities(&self) -> &AudioCapabilities {
        &self.capabilities
    }

    fn audio_update(&mut self) -> GoudResult<()> {
        // Audio-specific per-frame work (distinct from the generic lifecycle update).
        Ok(())
    }

    fn play(&mut self, _handle: SoundHandle, _config: &PlayConfig) -> GoudResult<PlaybackId> {
        // Start playback. Return an ID for the active instance.
        Ok(PlaybackId(1))
    }

    fn stop(&mut self, _id: PlaybackId) -> GoudResult<()> { Ok(()) }
    fn pause(&mut self, _id: PlaybackId) -> GoudResult<()> { Ok(()) }
    fn resume(&mut self, _id: PlaybackId) -> GoudResult<()> { Ok(()) }
    fn is_playing(&self, _id: PlaybackId) -> bool { false }

    fn set_volume(&mut self, _id: PlaybackId, _volume: f32) -> GoudResult<()> { Ok(()) }
    fn set_master_volume(&mut self, volume: f32) { self.master_volume = volume; }
    fn set_channel_volume(&mut self, _channel: AudioChannel, _volume: f32) {}

    fn set_listener_position(&mut self, _pos: [f32; 3]) {}
    fn set_source_position(&mut self, _id: PlaybackId, _pos: [f32; 3]) -> GoudResult<()> { Ok(()) }
}
```

All three trait impls are required. There is no default implementation for any subsystem method — this is intentional so that each backend is explicit about what it supports.

---

## Registration and Swapping

`ProviderRegistry` holds one boxed trait object per subsystem. Build it with `ProviderRegistryBuilder`:

```rust
let registry = ProviderRegistryBuilder::new()
    .with_renderer(OpenGLRenderProvider::new(backend))
    .with_audio(RodioAudioProvider::new())
    .build();
// physics and input default to null providers
```

Any slot left unconfigured defaults to its null implementation. This lets you enable subsystems incrementally — a game with no physics needs no physics configuration.

`ProviderRegistry` fields are public and hold the concrete `Box<dyn XxxProvider>`:

```rust
pub struct ProviderRegistry {
    pub render:  Box<dyn RenderProvider>,
    pub physics: Box<dyn PhysicsProvider>,
    pub audio:   Box<dyn AudioProvider>,
    pub input:   Box<dyn InputProvider>,
}
```

`WindowProvider` is not in `ProviderRegistry`. It is stored directly in `GoudGame` because it is `!Send + !Sync`.

**FFI SDK users** select providers by enum at engine initialization rather than through the builder. Custom provider implementations require the Rust SDK. The capability query API in `ffi/providers.rs` handles provider selection for SDK consumers.

---

## Thread Safety

All provider traits except `WindowProvider` require `Send + Sync + 'static`. This allows `ProviderRegistry` to be accessed from worker threads — for example, during asset streaming where textures or sound data may be loaded on a background thread.

`WindowProvider` is `!Send + !Sync`. GLFW requires all window operations on the main thread, and there is no safe way to enforce this at runtime on arbitrary backends. The type system enforces it instead: `WindowProvider` lacks the `Provider` supertrait (which requires `Send + Sync`), and it is stored outside `ProviderRegistry` in `GoudGame` directly. This makes `GoudGame` itself `!Send` when a native window is present, matching the constraint of the underlying platform layer.

If an async executor is added to the engine in the future, `WindowProvider` calls must be scheduled on the main thread through a `MainThreadScheduler`.

---

## Layer Placement

All paths below are relative to `goud_engine/src/`.

```
Layer 1 — Foundation (core/)
  core/providers/            -- canonical trait definitions
  core/providers/registry.rs -- ProviderRegistry
  core/providers/builder.rs  -- ProviderRegistryBuilder
  core/providers/impls/      -- null implementations
                                (NullRenderProvider, NullAudioProvider,
                                 NullPhysicsProvider, NullInputProvider,
                                 NullWindowProvider)

Layer 2 — Libs (libs/)
  libs/providers/            -- re-exports core traits
  libs/providers/impls/      -- native implementations
                                (OpenGLRenderProvider, RodioAudioProvider,
                                 GlfwWindowProvider, GlfwInputProvider)

Layer 5 — FFI (ffi/)
  ffi/providers.rs           -- enum selection for SDK initialization

External — SDKs (sdks/)
  generated from goud_sdk.schema.json  -- C#, Python, TypeScript wrappers
```

Provider trait definitions are canonical at Layer 1 (`core/providers/`). Layer 2 (`libs/providers/`) re-exports those traits and provides native implementations. This allows native implementations to depend on the trait definitions without upward imports — the `libs/providers/mod.rs` shim explicitly states that `crate::core::providers` is the canonical source. `ProviderRegistry` and the builder also live at Layer 1 because they are foundational types alongside `GoudError` and `GoudResult`.

---

## Design Decisions

The provider system was designed in RFC-0001 and extended for networking in RFC-0002. Key decisions:

**Object-safe dynamic dispatch.** All traits are object-safe (no associated types, no generic methods) and stored as `Box<dyn XxxProvider>`. Dynamic dispatch is acceptable because provider calls are coarse-grained — per-frame or per-batch, not per-vertex. The internal `RenderBackend` trait, which is not object-safe, remains an implementation detail inside concrete render providers.

**Explicit null providers.** Each subsystem has an explicit `Null*Provider` struct rather than relying on `Option` wrappers or default method implementations. Null providers are visible in the registry, debuggable, and testable. The `NullRenderProvider::name()` returns `"null"`, which makes it easy to detect misconfigured tests or games.

**Provider-owned resources.** Providers own their GPU and OS resources. There is no shared resource pool across providers. Handles from one provider are invalid with another.

**No hot-swap in release builds.** Providers cannot be swapped at runtime in a shipping build. In dev mode, hot-swap is supported using generational handle invalidation. The constraint: the replacement must pass `init()` before the old provider is dropped, and all existing handles must produce errors (not UB) after the swap.

**Vec2 as `[f32; 2]`.** Physics and audio traits use `[f32; 2]` and `[f32; 3]` instead of a named vector type. This avoids a dependency on any particular math library in the trait definitions. Concrete implementations convert to their internal types at the boundary.

See [RFC-0001](../../rfcs/RFC-0001-provider-trait-pattern.md) for the full rationale and alternatives considered. See [RFC-0002](../../rfcs/RFC-0002-network-provider-trait.md) for the planned `NetworkProvider` extension.
