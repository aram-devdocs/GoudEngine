//! NullBackend — minimal RenderBackend for CPU-only benchmarks.
//!
//! Shared between benchmark files via `#[path]` module include.

use goud_engine::core::error::GoudResult;
use goud_engine::libs::graphics::backend::capabilities::{BackendCapabilities, BackendInfo};
use goud_engine::libs::graphics::backend::types::{
    BufferHandle, BufferType, BufferUsage, PrimitiveTopology, RenderTargetDesc, RenderTargetHandle,
    ShaderHandle, TextureFilter, TextureFormat, TextureHandle, TextureWrap, VertexLayout,
};
use goud_engine::libs::graphics::backend::{
    BlendFactor, BufferOps, ClearOps, CullFace, DrawOps, FrameOps, RenderBackend, RenderTargetOps,
    ShaderOps, StateOps, TextureOps,
};

pub struct NullBackend {
    info: BackendInfo,
}

impl NullBackend {
    pub fn new() -> Self {
        Self {
            info: BackendInfo {
                name: "NullBackend",
                version: String::new(),
                vendor: String::new(),
                renderer: String::new(),
                capabilities: BackendCapabilities::default(),
            },
        }
    }
}

impl RenderBackend for NullBackend {
    fn info(&self) -> &BackendInfo {
        &self.info
    }
}

impl FrameOps for NullBackend {
    fn begin_frame(&mut self) -> GoudResult<()> {
        Ok(())
    }
    fn end_frame(&mut self) -> GoudResult<()> {
        Ok(())
    }
}

impl ClearOps for NullBackend {
    fn set_clear_color(&mut self, _r: f32, _g: f32, _b: f32, _a: f32) {}
    fn clear_color(&mut self) {}
    fn clear_depth(&mut self) {}
}

impl StateOps for NullBackend {
    fn set_viewport(&mut self, _x: i32, _y: i32, _width: u32, _height: u32) {}
    fn enable_depth_test(&mut self) {}
    fn disable_depth_test(&mut self) {}
    fn enable_blending(&mut self) {}
    fn disable_blending(&mut self) {}
    fn set_blend_func(&mut self, _src: BlendFactor, _dst: BlendFactor) {}
    fn enable_culling(&mut self) {}
    fn disable_culling(&mut self) {}
    fn set_cull_face(&mut self, _face: CullFace) {}
    fn set_depth_func(&mut self, _func: goud_engine::libs::graphics::backend::types::DepthFunc) {}
    fn set_front_face(&mut self, _face: goud_engine::libs::graphics::backend::types::FrontFace) {}
    fn set_depth_mask(&mut self, _enabled: bool) {}
    fn set_line_width(&mut self, _width: f32) {}
}

impl BufferOps for NullBackend {
    fn create_buffer(
        &mut self,
        _buffer_type: BufferType,
        _usage: BufferUsage,
        _data: &[u8],
    ) -> GoudResult<BufferHandle> {
        Ok(BufferHandle::new(0, 1))
    }
    fn update_buffer(
        &mut self,
        _handle: BufferHandle,
        _offset: usize,
        _data: &[u8],
    ) -> GoudResult<()> {
        Ok(())
    }
    fn destroy_buffer(&mut self, _handle: BufferHandle) -> bool {
        true
    }
    fn is_buffer_valid(&self, _handle: BufferHandle) -> bool {
        true
    }
    fn buffer_size(&self, _handle: BufferHandle) -> Option<usize> {
        Some(0)
    }
    fn bind_buffer(&mut self, _handle: BufferHandle) -> GoudResult<()> {
        Ok(())
    }
    fn unbind_buffer(&mut self, _buffer_type: BufferType) {}
}

impl TextureOps for NullBackend {
    fn create_texture(
        &mut self,
        _width: u32,
        _height: u32,
        _format: TextureFormat,
        _filter: TextureFilter,
        _wrap: TextureWrap,
        _data: &[u8],
    ) -> GoudResult<TextureHandle> {
        Ok(TextureHandle::new(0, 1))
    }
    fn update_texture(
        &mut self,
        _handle: TextureHandle,
        _x: u32,
        _y: u32,
        _width: u32,
        _height: u32,
        _data: &[u8],
    ) -> GoudResult<()> {
        Ok(())
    }
    fn destroy_texture(&mut self, _handle: TextureHandle) -> bool {
        true
    }
    fn is_texture_valid(&self, _handle: TextureHandle) -> bool {
        true
    }
    fn texture_size(&self, _handle: TextureHandle) -> Option<(u32, u32)> {
        Some((64, 64))
    }
    fn bind_texture(&mut self, _handle: TextureHandle, _unit: u32) -> GoudResult<()> {
        Ok(())
    }
    fn unbind_texture(&mut self, _unit: u32) {}
}

impl RenderTargetOps for NullBackend {
    fn create_render_target(&mut self, _desc: &RenderTargetDesc) -> GoudResult<RenderTargetHandle> {
        Ok(RenderTargetHandle::new(0, 1))
    }
    fn destroy_render_target(&mut self, _handle: RenderTargetHandle) -> bool {
        true
    }
    fn is_render_target_valid(&self, _handle: RenderTargetHandle) -> bool {
        true
    }
    fn bind_render_target(&mut self, _handle: Option<RenderTargetHandle>) -> GoudResult<()> {
        Ok(())
    }
    fn render_target_texture(&self, _handle: RenderTargetHandle) -> Option<TextureHandle> {
        Some(TextureHandle::new(0, 1))
    }
}

impl ShaderOps for NullBackend {
    fn create_shader(
        &mut self,
        _vertex_src: &str,
        _fragment_src: &str,
    ) -> GoudResult<ShaderHandle> {
        Ok(ShaderHandle::new(0, 1))
    }
    fn destroy_shader(&mut self, _handle: ShaderHandle) -> bool {
        true
    }
    fn is_shader_valid(&self, _handle: ShaderHandle) -> bool {
        true
    }
    fn bind_shader(&mut self, _handle: ShaderHandle) -> GoudResult<()> {
        Ok(())
    }
    fn unbind_shader(&mut self) {}
    fn get_uniform_location(&self, _handle: ShaderHandle, _name: &str) -> Option<i32> {
        Some(0)
    }
    fn set_uniform_int(&mut self, _location: i32, _value: i32) {}
    fn set_uniform_float(&mut self, _location: i32, _value: f32) {}
    fn set_uniform_vec2(&mut self, _location: i32, _x: f32, _y: f32) {}
    fn set_uniform_vec3(&mut self, _location: i32, _x: f32, _y: f32, _z: f32) {}
    fn set_uniform_vec4(&mut self, _location: i32, _x: f32, _y: f32, _z: f32, _w: f32) {}
    fn set_uniform_mat4(&mut self, _location: i32, _matrix: &[f32; 16]) {}
}

impl DrawOps for NullBackend {
    fn set_vertex_attributes(&mut self, _layout: &VertexLayout) {}
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
