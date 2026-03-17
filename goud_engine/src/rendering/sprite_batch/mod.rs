//! Sprite Batch Renderer for efficient 2D sprite rendering.
//!
//! This module provides a high-performance sprite batching system that:
//! - **Batches sprites**: Groups sprites by texture to minimize draw calls
//! - **Sorts by Z-layer**: Ensures correct render order
//! - **Manages vertex buffers**: Dynamic vertex buffer resizing
//! - **Handles transforms**: Integrates with Transform2D component
//!
//! # Architecture
//!
//! The sprite batch system uses a gather-sort-batch-render pipeline:
//!
//! 1. **Gather**: Query all entities with Sprite + Transform2D
//! 2. **Sort**: Order sprites by Z-layer and texture for efficient batching
//! 3. **Batch**: Group consecutive sprites with same texture into batches
//! 4. **Render**: Submit vertex data and draw calls to GPU
//!
//! # Performance
//!
//! Target performance: <100 draw calls for 10,000 sprites (100:1 batch ratio)
//!
//! # Example
//!
//! ```rust,ignore
//! use goud_engine::graphics::sprite_batch::{SpriteBatch, SpriteBatchConfig};
//! use goud_engine::graphics::backend::OpenGLBackend;
//! use goud_engine::ecs::World;
//!
//! let backend = OpenGLBackend::new()?;
//! let mut batch = SpriteBatch::new(backend, SpriteBatchConfig::default());
//!
//! // Each frame
//! batch.begin();
//! batch.draw_sprites(&world);
//! batch.end();
//! ```

pub mod batch;
pub mod config;
pub mod resources;
pub mod types;

#[cfg(all(test, feature = "legacy-glfw-opengl"))]
mod batching_tests;
#[cfg(all(test, feature = "legacy-glfw-opengl"))]
mod gl_tests;
#[cfg(all(test, feature = "legacy-glfw-opengl"))]
mod integration_tests;
#[cfg(test)]
mod tests;

pub use batch::SpriteBatch;
pub use config::SpriteBatchConfig;
pub use types::{SpriteBatchEntry, SpriteInstance, SpriteVertex};
