//! Sub-trait and `RenderBackend` implementations for `NullBackend`.

use crate::libs::error::{GoudError, GoudResult};
use crate::libs::graphics::backend::blend::{BlendFactor, CullFace};
use crate::libs::graphics::backend::capabilities::BackendInfo;
use crate::libs::graphics::backend::render_backend::{
    BufferOps, ClearOps, DrawOps, FrameOps, RenderBackend, ShaderOps, StateOps, TextureOps,
};
use crate::libs::graphics::backend::types::{
    BufferHandle, BufferType, BufferUsage, DepthFunc, FrontFace, PrimitiveTopology, ShaderHandle,
    TextureFilter, TextureFormat, TextureHandle, TextureWrap, VertexLayout,
};

use super::backend::{NullBufferMeta, NullTextureMeta};
use super::NullBackend;

// ========================================================================
// RenderBackend (supertrait)
// ========================================================================

impl RenderBackend for NullBackend {
    fn info(&self) -> &BackendInfo {
        &self.info
    }
}

// ========================================================================
// FrameOps
// ========================================================================

impl FrameOps for NullBackend {
    fn begin_frame(&mut self) -> GoudResult<()> {
        Ok(())
    }

    fn end_frame(&mut self) -> GoudResult<()> {
        Ok(())
    }
}

// ========================================================================
// ClearOps
// ========================================================================

impl ClearOps for NullBackend {
    fn set_clear_color(&mut self, r: f32, g: f32, b: f32, a: f32) {
        self.clear_color = [r, g, b, a];
    }

    fn clear_color(&mut self) {
        // no-op
    }

    fn clear_depth(&mut self) {
        // no-op
    }
}

// ========================================================================
// StateOps
// ========================================================================

impl StateOps for NullBackend {
    fn set_viewport(&mut self, x: i32, y: i32, width: u32, height: u32) {
        self.viewport = (x, y, width, height);
    }

    fn enable_depth_test(&mut self) {
        self.depth_test_enabled = true;
    }

    fn disable_depth_test(&mut self) {
        self.depth_test_enabled = false;
    }

    fn enable_blending(&mut self) {
        self.blending_enabled = true;
    }

    fn disable_blending(&mut self) {
        self.blending_enabled = false;
    }

    fn set_blend_func(&mut self, _src: BlendFactor, _dst: BlendFactor) {
        // no-op
    }

    fn enable_culling(&mut self) {
        self.culling_enabled = true;
    }

    fn disable_culling(&mut self) {
        self.culling_enabled = false;
    }

    fn set_cull_face(&mut self, _face: CullFace) {
        // no-op
    }

    fn set_depth_func(&mut self, _func: DepthFunc) {
        // no-op
    }

    fn set_front_face(&mut self, _face: FrontFace) {
        // no-op
    }

    fn set_depth_mask(&mut self, enabled: bool) {
        self.depth_mask_enabled = enabled;
    }

    fn set_line_width(&mut self, width: f32) {
        self.line_width = width;
    }
}

// ========================================================================
// BufferOps
// ========================================================================

impl BufferOps for NullBackend {
    fn create_buffer(
        &mut self,
        buffer_type: BufferType,
        _usage: BufferUsage,
        data: &[u8],
    ) -> GoudResult<BufferHandle> {
        let handle = self.buffer_allocator.allocate();
        self.buffers.insert(
            handle,
            NullBufferMeta {
                size: data.len(),
                _buffer_type: buffer_type,
            },
        );
        Ok(handle)
    }

    fn update_buffer(
        &mut self,
        handle: BufferHandle,
        _offset: usize,
        _data: &[u8],
    ) -> GoudResult<()> {
        if self.buffer_allocator.is_alive(handle) {
            Ok(())
        } else {
            Err(GoudError::InvalidHandle)
        }
    }

    fn destroy_buffer(&mut self, handle: BufferHandle) -> bool {
        if self.buffer_allocator.deallocate(handle) {
            self.buffers.remove(&handle);
            true
        } else {
            false
        }
    }

    fn is_buffer_valid(&self, handle: BufferHandle) -> bool {
        self.buffer_allocator.is_alive(handle)
    }

    fn buffer_size(&self, handle: BufferHandle) -> Option<usize> {
        self.buffers.get(&handle).map(|m| m.size)
    }

    fn bind_buffer(&mut self, handle: BufferHandle) -> GoudResult<()> {
        if self.buffer_allocator.is_alive(handle) {
            Ok(())
        } else {
            Err(GoudError::InvalidHandle)
        }
    }

    fn unbind_buffer(&mut self, _buffer_type: BufferType) {
        // no-op
    }
}

// ========================================================================
// TextureOps
// ========================================================================

impl TextureOps for NullBackend {
    fn create_texture(
        &mut self,
        width: u32,
        height: u32,
        _format: TextureFormat,
        _filter: TextureFilter,
        _wrap: TextureWrap,
        _data: &[u8],
    ) -> GoudResult<TextureHandle> {
        let handle = self.texture_allocator.allocate();
        self.textures
            .insert(handle, NullTextureMeta { width, height });
        Ok(handle)
    }

    fn update_texture(
        &mut self,
        handle: TextureHandle,
        _x: u32,
        _y: u32,
        _width: u32,
        _height: u32,
        _data: &[u8],
    ) -> GoudResult<()> {
        if self.texture_allocator.is_alive(handle) {
            Ok(())
        } else {
            Err(GoudError::InvalidHandle)
        }
    }

    fn destroy_texture(&mut self, handle: TextureHandle) -> bool {
        if self.texture_allocator.deallocate(handle) {
            self.textures.remove(&handle);
            true
        } else {
            false
        }
    }

    fn is_texture_valid(&self, handle: TextureHandle) -> bool {
        self.texture_allocator.is_alive(handle)
    }

    fn texture_size(&self, handle: TextureHandle) -> Option<(u32, u32)> {
        self.textures.get(&handle).map(|m| (m.width, m.height))
    }

    fn bind_texture(&mut self, handle: TextureHandle, _unit: u32) -> GoudResult<()> {
        if self.texture_allocator.is_alive(handle) {
            Ok(())
        } else {
            Err(GoudError::InvalidHandle)
        }
    }

    fn unbind_texture(&mut self, _unit: u32) {
        // no-op
    }
}

// ========================================================================
// ShaderOps
// ========================================================================

impl ShaderOps for NullBackend {
    fn create_shader(
        &mut self,
        _vertex_src: &str,
        _fragment_src: &str,
    ) -> GoudResult<ShaderHandle> {
        let handle = self.shader_allocator.allocate();
        Ok(handle)
    }

    fn destroy_shader(&mut self, handle: ShaderHandle) -> bool {
        self.shader_allocator.deallocate(handle)
    }

    fn is_shader_valid(&self, handle: ShaderHandle) -> bool {
        self.shader_allocator.is_alive(handle)
    }

    fn bind_shader(&mut self, handle: ShaderHandle) -> GoudResult<()> {
        if self.shader_allocator.is_alive(handle) {
            Ok(())
        } else {
            Err(GoudError::InvalidHandle)
        }
    }

    fn unbind_shader(&mut self) {
        // no-op
    }

    fn get_uniform_location(&self, handle: ShaderHandle, _name: &str) -> Option<i32> {
        if self.shader_allocator.is_alive(handle) {
            Some(0)
        } else {
            None
        }
    }

    fn set_uniform_int(&mut self, _location: i32, _value: i32) {
        // no-op
    }

    fn set_uniform_float(&mut self, _location: i32, _value: f32) {
        // no-op
    }

    fn set_uniform_vec2(&mut self, _location: i32, _x: f32, _y: f32) {
        // no-op
    }

    fn set_uniform_vec3(&mut self, _location: i32, _x: f32, _y: f32, _z: f32) {
        // no-op
    }

    fn set_uniform_vec4(&mut self, _location: i32, _x: f32, _y: f32, _z: f32, _w: f32) {
        // no-op
    }

    fn set_uniform_mat4(&mut self, _location: i32, _matrix: &[f32; 16]) {
        // no-op
    }
}

// ========================================================================
// DrawOps
// ========================================================================

impl DrawOps for NullBackend {
    fn set_vertex_attributes(&mut self, _layout: &VertexLayout) {
        // no-op
    }

    fn draw_arrays(
        &mut self,
        _topology: PrimitiveTopology,
        _first: u32,
        _count: u32,
    ) -> GoudResult<()> {
        Ok(())
    }

    fn draw_indexed(
        &mut self,
        _topology: PrimitiveTopology,
        _count: u32,
        _offset: usize,
    ) -> GoudResult<()> {
        Ok(())
    }

    fn draw_indexed_u16(
        &mut self,
        _topology: PrimitiveTopology,
        _count: u32,
        _offset: usize,
    ) -> GoudResult<()> {
        Ok(())
    }

    fn draw_arrays_instanced(
        &mut self,
        _topology: PrimitiveTopology,
        _first: u32,
        _count: u32,
        _instance_count: u32,
    ) -> GoudResult<()> {
        Ok(())
    }

    fn draw_indexed_instanced(
        &mut self,
        _topology: PrimitiveTopology,
        _count: u32,
        _offset: usize,
        _instance_count: u32,
    ) -> GoudResult<()> {
        Ok(())
    }
}
