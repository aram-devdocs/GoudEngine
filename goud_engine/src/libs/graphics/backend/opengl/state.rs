//! OpenGL render state management: frame control, clear, viewport, depth, blending, culling.

use super::{
    super::{BackendInfo, BlendFactor, CullFace, RenderBackend},
    backend::OpenGLBackend,
    conversions,
};
use crate::core::error::GoudResult;
use crate::libs::graphics::backend::types::{DepthFunc, FrontFace};

impl RenderBackend for OpenGLBackend {
    fn info(&self) -> &BackendInfo {
        &self.info
    }

    fn begin_frame(&mut self) -> GoudResult<()> {
        // OpenGL doesn't need explicit frame begin
        Ok(())
    }

    fn end_frame(&mut self) -> GoudResult<()> {
        // OpenGL doesn't need explicit frame end
        // Swap buffers is handled by windowing system
        Ok(())
    }

    fn set_clear_color(&mut self, r: f32, g: f32, b: f32, a: f32) {
        self.clear_color = [r, g, b, a];
        unsafe {
            gl::ClearColor(r, g, b, a);
        }
    }

    fn clear_color(&mut self) {
        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }
    }

    fn clear_depth(&mut self) {
        unsafe {
            gl::Clear(gl::DEPTH_BUFFER_BIT);
        }
    }

    fn set_viewport(&mut self, x: i32, y: i32, width: u32, height: u32) {
        unsafe {
            gl::Viewport(x, y, width as i32, height as i32);
        }
    }

    fn enable_depth_test(&mut self) {
        unsafe {
            gl::Enable(gl::DEPTH_TEST);
        }
    }

    fn disable_depth_test(&mut self) {
        unsafe {
            gl::Disable(gl::DEPTH_TEST);
        }
    }

    fn enable_blending(&mut self) {
        unsafe {
            gl::Enable(gl::BLEND);
            // Set standard alpha blending function
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
        }
    }

    fn disable_blending(&mut self) {
        unsafe {
            gl::Disable(gl::BLEND);
        }
    }

    fn set_blend_func(&mut self, src: BlendFactor, dst: BlendFactor) {
        let src_gl = conversions::blend_factor_to_gl(src);
        let dst_gl = conversions::blend_factor_to_gl(dst);
        unsafe {
            gl::BlendFunc(src_gl, dst_gl);
        }
    }

    fn enable_culling(&mut self) {
        unsafe {
            gl::Enable(gl::CULL_FACE);
        }
    }

    fn disable_culling(&mut self) {
        unsafe {
            gl::Disable(gl::CULL_FACE);
        }
    }

    fn set_cull_face(&mut self, face: CullFace) {
        let gl_face = match face {
            CullFace::Front => gl::FRONT,
            CullFace::Back => gl::BACK,
            CullFace::FrontAndBack => gl::FRONT_AND_BACK,
        };
        unsafe {
            gl::CullFace(gl_face);
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
        // SAFETY: Valid GL enum passed to DepthFunc
        unsafe {
            gl::DepthFunc(gl_func);
        }
    }

    fn set_front_face(&mut self, face: FrontFace) {
        let gl_face = match face {
            FrontFace::Ccw => gl::CCW,
            FrontFace::Cw => gl::CW,
        };
        // SAFETY: Valid GL enum passed to FrontFace
        unsafe {
            gl::FrontFace(gl_face);
        }
    }

    fn set_depth_mask(&mut self, enabled: bool) {
        // SAFETY: Boolean mapped to GL_TRUE/GL_FALSE
        unsafe {
            gl::DepthMask(if enabled { gl::TRUE } else { gl::FALSE });
        }
    }

    fn set_line_width(&mut self, width: f32) {
        // SAFETY: Positive float passed to LineWidth
        unsafe {
            gl::LineWidth(width);
        }
    }

    // Buffer, texture, shader, draw-call and vertex-attribute methods are implemented
    // in their respective submodules via separate `impl RenderBackend for OpenGLBackend`
    // blocks in buffer_ops.rs, texture_ops.rs, shader_ops.rs, and draw_calls.rs.
    //
    // Rust allows splitting trait implementations across multiple files as long as
    // the trait impl block is declared only once. We use a single impl block here
    // and forward to helper functions defined in the other modules.

    // ============================================================================
    // Buffer Operations — forwarded to buffer_ops
    // ============================================================================

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

    // ============================================================================
    // Texture Operations — forwarded to texture_ops
    // ============================================================================

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

    // ============================================================================
    // Shader Operations — forwarded to shader_ops
    // ============================================================================

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

    // ============================================================================
    // Vertex Array & Draw Calls — forwarded to draw_calls
    // ============================================================================

    fn set_vertex_attributes(
        &mut self,
        layout: &crate::libs::graphics::backend::types::VertexLayout,
    ) {
        super::draw_calls::set_vertex_attributes(layout)
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
