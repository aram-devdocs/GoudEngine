//! State management sub-trait for `RenderBackend`.

use crate::libs::graphics::backend::blend::{BlendFactor, CullFace};
use crate::libs::graphics::backend::types::{DepthFunc, FrontFace};

/// GPU state management operations.
///
/// Controls viewport, depth testing, blending, face culling,
/// and other pipeline state.
pub trait StateOps {
    /// Sets the viewport rectangle.
    ///
    /// # Arguments
    /// * `x` - X coordinate of lower-left corner
    /// * `y` - Y coordinate of lower-left corner
    /// * `width` - Viewport width in pixels
    /// * `height` - Viewport height in pixels
    fn set_viewport(&mut self, x: i32, y: i32, width: u32, height: u32);

    /// Enables depth testing.
    fn enable_depth_test(&mut self);

    /// Disables depth testing.
    fn disable_depth_test(&mut self);

    /// Enables alpha blending.
    fn enable_blending(&mut self);

    /// Disables alpha blending.
    fn disable_blending(&mut self);

    /// Sets the blend function.
    ///
    /// # Arguments
    /// * `src` - Source blend factor
    /// * `dst` - Destination blend factor
    fn set_blend_func(&mut self, src: BlendFactor, dst: BlendFactor);

    /// Enables face culling.
    fn enable_culling(&mut self);

    /// Disables face culling.
    fn disable_culling(&mut self);

    /// Sets which faces to cull.
    fn set_cull_face(&mut self, face: CullFace);

    /// Sets the depth comparison function.
    fn set_depth_func(&mut self, func: DepthFunc);

    /// Sets the front face winding order.
    fn set_front_face(&mut self, face: FrontFace);

    /// Enables or disables writing to the depth buffer.
    fn set_depth_mask(&mut self, enabled: bool);

    /// Enables or disables hardware multisampling when the backend supports it.
    fn set_multisampling_enabled(&mut self, _enabled: bool) {}

    /// Sets the line width for line primitives.
    fn set_line_width(&mut self, width: f32);
}
