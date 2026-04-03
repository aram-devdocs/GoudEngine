use crate::libs::error::GoudResult;

use super::NativeRenderBackend;
use crate::libs::graphics::backend::render_backend::{
    BufferOps, RenderTargetOps, ShaderOps, TextureOps,
};
use crate::libs::graphics::backend::types::{
    BufferHandle, BufferType, BufferUsage, RenderTargetDesc, RenderTargetHandle, ShaderHandle,
    TextureFilter, TextureFormat, TextureHandle, TextureWrap,
};

/// Dispatch a method call to the active backend variant.
macro_rules! dispatch {
    ($self:expr, $method:ident $(, $arg:expr)*) => {
        match $self {
            #[cfg(feature = "legacy-glfw-opengl")]
            Self::OpenGlLegacy(b) => b.$method($($arg),*),
            #[cfg(any(all(feature = "native", feature = "wgpu-backend"), feature = "xbox-gdk", feature = "sdl-window", feature = "switch-vulkan"))]
            Self::Wgpu(b) => b.$method($($arg),*),
        }
    };
}

impl BufferOps for NativeRenderBackend {
    fn create_buffer(
        &mut self,
        buffer_type: BufferType,
        usage: BufferUsage,
        data: &[u8],
    ) -> GoudResult<BufferHandle> {
        dispatch!(self, create_buffer, buffer_type, usage, data)
    }

    fn update_buffer(
        &mut self,
        handle: BufferHandle,
        offset: usize,
        data: &[u8],
    ) -> GoudResult<()> {
        dispatch!(self, update_buffer, handle, offset, data)
    }

    fn destroy_buffer(&mut self, handle: BufferHandle) -> bool {
        dispatch!(self, destroy_buffer, handle)
    }

    fn is_buffer_valid(&self, handle: BufferHandle) -> bool {
        dispatch!(self, is_buffer_valid, handle)
    }

    fn buffer_size(&self, handle: BufferHandle) -> Option<usize> {
        dispatch!(self, buffer_size, handle)
    }

    fn bind_buffer(&mut self, handle: BufferHandle) -> GoudResult<()> {
        dispatch!(self, bind_buffer, handle)
    }

    fn unbind_buffer(&mut self, buffer_type: BufferType) {
        dispatch!(self, unbind_buffer, buffer_type)
    }

    fn supports_storage_buffers(&self) -> bool {
        dispatch!(self, supports_storage_buffers)
    }

    fn create_storage_buffer(&mut self, data: &[u8]) -> GoudResult<BufferHandle> {
        dispatch!(self, create_storage_buffer, data)
    }

    fn update_storage_buffer(
        &mut self,
        handle: BufferHandle,
        offset: usize,
        data: &[u8],
    ) -> GoudResult<()> {
        dispatch!(self, update_storage_buffer, handle, offset, data)
    }

    fn bind_storage_buffer(&mut self, handle: BufferHandle, binding: u32) -> GoudResult<()> {
        dispatch!(self, bind_storage_buffer, handle, binding)
    }

    fn unbind_storage_buffer(&mut self) {
        dispatch!(self, unbind_storage_buffer)
    }
}

impl TextureOps for NativeRenderBackend {
    fn create_texture(
        &mut self,
        width: u32,
        height: u32,
        format: TextureFormat,
        filter: TextureFilter,
        wrap: TextureWrap,
        data: &[u8],
    ) -> GoudResult<TextureHandle> {
        dispatch!(
            self,
            create_texture,
            width,
            height,
            format,
            filter,
            wrap,
            data
        )
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
        dispatch!(self, update_texture, handle, x, y, width, height, data)
    }

    fn destroy_texture(&mut self, handle: TextureHandle) -> bool {
        dispatch!(self, destroy_texture, handle)
    }

    fn is_texture_valid(&self, handle: TextureHandle) -> bool {
        dispatch!(self, is_texture_valid, handle)
    }

    fn texture_size(&self, handle: TextureHandle) -> Option<(u32, u32)> {
        dispatch!(self, texture_size, handle)
    }

    fn bind_texture(&mut self, handle: TextureHandle, unit: u32) -> GoudResult<()> {
        dispatch!(self, bind_texture, handle, unit)
    }

    fn unbind_texture(&mut self, unit: u32) {
        dispatch!(self, unbind_texture, unit)
    }

    fn create_compressed_texture(
        &mut self,
        width: u32,
        height: u32,
        format: TextureFormat,
        data: &[u8],
        mip_levels: u32,
    ) -> GoudResult<TextureHandle> {
        dispatch!(
            self,
            create_compressed_texture,
            width,
            height,
            format,
            data,
            mip_levels
        )
    }
}

impl RenderTargetOps for NativeRenderBackend {
    fn create_render_target(&mut self, desc: &RenderTargetDesc) -> GoudResult<RenderTargetHandle> {
        dispatch!(self, create_render_target, desc)
    }

    fn destroy_render_target(&mut self, handle: RenderTargetHandle) -> bool {
        dispatch!(self, destroy_render_target, handle)
    }

    fn is_render_target_valid(&self, handle: RenderTargetHandle) -> bool {
        dispatch!(self, is_render_target_valid, handle)
    }

    fn bind_render_target(&mut self, handle: Option<RenderTargetHandle>) -> GoudResult<()> {
        dispatch!(self, bind_render_target, handle)
    }

    fn render_target_texture(&self, handle: RenderTargetHandle) -> Option<TextureHandle> {
        dispatch!(self, render_target_texture, handle)
    }
}

impl ShaderOps for NativeRenderBackend {
    fn create_shader(&mut self, vertex_src: &str, fragment_src: &str) -> GoudResult<ShaderHandle> {
        dispatch!(self, create_shader, vertex_src, fragment_src)
    }

    fn destroy_shader(&mut self, handle: ShaderHandle) -> bool {
        dispatch!(self, destroy_shader, handle)
    }

    fn is_shader_valid(&self, handle: ShaderHandle) -> bool {
        dispatch!(self, is_shader_valid, handle)
    }

    fn bind_shader(&mut self, handle: ShaderHandle) -> GoudResult<()> {
        dispatch!(self, bind_shader, handle)
    }

    fn unbind_shader(&mut self) {
        dispatch!(self, unbind_shader)
    }

    fn get_uniform_location(&self, handle: ShaderHandle, name: &str) -> Option<i32> {
        dispatch!(self, get_uniform_location, handle, name)
    }

    fn set_uniform_int(&mut self, location: i32, value: i32) {
        dispatch!(self, set_uniform_int, location, value)
    }

    fn set_uniform_float(&mut self, location: i32, value: f32) {
        dispatch!(self, set_uniform_float, location, value)
    }

    fn set_uniform_vec2(&mut self, location: i32, x: f32, y: f32) {
        dispatch!(self, set_uniform_vec2, location, x, y)
    }

    fn set_uniform_vec3(&mut self, location: i32, x: f32, y: f32, z: f32) {
        dispatch!(self, set_uniform_vec3, location, x, y, z)
    }

    fn set_uniform_vec4(&mut self, location: i32, x: f32, y: f32, z: f32, w: f32) {
        dispatch!(self, set_uniform_vec4, location, x, y, z, w)
    }

    fn set_uniform_mat4(&mut self, location: i32, matrix: &[f32; 16]) {
        dispatch!(self, set_uniform_mat4, location, matrix)
    }
}
