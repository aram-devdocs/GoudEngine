# wasm/ — WebAssembly Browser Target

Browser entry point via `wasm-bindgen`. Uses wgpu backend (not OpenGL).

## Key Files

- `mod.rs` — `WasmGame` struct and lifecycle
- `ecs_ops.rs` — Entity/component CRUD
- `input.rs` — Key/mouse state queries
- `collision.rs` — Pure math collision detection
- `rendering.rs` — Sprite drawing and texture management
- `sprite_renderer.rs` — wgpu sprite batcher (not shared with native GL renderer)

## Construction

- `WasmGame::new(w, h, title)` — ECS-only (headless)
- `WasmGame::create_with_canvas(canvas, w, h, title)` — Full rendering to canvas

## Gotchas

- Target: `wasm32-unknown-unknown` only
- Texture handles are 1-based (0 = white fallback)
- Input is push-based: JS calls `press_key()`, `release_key()`
