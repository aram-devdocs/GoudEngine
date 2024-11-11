
pub mod buffer;
pub mod renderer;
pub mod shader;
pub mod sprite;
pub mod textures;
pub mod vao;
pub mod vertex_attribute;

// Re-export commonly used structs and traits
pub use buffer::BufferObject;
pub use renderer::{renderer2d, Renderer};
pub use shader::ShaderProgram;
pub use vao::Vao;
pub use vertex_attribute::VertexAttribute;

// Utility functions
pub fn clear() {
    unsafe {
        gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
    }
}
