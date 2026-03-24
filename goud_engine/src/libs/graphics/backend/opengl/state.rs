//! OpenGL render state management and sub-trait forwarding helpers.
use super::{
    super::{
        BackendInfo, BlendFactor, BufferOps, ClearOps, CullFace, DrawOps, FrameOps, RenderBackend,
        ShaderOps, StateOps, TextureOps,
    },
    backend::OpenGLBackend,
    conversions, gl_check_debug,
};
use crate::libs::error::GoudResult;
use crate::libs::graphics::backend::types::{DepthFunc, FrontFace, VertexBufferBinding};

mod readback;
#[cfg(test)]
mod tests;

fn clamp_line_width(width: f32, supported_range: [f32; 2]) -> Option<f32> {
    if !width.is_finite() || width <= 0.0 {
        return None;
    }

    let min = supported_range[0].max(1.0);
    let max = supported_range[1].max(min);
    Some(width.clamp(min, max))
}

impl OpenGLBackend {
    /// Standalone framebuffer readback (no mutable backend reference needed).
    pub fn read_framebuffer_standalone(width: u32, height: u32) -> Result<Vec<u8>, String> {
        readback::read_default_framebuffer_rgba8_standalone(width, height)
    }
}

impl RenderBackend for OpenGLBackend {
    fn info(&self) -> &BackendInfo {
        &self.info
    }

    fn bind_default_vertex_array(&mut self) {
        self.bind_default_vao();
    }

    fn validate_text_draw_state(&self) -> Result<(), String> {
        self.validate_bound_text_draw_state()
    }

    fn read_default_framebuffer_rgba8(
        &mut self,
        width: u32,
        height: u32,
    ) -> Result<Vec<u8>, String> {
        readback::read_default_framebuffer_rgba8(self, width, height)
    }
}

// ============================================================================
// FrameOps
// ============================================================================

impl FrameOps for OpenGLBackend {
    fn begin_frame(&mut self) -> GoudResult<()> {
        // OpenGL doesn't need explicit frame begin
        Ok(())
    }

    fn end_frame(&mut self) -> GoudResult<()> {
        // OpenGL doesn't need explicit frame end
        // Swap buffers is handled by windowing system
        Ok(())
    }
}

// ============================================================================
// ClearOps
// ============================================================================

impl ClearOps for OpenGLBackend {
    fn set_clear_color(&mut self, r: f32, g: f32, b: f32, a: f32) {
        self.clear_color = [r, g, b, a];
        // SAFETY: RGBA floats in [0.0, 1.0] are valid arguments for ClearColor.
        unsafe {
            gl::ClearColor(r, g, b, a);
        }
        gl_check_debug!("set_clear_color");
    }

    fn clear_color(&mut self) {
        // SAFETY: COLOR_BUFFER_BIT is a valid mask for gl::Clear.
        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }
        gl_check_debug!("clear_color");
    }

    fn clear_depth(&mut self) {
        // SAFETY: DEPTH_BUFFER_BIT is a valid mask for gl::Clear.
        unsafe {
            gl::Clear(gl::DEPTH_BUFFER_BIT);
        }
        gl_check_debug!("clear_depth");
    }
}

// ============================================================================
// StateOps
// ============================================================================

impl StateOps for OpenGLBackend {
    fn set_viewport(&mut self, x: i32, y: i32, width: u32, height: u32) {
        if self.active_render_target.is_none() {
            self.default_viewport = (x, y, width, height);
        }
        // SAFETY: x, y are signed integers and width/height are cast from valid u32 values.
        unsafe {
            gl::Viewport(x, y, width as i32, height as i32);
        }
        gl_check_debug!("set_viewport");
    }

    fn enable_depth_test(&mut self) {
        if !self.depth_test_enabled {
            self.depth_test_enabled = true;
            // SAFETY: DEPTH_TEST is a valid capability enum for gl::Enable.
            unsafe {
                gl::Enable(gl::DEPTH_TEST);
            }
            gl_check_debug!("enable_depth_test");
        }
    }

    fn disable_depth_test(&mut self) {
        if self.depth_test_enabled {
            self.depth_test_enabled = false;
            // SAFETY: DEPTH_TEST is a valid capability enum for gl::Disable.
            unsafe {
                gl::Disable(gl::DEPTH_TEST);
            }
            gl_check_debug!("disable_depth_test");
        }
    }

    fn enable_blending(&mut self) {
        if !self.blend_enabled {
            self.blend_enabled = true;
            // SAFETY: BLEND is a valid capability enum.
            unsafe {
                gl::Enable(gl::BLEND);
            }
        }
        // Set standard alpha blending (may differ from cached values).
        let src = gl::SRC_ALPHA;
        let dst = gl::ONE_MINUS_SRC_ALPHA;
        if self.cached_blend_src != src || self.cached_blend_dst != dst {
            self.cached_blend_src = src;
            self.cached_blend_dst = dst;
            // SAFETY: SRC_ALPHA and ONE_MINUS_SRC_ALPHA are valid blend factors.
            unsafe {
                gl::BlendFunc(src, dst);
            }
        }
        gl_check_debug!("enable_blending");
    }

    fn disable_blending(&mut self) {
        if self.blend_enabled {
            self.blend_enabled = false;
            // SAFETY: BLEND is a valid capability enum for gl::Disable.
            unsafe {
                gl::Disable(gl::BLEND);
            }
            gl_check_debug!("disable_blending");
        }
    }

    fn set_blend_func(&mut self, src: BlendFactor, dst: BlendFactor) {
        let src_gl = conversions::blend_factor_to_gl(src);
        let dst_gl = conversions::blend_factor_to_gl(dst);
        if self.cached_blend_src != src_gl || self.cached_blend_dst != dst_gl {
            self.cached_blend_src = src_gl;
            self.cached_blend_dst = dst_gl;
            // SAFETY: src_gl and dst_gl are valid GL blend factor enums.
            unsafe {
                gl::BlendFunc(src_gl, dst_gl);
            }
            gl_check_debug!("set_blend_func");
        }
    }

    fn enable_culling(&mut self) {
        if !self.cull_enabled {
            self.cull_enabled = true;
            // SAFETY: CULL_FACE is a valid capability enum for gl::Enable.
            unsafe {
                gl::Enable(gl::CULL_FACE);
            }
            gl_check_debug!("enable_culling");
        }
    }

    fn disable_culling(&mut self) {
        if self.cull_enabled {
            self.cull_enabled = false;
            // SAFETY: CULL_FACE is a valid capability enum for gl::Disable.
            unsafe {
                gl::Disable(gl::CULL_FACE);
            }
            gl_check_debug!("disable_culling");
        }
    }

    fn set_cull_face(&mut self, face: CullFace) {
        let gl_face = match face {
            CullFace::Front => gl::FRONT,
            CullFace::Back => gl::BACK,
            CullFace::FrontAndBack => gl::FRONT_AND_BACK,
        };
        if self.cached_cull_face != gl_face {
            self.cached_cull_face = gl_face;
            // SAFETY: gl_face is a valid GL face enum.
            unsafe {
                gl::CullFace(gl_face);
            }
            gl_check_debug!("set_cull_face");
        }
    }

    fn set_depth_func(&mut self, func: DepthFunc) {
        let gl_func = match func {
            DepthFunc::Always => gl::ALWAYS,
            DepthFunc::Never => gl::NEVER,
            DepthFunc::Less => gl::LESS,
            DepthFunc::LessEqual => gl::LEQUAL,
            DepthFunc::Greater => gl::GREATER,
            DepthFunc::GreaterEqual => gl::GEQUAL,
            DepthFunc::Equal => gl::EQUAL,
            DepthFunc::NotEqual => gl::NOTEQUAL,
        };
        if self.cached_depth_func != gl_func {
            self.cached_depth_func = gl_func;
            // SAFETY: Valid GL enum passed to DepthFunc.
            unsafe {
                gl::DepthFunc(gl_func);
            }
        }
    }

    fn set_front_face(&mut self, face: FrontFace) {
        let gl_face = match face {
            FrontFace::Ccw => gl::CCW,
            FrontFace::Cw => gl::CW,
        };
        if self.cached_front_face != gl_face {
            self.cached_front_face = gl_face;
            // SAFETY: Valid GL enum passed to FrontFace.
            unsafe {
                gl::FrontFace(gl_face);
            }
        }
    }

    fn set_depth_mask(&mut self, enabled: bool) {
        if self.depth_write_enabled != enabled {
            self.depth_write_enabled = enabled;
            // SAFETY: Boolean mapped to GL_TRUE/GL_FALSE.
            unsafe {
                gl::DepthMask(if enabled { gl::TRUE } else { gl::FALSE });
            }
        }
    }

    fn set_multisampling_enabled(&mut self, enabled: bool) {
        // SAFETY: MULTISAMPLE is a valid OpenGL capability enum.
        unsafe {
            if enabled {
                gl::Enable(gl::MULTISAMPLE);
            } else {
                gl::Disable(gl::MULTISAMPLE);
            }
        }
        gl_check_debug!("set_multisampling_enabled");
    }

    fn set_line_width(&mut self, width: f32) {
        let Some(width) = clamp_line_width(width, self.line_width_range) else {
            return;
        };
        // SAFETY: Positive float passed to LineWidth.
        unsafe {
            gl::LineWidth(width);
        }
        gl_check_debug!("set_line_width");
    }
}

// ============================================================================
// BufferOps — forwarded to buffer_ops module
// ============================================================================

impl BufferOps for OpenGLBackend {
    fn create_buffer(
        &mut self,
        buffer_type: crate::libs::graphics::backend::types::BufferType,
        usage: crate::libs::graphics::backend::types::BufferUsage,
        data: &[u8],
    ) -> GoudResult<crate::libs::graphics::backend::types::BufferHandle> {
        super::buffer_ops::create_buffer(self, buffer_type, usage, data)
    }

    fn update_buffer(
        &mut self,
        handle: crate::libs::graphics::backend::types::BufferHandle,
        offset: usize,
        data: &[u8],
    ) -> GoudResult<()> {
        super::buffer_ops::update_buffer(self, handle, offset, data)
    }

    fn destroy_buffer(
        &mut self,
        handle: crate::libs::graphics::backend::types::BufferHandle,
    ) -> bool {
        super::buffer_ops::destroy_buffer(self, handle)
    }

    fn is_buffer_valid(&self, handle: crate::libs::graphics::backend::types::BufferHandle) -> bool {
        super::buffer_ops::is_buffer_valid(self, handle)
    }

    fn buffer_size(
        &self,
        handle: crate::libs::graphics::backend::types::BufferHandle,
    ) -> Option<usize> {
        super::buffer_ops::buffer_size(self, handle)
    }

    fn bind_buffer(
        &mut self,
        handle: crate::libs::graphics::backend::types::BufferHandle,
    ) -> GoudResult<()> {
        super::buffer_ops::bind_buffer(self, handle)
    }

    fn unbind_buffer(&mut self, buffer_type: crate::libs::graphics::backend::types::BufferType) {
        super::buffer_ops::unbind_buffer(self, buffer_type)
    }
}

// ============================================================================
// TextureOps — forwarded to texture_ops module
// ============================================================================

impl TextureOps for OpenGLBackend {
    fn create_texture(
        &mut self,
        width: u32,
        height: u32,
        format: crate::libs::graphics::backend::types::TextureFormat,
        filter: crate::libs::graphics::backend::types::TextureFilter,
        wrap: crate::libs::graphics::backend::types::TextureWrap,
        data: &[u8],
    ) -> GoudResult<crate::libs::graphics::backend::types::TextureHandle> {
        super::texture_ops::create_texture(self, width, height, format, filter, wrap, data)
    }

    fn update_texture(
        &mut self,
        handle: crate::libs::graphics::backend::types::TextureHandle,
        x: u32,
        y: u32,
        width: u32,
        height: u32,
        data: &[u8],
    ) -> GoudResult<()> {
        super::texture_ops::update_texture(self, handle, x, y, width, height, data)
    }

    fn destroy_texture(
        &mut self,
        handle: crate::libs::graphics::backend::types::TextureHandle,
    ) -> bool {
        super::texture_ops::destroy_texture(self, handle)
    }

    fn is_texture_valid(
        &self,
        handle: crate::libs::graphics::backend::types::TextureHandle,
    ) -> bool {
        super::texture_ops::is_texture_valid(self, handle)
    }

    fn texture_size(
        &self,
        handle: crate::libs::graphics::backend::types::TextureHandle,
    ) -> Option<(u32, u32)> {
        super::texture_ops::texture_size(self, handle)
    }

    fn bind_texture(
        &mut self,
        handle: crate::libs::graphics::backend::types::TextureHandle,
        unit: u32,
    ) -> GoudResult<()> {
        super::texture_ops::bind_texture(self, handle, unit)
    }

    fn unbind_texture(&mut self, unit: u32) {
        super::texture_ops::unbind_texture(self, unit)
    }
}

// ============================================================================
// ShaderOps — forwarded to shader_ops module
// ============================================================================

impl ShaderOps for OpenGLBackend {
    fn create_shader(
        &mut self,
        vertex_src: &str,
        fragment_src: &str,
    ) -> GoudResult<crate::libs::graphics::backend::types::ShaderHandle> {
        super::shader_ops::create_shader(self, vertex_src, fragment_src)
    }

    fn destroy_shader(
        &mut self,
        handle: crate::libs::graphics::backend::types::ShaderHandle,
    ) -> bool {
        super::shader_ops::destroy_shader(self, handle)
    }

    fn is_shader_valid(&self, handle: crate::libs::graphics::backend::types::ShaderHandle) -> bool {
        super::shader_ops::is_shader_valid(self, handle)
    }

    fn bind_shader(
        &mut self,
        handle: crate::libs::graphics::backend::types::ShaderHandle,
    ) -> GoudResult<()> {
        super::shader_ops::bind_shader(self, handle)
    }

    fn unbind_shader(&mut self) {
        super::shader_ops::unbind_shader(self)
    }

    fn get_uniform_location(
        &self,
        handle: crate::libs::graphics::backend::types::ShaderHandle,
        name: &str,
    ) -> Option<i32> {
        super::shader_ops::get_uniform_location(self, handle, name)
    }

    fn set_uniform_int(&mut self, location: i32, value: i32) {
        super::shader_ops::set_uniform_int(location, value)
    }

    fn set_uniform_float(&mut self, location: i32, value: f32) {
        super::shader_ops::set_uniform_float(location, value)
    }

    fn set_uniform_vec2(&mut self, location: i32, x: f32, y: f32) {
        super::shader_ops::set_uniform_vec2(location, x, y)
    }

    fn set_uniform_vec3(&mut self, location: i32, x: f32, y: f32, z: f32) {
        super::shader_ops::set_uniform_vec3(location, x, y, z)
    }

    fn set_uniform_vec4(&mut self, location: i32, x: f32, y: f32, z: f32, w: f32) {
        super::shader_ops::set_uniform_vec4(location, x, y, z, w)
    }

    fn set_uniform_mat4(&mut self, location: i32, matrix: &[f32; 16]) {
        super::shader_ops::set_uniform_mat4(location, matrix)
    }
}

// ============================================================================
// DrawOps — forwarded to draw_calls module
// ============================================================================

impl DrawOps for OpenGLBackend {
    fn set_vertex_attributes(
        &mut self,
        layout: &crate::libs::graphics::backend::types::VertexLayout,
    ) {
        super::draw_calls::set_vertex_attributes_cached(self, layout)
    }

    fn set_vertex_bindings(&mut self, bindings: &[VertexBufferBinding]) -> GoudResult<()> {
        super::draw_calls::set_vertex_bindings_cached(self, bindings)
    }

    fn draw_arrays(
        &mut self,
        topology: crate::libs::graphics::backend::types::PrimitiveTopology,
        first: u32,
        count: u32,
    ) -> GoudResult<()> {
        super::draw_calls::draw_arrays(self, topology, first, count)
    }

    fn draw_indexed(
        &mut self,
        topology: crate::libs::graphics::backend::types::PrimitiveTopology,
        count: u32,
        offset: usize,
    ) -> GoudResult<()> {
        super::draw_calls::draw_indexed(self, topology, count, offset)
    }

    fn draw_indexed_u16(
        &mut self,
        topology: crate::libs::graphics::backend::types::PrimitiveTopology,
        count: u32,
        offset: usize,
    ) -> GoudResult<()> {
        super::draw_calls::draw_indexed_u16(self, topology, count, offset)
    }

    fn draw_arrays_instanced(
        &mut self,
        topology: crate::libs::graphics::backend::types::PrimitiveTopology,
        first: u32,
        count: u32,
        instance_count: u32,
    ) -> GoudResult<()> {
        super::draw_calls::draw_arrays_instanced(self, topology, first, count, instance_count)
    }

    fn draw_indexed_instanced(
        &mut self,
        topology: crate::libs::graphics::backend::types::PrimitiveTopology,
        count: u32,
        offset: usize,
        instance_count: u32,
    ) -> GoudResult<()> {
        super::draw_calls::draw_indexed_instanced(self, topology, count, offset, instance_count)
    }
}
