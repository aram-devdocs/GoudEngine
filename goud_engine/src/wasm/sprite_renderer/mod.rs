//! Immediate-mode sprite batcher for wgpu on WebAssembly.
//!
//! Collects draw calls during a frame, then flushes them in a single
//! render pass at `end_frame()`. Sprites are batched by texture to
//! minimise bind group switches.

mod renderer_core;
mod texture;
mod types;

pub use renderer_core::WgpuSpriteRenderer;
pub use texture::create_texture_entry;
pub use types::{RenderStats, SpriteVertex, TextureEntry};
