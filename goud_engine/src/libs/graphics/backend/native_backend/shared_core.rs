use crate::libs::error::GoudResult;

use super::SharedNativeRenderBackend;
use crate::libs::graphics::backend::render_backend::{ClearOps, FrameOps, RenderBackend, StateOps};

impl RenderBackend for SharedNativeRenderBackend {
    fn info(&self) -> &crate::libs::graphics::backend::capabilities::BackendInfo {
        &self.info
    }

    fn bind_default_vertex_array(&mut self) {
        self.lock().bind_default_vertex_array();
    }

    fn validate_text_draw_state(&self) -> Result<(), String> {
        self.lock().validate_text_draw_state()
    }

    fn read_default_framebuffer_rgba8(
        &mut self,
        width: u32,
        height: u32,
    ) -> Result<Vec<u8>, String> {
        self.lock().read_default_framebuffer_rgba8(width, height)
    }
}

impl FrameOps for SharedNativeRenderBackend {
    fn begin_frame(&mut self) -> GoudResult<()> {
        self.lock().begin_frame()
    }

    fn end_frame(&mut self) -> GoudResult<()> {
        self.lock().end_frame()
    }
}

impl ClearOps for SharedNativeRenderBackend {
    fn set_clear_color(&mut self, r: f32, g: f32, b: f32, a: f32) {
        self.lock().set_clear_color(r, g, b, a);
    }

    fn clear_color(&mut self) {
        self.lock().clear_color();
    }

    fn clear_depth(&mut self) {
        self.lock().clear_depth();
    }
}

impl StateOps for SharedNativeRenderBackend {
    fn set_viewport(&mut self, x: i32, y: i32, width: u32, height: u32) {
        self.lock().set_viewport(x, y, width, height);
    }

    fn enable_depth_test(&mut self) {
        self.lock().enable_depth_test();
    }

    fn disable_depth_test(&mut self) {
        self.lock().disable_depth_test();
    }

    fn enable_blending(&mut self) {
        self.lock().enable_blending();
    }

    fn disable_blending(&mut self) {
        self.lock().disable_blending();
    }

    fn set_blend_func(
        &mut self,
        src: crate::libs::graphics::backend::BlendFactor,
        dst: crate::libs::graphics::backend::BlendFactor,
    ) {
        self.lock().set_blend_func(src, dst);
    }

    fn enable_culling(&mut self) {
        self.lock().enable_culling();
    }

    fn disable_culling(&mut self) {
        self.lock().disable_culling();
    }

    fn set_cull_face(&mut self, face: crate::libs::graphics::backend::CullFace) {
        self.lock().set_cull_face(face);
    }

    fn set_depth_func(&mut self, func: crate::libs::graphics::backend::types::DepthFunc) {
        self.lock().set_depth_func(func);
    }

    fn set_front_face(&mut self, face: crate::libs::graphics::backend::types::FrontFace) {
        self.lock().set_front_face(face);
    }

    fn set_depth_mask(&mut self, enabled: bool) {
        self.lock().set_depth_mask(enabled);
    }

    fn set_line_width(&mut self, width: f32) {
        self.lock().set_line_width(width);
    }
}
