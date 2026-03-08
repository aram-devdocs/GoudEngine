//! High-level rendering systems that integrate ECS, assets, and graphics.
//!
//! This module sits at the Engine layer and bridges the gap between
//! the low-level graphics backend (`libs::graphics`) and the ECS world.

mod render_system;
pub mod sprite_batch;
pub mod text;

pub use render_system::SpriteRenderSystem;
pub use text::TextRenderSystem;
