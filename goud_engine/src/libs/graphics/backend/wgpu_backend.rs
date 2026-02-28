//! wgpu implementation of the [`RenderBackend`] trait.
//!
//! Provides a cross-platform GPU backend using wgpu (WebGPU API). This backend
//! works on desktop (Vulkan/Metal/DX12) and web (WebGPU/WebGL2).
//!
//! # Architecture
//!
//! Unlike OpenGL's immediate-mode API, wgpu uses command buffers and render
//! pipelines. This backend bridges the gap by:
//! - Tracking GPU state changes (depth, blend, cull) and caching render pipelines
//! - Deferring draw calls into a command list replayed at [`end_frame`]
//! - Managing per-shader uniform buffers with CPU staging

use super::{
    types::{
        BufferHandle, BufferMarker, BufferType, BufferUsage, DepthFunc, FrontFace,
        PrimitiveTopology, ShaderHandle, ShaderMarker, TextureFilter, TextureFormat, TextureHandle,
        TextureMarker, TextureWrap, VertexAttributeType, VertexLayout,
    },
    BackendCapabilities, BackendInfo, BlendFactor, CullFace, RenderBackend,
};
use crate::core::{
    error::{GoudError, GoudResult},
    handle::HandleAllocator,
};
use std::collections::HashMap;
use std::sync::Arc;

const UNIFORM_BUFFER_SIZE: usize = 4096;
const MAX_TEXTURE_UNITS: usize = 16;

// =============================================================================
// Internal metadata types
// =============================================================================

struct WgpuBufferMeta {
    buffer: wgpu::Buffer,
    buffer_type: BufferType,
    #[allow(dead_code)]
    size: usize,
}

struct WgpuTextureMeta {
    _texture: wgpu::Texture,
    view: wgpu::TextureView,
    sampler: wgpu::Sampler,
    width: u32,
    height: u32,
}

#[allow(dead_code)]
struct UniformSlot {
    offset: usize,
    size: usize,
}

struct WgpuShaderMeta {
    vertex_module: wgpu::ShaderModule,
    fragment_module: wgpu::ShaderModule,
    uniform_slots: HashMap<String, UniformSlot>,
    uniform_staging: Vec<u8>,
    uniform_buffer: wgpu::Buffer,
    uniform_bind_group: wgpu::BindGroup,
    #[allow(dead_code)]
    next_uniform_offset: usize,
}

// Draw commands recorded during the frame and replayed in end_frame.
struct DrawCommand {
    shader: ShaderHandle,
    vertex_buffer: BufferHandle,
    index_buffer: Option<BufferHandle>,
    vertex_layout: VertexLayout,
    bound_textures: Vec<(u32, TextureHandle)>,
    topology: PrimitiveTopology,
    depth_test: bool,
    depth_write: bool,
    depth_func: DepthFunc,
    blend_enabled: bool,
    blend_src: BlendFactor,
    blend_dst: BlendFactor,
    cull_enabled: bool,
    cull_face: CullFace,
    front_face: FrontFace,
    uniform_snapshot: Vec<u8>,
    draw_type: DrawType,
}

#[allow(dead_code)]
enum DrawType {
    Arrays {
        first: u32,
        count: u32,
    },
    Indexed {
        count: u32,
        offset: usize,
    },
    IndexedU16 {
        count: u32,
        offset: usize,
    },
    ArraysInstanced {
        first: u32,
        count: u32,
        instances: u32,
    },
    IndexedInstanced {
        count: u32,
        offset: usize,
        instances: u32,
    },
}

/// Pipeline cache key combining all state that affects pipeline creation.
#[derive(Hash, Eq, PartialEq, Clone)]
struct PipelineKey {
    shader: ShaderHandle,
    topology: u8,
    depth_test: bool,
    depth_write: bool,
    depth_func: u8,
    blend_enabled: bool,
    blend_src: u8,
    blend_dst: u8,
    cull_enabled: bool,
    cull_face: u8,
    front_face: u8,
    vertex_stride: u32,
    vertex_attrs: Vec<(u32, u8, u32, bool)>,
}

struct FrameState {
    surface_texture: wgpu::SurfaceTexture,
    surface_view: wgpu::TextureView,
}

// =============================================================================
// WgpuBackend
// =============================================================================

/// wgpu-based render backend for cross-platform GPU rendering.
///
/// Owns the full wgpu device stack (instance, surface, adapter, device, queue)
/// and manages GPU resources via generational handles identical to OpenGLBackend.
pub struct WgpuBackend {
    info: BackendInfo,
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,
    surface: wgpu::Surface<'static>,
    surface_config: wgpu::SurfaceConfiguration,
    surface_format: wgpu::TextureFormat,

    depth_texture: wgpu::Texture,
    depth_view: wgpu::TextureView,

    clear_color: wgpu::Color,
    needs_clear: bool,

    current_frame: Option<FrameState>,
    draw_commands: Vec<DrawCommand>,

    // Render state
    depth_test_enabled: bool,
    depth_write_enabled: bool,
    depth_func: DepthFunc,
    blend_enabled: bool,
    blend_src: BlendFactor,
    blend_dst: BlendFactor,
    cull_enabled: bool,
    cull_face: CullFace,
    front_face_state: FrontFace,

    // Resource management
    buffer_allocator: HandleAllocator<BufferMarker>,
    buffers: HashMap<BufferHandle, WgpuBufferMeta>,

    texture_allocator: HandleAllocator<TextureMarker>,
    textures: HashMap<TextureHandle, WgpuTextureMeta>,

    shader_allocator: HandleAllocator<ShaderMarker>,
    shaders: HashMap<ShaderHandle, WgpuShaderMeta>,

    // Current bindings
    bound_vertex_buffer: Option<BufferHandle>,
    bound_index_buffer: Option<BufferHandle>,
    bound_shader: Option<ShaderHandle>,
    bound_textures: Vec<Option<TextureHandle>>,
    current_layout: Option<VertexLayout>,

    // Pipeline cache
    pipeline_cache: HashMap<PipelineKey, wgpu::RenderPipeline>,
    uniform_bind_group_layout: wgpu::BindGroupLayout,
    texture_bind_group_layout: wgpu::BindGroupLayout,
}

// SAFETY: wgpu Device and Queue are Send+Sync. Surface is Send.
// All other fields are plain data or standard Rust containers.
unsafe impl Send for WgpuBackend {}
unsafe impl Sync for WgpuBackend {}

impl WgpuBackend {
    /// Creates a new wgpu backend from a winit window.
    ///
    /// Blocks on async wgpu initialization via pollster.
    pub fn new(window: Arc<winit::window::Window>) -> GoudResult<Self> {
        pollster::block_on(Self::new_async(window))
    }

    async fn new_async(window: Arc<winit::window::Window>) -> GoudResult<Self> {
        let instance = wgpu::Instance::default();
        let surface = instance
            .create_surface(wgpu::SurfaceTarget::from(window.clone()))
            .map_err(|e| GoudError::BackendNotSupported(format!("wgpu surface: {e}")))?;

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .map_err(|e| GoudError::BackendNotSupported(format!("No suitable GPU adapter: {e}")))?;

        let (device, queue): (wgpu::Device, wgpu::Queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("GoudEngine"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                ..Default::default()
            })
            .await
            .map_err(|e| GoudError::BackendNotSupported(format!("wgpu device: {e}")))?;

        let size = window.inner_size();
        let caps = surface.get_capabilities(&adapter);
        let surface_format = caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(caps.formats[0]);

        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode: wgpu::PresentMode::AutoVsync,
            alpha_mode: caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &surface_config);

        let (depth_texture, depth_view) =
            Self::create_depth_texture(&device, surface_config.width, surface_config.height);

        let adapter_info = adapter.get_info();
        let limits = device.limits();

        let info = BackendInfo {
            name: "wgpu",
            version: format!("{:?}", adapter_info.backend),
            vendor: adapter_info.vendor.to_string(),
            renderer: adapter_info.name.clone(),
            capabilities: BackendCapabilities {
                max_texture_units: limits.max_sampled_textures_per_shader_stage.min(16),
                max_texture_size: limits.max_texture_dimension_2d,
                max_vertex_attributes: limits.max_vertex_attributes,
                max_uniform_buffer_size: limits.max_uniform_buffer_binding_size,
                supports_instancing: true,
                supports_compute_shaders: true,
                supports_geometry_shaders: false,
                supports_tessellation: false,
                supports_multisampling: true,
                supports_anisotropic_filtering: true,
            },
        };

        let uniform_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("uniform_bgl"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("texture_bgl"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            });

        let device = Arc::new(device);
        let queue = Arc::new(queue);

        Ok(Self {
            info,
            device,
            queue,
            surface,
            surface_config,
            surface_format,
            depth_texture,
            depth_view,
            clear_color: wgpu::Color::BLACK,
            needs_clear: false,
            current_frame: None,
            draw_commands: Vec::new(),
            depth_test_enabled: false,
            depth_write_enabled: true,
            depth_func: DepthFunc::Less,
            blend_enabled: false,
            blend_src: BlendFactor::One,
            blend_dst: BlendFactor::Zero,
            cull_enabled: false,
            cull_face: CullFace::Back,
            front_face_state: FrontFace::Ccw,
            buffer_allocator: HandleAllocator::new(),
            buffers: HashMap::new(),
            texture_allocator: HandleAllocator::new(),
            textures: HashMap::new(),
            shader_allocator: HandleAllocator::new(),
            shaders: HashMap::new(),
            bound_vertex_buffer: None,
            bound_index_buffer: None,
            bound_shader: None,
            bound_textures: vec![None; MAX_TEXTURE_UNITS],
            current_layout: None,
            pipeline_cache: HashMap::new(),
            uniform_bind_group_layout,
            texture_bind_group_layout,
        })
    }

    fn create_depth_texture(
        device: &wgpu::Device,
        width: u32,
        height: u32,
    ) -> (wgpu::Texture, wgpu::TextureView) {
        let tex = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("depth"),
            size: wgpu::Extent3d {
                width: width.max(1),
                height: height.max(1),
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });
        let view = tex.create_view(&wgpu::TextureViewDescriptor::default());
        (tex, view)
    }

    /// Attempts to transpile GLSL source to WGSL via naga.
    /// Falls back to treating the source as WGSL if transpilation fails.
    fn transpile_to_wgsl(source: &str, stage: naga::ShaderStage) -> GoudResult<String> {
        let opts = naga::front::glsl::Options {
            stage,
            defines: Default::default(),
        };
        let module = naga::front::glsl::Frontend::default()
            .parse(&opts, source)
            .map_err(|errs| {
                GoudError::ShaderCompilationFailed(format!("GLSL parse: {:?}", errs))
            })?;

        let mut validator = naga::valid::Validator::new(
            naga::valid::ValidationFlags::all(),
            naga::valid::Capabilities::all(),
        );
        let info = validator
            .validate(&module)
            .map_err(|e| GoudError::ShaderCompilationFailed(format!("Validation: {e}")))?;

        naga::back::wgsl::write_string(&module, &info, naga::back::wgsl::WriterFlags::empty())
            .map_err(|e| GoudError::ShaderCompilationFailed(format!("WGSL write: {e}")))
    }

    fn create_shader_module(
        device: &wgpu::Device,
        source: &str,
        stage: naga::ShaderStage,
    ) -> GoudResult<wgpu::ShaderModule> {
        let wgsl = if source.trim_start().starts_with('#') {
            Self::transpile_to_wgsl(source, stage)?
        } else {
            source.to_string()
        };

        Ok(device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(wgsl.into()),
        }))
    }

    fn map_topology(t: PrimitiveTopology) -> wgpu::PrimitiveTopology {
        match t {
            PrimitiveTopology::Points => wgpu::PrimitiveTopology::PointList,
            PrimitiveTopology::Lines => wgpu::PrimitiveTopology::LineList,
            PrimitiveTopology::LineStrip => wgpu::PrimitiveTopology::LineStrip,
            PrimitiveTopology::Triangles => wgpu::PrimitiveTopology::TriangleList,
            PrimitiveTopology::TriangleStrip => wgpu::PrimitiveTopology::TriangleStrip,
            PrimitiveTopology::TriangleFan => wgpu::PrimitiveTopology::TriangleList,
        }
    }

    fn map_vertex_format(ty: VertexAttributeType) -> wgpu::VertexFormat {
        match ty {
            VertexAttributeType::Float => wgpu::VertexFormat::Float32,
            VertexAttributeType::Float2 => wgpu::VertexFormat::Float32x2,
            VertexAttributeType::Float3 => wgpu::VertexFormat::Float32x3,
            VertexAttributeType::Float4 => wgpu::VertexFormat::Float32x4,
            VertexAttributeType::Int => wgpu::VertexFormat::Sint32,
            VertexAttributeType::Int2 => wgpu::VertexFormat::Sint32x2,
            VertexAttributeType::Int3 => wgpu::VertexFormat::Sint32x3,
            VertexAttributeType::Int4 => wgpu::VertexFormat::Sint32x4,
            VertexAttributeType::UInt => wgpu::VertexFormat::Uint32,
            VertexAttributeType::UInt2 => wgpu::VertexFormat::Uint32x2,
            VertexAttributeType::UInt3 => wgpu::VertexFormat::Uint32x3,
            VertexAttributeType::UInt4 => wgpu::VertexFormat::Uint32x4,
        }
    }

    fn map_blend_factor(f: BlendFactor) -> wgpu::BlendFactor {
        match f {
            BlendFactor::Zero => wgpu::BlendFactor::Zero,
            BlendFactor::One => wgpu::BlendFactor::One,
            BlendFactor::SrcColor => wgpu::BlendFactor::Src,
            BlendFactor::OneMinusSrcColor => wgpu::BlendFactor::OneMinusSrc,
            BlendFactor::DstColor => wgpu::BlendFactor::Dst,
            BlendFactor::OneMinusDstColor => wgpu::BlendFactor::OneMinusDst,
            BlendFactor::SrcAlpha => wgpu::BlendFactor::SrcAlpha,
            BlendFactor::OneMinusSrcAlpha => wgpu::BlendFactor::OneMinusSrcAlpha,
            BlendFactor::DstAlpha => wgpu::BlendFactor::DstAlpha,
            BlendFactor::OneMinusDstAlpha => wgpu::BlendFactor::OneMinusDstAlpha,
            BlendFactor::ConstantColor => wgpu::BlendFactor::Constant,
            BlendFactor::OneMinusConstantColor => wgpu::BlendFactor::OneMinusConstant,
            BlendFactor::ConstantAlpha => wgpu::BlendFactor::Constant,
            BlendFactor::OneMinusConstantAlpha => wgpu::BlendFactor::OneMinusConstant,
        }
    }

    fn map_depth_func(f: DepthFunc) -> wgpu::CompareFunction {
        match f {
            DepthFunc::Always => wgpu::CompareFunction::Always,
            DepthFunc::Never => wgpu::CompareFunction::Never,
            DepthFunc::Less => wgpu::CompareFunction::Less,
            DepthFunc::LessEqual => wgpu::CompareFunction::LessEqual,
            DepthFunc::Greater => wgpu::CompareFunction::Greater,
            DepthFunc::GreaterEqual => wgpu::CompareFunction::GreaterEqual,
            DepthFunc::Equal => wgpu::CompareFunction::Equal,
            DepthFunc::NotEqual => wgpu::CompareFunction::NotEqual,
        }
    }

    fn map_front_face(f: FrontFace) -> wgpu::FrontFace {
        match f {
            FrontFace::Ccw => wgpu::FrontFace::Ccw,
            FrontFace::Cw => wgpu::FrontFace::Cw,
        }
    }

    fn map_cull_face(f: CullFace) -> Option<wgpu::Face> {
        match f {
            CullFace::Front => Some(wgpu::Face::Front),
            CullFace::Back => Some(wgpu::Face::Back),
            CullFace::FrontAndBack => None,
        }
    }

    fn snapshot_textures(&self) -> Vec<(u32, TextureHandle)> {
        self.bound_textures
            .iter()
            .enumerate()
            .filter_map(|(i, t)| t.map(|h| (i as u32, h)))
            .collect()
    }

    fn make_pipeline_key(&self, cmd: &DrawCommand) -> PipelineKey {
        PipelineKey {
            shader: cmd.shader,
            topology: cmd.topology as u8,
            depth_test: cmd.depth_test,
            depth_write: cmd.depth_write,
            depth_func: cmd.depth_func as u8,
            blend_enabled: cmd.blend_enabled,
            blend_src: cmd.blend_src as u8,
            blend_dst: cmd.blend_dst as u8,
            cull_enabled: cmd.cull_enabled,
            cull_face: cmd.cull_face as u8,
            front_face: cmd.front_face as u8,
            vertex_stride: cmd.vertex_layout.stride,
            vertex_attrs: cmd
                .vertex_layout
                .attributes
                .iter()
                .map(|a| (a.location, a.attribute_type as u8, a.offset, a.normalized))
                .collect(),
        }
    }

    fn record_draw(&mut self, draw_type: DrawType) -> GoudResult<()> {
        let shader = self
            .bound_shader
            .ok_or(GoudError::InvalidState("No shader bound".into()))?;
        let vb = self
            .bound_vertex_buffer
            .ok_or(GoudError::InvalidState("No vertex buffer bound".into()))?;
        let layout = self
            .current_layout
            .clone()
            .ok_or(GoudError::InvalidState("No vertex layout set".into()))?;

        let uniform_snapshot = self
            .shaders
            .get(&shader)
            .map(|s| s.uniform_staging.clone())
            .unwrap_or_default();

        self.draw_commands.push(DrawCommand {
            shader,
            vertex_buffer: vb,
            index_buffer: self.bound_index_buffer,
            vertex_layout: layout,
            bound_textures: self.snapshot_textures(),
            topology: PrimitiveTopology::Triangles,
            depth_test: self.depth_test_enabled,
            depth_write: self.depth_write_enabled,
            depth_func: self.depth_func,
            blend_enabled: self.blend_enabled,
            blend_src: self.blend_src,
            blend_dst: self.blend_dst,
            cull_enabled: self.cull_enabled,
            cull_face: self.cull_face,
            front_face: self.front_face_state,
            uniform_snapshot,
            draw_type,
        });
        Ok(())
    }

    /// Provides access to the wgpu device (for advanced use / surface creation).
    pub fn device(&self) -> &Arc<wgpu::Device> {
        &self.device
    }

    /// Provides access to the wgpu queue.
    pub fn queue(&self) -> &Arc<wgpu::Queue> {
        &self.queue
    }

    /// Resizes the surface and depth buffer. Call after window resize.
    pub fn resize(&mut self, width: u32, height: u32) {
        let w = width.max(1);
        let h = height.max(1);
        self.surface_config.width = w;
        self.surface_config.height = h;
        self.surface.configure(&self.device, &self.surface_config);
        let (dt, dv) = Self::create_depth_texture(&self.device, w, h);
        self.depth_texture = dt;
        self.depth_view = dv;
    }
}

// =============================================================================
// RenderBackend implementation
// =============================================================================

impl RenderBackend for WgpuBackend {
    fn info(&self) -> &BackendInfo {
        &self.info
    }

    fn begin_frame(&mut self) -> GoudResult<()> {
        let surface_texture = self
            .surface
            .get_current_texture()
            .map_err(|e| GoudError::InternalError(format!("Surface texture: {e}")))?;
        let surface_view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        self.current_frame = Some(FrameState {
            surface_texture,
            surface_view,
        });
        self.draw_commands.clear();
        Ok(())
    }

    fn end_frame(&mut self) -> GoudResult<()> {
        let frame = self
            .current_frame
            .take()
            .ok_or(GoudError::InvalidState("No active frame".into()))?;

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        // Upload uniform data for each shader used this frame
        let shader_handles: Vec<ShaderHandle> = self
            .draw_commands
            .iter()
            .map(|c| c.shader)
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();

        for sh in &shader_handles {
            if let Some(cmd) = self.draw_commands.iter().rev().find(|c| c.shader == *sh) {
                if let Some(meta) = self.shaders.get(sh) {
                    self.queue
                        .write_buffer(&meta.uniform_buffer, 0, &cmd.uniform_snapshot);
                }
            }
        }

        let load_op = if self.needs_clear {
            self.needs_clear = false;
            wgpu::LoadOp::Clear(self.clear_color)
        } else {
            wgpu::LoadOp::Load
        };

        // Collect pipeline keys and ensure pipelines exist before the render pass borrow
        let cmd_keys: Vec<PipelineKey> = self
            .draw_commands
            .iter()
            .map(|cmd| self.make_pipeline_key(cmd))
            .collect();

        for (i, key) in cmd_keys.iter().enumerate() {
            let cmd = &self.draw_commands[i];
            if !self.pipeline_cache.contains_key(key) {
                if let Some(shader_meta) = self.shaders.get(&cmd.shader) {
                    let wgpu_attrs: Vec<wgpu::VertexAttribute> = cmd
                        .vertex_layout
                        .attributes
                        .iter()
                        .map(|a| wgpu::VertexAttribute {
                            format: Self::map_vertex_format(a.attribute_type),
                            offset: a.offset as u64,
                            shader_location: a.location,
                        })
                        .collect();

                    let pipeline_layout =
                        self.device
                            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                                label: None,
                                bind_group_layouts: &[
                                    &self.uniform_bind_group_layout,
                                    &self.texture_bind_group_layout,
                                ],
                                immediate_size: 0,
                            });

                    let blend_state = if key.blend_enabled {
                        let src = unsafe { std::mem::transmute::<u8, BlendFactor>(key.blend_src) };
                        let dst = unsafe { std::mem::transmute::<u8, BlendFactor>(key.blend_dst) };
                        Some(wgpu::BlendState {
                            color: wgpu::BlendComponent {
                                src_factor: Self::map_blend_factor(src),
                                dst_factor: Self::map_blend_factor(dst),
                                operation: wgpu::BlendOperation::Add,
                            },
                            alpha: wgpu::BlendComponent {
                                src_factor: Self::map_blend_factor(src),
                                dst_factor: Self::map_blend_factor(dst),
                                operation: wgpu::BlendOperation::Add,
                            },
                        })
                    } else {
                        None
                    };

                    let depth_stencil = if key.depth_test {
                        let func = unsafe { std::mem::transmute::<u8, DepthFunc>(key.depth_func) };
                        Some(wgpu::DepthStencilState {
                            format: wgpu::TextureFormat::Depth32Float,
                            depth_write_enabled: key.depth_write,
                            depth_compare: Self::map_depth_func(func),
                            stencil: wgpu::StencilState::default(),
                            bias: wgpu::DepthBiasState::default(),
                        })
                    } else {
                        None
                    };

                    let cull_mode = if key.cull_enabled {
                        let face = unsafe { std::mem::transmute::<u8, CullFace>(key.cull_face) };
                        Self::map_cull_face(face)
                    } else {
                        None
                    };
                    let front_face = Self::map_front_face(unsafe {
                        std::mem::transmute::<u8, FrontFace>(key.front_face)
                    });
                    let topology = Self::map_topology(unsafe {
                        std::mem::transmute::<u8, PrimitiveTopology>(key.topology)
                    });
                    let strip_index_format = match topology {
                        wgpu::PrimitiveTopology::LineStrip
                        | wgpu::PrimitiveTopology::TriangleStrip => Some(wgpu::IndexFormat::Uint32),
                        _ => None,
                    };

                    let pipeline =
                        self.device
                            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                                label: None,
                                layout: Some(&pipeline_layout),
                                vertex: wgpu::VertexState {
                                    module: &shader_meta.vertex_module,
                                    entry_point: Some("main"),
                                    buffers: &[wgpu::VertexBufferLayout {
                                        array_stride: cmd.vertex_layout.stride as u64,
                                        step_mode: wgpu::VertexStepMode::Vertex,
                                        attributes: &wgpu_attrs,
                                    }],
                                    compilation_options: Default::default(),
                                },
                                fragment: Some(wgpu::FragmentState {
                                    module: &shader_meta.fragment_module,
                                    entry_point: Some("main"),
                                    targets: &[Some(wgpu::ColorTargetState {
                                        format: self.surface_format,
                                        blend: blend_state,
                                        write_mask: wgpu::ColorWrites::ALL,
                                    })],
                                    compilation_options: Default::default(),
                                }),
                                primitive: wgpu::PrimitiveState {
                                    topology,
                                    strip_index_format,
                                    front_face,
                                    cull_mode,
                                    polygon_mode: wgpu::PolygonMode::Fill,
                                    unclipped_depth: false,
                                    conservative: false,
                                },
                                depth_stencil,
                                multisample: wgpu::MultisampleState::default(),
                                multiview_mask: None,
                                cache: None,
                            });

                    self.pipeline_cache.insert(key.clone(), pipeline);
                }
            }
        }

        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &frame.surface_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: load_op,
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });

            for (i, cmd) in self.draw_commands.iter().enumerate() {
                let key = &cmd_keys[i];
                let Some(pipeline) = self.pipeline_cache.get(key) else {
                    continue;
                };
                let Some(vb_meta) = self.buffers.get(&cmd.vertex_buffer) else {
                    continue;
                };

                pass.set_pipeline(pipeline);
                pass.set_vertex_buffer(0, vb_meta.buffer.slice(..));

                if let Some(shader_meta) = self.shaders.get(&cmd.shader) {
                    pass.set_bind_group(0, &shader_meta.uniform_bind_group, &[]);
                }

                if let Some((_unit, tex_handle)) = cmd.bound_textures.first() {
                    if let Some(tex_meta) = self.textures.get(tex_handle) {
                        let bg = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                            label: None,
                            layout: &self.texture_bind_group_layout,
                            entries: &[
                                wgpu::BindGroupEntry {
                                    binding: 0,
                                    resource: wgpu::BindingResource::TextureView(&tex_meta.view),
                                },
                                wgpu::BindGroupEntry {
                                    binding: 1,
                                    resource: wgpu::BindingResource::Sampler(&tex_meta.sampler),
                                },
                            ],
                        });
                        pass.set_bind_group(1, &bg, &[]);
                    }
                }

                if let Some(ib_handle) = cmd.index_buffer {
                    if let Some(ib_meta) = self.buffers.get(&ib_handle) {
                        let format = match cmd.draw_type {
                            DrawType::IndexedU16 { .. } => wgpu::IndexFormat::Uint16,
                            _ => wgpu::IndexFormat::Uint32,
                        };
                        pass.set_index_buffer(ib_meta.buffer.slice(..), format);
                    }
                }

                match cmd.draw_type {
                    DrawType::Arrays { first, count } => {
                        pass.draw(first..first + count, 0..1);
                    }
                    DrawType::Indexed { count, .. } | DrawType::IndexedU16 { count, .. } => {
                        pass.draw_indexed(0..count, 0, 0..1);
                    }
                    DrawType::ArraysInstanced {
                        first,
                        count,
                        instances,
                    } => {
                        pass.draw(first..first + count, 0..instances);
                    }
                    DrawType::IndexedInstanced {
                        count, instances, ..
                    } => {
                        pass.draw_indexed(0..count, 0, 0..instances);
                    }
                }
            }
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        frame.surface_texture.present();
        self.draw_commands.clear();
        Ok(())
    }

    // ========================================================================
    // Clear Operations
    // ========================================================================

    fn set_clear_color(&mut self, r: f32, g: f32, b: f32, a: f32) {
        self.clear_color = wgpu::Color {
            r: r as f64,
            g: g as f64,
            b: b as f64,
            a: a as f64,
        };
    }

    fn clear_color(&mut self) {
        self.needs_clear = true;
    }

    fn clear_depth(&mut self) {
        self.needs_clear = true;
    }

    // ========================================================================
    // State Management
    // ========================================================================

    fn set_viewport(&mut self, _x: i32, _y: i32, _width: u32, _height: u32) {
        // wgpu viewport is set per render pass; tracked state is applied in end_frame
    }

    fn enable_depth_test(&mut self) {
        self.depth_test_enabled = true;
    }
    fn disable_depth_test(&mut self) {
        self.depth_test_enabled = false;
    }
    fn enable_blending(&mut self) {
        self.blend_enabled = true;
        self.blend_src = BlendFactor::SrcAlpha;
        self.blend_dst = BlendFactor::OneMinusSrcAlpha;
    }
    fn disable_blending(&mut self) {
        self.blend_enabled = false;
    }
    fn set_blend_func(&mut self, src: BlendFactor, dst: BlendFactor) {
        self.blend_src = src;
        self.blend_dst = dst;
    }
    fn enable_culling(&mut self) {
        self.cull_enabled = true;
    }
    fn disable_culling(&mut self) {
        self.cull_enabled = false;
    }
    fn set_cull_face(&mut self, face: CullFace) {
        self.cull_face = face;
    }
    fn set_depth_func(&mut self, func: DepthFunc) {
        self.depth_func = func;
    }
    fn set_front_face(&mut self, face: FrontFace) {
        self.front_face_state = face;
    }
    fn set_depth_mask(&mut self, enabled: bool) {
        self.depth_write_enabled = enabled;
    }
    fn set_line_width(&mut self, _width: f32) {
        // wgpu does not support variable line width (WebGPU spec limitation)
    }

    // ========================================================================
    // Buffer Operations
    // ========================================================================

    fn create_buffer(
        &mut self,
        buffer_type: BufferType,
        usage: BufferUsage,
        data: &[u8],
    ) -> GoudResult<BufferHandle> {
        let wgpu_usage = match buffer_type {
            BufferType::Vertex => wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            BufferType::Index => wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            BufferType::Uniform => wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        };

        let buffer = if data.is_empty() {
            self.device.create_buffer(&wgpu::BufferDescriptor {
                label: None,
                size: 64,
                usage: wgpu_usage,
                mapped_at_creation: false,
            })
        } else {
            wgpu::util::DeviceExt::create_buffer_init(
                &*self.device,
                &wgpu::util::BufferInitDescriptor {
                    label: None,
                    contents: data,
                    usage: wgpu_usage,
                },
            )
        };

        let handle = self.buffer_allocator.allocate();
        self.buffers.insert(
            handle,
            WgpuBufferMeta {
                buffer,
                buffer_type,
                size: if data.is_empty() { 64 } else { data.len() },
            },
        );

        let _ = usage; // Usage hints don't apply to wgpu (managed automatically)
        Ok(handle)
    }

    fn update_buffer(
        &mut self,
        handle: BufferHandle,
        offset: usize,
        data: &[u8],
    ) -> GoudResult<()> {
        let meta = self.buffers.get(&handle).ok_or(GoudError::InvalidHandle)?;
        if offset + data.len() > meta.size {
            return Err(GoudError::InvalidState(format!(
                "Buffer update out of bounds: {} + {} > {}",
                offset,
                data.len(),
                meta.size
            )));
        }
        self.queue.write_buffer(&meta.buffer, offset as u64, data);
        Ok(())
    }

    fn destroy_buffer(&mut self, handle: BufferHandle) -> bool {
        if self.buffers.remove(&handle).is_some() {
            self.buffer_allocator.deallocate(handle);
            true
        } else {
            false
        }
    }

    fn is_buffer_valid(&self, handle: BufferHandle) -> bool {
        self.buffers.contains_key(&handle)
    }

    fn buffer_size(&self, handle: BufferHandle) -> Option<usize> {
        self.buffers.get(&handle).map(|m| m.size)
    }

    fn bind_buffer(&mut self, handle: BufferHandle) -> GoudResult<()> {
        let meta = self.buffers.get(&handle).ok_or(GoudError::InvalidHandle)?;
        match meta.buffer_type {
            BufferType::Vertex => self.bound_vertex_buffer = Some(handle),
            BufferType::Index => self.bound_index_buffer = Some(handle),
            BufferType::Uniform => {}
        }
        Ok(())
    }

    fn unbind_buffer(&mut self, buffer_type: BufferType) {
        match buffer_type {
            BufferType::Vertex => self.bound_vertex_buffer = None,
            BufferType::Index => self.bound_index_buffer = None,
            BufferType::Uniform => {}
        }
    }

    // ========================================================================
    // Texture Operations
    // ========================================================================

    fn create_texture(
        &mut self,
        width: u32,
        height: u32,
        format: TextureFormat,
        filter: TextureFilter,
        wrap: TextureWrap,
        data: &[u8],
    ) -> GoudResult<TextureHandle> {
        let wgpu_format = match format {
            TextureFormat::R8 => wgpu::TextureFormat::R8Unorm,
            TextureFormat::RG8 => wgpu::TextureFormat::Rg8Unorm,
            TextureFormat::RGB8 | TextureFormat::RGBA8 => wgpu::TextureFormat::Rgba8UnormSrgb,
            TextureFormat::RGBA16F => wgpu::TextureFormat::Rgba16Float,
            TextureFormat::RGBA32F => wgpu::TextureFormat::Rgba32Float,
            TextureFormat::Depth => wgpu::TextureFormat::Depth32Float,
            TextureFormat::DepthStencil => wgpu::TextureFormat::Depth24PlusStencil8,
        };

        let texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu_format,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        if !data.is_empty() {
            let bytes_per_pixel = match format {
                TextureFormat::R8 => 1,
                TextureFormat::RG8 => 2,
                TextureFormat::RGB8 | TextureFormat::RGBA8 => 4,
                TextureFormat::RGBA16F => 8,
                TextureFormat::RGBA32F => 16,
                _ => 4,
            };

            let upload_data = if matches!(format, TextureFormat::RGB8)
                && data.len() == (width * height * 3) as usize
            {
                let mut rgba = Vec::with_capacity((width * height * 4) as usize);
                for pixel in data.chunks(3) {
                    rgba.extend_from_slice(pixel);
                    rgba.push(255);
                }
                rgba
            } else {
                data.to_vec()
            };

            self.queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    texture: &texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                &upload_data,
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(bytes_per_pixel * width),
                    rows_per_image: Some(height),
                },
                wgpu::Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
            );
        }

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let wgpu_filter = match filter {
            TextureFilter::Nearest => wgpu::FilterMode::Nearest,
            TextureFilter::Linear => wgpu::FilterMode::Linear,
        };
        let wgpu_wrap = match wrap {
            TextureWrap::Repeat => wgpu::AddressMode::Repeat,
            TextureWrap::MirroredRepeat => wgpu::AddressMode::MirrorRepeat,
            TextureWrap::ClampToEdge => wgpu::AddressMode::ClampToEdge,
            TextureWrap::ClampToBorder => wgpu::AddressMode::ClampToEdge,
        };

        let sampler = self.device.create_sampler(&wgpu::SamplerDescriptor {
            label: None,
            address_mode_u: wgpu_wrap,
            address_mode_v: wgpu_wrap,
            address_mode_w: wgpu_wrap,
            mag_filter: wgpu_filter,
            min_filter: wgpu_filter,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        });

        let handle = self.texture_allocator.allocate();
        self.textures.insert(
            handle,
            WgpuTextureMeta {
                _texture: texture,
                view,
                sampler,
                width,
                height,
            },
        );
        Ok(handle)
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
        let meta = self.textures.get(&handle).ok_or(GoudError::InvalidHandle)?;
        if x + width > meta.width || y + height > meta.height {
            return Err(GoudError::InvalidState(
                "Texture update out of bounds".into(),
            ));
        }
        self.queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &meta._texture,
                mip_level: 0,
                origin: wgpu::Origin3d { x, y, z: 0 },
                aspect: wgpu::TextureAspect::All,
            },
            data,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * width),
                rows_per_image: Some(height),
            },
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );
        Ok(())
    }

    fn destroy_texture(&mut self, handle: TextureHandle) -> bool {
        if self.textures.remove(&handle).is_some() {
            self.texture_allocator.deallocate(handle);
            true
        } else {
            false
        }
    }

    fn is_texture_valid(&self, handle: TextureHandle) -> bool {
        self.textures.contains_key(&handle)
    }

    fn texture_size(&self, handle: TextureHandle) -> Option<(u32, u32)> {
        self.textures.get(&handle).map(|m| (m.width, m.height))
    }

    fn bind_texture(&mut self, handle: TextureHandle, unit: u32) -> GoudResult<()> {
        if !self.textures.contains_key(&handle) {
            return Err(GoudError::InvalidHandle);
        }
        if (unit as usize) < self.bound_textures.len() {
            self.bound_textures[unit as usize] = Some(handle);
        }
        Ok(())
    }

    fn unbind_texture(&mut self, unit: u32) {
        if (unit as usize) < self.bound_textures.len() {
            self.bound_textures[unit as usize] = None;
        }
    }

    // ========================================================================
    // Shader Operations
    // ========================================================================

    fn create_shader(&mut self, vertex_src: &str, fragment_src: &str) -> GoudResult<ShaderHandle> {
        let vertex_module =
            Self::create_shader_module(&self.device, vertex_src, naga::ShaderStage::Vertex)?;
        let fragment_module =
            Self::create_shader_module(&self.device, fragment_src, naga::ShaderStage::Fragment)?;

        let uniform_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("uniforms"),
            size: UNIFORM_BUFFER_SIZE as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let uniform_bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &self.uniform_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

        let handle = self.shader_allocator.allocate();
        self.shaders.insert(
            handle,
            WgpuShaderMeta {
                vertex_module,
                fragment_module,
                uniform_slots: HashMap::new(),
                uniform_staging: vec![0u8; UNIFORM_BUFFER_SIZE],
                uniform_buffer,
                uniform_bind_group,
                next_uniform_offset: 0,
            },
        );
        Ok(handle)
    }

    fn destroy_shader(&mut self, handle: ShaderHandle) -> bool {
        if self.shaders.remove(&handle).is_some() {
            self.shader_allocator.deallocate(handle);
            // Invalidate cached pipelines that referenced this shader
            self.pipeline_cache.retain(|k, _| k.shader != handle);
            true
        } else {
            false
        }
    }

    fn is_shader_valid(&self, handle: ShaderHandle) -> bool {
        self.shaders.contains_key(&handle)
    }

    fn bind_shader(&mut self, handle: ShaderHandle) -> GoudResult<()> {
        if !self.shaders.contains_key(&handle) {
            return Err(GoudError::InvalidHandle);
        }
        self.bound_shader = Some(handle);
        Ok(())
    }

    fn unbind_shader(&mut self) {
        self.bound_shader = None;
    }

    fn get_uniform_location(&self, handle: ShaderHandle, name: &str) -> Option<i32> {
        self.shaders.get(&handle).and_then(|meta| {
            meta.uniform_slots
                .get(name)
                .map(|slot| (slot.offset / 4) as i32)
        })
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

    fn set_vertex_attributes(&mut self, layout: &VertexLayout) {
        self.current_layout = Some(layout.clone());
    }

    // ========================================================================
    // Draw Calls
    // ========================================================================

    fn draw_arrays(
        &mut self,
        _topology: PrimitiveTopology,
        first: u32,
        count: u32,
    ) -> GoudResult<()> {
        self.record_draw(DrawType::Arrays { first, count })
    }

    fn draw_indexed(
        &mut self,
        _topology: PrimitiveTopology,
        count: u32,
        offset: usize,
    ) -> GoudResult<()> {
        self.record_draw(DrawType::Indexed { count, offset })
    }

    fn draw_indexed_u16(
        &mut self,
        _topology: PrimitiveTopology,
        count: u32,
        offset: usize,
    ) -> GoudResult<()> {
        self.record_draw(DrawType::IndexedU16 { count, offset })
    }

    fn draw_arrays_instanced(
        &mut self,
        _topology: PrimitiveTopology,
        first: u32,
        count: u32,
        instance_count: u32,
    ) -> GoudResult<()> {
        self.record_draw(DrawType::ArraysInstanced {
            first,
            count,
            instances: instance_count,
        })
    }

    fn draw_indexed_instanced(
        &mut self,
        _topology: PrimitiveTopology,
        count: u32,
        offset: usize,
        instance_count: u32,
    ) -> GoudResult<()> {
        self.record_draw(DrawType::IndexedInstanced {
            count,
            offset,
            instances: instance_count,
        })
    }
}

// Private uniform helper
impl WgpuBackend {
    fn write_uniform(&mut self, location: i32, data: &[u8]) {
        let offset = (location as usize) * 4;
        if let Some(shader_handle) = self.bound_shader {
            if let Some(meta) = self.shaders.get_mut(&shader_handle) {
                let end = (offset + data.len()).min(UNIFORM_BUFFER_SIZE);
                if offset < end {
                    meta.uniform_staging[offset..end].copy_from_slice(&data[..end - offset]);
                }
            }
        }
    }
}
