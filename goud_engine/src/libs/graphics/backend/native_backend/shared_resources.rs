use crate::libs::error::GoudResult;

use super::SharedNativeRenderBackend;
use crate::libs::graphics::backend::render_backend::{
    BufferOps, RenderTargetOps, ShaderOps, TextureOps,
};
use crate::libs::graphics::backend::types::{
    BufferHandle, BufferType, BufferUsage, RenderTargetDesc, RenderTargetHandle, ShaderHandle,
    TextureFilter, TextureFormat, TextureHandle, TextureWrap,
};

impl BufferOps for SharedNativeRenderBackend {
    fn create_buffer(
        &mut self,
        buffer_type: BufferType,
        usage: BufferUsage,
        data: &[u8],
    ) -> GoudResult<BufferHandle> {
        self.lock().create_buffer(buffer_type, usage, data)
    }

    fn update_buffer(
        &mut self,
        handle: BufferHandle,
        offset: usize,
        data: &[u8],
    ) -> GoudResult<()> {
        self.lock().update_buffer(handle, offset, data)
    }

    fn destroy_buffer(&mut self, handle: BufferHandle) -> bool {
        self.lock().destroy_buffer(handle)
    }

    fn is_buffer_valid(&self, handle: BufferHandle) -> bool {
        self.lock().is_buffer_valid(handle)
    }

    fn buffer_size(&self, handle: BufferHandle) -> Option<usize> {
        self.lock().buffer_size(handle)
    }

    fn bind_buffer(&mut self, handle: BufferHandle) -> GoudResult<()> {
        self.lock().bind_buffer(handle)
    }

    fn unbind_buffer(&mut self, buffer_type: BufferType) {
        self.lock().unbind_buffer(buffer_type);
    }
}

impl TextureOps for SharedNativeRenderBackend {
    fn create_texture(
        &mut self,
        width: u32,
        height: u32,
        format: TextureFormat,
        filter: TextureFilter,
        wrap: TextureWrap,
        data: &[u8],
    ) -> GoudResult<TextureHandle> {
        self.lock()
            .create_texture(width, height, format, filter, wrap, data)
    }

    fn update_texture(
        &mut self,
        handle: TextureHandle,
        x: u32,
        y: u32,
        width: u32,
        height: u32,
        data: &[u8],
    ) -> GoudResult<()> {
        self.lock()
            .update_texture(handle, x, y, width, height, data)
    }

    fn destroy_texture(&mut self, handle: TextureHandle) -> bool {
        self.lock().destroy_texture(handle)
    }

    fn is_texture_valid(&self, handle: TextureHandle) -> bool {
        self.lock().is_texture_valid(handle)
    }

    fn texture_size(&self, handle: TextureHandle) -> Option<(u32, u32)> {
        self.lock().texture_size(handle)
    }

    fn bind_texture(&mut self, handle: TextureHandle, unit: u32) -> GoudResult<()> {
        self.lock().bind_texture(handle, unit)
    }

    fn unbind_texture(&mut self, unit: u32) {
        self.lock().unbind_texture(unit);
    }

    fn create_compressed_texture(
        &mut self,
        width: u32,
        height: u32,
        format: TextureFormat,
        data: &[u8],
        mip_levels: u32,
    ) -> GoudResult<TextureHandle> {
        self.lock()
            .create_compressed_texture(width, height, format, data, mip_levels)
    }
}

impl RenderTargetOps for SharedNativeRenderBackend {
    fn create_render_target(&mut self, desc: &RenderTargetDesc) -> GoudResult<RenderTargetHandle> {
        self.lock().create_render_target(desc)
    }

    fn destroy_render_target(&mut self, handle: RenderTargetHandle) -> bool {
        self.lock().destroy_render_target(handle)
    }

    fn is_render_target_valid(&self, handle: RenderTargetHandle) -> bool {
        self.lock().is_render_target_valid(handle)
    }

    fn bind_render_target(&mut self, handle: Option<RenderTargetHandle>) -> GoudResult<()> {
        self.lock().bind_render_target(handle)
    }

    fn render_target_texture(&self, handle: RenderTargetHandle) -> Option<TextureHandle> {
        self.lock().render_target_texture(handle)
    }
}

impl ShaderOps for SharedNativeRenderBackend {
    fn create_shader(&mut self, vertex_src: &str, fragment_src: &str) -> GoudResult<ShaderHandle> {
        self.lock().create_shader(vertex_src, fragment_src)
    }

    fn destroy_shader(&mut self, handle: ShaderHandle) -> bool {
        self.lock().destroy_shader(handle)
    }

    fn is_shader_valid(&self, handle: ShaderHandle) -> bool {
        self.lock().is_shader_valid(handle)
    }

    fn bind_shader(&mut self, handle: ShaderHandle) -> GoudResult<()> {
        self.lock().bind_shader(handle)
    }

    fn unbind_shader(&mut self) {
        self.lock().unbind_shader();
    }

    fn get_uniform_location(&self, handle: ShaderHandle, name: &str) -> Option<i32> {
        self.lock().get_uniform_location(handle, name)
    }

    fn set_uniform_int(&mut self, location: i32, value: i32) {
        self.lock().set_uniform_int(location, value);
    }

    fn set_uniform_float(&mut self, location: i32, value: f32) {
        self.lock().set_uniform_float(location, value);
    }

    fn set_uniform_vec2(&mut self, location: i32, x: f32, y: f32) {
        self.lock().set_uniform_vec2(location, x, y);
    }

    fn set_uniform_vec3(&mut self, location: i32, x: f32, y: f32, z: f32) {
        self.lock().set_uniform_vec3(location, x, y, z);
    }

    fn set_uniform_vec4(&mut self, location: i32, x: f32, y: f32, z: f32, w: f32) {
        self.lock().set_uniform_vec4(location, x, y, z, w);
    }

    fn set_uniform_mat4(&mut self, location: i32, matrix: &[f32; 16]) {
        self.lock().set_uniform_mat4(location, matrix);
    }
}
