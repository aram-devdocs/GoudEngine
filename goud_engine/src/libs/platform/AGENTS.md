# platform/ -- Platform Abstraction Layer

## Purpose

Abstracts windowing, input polling, and buffer presentation behind the
`PlatformBackend` trait. The engine picks a concrete backend at startup;
higher layers never touch platform-specific APIs directly.

## Files

- `mod.rs` -- `WindowConfig` struct, `PlatformBackend` trait, unit tests
- `glfw_platform.rs` -- GLFW + OpenGL 3.3 Core backend (desktop only)
- `winit_platform.rs` -- winit backend (desktop now, web later; requires `wgpu-backend` + `native` features)

## PlatformBackend Trait

```rust
trait PlatformBackend {
    fn should_close(&self) -> bool;
    fn set_should_close(&mut self, should_close: bool);
    fn poll_events(&mut self, input: &mut InputManager) -> f32; // returns delta time
    fn swap_buffers(&mut self);
    fn get_size(&self) -> (u32, u32);            // logical window size
    fn get_framebuffer_size(&self) -> (u32, u32); // physical pixel size (HiDPI)
}
```

## Feature Gates

- `GlfwPlatform` requires `#[cfg(feature = "native")]`
- `WinitPlatform` requires `#[cfg(all(feature = "wgpu-backend", feature = "native"))]`

## Gotchas

- GLFW uses a `thread_local!` singleton -- only one `GlfwPlatform` per thread
- GLFW must run on the main thread. Neither backend is `Send` or `Sync`
- `WinitPlatform` maps winit key codes directly to the engine's platform-neutral
  input enums
- `WinitPlatform::swap_buffers()` is a no-op -- wgpu handles presentation
- `get_framebuffer_size()` can differ from `get_size()` on Retina/HiDPI displays

## Dependencies

Layer 1 (Core). Imports from `crate::core::error`, `crate::core::math`,
and `crate::ecs::InputManager`. NEVER imports from Layer 2+ (engine, ffi, sdk).
