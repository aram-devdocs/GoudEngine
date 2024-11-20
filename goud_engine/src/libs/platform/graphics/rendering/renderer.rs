use crate::types::{SpriteMap, TextureManager, Tiled};

pub mod renderer2d;
pub mod renderer3d;

/// Base Renderer trait
///
/// Defines common functionality for renderers.
pub trait Renderer {
    /// Renders the scene.
    // TODO: We need to abstract this so it works better for 3d
    fn render(&mut self, sprites: SpriteMap, texture_manager: &TextureManager, selected_map: Option<&Tiled>);

    /// Terminates the renderer.
    fn terminate(&self);
}
