# libs/graphics/ — Graphics Subsystem

## Purpose

Rendering infrastructure: 2D sprite batching, 3D rendering, and OpenGL backend abstraction.

## Files

- `sprite_batch.rs` — SpriteBatch for efficient 2D rendering (batches draw calls)
- `renderer3d.rs` — 3D renderer (primitives, dynamic lighting, 3D camera)
- `backend/` — OpenGL backend (all raw GL calls live here)
- `mod.rs` — Renderer trait definition and module re-exports

## Patterns

- `Renderer` trait abstracts the backend — future backends can swap in
- SpriteBatch minimizes state changes by batching draw calls
- 2D uses orthographic camera; 3D uses perspective camera
- All raw OpenGL calls MUST go through `backend/` module

## Anti-Patterns

- NEVER make raw `gl::` calls outside `backend/` module
- NEVER mix 2D and 3D rendering in the same pass
- NEVER create GPU resources without corresponding cleanup

## Testing

- Tests requiring GL context MUST use `test_helpers::init_test_context()`
- Math/logic tests (camera matrices, batch sorting) should NOT require GL
- Shader compilation tests need a valid GL context

## Dependencies

Layer 1 (Core). No imports from ecs/, assets/, ffi/, or sdk/.
