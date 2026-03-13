# wasm/ — WebAssembly Browser Target

## Purpose

Exposes the engine to browsers via `wasm-bindgen`. Provides a `WasmGame` handle that
JavaScript/TypeScript code uses for ECS, input, collision detection, and rendering.
Uses wgpu (WebGPU/WebGL) instead of the native OpenGL backend.

## Architecture

```
Browser JS/TS  -->  wasm_bindgen exports  -->  ECS World + wgpu renderer
  (canvas,           (this module)            (entities, components,
   events,                                     sprites, textures)
   rAF loop)
```

The browser owns the game loop (requestAnimationFrame). JS calls `begin_frame()` and
`end_frame()` each tick, with input events pushed in between.

## Files

- `mod.rs` — `WasmGame` struct and core lifecycle (construction, frame timing, basic
  ECS ops, input state, rendering). This is the main `#[wasm_bindgen]` export.
- `ecs_ops.rs` — Entity/component CRUD: spawn, despawn, Transform2D, Sprite, Name
- `input.rs` — Key/mouse state queries and action mapping
- `collision.rs` — AABB, circle-circle, circle-AABB collision (pure math, no GL)
- `rendering.rs` — Sprite drawing, texture management, render stats
- `sprite_renderer.rs` — Immediate-mode wgpu sprite batcher. Collects draw calls,
  flushes in a single render pass per frame. Batches by texture.
- `texture_loader.rs` — Async texture loading via browser Fetch API + `image` crate
  decoding + wgpu upload

## Two Construction Modes

- `WasmGame::new(w, h, title)` — ECS-only, no rendering (headless/logic-only)
- `WasmGame::create_with_canvas(canvas, w, h, title)` — Full rendering via wgpu
  attached to an HTML canvas element

## Rendering Backend

Uses wgpu (not OpenGL). The sprite renderer (`sprite_renderer.rs`) is a standalone
immediate-mode batcher with its own vertex format, pipeline, and texture bind groups.
Not shared with the native OpenGL renderer in `libs/graphics/`.

## Testing

Tests are gated with `#[cfg(all(test, target_arch = "wasm32"))]`. They cover ECS ops,
input state, frame timing, and canvas resize. Rendering tests require a browser
environment and are not run in standard `cargo test`.

## Dependencies

Imports from `core::math`, `ecs::components`, `ecs::Entity`, `ecs::World`, and
`assets::loaders`. External: `wasm-bindgen`, `web-sys`, `wgpu`, `bytemuck`, `image`.

## Gotchas

- This module is only compiled with `--target wasm32-unknown-unknown`
- The `web` feature flag controls panic hook setup
- Texture handles are 1-based (0 is the white fallback texture for `draw_quad`)
- Input state is push-based: JS event handlers call `press_key()`, `release_key()`, etc.
