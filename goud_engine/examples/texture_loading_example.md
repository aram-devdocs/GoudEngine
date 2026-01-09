# OpenGL Texture Operations Example

This document demonstrates how to use the OpenGL backend's texture operations for efficient texture management in the GoudEngine.

## Overview

The OpenGL backend provides comprehensive texture operations with:
- **Generational Handles**: Type-safe texture references with automatic slot reuse
- **Multiple Formats**: Support for R8, RG8, RGB8, RGBA8, floating-point, and depth formats
- **Filtering Options**: Nearest or linear filtering for minification/magnification
- **Wrapping Modes**: Repeat, mirrored repeat, clamp to edge, or clamp to border
- **Partial Updates**: Update texture regions without re-uploading entire texture
- **Multi-Texturing**: Bind multiple textures to different units simultaneously

## Basic Usage

### Creating a Texture

```rust
use goud_engine::graphics::backend::{
    OpenGLBackend, RenderBackend,
    TextureFormat, TextureFilter, TextureWrap
};

fn create_simple_texture(backend: &mut OpenGLBackend) -> TextureHandle {
    // Create a 256x256 RGBA texture with linear filtering
    let width = 256;
    let height = 256;
    let pixels: Vec<u8> = vec![255; width * height * 4]; // White texture

    backend.create_texture(
        width as u32,
        height as u32,
        TextureFormat::RGBA8,
        TextureFilter::Linear,
        TextureWrap::Repeat,
        &pixels
    ).expect("Failed to create texture")
}
```

See the rest of the documentation in the file for complete usage examples.
