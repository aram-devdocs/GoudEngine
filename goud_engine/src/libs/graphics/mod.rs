pub mod components;
pub mod renderer;
pub mod renderer2d;
pub mod renderer3d;

/// Clears the color and depth buffers.
pub fn clear() {
    unsafe {
        gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
    }
}
