use crate::libs::error::GoudResult;

use super::NativeRenderBackend;
use crate::libs::graphics::backend::render_backend::{
    BufferOps, RenderTargetOps, ShaderOps, TextureOps,
};
use crate::libs::graphics::backend::types::{
    BufferHandle, BufferType, BufferUsage, RenderTargetDesc, RenderTargetHandle, ShaderHandle,
    TextureFilter, TextureFormat, TextureHandle, TextureWrap,
};

impl BufferOps for NativeRenderBackend {
    fn create_buffer(
        &mut self,
        buffer_type: BufferType,
        usage: BufferUsage,
        data: &[u8],
    ) -> GoudResult<BufferHandle> {
        match self {
            #[cfg(feature = "legacy-glfw-opengl")]
            Self::OpenGlLegacy(backend) => backend.create_buffer(buffer_type, usage, data),
            #[cfg(all(feature = "native", feature = "wgpu-backend"))]
            Self::Wgpu(backend) => backend.create_buffer(buffer_type, usage, data),
        }
    }

    fn update_buffer(
        &mut self,
        handle: BufferHandle,
        offset: usize,
        data: &[u8],
    ) -> GoudResult<()> {
        match self {
            #[cfg(feature = "legacy-glfw-opengl")]
            Self::OpenGlLegacy(backend) => backend.update_buffer(handle, offset, data),
            #[cfg(all(feature = "native", feature = "wgpu-backend"))]
            Self::Wgpu(backend) => backend.update_buffer(handle, offset, data),
        }
    }

    fn destroy_buffer(&mut self, handle: BufferHandle) -> bool {
        match self {
            #[cfg(feature = "legacy-glfw-opengl")]
            Self::OpenGlLegacy(backend) => backend.destroy_buffer(handle),
            #[cfg(all(feature = "native", feature = "wgpu-backend"))]
            Self::Wgpu(backend) => backend.destroy_buffer(handle),
        }
    }

    fn is_buffer_valid(&self, handle: BufferHandle) -> bool {
        match self {
            #[cfg(feature = "legacy-glfw-opengl")]
            Self::OpenGlLegacy(backend) => backend.is_buffer_valid(handle),
            #[cfg(all(feature = "native", feature = "wgpu-backend"))]
            Self::Wgpu(backend) => backend.is_buffer_valid(handle),
        }
    }

    fn buffer_size(&self, handle: BufferHandle) -> Option<usize> {
        match self {
            #[cfg(feature = "legacy-glfw-opengl")]
            Self::OpenGlLegacy(backend) => backend.buffer_size(handle),
            #[cfg(all(feature = "native", feature = "wgpu-backend"))]
            Self::Wgpu(backend) => backend.buffer_size(handle),
        }
    }

    fn bind_buffer(&mut self, handle: BufferHandle) -> GoudResult<()> {
        match self {
            #[cfg(feature = "legacy-glfw-opengl")]
            Self::OpenGlLegacy(backend) => backend.bind_buffer(handle),
            #[cfg(all(feature = "native", feature = "wgpu-backend"))]
            Self::Wgpu(backend) => backend.bind_buffer(handle),
        }
    }

    fn unbind_buffer(&mut self, buffer_type: BufferType) {
        match self {
            #[cfg(feature = "legacy-glfw-opengl")]
            Self::OpenGlLegacy(backend) => backend.unbind_buffer(buffer_type),
            #[cfg(all(feature = "native", feature = "wgpu-backend"))]
            Self::Wgpu(backend) => backend.unbind_buffer(buffer_type),
        }
    }

    fn supports_storage_buffers(&self) -> bool {
        match self {
            #[cfg(feature = "legacy-glfw-opengl")]
            Self::OpenGlLegacy(backend) => backend.supports_storage_buffers(),
            #[cfg(all(feature = "native", feature = "wgpu-backend"))]
            Self::Wgpu(backend) => backend.supports_storage_buffers(),
        }
    }

    fn create_storage_buffer(&mut self, data: &[u8]) -> GoudResult<BufferHandle> {
        match self {
            #[cfg(feature = "legacy-glfw-opengl")]
            Self::OpenGlLegacy(backend) => backend.create_storage_buffer(data),
            #[cfg(all(feature = "native", feature = "wgpu-backend"))]
            Self::Wgpu(backend) => backend.create_storage_buffer(data),
        }
    }

    fn update_storage_buffer(
        &mut self,
        handle: BufferHandle,
        offset: usize,
        data: &[u8],
    ) -> GoudResult<()> {
        match self {
            #[cfg(feature = "legacy-glfw-opengl")]
            Self::OpenGlLegacy(backend) => backend.update_storage_buffer(handle, offset, data),
            #[cfg(all(feature = "native", feature = "wgpu-backend"))]
            Self::Wgpu(backend) => backend.update_storage_buffer(handle, offset, data),
        }
    }

    fn bind_storage_buffer(&mut self, handle: BufferHandle, binding: u32) -> GoudResult<()> {
        match self {
            #[cfg(feature = "legacy-glfw-opengl")]
            Self::OpenGlLegacy(backend) => backend.bind_storage_buffer(handle, binding),
            #[cfg(all(feature = "native", feature = "wgpu-backend"))]
            Self::Wgpu(backend) => backend.bind_storage_buffer(handle, binding),
        }
    }

    fn unbind_storage_buffer(&mut self) {
        match self {
            #[cfg(feature = "legacy-glfw-opengl")]
            Self::OpenGlLegacy(backend) => backend.unbind_storage_buffer(),
            #[cfg(all(feature = "native", feature = "wgpu-backend"))]
            Self::Wgpu(backend) => backend.unbind_storage_buffer(),
        }
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
        match self {
            #[cfg(feature = "legacy-glfw-opengl")]
            Self::OpenGlLegacy(backend) => {
                backend.create_texture(width, height, format, filter, wrap, data)
            }
            #[cfg(all(feature = "native", feature = "wgpu-backend"))]
            Self::Wgpu(backend) => {
                backend.create_texture(width, height, format, filter, wrap, data)
            }
        }
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
        match self {
            #[cfg(feature = "legacy-glfw-opengl")]
            Self::OpenGlLegacy(backend) => {
                backend.update_texture(handle, x, y, width, height, data)
            }
            #[cfg(all(feature = "native", feature = "wgpu-backend"))]
            Self::Wgpu(backend) => backend.update_texture(handle, x, y, width, height, data),
        }
    }

    fn destroy_texture(&mut self, handle: TextureHandle) -> bool {
        match self {
            #[cfg(feature = "legacy-glfw-opengl")]
            Self::OpenGlLegacy(backend) => backend.destroy_texture(handle),
            #[cfg(all(feature = "native", feature = "wgpu-backend"))]
            Self::Wgpu(backend) => backend.destroy_texture(handle),
        }
    }

    fn is_texture_valid(&self, handle: TextureHandle) -> bool {
        match self {
            #[cfg(feature = "legacy-glfw-opengl")]
            Self::OpenGlLegacy(backend) => backend.is_texture_valid(handle),
            #[cfg(all(feature = "native", feature = "wgpu-backend"))]
            Self::Wgpu(backend) => backend.is_texture_valid(handle),
        }
    }

    fn texture_size(&self, handle: TextureHandle) -> Option<(u32, u32)> {
        match self {
            #[cfg(feature = "legacy-glfw-opengl")]
            Self::OpenGlLegacy(backend) => backend.texture_size(handle),
            #[cfg(all(feature = "native", feature = "wgpu-backend"))]
            Self::Wgpu(backend) => backend.texture_size(handle),
        }
    }

    fn bind_texture(&mut self, handle: TextureHandle, unit: u32) -> GoudResult<()> {
        match self {
            #[cfg(feature = "legacy-glfw-opengl")]
            Self::OpenGlLegacy(backend) => backend.bind_texture(handle, unit),
            #[cfg(all(feature = "native", feature = "wgpu-backend"))]
            Self::Wgpu(backend) => backend.bind_texture(handle, unit),
        }
    }

    fn unbind_texture(&mut self, unit: u32) {
        match self {
            #[cfg(feature = "legacy-glfw-opengl")]
            Self::OpenGlLegacy(backend) => backend.unbind_texture(unit),
            #[cfg(all(feature = "native", feature = "wgpu-backend"))]
            Self::Wgpu(backend) => backend.unbind_texture(unit),
        }
    }

    fn create_compressed_texture(
        &mut self,
        width: u32,
        height: u32,
        format: TextureFormat,
        data: &[u8],
        mip_levels: u32,
    ) -> GoudResult<TextureHandle> {
        match self {
            #[cfg(feature = "legacy-glfw-opengl")]
            Self::OpenGlLegacy(backend) => {
                backend.create_compressed_texture(width, height, format, data, mip_levels)
            }
            #[cfg(all(feature = "native", feature = "wgpu-backend"))]
            Self::Wgpu(backend) => {
                backend.create_compressed_texture(width, height, format, data, mip_levels)
            }
        }
    }
}

impl RenderTargetOps for NativeRenderBackend {
    fn create_render_target(&mut self, desc: &RenderTargetDesc) -> GoudResult<RenderTargetHandle> {
        match self {
            #[cfg(feature = "legacy-glfw-opengl")]
            Self::OpenGlLegacy(backend) => backend.create_render_target(desc),
            #[cfg(all(feature = "native", feature = "wgpu-backend"))]
            Self::Wgpu(backend) => backend.create_render_target(desc),
        }
    }

    fn destroy_render_target(&mut self, handle: RenderTargetHandle) -> bool {
        match self {
            #[cfg(feature = "legacy-glfw-opengl")]
            Self::OpenGlLegacy(backend) => backend.destroy_render_target(handle),
            #[cfg(all(feature = "native", feature = "wgpu-backend"))]
            Self::Wgpu(backend) => backend.destroy_render_target(handle),
        }
    }

    fn is_render_target_valid(&self, handle: RenderTargetHandle) -> bool {
        match self {
            #[cfg(feature = "legacy-glfw-opengl")]
            Self::OpenGlLegacy(backend) => backend.is_render_target_valid(handle),
            #[cfg(all(feature = "native", feature = "wgpu-backend"))]
            Self::Wgpu(backend) => backend.is_render_target_valid(handle),
        }
    }

    fn bind_render_target(&mut self, handle: Option<RenderTargetHandle>) -> GoudResult<()> {
        match self {
            #[cfg(feature = "legacy-glfw-opengl")]
            Self::OpenGlLegacy(backend) => backend.bind_render_target(handle),
            #[cfg(all(feature = "native", feature = "wgpu-backend"))]
            Self::Wgpu(backend) => backend.bind_render_target(handle),
        }
    }

    fn render_target_texture(&self, handle: RenderTargetHandle) -> Option<TextureHandle> {
        match self {
            #[cfg(feature = "legacy-glfw-opengl")]
            Self::OpenGlLegacy(backend) => backend.render_target_texture(handle),
            #[cfg(all(feature = "native", feature = "wgpu-backend"))]
            Self::Wgpu(backend) => backend.render_target_texture(handle),
        }
    }
}

impl ShaderOps for NativeRenderBackend {
    fn create_shader(&mut self, vertex_src: &str, fragment_src: &str) -> GoudResult<ShaderHandle> {
        match self {
            #[cfg(feature = "legacy-glfw-opengl")]
            Self::OpenGlLegacy(backend) => backend.create_shader(vertex_src, fragment_src),
            #[cfg(all(feature = "native", feature = "wgpu-backend"))]
            Self::Wgpu(backend) => backend.create_shader(vertex_src, fragment_src),
        }
    }

    fn destroy_shader(&mut self, handle: ShaderHandle) -> bool {
        match self {
            #[cfg(feature = "legacy-glfw-opengl")]
            Self::OpenGlLegacy(backend) => backend.destroy_shader(handle),
            #[cfg(all(feature = "native", feature = "wgpu-backend"))]
            Self::Wgpu(backend) => backend.destroy_shader(handle),
        }
    }

    fn is_shader_valid(&self, handle: ShaderHandle) -> bool {
        match self {
            #[cfg(feature = "legacy-glfw-opengl")]
            Self::OpenGlLegacy(backend) => backend.is_shader_valid(handle),
            #[cfg(all(feature = "native", feature = "wgpu-backend"))]
            Self::Wgpu(backend) => backend.is_shader_valid(handle),
        }
    }

    fn bind_shader(&mut self, handle: ShaderHandle) -> GoudResult<()> {
        match self {
            #[cfg(feature = "legacy-glfw-opengl")]
            Self::OpenGlLegacy(backend) => backend.bind_shader(handle),
            #[cfg(all(feature = "native", feature = "wgpu-backend"))]
            Self::Wgpu(backend) => backend.bind_shader(handle),
        }
    }

    fn unbind_shader(&mut self) {
        match self {
            #[cfg(feature = "legacy-glfw-opengl")]
            Self::OpenGlLegacy(backend) => backend.unbind_shader(),
            #[cfg(all(feature = "native", feature = "wgpu-backend"))]
            Self::Wgpu(backend) => backend.unbind_shader(),
        }
    }

    fn get_uniform_location(&self, handle: ShaderHandle, name: &str) -> Option<i32> {
        match self {
            #[cfg(feature = "legacy-glfw-opengl")]
            Self::OpenGlLegacy(backend) => backend.get_uniform_location(handle, name),
            #[cfg(all(feature = "native", feature = "wgpu-backend"))]
            Self::Wgpu(backend) => backend.get_uniform_location(handle, name),
        }
    }

    fn set_uniform_int(&mut self, location: i32, value: i32) {
        match self {
            #[cfg(feature = "legacy-glfw-opengl")]
            Self::OpenGlLegacy(backend) => backend.set_uniform_int(location, value),
            #[cfg(all(feature = "native", feature = "wgpu-backend"))]
            Self::Wgpu(backend) => backend.set_uniform_int(location, value),
        }
    }

    fn set_uniform_float(&mut self, location: i32, value: f32) {
        match self {
            #[cfg(feature = "legacy-glfw-opengl")]
            Self::OpenGlLegacy(backend) => backend.set_uniform_float(location, value),
            #[cfg(all(feature = "native", feature = "wgpu-backend"))]
            Self::Wgpu(backend) => backend.set_uniform_float(location, value),
        }
    }

    fn set_uniform_vec2(&mut self, location: i32, x: f32, y: f32) {
        match self {
            #[cfg(feature = "legacy-glfw-opengl")]
            Self::OpenGlLegacy(backend) => backend.set_uniform_vec2(location, x, y),
            #[cfg(all(feature = "native", feature = "wgpu-backend"))]
            Self::Wgpu(backend) => backend.set_uniform_vec2(location, x, y),
        }
    }

    fn set_uniform_vec3(&mut self, location: i32, x: f32, y: f32, z: f32) {
        match self {
            #[cfg(feature = "legacy-glfw-opengl")]
            Self::OpenGlLegacy(backend) => backend.set_uniform_vec3(location, x, y, z),
            #[cfg(all(feature = "native", feature = "wgpu-backend"))]
            Self::Wgpu(backend) => backend.set_uniform_vec3(location, x, y, z),
        }
    }

    fn set_uniform_vec4(&mut self, location: i32, x: f32, y: f32, z: f32, w: f32) {
        match self {
            #[cfg(feature = "legacy-glfw-opengl")]
            Self::OpenGlLegacy(backend) => backend.set_uniform_vec4(location, x, y, z, w),
            #[cfg(all(feature = "native", feature = "wgpu-backend"))]
            Self::Wgpu(backend) => backend.set_uniform_vec4(location, x, y, z, w),
        }
    }

    fn set_uniform_mat4(&mut self, location: i32, matrix: &[f32; 16]) {
        match self {
            #[cfg(feature = "legacy-glfw-opengl")]
            Self::OpenGlLegacy(backend) => backend.set_uniform_mat4(location, matrix),
            #[cfg(all(feature = "native", feature = "wgpu-backend"))]
            Self::Wgpu(backend) => backend.set_uniform_mat4(location, matrix),
        }
    }
}
