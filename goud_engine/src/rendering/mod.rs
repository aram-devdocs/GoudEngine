//! High-level rendering systems that integrate ECS, assets, and graphics.
//!
//! This module sits at the Engine layer and bridges the gap between
//! the low-level graphics backend (`libs::graphics`) and the ECS world.

mod render_system;
pub mod sprite_batch;
pub mod text;
pub mod texture_atlas;
mod ui_render_system;
pub mod viewport;

pub use render_system::SpriteRenderSystem;
pub use text::TextRenderSystem;
#[cfg(any(feature = "native", test))]
pub(crate) use ui_render_system::ensure_ui_asset_loaders;
pub use ui_render_system::{UiRenderStats, UiRenderSystem};
pub use viewport::{
    compute_render_viewport, compute_render_viewport_with_aspect_lock, AspectRatioLock,
    RenderViewport, SafeAreaInsets, ViewportScaleMode,
};
