//! OpenGL Backend Implementation
//!
//! This module provides an OpenGL 3.3 Core implementation of the `RenderBackend` trait.
//! It manages OpenGL state, resources (buffers, textures, shaders), and rendering operations.
//!
//! The implementation is split into focused submodules:
//! - `backend`: Struct definition, constructor, and `Drop` cleanup
//! - `state`: `RenderBackend` trait impl (state management + forwarding)
//! - `buffer_ops`: Buffer create/update/destroy/bind
//! - `texture_ops`: Texture create/update/destroy/bind
//! - `shader_ops`: Shader compile/link/destroy/bind and uniform setters
//! - `draw_calls`: Vertex attribute setup and draw dispatch
//! - `conversions`: Engine-type-to-GL-constant helpers
//! - `gl_tests`: Unit and integration tests

use std::collections::HashMap;
use std::sync::Mutex;

mod backend;
mod buffer_ops;
mod conversions;
mod draw_calls;
mod gl_tests;
mod render_target_ops;
mod shader_ops;
mod state;
mod texture_ops;

// Re-export the backend struct so existing code using `opengl::OpenGLBackend` still works.
pub use backend::OpenGLBackend;

// ============================================================================
// Internal metadata types — used across submodules
// ============================================================================

/// Internal buffer metadata stored alongside the OpenGL buffer ID.
#[derive(Debug, Clone)]
struct BufferMetadata {
    /// OpenGL buffer object ID
    gl_id: u32,
    /// Type of buffer (Vertex, Index, Uniform)
    buffer_type: super::types::BufferType,
    /// Usage hint (Static, Dynamic, Stream) — reserved for future rebind optimisation
    _usage: super::types::BufferUsage,
    /// Size of buffer in bytes
    size: usize,
}

/// Internal texture metadata stored alongside the OpenGL texture ID.
#[derive(Debug, Clone)]
struct TextureMetadata {
    /// OpenGL texture object ID
    gl_id: u32,
    /// Texture width in pixels
    width: u32,
    /// Texture height in pixels
    height: u32,
    /// Pixel format
    format: super::types::TextureFormat,
    /// Filtering mode — reserved for future texture parameter queries
    _filter: super::types::TextureFilter,
    /// Wrapping mode — reserved for future texture parameter queries
    _wrap: super::types::TextureWrap,
}

/// Internal render-target metadata stored alongside the OpenGL framebuffer ID.
#[derive(Debug, Clone)]
struct RenderTargetMetadata {
    /// OpenGL framebuffer object ID.
    framebuffer_id: u32,
    /// Texture attached to color attachment 0.
    color_texture: super::types::TextureHandle,
    /// Optional depth renderbuffer.
    depth_renderbuffer: Option<u32>,
    /// Render-target width in pixels.
    width: u32,
    /// Render-target height in pixels.
    height: u32,
}

/// Internal shader metadata stored alongside the OpenGL shader program ID.
#[derive(Debug)]
struct ShaderMetadata {
    /// OpenGL shader program ID
    gl_id: u32,
    /// Cached uniform locations by name.
    /// Uses `Mutex` for interior mutability so `get_uniform_location` can
    /// populate the cache without requiring `&mut self` on the backend.
    uniform_locations: Mutex<HashMap<String, i32>>,
}

// ============================================================================
// Debug-only GL error checking macro
// ============================================================================

/// Checks `glGetError()` in debug builds and logs any errors.
/// Compiles to nothing in release builds (zero overhead).
macro_rules! gl_check_debug {
    ($op:expr) => {
        #[cfg(debug_assertions)]
        {
            // SAFETY: glGetError is always safe to call with a valid GL context
            let err = unsafe { gl::GetError() };
            if err != gl::NO_ERROR {
                log::error!(
                    "GL error 0x{:X} after {} at {}:{}",
                    err,
                    $op,
                    file!(),
                    line!()
                );
            }
        }
    };
}

pub(super) use gl_check_debug;
