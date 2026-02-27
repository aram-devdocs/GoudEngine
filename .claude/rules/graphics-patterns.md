---
globs:
  - "**/graphics/**"
  - "**/renderer*"
---

# Graphics Subsystem Patterns

## Architecture

- **Renderer trait** abstracts the rendering backend — all renderers implement this trait
- **Renderer2D**: sprites, 2D camera (orthographic), Tiled map support
- **Renderer3D**: 3D primitives, dynamic lighting, 3D camera (perspective)
- **SpriteBatch** batches draw calls to minimize GPU state changes
- Renderer type selected at `GoudGame` initialization time

## OpenGL Isolation

All raw OpenGL calls (`gl::` namespace) MUST live in `libs/graphics/backend/`. No other module may call OpenGL directly. This ensures:

- Backend can be swapped (Vulkan, Metal) without touching higher layers
- GL state management stays centralized
- Easier to test non-GL logic in isolation

## Shader Programs

- Compile vertex + fragment shaders, then link into a program
- Always validate after linking (check `GL_LINK_STATUS`)
- Shader source loaded via asset system, not hardcoded strings

## Camera System

- `Camera2D`: orthographic projection, position + zoom
- `Camera3D`: perspective projection, position + target + up vector
- Cameras are separate from renderers — a renderer receives a camera reference

## Testing

- Tests that need a GL context: use `test_helpers::init_test_context()`
- Math-only tests (matrix calculations, projections): no GL context needed
- Texture tests may require valid image files in `assets/`
