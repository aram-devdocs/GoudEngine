//! Clear operations sub-trait for `RenderBackend`.

/// Clear-buffer operations.
///
/// Controls clear color and clearing of color/depth buffers.
pub trait ClearOps {
    /// Sets the clear color for subsequent clear operations.
    ///
    /// # Arguments
    /// * `r` - Red component (0.0 to 1.0)
    /// * `g` - Green component (0.0 to 1.0)
    /// * `b` - Blue component (0.0 to 1.0)
    /// * `a` - Alpha component (0.0 to 1.0)
    fn set_clear_color(&mut self, r: f32, g: f32, b: f32, a: f32);

    /// Clears the color buffer using the current clear color.
    fn clear_color(&mut self);

    /// Clears the depth buffer.
    fn clear_depth(&mut self);

    /// Clears both color and depth buffers.
    ///
    /// Default implementation calls both clear methods, but backends
    /// can override for more efficient combined operations.
    fn clear(&mut self) {
        self.clear_color();
        self.clear_depth();
    }
}
