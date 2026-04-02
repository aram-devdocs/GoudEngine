//! Xbox GDK wgpu initialization path.
//!
//! Separated from `init.rs` to stay within the 500-line file limit.
//! TODO(xbox-gdk): Extract shared init logic (bind group layouts, fallback
//! textures, surface config) into a common helper before promoting this PoC
//! to production. Currently duplicates `new_async` to avoid refactoring the
//! existing init path during the feasibility study.

use super::{
    BackendCapabilities, BackendInfo, BlendFactor, CullFace, DepthFunc, FrontFace, HashMap,
    PrimitiveTopology, ShaderLanguage, WgpuBackend,
};
use crate::core::{
    error::{GoudError, GoudResult},
    handle::HandleAllocator,
};
use std::sync::Arc;

use super::init::{MAX_TEXTURE_UNITS, UNIFORM_BUFFER_SIZE};

impl WgpuBackend {
    /// Creates a new wgpu backend from a raw Xbox GDK window handle.
    ///
    /// Forces the DX12 backend since Xbox GDK uses DirectX 12 natively.
    /// Blocks on async wgpu initialization via pollster.
    pub fn new_from_raw_handle(
        handle: Arc<super::xbox_surface::XboxWindowHandle>,
        width: u32,
        height: u32,
    ) -> GoudResult<Self> {
        pollster::block_on(Self::new_xbox_async(handle, width, height))
    }

    async fn new_xbox_async(
        handle: Arc<super::xbox_surface::XboxWindowHandle>,
        width: u32,
        height: u32,
    ) -> GoudResult<Self> {
        let mut instance_desc = wgpu::InstanceDescriptor::new_without_display_handle();
        instance_desc.backends = wgpu::Backends::DX12;
        let instance = wgpu::Instance::new(instance_desc);

        let surface = instance
            .create_surface(wgpu::SurfaceTarget::from(handle))
            .map_err(|e| GoudError::BackendNotSupported(format!("wgpu Xbox surface: {e}")))?;

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .map_err(|e| {
                GoudError::BackendNotSupported(format!("No suitable DX12 adapter: {e}"))
            })?;

        let (device, queue): (wgpu::Device, wgpu::Queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("GoudEngine-Xbox"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                ..Default::default()
            })
            .await
            .map_err(|e| GoudError::BackendNotSupported(format!("wgpu device: {e}")))?;

        let caps = surface.get_capabilities(&adapter);
        let surface_format = caps
            .formats
            .iter()
            .find(|f| !f.is_srgb())
            .copied()
            .unwrap_or(caps.formats[0]);
        let surface_supports_copy_src = caps.usages.contains(wgpu::TextureUsages::COPY_SRC);

        let w = width.max(1);
        let h = height.max(1);
        let surface_config = wgpu::SurfaceConfiguration {
            usage: if surface_supports_copy_src {
                wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC
            } else {
                wgpu::TextureUsages::RENDER_ATTACHMENT
            },
            format: surface_format,
            width: w,
            height: h,
            present_mode: wgpu::PresentMode::AutoVsync,
            alpha_mode: caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &surface_config);

        let (depth_texture, depth_view) = Self::create_depth_texture(&device, w, h);

        let adapter_info = adapter.get_info();
        let limits = device.limits();

        let info = BackendInfo {
            name: "wgpu-xbox",
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
            window: None,
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
        })
    }
}
