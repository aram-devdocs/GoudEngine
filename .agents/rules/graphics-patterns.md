---
globs:
  - "**/graphics/**"
  - "**/renderer*"
---

# Graphics Subsystem Patterns

## Architecture

The default render backend is wgpu (`wgpu-backend` feature, enabled by the `native` default feature). OpenGL 3.3 is available as a legacy fallback via the `legacy-glfw-opengl` feature flag.

- **Renderer trait** abstracts the rendering backend — all renderers implement this trait
- **Renderer2D**: sprites, 2D camera (orthographic), Tiled map support
- **Renderer3D**: 3D primitives, dynamic lighting, 3D camera (perspective)
- **SpriteBatch** batches draw calls to minimize GPU state changes
- Renderer type selected at `GoudGame` initialization time

## GPU Backend Isolation

All raw GPU calls (`wgpu::` for the default wgpu backend, `gl::` for the legacy OpenGL backend) MUST live in `libs/graphics/backend/`. No other module may call GPU APIs directly. This ensures:

- The active backend is selected at engine initialization via `RenderBackendKind`
- GPU state management stays centralized
- Easier to test non-GPU logic in isolation

## Shader Programs

- Compile vertex + fragment shaders, then link into a program
- Always validate after linking (check link status via backend API)
- Shader source loaded via asset system, not hardcoded strings

## Camera System

- `Camera2D`: orthographic projection, position + zoom
- `Camera3D`: perspective projection, position + target + up vector
- Cameras are separate from renderers — a renderer receives a camera reference

## Testing

- Tests that need a GL context: use `test_helpers::init_test_context()`
- Math-only tests (matrix calculations, projections): no GL context needed
- Texture tests may require valid image files in `assets/`
