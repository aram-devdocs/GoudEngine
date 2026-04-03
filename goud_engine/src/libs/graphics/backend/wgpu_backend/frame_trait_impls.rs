//! Texture and shader trait implementations for `WgpuBackend`.
//!
//! Split from `frame.rs` to stay within the 500-line file limit.

use super::{
    super::types::{TextureFilter, TextureFormat, TextureWrap},
    ShaderHandle, ShaderOps, TextureHandle, TextureOps, WgpuBackend,
};
use crate::libs::error::GoudResult;

// ========================================================================
// TextureOps (delegated to texture.rs)
// ========================================================================

impl TextureOps for WgpuBackend {
    fn create_texture(
        &mut self,
        width: u32,
        height: u32,
        format: TextureFormat,
        filter: TextureFilter,
        wrap: TextureWrap,
        data: &[u8],
    ) -> GoudResult<TextureHandle> {
        self.create_texture_impl(width, height, format, filter, wrap, data)
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
        self.update_texture_impl(handle, x, y, width, height, data)
    }

    fn destroy_texture(&mut self, handle: TextureHandle) -> bool {
        self.destroy_texture_impl(handle)
    }

    fn is_texture_valid(&self, handle: TextureHandle) -> bool {
        self.is_texture_valid_impl(handle)
    }

    fn texture_size(&self, handle: TextureHandle) -> Option<(u32, u32)> {
        self.texture_size_impl(handle)
    }

    fn bind_texture(&mut self, handle: TextureHandle, unit: u32) -> GoudResult<()> {
        self.bind_texture_impl(handle, unit)
    }

    fn unbind_texture(&mut self, unit: u32) {
        self.unbind_texture_impl(unit);
    }
}

// ========================================================================
// ShaderOps (delegated to shader.rs)
// ========================================================================

impl ShaderOps for WgpuBackend {
    fn create_shader(&mut self, vertex_src: &str, fragment_src: &str) -> GoudResult<ShaderHandle> {
        self.create_shader_impl(vertex_src, fragment_src)
    }

    fn destroy_shader(&mut self, handle: ShaderHandle) -> bool {
        self.destroy_shader_impl(handle)
    }

    fn is_shader_valid(&self, handle: ShaderHandle) -> bool {
        self.is_shader_valid_impl(handle)
    }

    fn bind_shader(&mut self, handle: ShaderHandle) -> GoudResult<()> {
        self.bind_shader_impl(handle)
    }

    fn unbind_shader(&mut self) {
        self.unbind_shader_impl();
    }

    fn get_uniform_location(&self, handle: ShaderHandle, name: &str) -> Option<i32> {
        self.get_uniform_location_impl(handle, name)
    }

    fn set_uniform_int(&mut self, location: i32, value: i32) {
        self.write_uniform(location, &value.to_le_bytes());
    }

    fn set_uniform_float(&mut self, location: i32, value: f32) {
        self.write_uniform(location, &value.to_le_bytes());
    }

    fn set_uniform_vec2(&mut self, location: i32, x: f32, y: f32) {
        let mut buf = [0u8; 8];
        buf[0..4].copy_from_slice(&x.to_le_bytes());
        buf[4..8].copy_from_slice(&y.to_le_bytes());
        self.write_uniform(location, &buf);
    }

    fn set_uniform_vec3(&mut self, location: i32, x: f32, y: f32, z: f32) {
        let mut buf = [0u8; 12];
        buf[0..4].copy_from_slice(&x.to_le_bytes());
        buf[4..8].copy_from_slice(&y.to_le_bytes());
        buf[8..12].copy_from_slice(&z.to_le_bytes());
        self.write_uniform(location, &buf);
    }

    fn set_uniform_vec4(&mut self, location: i32, x: f32, y: f32, z: f32, w: f32) {
        let mut buf = [0u8; 16];
        buf[0..4].copy_from_slice(&x.to_le_bytes());
        buf[4..8].copy_from_slice(&y.to_le_bytes());
        buf[8..12].copy_from_slice(&z.to_le_bytes());
        buf[12..16].copy_from_slice(&w.to_le_bytes());
        self.write_uniform(location, &buf);
    }

    fn set_uniform_mat4(&mut self, location: i32, matrix: &[f32; 16]) {
        self.write_uniform(location, bytemuck::cast_slice(matrix));
    }
}
