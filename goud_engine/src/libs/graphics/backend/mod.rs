//! Render Backend Abstraction Layer
//!
//! This module provides a graphics API-agnostic interface for rendering operations.
//! The `RenderBackend` trait abstracts common GPU operations, enabling support for
//! multiple graphics APIs (OpenGL, Vulkan, Metal, WebGPU) through a unified interface.
//!
//! # Architecture
//!
//! The backend system consists of:
//! - **RenderBackend Trait**: Main abstraction defining all graphics operations
//! - **GPU Resource Types**: Handle-based references to buffers, textures, shaders
//! - **Backend Implementations**: Concrete implementations for each graphics API
//!
//! # Example
//!
//! ```rust,ignore
//! use goud_engine::graphics::backend::{RenderBackend, OpenGLBackend};
//!
//! let mut backend = OpenGLBackend::new()?;
//! backend.clear_color(0.1, 0.1, 0.1, 1.0);
//! backend.clear();
//! ```

pub mod blend;
pub mod capabilities;
#[cfg(feature = "native")]
pub mod opengl;
pub mod render_backend;
pub mod types;
#[cfg(feature = "wgpu-backend")]
pub mod wgpu_backend;

#[cfg(test)]
mod tests;

// Re-export for convenience
#[allow(unused_imports)]
pub use self::blend::*;
pub use self::capabilities::*;
pub use self::render_backend::*;
#[allow(unused_imports)]
pub use self::types::*;
