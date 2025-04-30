pub mod components;
pub mod renderer;
pub mod renderer2d;
pub mod renderer3d;

/// Base Renderer trait
///
/// Defines common functionality for renderers.

// Utility functions
pub fn clear() {
    unsafe {
        gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
    }
}
