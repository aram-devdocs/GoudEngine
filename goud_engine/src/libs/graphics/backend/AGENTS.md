# libs/graphics/backend/ — GPU Backend

## Purpose

All raw GPU calls live here. This is the ONLY module that may use `gl::` or `wgpu::` APIs directly.
Two implementations: wgpu (default, cross-platform) and OpenGL 3.3 (legacy).

## Files

- `opengl/` — OpenGL 3.3 backend (legacy; draw calls, state management, context)
- `wgpu_backend/` — wgpu backend (Vulkan/Metal/DX12/WebGPU cross-platform rendering)
- `native_backend/` — Runtime enum wrapper selecting wgpu or OpenGL
- `render_backend/` — `RenderBackend` trait definition (composed sub-traits)
- `types.rs` — GPU-side type mappings (vertex formats, buffer types, texture formats)
- `mod.rs` — Backend re-exports

## Patterns

- Shader compilation and linking happen here
- Buffer management is encapsulated in this module
- Type mappings translate engine types to backend-specific enums/constants
- All GPU calls wrapped with error checking where appropriate
- Both backends implement the same `RenderBackend` trait — renderer code is backend-agnostic

## Anti-Patterns

- NEVER use `gl::` or `wgpu::` calls outside this module — this is the sole GPU backend
- NEVER expose raw GPU handles (GLuint, wgpu::Buffer, etc.) to code outside this module
- NEVER skip error checking in debug builds

## Dependencies

Leaf module. Depends on the `gl` crate (OpenGL), `wgpu` crate, and engine core types.
