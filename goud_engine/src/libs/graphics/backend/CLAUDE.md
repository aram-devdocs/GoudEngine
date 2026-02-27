# libs/graphics/backend/ — OpenGL Backend

## Purpose

All raw OpenGL calls live here. This is the ONLY module that may use the `gl::` namespace directly.

## Files

- `opengl.rs` — OpenGL function wrappers (draw calls, state management, context)
- `types.rs` — GPU-side type mappings (vertex formats, buffer types, texture formats)
- `mod.rs` — Backend re-exports

## Patterns

- Shader compilation and linking happen here
- Buffer management (VBO, VAO, EBO) is encapsulated in this module
- Type mappings translate engine types to OpenGL enums/constants
- All GL calls wrapped with error checking where appropriate

## Anti-Patterns

- NEVER use `gl::` calls outside this module — this is the sole graphics backend
- NEVER expose raw GL handles (GLuint, etc.) to code outside this module
- NEVER skip GL error checking in debug builds

## Dependencies

Leaf module. Depends only on the `gl` crate and engine core types.
