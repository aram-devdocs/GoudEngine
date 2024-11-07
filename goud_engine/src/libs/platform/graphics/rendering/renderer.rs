
pub mod renderer2d;
pub mod renderer3d;

/// Base Renderer trait
///
/// Defines common functionality for renderers.
pub trait Renderer {
    /// Renders the scene.
    fn render(&mut self);
}