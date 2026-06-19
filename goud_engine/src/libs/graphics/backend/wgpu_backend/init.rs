//! wgpu backend initialization: device/surface setup and public accessors.

use super::{
    BackendCapabilities, BackendInfo, BlendFactor, CullFace, DepthFunc, FrontFace, HashMap,
    PrimitiveTopology, ShaderLanguage, TextureOps, WgpuBackend,
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
    pub fn new(window: Arc<winit::window::Window>, vsync: bool) -> GoudResult<Self> {
        pollster::block_on(Self::new_async(window, vsync))
    }

    async fn new_async(window: Arc<winit::window::Window>, vsync: bool) -> GoudResult<Self> {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_with_display_handle(
            Box::new(window.clone()),
        ));
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
        // Prefer a non-sRGB surface so blending stays in gamma-encoded space, matching OpenGL's default behavior (no GL_FRAMEBUFFER_SRGB). An sRGB surface applies hardware gamma expansion on read, changing blend results.
        let surface_format = caps
            .formats
            .iter()
            .find(|f| !f.is_srgb())
            .copied()
            .unwrap_or(caps.formats[0]);
        let surface_supports_copy_src = caps.usages.contains(wgpu::TextureUsages::COPY_SRC);

        let surface_config = wgpu::SurfaceConfiguration {
            usage: if surface_supports_copy_src {
                wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC
            } else {
                wgpu::TextureUsages::RENDER_ATTACHMENT
            },
            format: surface_format,
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode: if vsync {
                wgpu::PresentMode::AutoVsync
            } else {
                wgpu::PresentMode::AutoNoVsync
            },
            alpha_mode: caps.alpha_modes[0],
            view_formats: vec![],
            // wgpu default; 2 allows double-buffering for smooth frame delivery on high-refresh displays
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
                max_uniform_buffer_size: limits.max_uniform_buffer_binding_size as u32,
                supports_instancing: true,
                supports_compute_shaders: true,
                supports_geometry_shaders: false,
                supports_tessellation: false,
                supports_multisampling: true,
                supports_anisotropic_filtering: true,
                supports_bc_compression: false,
            },
            shader_language: ShaderLanguage::Wgsl,
        };

        let uniform_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("uniform_bgl"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: true,
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

        // Create a cached 1x1 white fallback texture + bind group used for draws
        // without a bound texture.  Created once here to avoid per-frame allocation.
        let fallback_tex_bind_group = {
            let tex = device.create_texture(&wgpu::TextureDescriptor {
                label: Some("fallback-white-1x1"),
                size: wgpu::Extent3d {
                    width: 1,
                    height: 1,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            });
            queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    texture: &tex,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                &[255u8, 255, 255, 255],
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(4),
                    rows_per_image: Some(1),
                },
                wgpu::Extent3d {
                    width: 1,
                    height: 1,
                    depth_or_array_layers: 1,
                },
            );
            let view = tex.create_view(&wgpu::TextureViewDescriptor::default());
            let sampler = device.create_sampler(&wgpu::SamplerDescriptor::default());
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("fallback-texture-bg"),
                layout: &texture_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&sampler),
                    },
                ],
            })
        };

        let storage_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("storage_bgl"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let shadow_bind_group_layout = super::shadow_pass::create_shadow_bind_group_layout(&device);
        let fallback_shadow_bind_group = super::shadow_pass::create_fallback_shadow_bind_group(
            &device,
            &queue,
            &shadow_bind_group_layout,
        );

        // Create fallback storage buffer bind group (empty 64-byte buffer).
        let fallback_storage_bind_group = {
            let buf = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("fallback-storage"),
                size: 64,
                usage: wgpu::BufferUsages::STORAGE,
                mapped_at_creation: false,
            });
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("fallback-storage-bg"),
                layout: &storage_bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: buf.as_entire_binding(),
                }],
            })
        };

        Ok(Self {
            info,
            device,
            queue,
            surface: Some(surface),
            surface_config,
            surface_format,
            surface_supports_copy_src,
            wgpu_instance: instance,
            wgpu_adapter: adapter,
            window: Some(window),
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
            pending_destroy_buffers: Vec::new(),
            texture_allocator: HandleAllocator::new(),
            textures: HashMap::new(),
            shader_allocator: HandleAllocator::new(),
            shaders: HashMap::new(),
            bound_vertex_buffer: None,
            bound_index_buffer: None,
            bound_shader: None,
            bound_textures: vec![None; MAX_TEXTURE_UNITS],
            current_layout: None,
            current_vertex_bindings: Vec::new(),
            current_topology: PrimitiveTopology::Triangles,
            pipeline_cache: HashMap::new(),
            uniform_bind_group_layout,
            texture_bind_group_layout,
            storage_bind_group_layout,
            fallback_tex_bind_group,
            fallback_storage_bind_group,
            bound_storage_buffer: None,
            storage_bind_group_cache: HashMap::new(),
            uniform_ring: Vec::with_capacity(UNIFORM_BUFFER_SIZE * 64),
            shadow_bind_group_layout,
            shadow_depth_texture: None,
            shadow_depth_view: None,
            shadow_sample_view: None,
            shadow_sampler: None,
            shadow_bind_group: None,
            fallback_shadow_bind_group,
            shadow_draw_commands: Vec::new(),
            recording_shadow: false,
            shadow_map_size: 0,
            shadow_pipeline_cache: HashMap::new(),
            readback_requested: false,
            scratch_pipeline_keys: Vec::new(),
            scratch_shadow_pipeline_keys: Vec::new(),
            scratch_shadow_offsets: Vec::new(),
            scratch_shadow_grown_shaders: rustc_hash::FxHashSet::default(),
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
        if let Some(ref surface) = self.surface {
            surface.configure(&self.device, &self.surface_config);
        }
        let (dt, dv) = Self::create_depth_texture(&self.device, w, h);
        self.depth_texture = dt;
        self.depth_view = dv;
    }

    /// Drops the GPU surface. Used when the app is suspended on mobile.
    ///
    /// The device, queue, and all GPU resources (textures, buffers, pipelines)
    /// remain valid -- only the presentation surface is released.
    pub fn drop_surface(&mut self) {
        self.surface = None;
    }

    /// Recreates the GPU surface after a mobile resume.
    ///
    /// Uses the persisted wgpu instance and window handle. Returns an error
    /// on platforms without a winit window (e.g. Xbox GDK).
    pub fn recreate_surface(&mut self) -> GoudResult<()> {
        let window = self.window.as_ref().ok_or_else(|| {
            GoudError::InvalidState(
                "cannot recreate surface: no winit window (Xbox GDK does not support mobile suspend/resume)".into(),
            )
        })?;
        let surface = self
            .wgpu_instance
            .create_surface(wgpu::SurfaceTarget::from(window.clone()))
            .map_err(|e| GoudError::BackendNotSupported(format!("wgpu surface recreate: {e}")))?;
        surface.configure(&self.device, &self.surface_config);
        self.surface = Some(surface);
        Ok(())
    }
}
