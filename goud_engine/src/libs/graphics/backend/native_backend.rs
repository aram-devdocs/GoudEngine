//! Shared native render backend wrappers.

use std::sync::{Arc, Mutex, MutexGuard};

use crate::libs::error::GoudResult;

use super::{
    capabilities::BackendInfo,
    render_backend::{
        BufferOps, ClearOps, DrawOps, FrameOps, RenderBackend, ShaderOps, StateOps, TextureOps,
    },
    types::{
        BufferHandle, BufferType, BufferUsage, PrimitiveTopology, ShaderHandle, TextureFilter,
        TextureFormat, TextureHandle, TextureWrap, VertexLayout,
    },
};

#[cfg(feature = "legacy-glfw-opengl")]
use super::opengl::OpenGLBackend;
#[cfg(all(feature = "native", feature = "wgpu-backend"))]
use super::wgpu_backend::WgpuBackend;

/// Concrete native render backend choice.
pub enum NativeRenderBackend {
    #[cfg(feature = "legacy-glfw-opengl")]
    /// Legacy OpenGL backend selected through the explicit legacy feature gate.
    OpenGlLegacy(OpenGLBackend),
    #[cfg(all(feature = "native", feature = "wgpu-backend"))]
    /// Default wgpu backend used by the native runtime.
    Wgpu(WgpuBackend),
}

impl NativeRenderBackend {
    fn info(&self) -> &BackendInfo {
        match self {
            #[cfg(feature = "legacy-glfw-opengl")]
            Self::OpenGlLegacy(backend) => backend.info(),
            #[cfg(all(feature = "native", feature = "wgpu-backend"))]
            Self::Wgpu(backend) => backend.info(),
        }
    }

    pub(crate) fn info_clone(&self) -> BackendInfo {
        self.info().clone()
    }

    pub(crate) fn bind_texture_by_index(&mut self, index: u32, unit: u32) -> GoudResult<()> {
        match self {
            #[cfg(feature = "legacy-glfw-opengl")]
            Self::OpenGlLegacy(backend) => backend.bind_texture_by_index(index, unit),
            #[cfg(all(feature = "native", feature = "wgpu-backend"))]
            Self::Wgpu(backend) => backend.bind_texture_by_index(index, unit),
        }
    }

    pub(crate) fn resize_surface(&mut self, width: u32, height: u32) {
        match self {
            #[cfg(feature = "legacy-glfw-opengl")]
            Self::OpenGlLegacy(backend) => backend.set_viewport(0, 0, width, height),
            #[cfg(all(feature = "native", feature = "wgpu-backend"))]
            Self::Wgpu(backend) => backend.resize(width, height),
        }
    }
}

impl RenderBackend for NativeRenderBackend {
    fn info(&self) -> &BackendInfo {
        self.info()
    }

    fn bind_default_vertex_array(&mut self) {
        match self {
            #[cfg(feature = "legacy-glfw-opengl")]
            Self::OpenGlLegacy(backend) => backend.bind_default_vertex_array(),
            #[cfg(all(feature = "native", feature = "wgpu-backend"))]
            Self::Wgpu(backend) => backend.bind_default_vertex_array(),
        }
    }

    fn validate_text_draw_state(&self) -> Result<(), String> {
        match self {
            #[cfg(feature = "legacy-glfw-opengl")]
            Self::OpenGlLegacy(backend) => backend.validate_text_draw_state(),
            #[cfg(all(feature = "native", feature = "wgpu-backend"))]
            Self::Wgpu(backend) => backend.validate_text_draw_state(),
        }
    }

    fn read_default_framebuffer_rgba8(
        &mut self,
        width: u32,
        height: u32,
    ) -> Result<Vec<u8>, String> {
        match self {
            #[cfg(feature = "legacy-glfw-opengl")]
            Self::OpenGlLegacy(backend) => backend.read_default_framebuffer_rgba8(width, height),
            #[cfg(all(feature = "native", feature = "wgpu-backend"))]
            Self::Wgpu(backend) => backend.read_default_framebuffer_rgba8(width, height),
        }
    }
}

impl FrameOps for NativeRenderBackend {
    fn begin_frame(&mut self) -> GoudResult<()> {
        match self {
            #[cfg(feature = "legacy-glfw-opengl")]
            Self::OpenGlLegacy(backend) => backend.begin_frame(),
            #[cfg(all(feature = "native", feature = "wgpu-backend"))]
            Self::Wgpu(backend) => backend.begin_frame(),
        }
    }

    fn end_frame(&mut self) -> GoudResult<()> {
        match self {
            #[cfg(feature = "legacy-glfw-opengl")]
            Self::OpenGlLegacy(backend) => backend.end_frame(),
            #[cfg(all(feature = "native", feature = "wgpu-backend"))]
            Self::Wgpu(backend) => backend.end_frame(),
        }
    }
}

impl ClearOps for NativeRenderBackend {
    fn set_clear_color(&mut self, r: f32, g: f32, b: f32, a: f32) {
        match self {
            #[cfg(feature = "legacy-glfw-opengl")]
            Self::OpenGlLegacy(backend) => backend.set_clear_color(r, g, b, a),
            #[cfg(all(feature = "native", feature = "wgpu-backend"))]
            Self::Wgpu(backend) => backend.set_clear_color(r, g, b, a),
        }
    }

    fn clear_color(&mut self) {
        match self {
            #[cfg(feature = "legacy-glfw-opengl")]
            Self::OpenGlLegacy(backend) => backend.clear_color(),
            #[cfg(all(feature = "native", feature = "wgpu-backend"))]
            Self::Wgpu(backend) => backend.clear_color(),
        }
    }

    fn clear_depth(&mut self) {
        match self {
            #[cfg(feature = "legacy-glfw-opengl")]
            Self::OpenGlLegacy(backend) => backend.clear_depth(),
            #[cfg(all(feature = "native", feature = "wgpu-backend"))]
            Self::Wgpu(backend) => backend.clear_depth(),
        }
    }
}

impl StateOps for NativeRenderBackend {
    fn set_viewport(&mut self, x: i32, y: i32, width: u32, height: u32) {
        match self {
            #[cfg(feature = "legacy-glfw-opengl")]
            Self::OpenGlLegacy(backend) => backend.set_viewport(x, y, width, height),
            #[cfg(all(feature = "native", feature = "wgpu-backend"))]
            Self::Wgpu(backend) => backend.set_viewport(x, y, width, height),
        }
    }

    fn enable_depth_test(&mut self) {
        match self {
            #[cfg(feature = "legacy-glfw-opengl")]
            Self::OpenGlLegacy(backend) => backend.enable_depth_test(),
            #[cfg(all(feature = "native", feature = "wgpu-backend"))]
            Self::Wgpu(backend) => backend.enable_depth_test(),
        }
    }

    fn disable_depth_test(&mut self) {
        match self {
            #[cfg(feature = "legacy-glfw-opengl")]
            Self::OpenGlLegacy(backend) => backend.disable_depth_test(),
            #[cfg(all(feature = "native", feature = "wgpu-backend"))]
            Self::Wgpu(backend) => backend.disable_depth_test(),
        }
    }

    fn enable_blending(&mut self) {
        match self {
            #[cfg(feature = "legacy-glfw-opengl")]
            Self::OpenGlLegacy(backend) => backend.enable_blending(),
            #[cfg(all(feature = "native", feature = "wgpu-backend"))]
            Self::Wgpu(backend) => backend.enable_blending(),
        }
    }

    fn disable_blending(&mut self) {
        match self {
            #[cfg(feature = "legacy-glfw-opengl")]
            Self::OpenGlLegacy(backend) => backend.disable_blending(),
            #[cfg(all(feature = "native", feature = "wgpu-backend"))]
            Self::Wgpu(backend) => backend.disable_blending(),
        }
    }

    fn set_blend_func(&mut self, src: super::BlendFactor, dst: super::BlendFactor) {
        match self {
            #[cfg(feature = "legacy-glfw-opengl")]
            Self::OpenGlLegacy(backend) => backend.set_blend_func(src, dst),
            #[cfg(all(feature = "native", feature = "wgpu-backend"))]
            Self::Wgpu(backend) => backend.set_blend_func(src, dst),
        }
    }

    fn enable_culling(&mut self) {
        match self {
            #[cfg(feature = "legacy-glfw-opengl")]
            Self::OpenGlLegacy(backend) => backend.enable_culling(),
            #[cfg(all(feature = "native", feature = "wgpu-backend"))]
            Self::Wgpu(backend) => backend.enable_culling(),
        }
    }

    fn disable_culling(&mut self) {
        match self {
            #[cfg(feature = "legacy-glfw-opengl")]
            Self::OpenGlLegacy(backend) => backend.disable_culling(),
            #[cfg(all(feature = "native", feature = "wgpu-backend"))]
            Self::Wgpu(backend) => backend.disable_culling(),
        }
    }

    fn set_cull_face(&mut self, face: super::CullFace) {
        match self {
            #[cfg(feature = "legacy-glfw-opengl")]
            Self::OpenGlLegacy(backend) => backend.set_cull_face(face),
            #[cfg(all(feature = "native", feature = "wgpu-backend"))]
            Self::Wgpu(backend) => backend.set_cull_face(face),
        }
    }

    fn set_depth_func(&mut self, func: super::types::DepthFunc) {
        match self {
            #[cfg(feature = "legacy-glfw-opengl")]
            Self::OpenGlLegacy(backend) => backend.set_depth_func(func),
            #[cfg(all(feature = "native", feature = "wgpu-backend"))]
            Self::Wgpu(backend) => backend.set_depth_func(func),
        }
    }

    fn set_front_face(&mut self, face: super::types::FrontFace) {
        match self {
            #[cfg(feature = "legacy-glfw-opengl")]
            Self::OpenGlLegacy(backend) => backend.set_front_face(face),
            #[cfg(all(feature = "native", feature = "wgpu-backend"))]
            Self::Wgpu(backend) => backend.set_front_face(face),
        }
    }

    fn set_depth_mask(&mut self, enabled: bool) {
        match self {
            #[cfg(feature = "legacy-glfw-opengl")]
            Self::OpenGlLegacy(backend) => backend.set_depth_mask(enabled),
            #[cfg(all(feature = "native", feature = "wgpu-backend"))]
            Self::Wgpu(backend) => backend.set_depth_mask(enabled),
        }
    }

    fn set_line_width(&mut self, width: f32) {
        match self {
            #[cfg(feature = "legacy-glfw-opengl")]
            Self::OpenGlLegacy(backend) => backend.set_line_width(width),
            #[cfg(all(feature = "native", feature = "wgpu-backend"))]
            Self::Wgpu(backend) => backend.set_line_width(width),
        }
    }
}

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

impl DrawOps for NativeRenderBackend {
    fn set_vertex_attributes(&mut self, layout: &VertexLayout) {
        match self {
            #[cfg(feature = "legacy-glfw-opengl")]
            Self::OpenGlLegacy(backend) => backend.set_vertex_attributes(layout),
            #[cfg(all(feature = "native", feature = "wgpu-backend"))]
            Self::Wgpu(backend) => backend.set_vertex_attributes(layout),
        }
    }

    fn draw_arrays(
        &mut self,
        topology: PrimitiveTopology,
        first: u32,
        count: u32,
    ) -> GoudResult<()> {
        match self {
            #[cfg(feature = "legacy-glfw-opengl")]
            Self::OpenGlLegacy(backend) => backend.draw_arrays(topology, first, count),
            #[cfg(all(feature = "native", feature = "wgpu-backend"))]
            Self::Wgpu(backend) => backend.draw_arrays(topology, first, count),
        }
    }

    fn draw_indexed(
        &mut self,
        topology: PrimitiveTopology,
        count: u32,
        offset: usize,
    ) -> GoudResult<()> {
        match self {
            #[cfg(feature = "legacy-glfw-opengl")]
            Self::OpenGlLegacy(backend) => backend.draw_indexed(topology, count, offset),
            #[cfg(all(feature = "native", feature = "wgpu-backend"))]
            Self::Wgpu(backend) => backend.draw_indexed(topology, count, offset),
        }
    }

    fn draw_indexed_u16(
        &mut self,
        topology: PrimitiveTopology,
        count: u32,
        offset: usize,
    ) -> GoudResult<()> {
        match self {
            #[cfg(feature = "legacy-glfw-opengl")]
            Self::OpenGlLegacy(backend) => backend.draw_indexed_u16(topology, count, offset),
            #[cfg(all(feature = "native", feature = "wgpu-backend"))]
            Self::Wgpu(backend) => backend.draw_indexed_u16(topology, count, offset),
        }
    }

    fn draw_arrays_instanced(
        &mut self,
        topology: PrimitiveTopology,
        first: u32,
        count: u32,
        instance_count: u32,
    ) -> GoudResult<()> {
        match self {
            #[cfg(feature = "legacy-glfw-opengl")]
            Self::OpenGlLegacy(backend) => {
                backend.draw_arrays_instanced(topology, first, count, instance_count)
            }
            #[cfg(all(feature = "native", feature = "wgpu-backend"))]
            Self::Wgpu(backend) => {
                backend.draw_arrays_instanced(topology, first, count, instance_count)
            }
        }
    }

    fn draw_indexed_instanced(
        &mut self,
        topology: PrimitiveTopology,
        count: u32,
        offset: usize,
        instance_count: u32,
    ) -> GoudResult<()> {
        match self {
            #[cfg(feature = "legacy-glfw-opengl")]
            Self::OpenGlLegacy(backend) => {
                backend.draw_indexed_instanced(topology, count, offset, instance_count)
            }
            #[cfg(all(feature = "native", feature = "wgpu-backend"))]
            Self::Wgpu(backend) => {
                backend.draw_indexed_instanced(topology, count, offset, instance_count)
            }
        }
    }
}

/// Cloneable native render backend handle backed by shared state.
#[derive(Clone)]
pub struct SharedNativeRenderBackend {
    inner: Arc<Mutex<NativeRenderBackend>>,
    info: BackendInfo,
}

impl SharedNativeRenderBackend {
    /// Wraps a concrete native backend in shared ownership for SDK and FFI runtime state.
    pub fn new(backend: NativeRenderBackend) -> Self {
        let info = backend.info_clone();
        Self {
            inner: Arc::new(Mutex::new(backend)),
            info,
        }
    }

    fn lock(&self) -> MutexGuard<'_, NativeRenderBackend> {
        self.inner
            .lock()
            .expect("native render backend mutex should not be poisoned")
    }

    pub(crate) fn bind_texture_by_index(&self, index: u32, unit: u32) -> GoudResult<()> {
        self.lock().bind_texture_by_index(index, unit)
    }

    pub(crate) fn resize_surface(&self, width: u32, height: u32) {
        self.lock().resize_surface(width, height);
    }
}

impl RenderBackend for SharedNativeRenderBackend {
    fn info(&self) -> &BackendInfo {
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

    fn set_blend_func(&mut self, src: super::BlendFactor, dst: super::BlendFactor) {
        self.lock().set_blend_func(src, dst);
    }

    fn enable_culling(&mut self) {
        self.lock().enable_culling();
    }

    fn disable_culling(&mut self) {
        self.lock().disable_culling();
    }

    fn set_cull_face(&mut self, face: super::CullFace) {
        self.lock().set_cull_face(face);
    }

    fn set_depth_func(&mut self, func: super::types::DepthFunc) {
        self.lock().set_depth_func(func);
    }

    fn set_front_face(&mut self, face: super::types::FrontFace) {
        self.lock().set_front_face(face);
    }

    fn set_depth_mask(&mut self, enabled: bool) {
        self.lock().set_depth_mask(enabled);
    }

    fn set_line_width(&mut self, width: f32) {
        self.lock().set_line_width(width);
    }
}

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

impl DrawOps for SharedNativeRenderBackend {
    fn set_vertex_attributes(&mut self, layout: &VertexLayout) {
        self.lock().set_vertex_attributes(layout);
    }

    fn draw_arrays(
        &mut self,
        topology: PrimitiveTopology,
        first: u32,
        count: u32,
    ) -> GoudResult<()> {
        self.lock().draw_arrays(topology, first, count)
    }

    fn draw_indexed(
        &mut self,
        topology: PrimitiveTopology,
        count: u32,
        offset: usize,
    ) -> GoudResult<()> {
        self.lock().draw_indexed(topology, count, offset)
    }

    fn draw_indexed_u16(
        &mut self,
        topology: PrimitiveTopology,
        count: u32,
        offset: usize,
    ) -> GoudResult<()> {
        self.lock().draw_indexed_u16(topology, count, offset)
    }

    fn draw_arrays_instanced(
        &mut self,
        topology: PrimitiveTopology,
        first: u32,
        count: u32,
        instance_count: u32,
    ) -> GoudResult<()> {
        self.lock()
            .draw_arrays_instanced(topology, first, count, instance_count)
    }

    fn draw_indexed_instanced(
        &mut self,
        topology: PrimitiveTopology,
        count: u32,
        offset: usize,
        instance_count: u32,
    ) -> GoudResult<()> {
        self.lock()
            .draw_indexed_instanced(topology, count, offset, instance_count)
    }
}
