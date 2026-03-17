//! wgpu backend initialization: device/surface setup and public accessors.

use super::{
    BackendCapabilities, BackendInfo, BlendFactor, CullFace, DepthFunc, FrontFace, HashMap,
    TextureOps, WgpuBackend,
};
use crate::core::{
    error::{GoudError, GoudResult},
    handle::HandleAllocator,
};
use std::sync::Arc;

/// Size (bytes) of the per-shader uniform staging buffer.
pub const UNIFORM_BUFFER_SIZE: usize = 4096;
/// Maximum number of simultaneously-bound texture units.
pub const MAX_TEXTURE_UNITS: usize = 16;

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
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
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
                supports_bc_compression: false,
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

        Ok(Self {
            info,
            device,
            queue,
            surface,
            surface_config,
            surface_format,
            depth_texture,
            depth_view,
            last_frame_readback: None,
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

    pub(super) fn create_depth_texture(
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

    /// Provides access to the wgpu device (for advanced use / surface creation).
    pub fn device(&self) -> &wgpu::Device {
        &self.device
    }

    /// Provides access to the wgpu queue.
    pub fn queue(&self) -> &wgpu::Queue {
        &self.queue
    }

    pub(crate) fn bind_texture_by_index(&mut self, index: u32, unit: u32) -> GoudResult<()> {
        let handle = self
            .textures
            .keys()
            .copied()
            .find(|handle| handle.index() == index)
            .ok_or(GoudError::InvalidHandle)?;
        self.bind_texture(handle, unit)
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
