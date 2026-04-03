//! GPU shadow pass resource management.
//!
//! Creates and manages the depth-only render pass resources used for shadow
//! mapping. The shadow pass renders the scene from the light's perspective
//! into an offscreen `Depth32Float` texture which is then sampled during
//! the main render pass.

use super::{convert, DrawType, PipelineKey, WgpuBackend};
use crate::libs::graphics::backend::VertexStepMode;

/// Creates the bind group layout for shadow depth texture + comparison sampler (group 3).
///
/// Shared across all init paths (winit, SDL, Switch, Xbox) to avoid duplication.
pub(super) fn create_shadow_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("shadow_bgl"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Depth,
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Comparison),
                count: None,
            },
        ],
    })
}

/// Creates a fallback 1x1 depth texture + bind group for draws without shadows.
///
/// Shared across all init paths to avoid duplication of ~60 lines of GPU resource creation.
pub(super) fn create_fallback_shadow_bind_group(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    layout: &wgpu::BindGroupLayout,
) -> wgpu::BindGroup {
    let tex = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("fallback-shadow-1x1"),
        size: wgpu::Extent3d {
            width: 1,
            height: 1,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Depth32Float,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    });
    // Initialize fallback depth to 1.0 via a clear render pass.
    let view = tex.create_view(&wgpu::TextureViewDescriptor::default());
    {
        let mut init_encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        let _ = init_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("fallback-shadow-clear"),
            color_attachments: &[],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &view,
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
        queue.submit(std::iter::once(init_encoder.finish()));
    }
    let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("fallback-shadow-sampler"),
        compare: Some(wgpu::CompareFunction::LessEqual),
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        ..Default::default()
    });
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("fallback-shadow-bg"),
        layout,
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
}

impl WgpuBackend {
    /// Lazily creates or resizes the shadow depth texture and associated views,
    /// sampler, and bind group.
    pub(super) fn ensure_shadow_resources_impl(&mut self, size: u32) {
        let size = size.max(1);
        if self.shadow_map_size == size && self.shadow_depth_texture.is_some() {
            return;
        }

        let tex = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("shadow-depth"),
            size: wgpu::Extent3d {
                width: size,
                height: size,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        let depth_view = tex.create_view(&wgpu::TextureViewDescriptor {
            label: Some("shadow-depth-attachment"),
            ..Default::default()
        });

        let sample_view = tex.create_view(&wgpu::TextureViewDescriptor {
            label: Some("shadow-depth-sample"),
            aspect: wgpu::TextureAspect::DepthOnly,
            ..Default::default()
        });

        let sampler = self.device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("shadow-comparison-sampler"),
            compare: Some(wgpu::CompareFunction::LessEqual),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            ..Default::default()
        });

        let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("shadow-bg"),
            layout: &self.shadow_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&sample_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });

        self.shadow_depth_texture = Some(tex);
        self.shadow_depth_view = Some(depth_view);
        self.shadow_sample_view = Some(sample_view);
        self.shadow_sampler = Some(sampler);
        self.shadow_bind_group = Some(bind_group);
        self.shadow_map_size = size;
    }

    /// Begins recording draw commands for the shadow pre-pass.
    ///
    /// While recording, `record_draw()` appends to `shadow_draw_commands`
    /// instead of `draw_commands`.
    pub(super) fn begin_shadow_recording_impl(&mut self) {
        self.recording_shadow = true;
        self.shadow_draw_commands.clear();
    }

    /// Ends shadow recording mode.
    pub(super) fn end_shadow_recording_impl(&mut self) {
        self.recording_shadow = false;
    }

    /// Requests that the current frame's surface be read back after rendering.
    ///
    /// Without calling this, the readback buffer is not prepared, avoiding
    /// the GPU stall cost on frames that do not need post-processing.
    pub(super) fn request_readback_impl(&mut self) {
        self.readback_requested = true;
    }

    /// Builds depth-only shadow pipelines for keys that are not yet cached.
    ///
    /// Shadow pipelines differ from main pipelines: they target `Depth32Float`
    /// with no color attachment and use a minimal depth-only shader.
    pub(super) fn build_missing_shadow_pipelines(&mut self, cmd_keys: &[PipelineKey]) {
        for (i, key) in cmd_keys.iter().enumerate() {
            if self.shadow_pipeline_cache.contains_key(key) {
                continue;
            }
            let cmd = &self.shadow_draw_commands[i];
            let shader_meta = match self.shaders.get(&cmd.shader) {
                Some(m) => m,
                None => continue,
            };

            let pipeline_layout =
                self.device
                    .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                        label: Some("shadow-pipeline-layout"),
                        bind_group_layouts: &[Some(&self.uniform_bind_group_layout)],
                        immediate_size: 0,
                    });

            let wgpu_attr_storage: Vec<Vec<wgpu::VertexAttribute>> = cmd
                .vertex_bindings
                .iter()
                .map(|binding| {
                    binding
                        .layout
                        .attributes
                        .iter()
                        .map(|a| wgpu::VertexAttribute {
                            format: convert::map_vertex_format(a.attribute_type),
                            offset: a.offset as u64,
                            shader_location: a.location,
                        })
                        .collect()
                })
                .collect();
            let vertex_buffers: Vec<_> = cmd
                .vertex_bindings
                .iter()
                .zip(wgpu_attr_storage.iter())
                .map(|(binding, attrs)| wgpu::VertexBufferLayout {
                    array_stride: binding.layout.stride as u64,
                    step_mode: match binding.step_mode {
                        VertexStepMode::Vertex => wgpu::VertexStepMode::Vertex,
                        VertexStepMode::Instance => wgpu::VertexStepMode::Instance,
                    },
                    attributes: attrs,
                })
                .collect();

            let pipeline = self
                .device
                .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some("shadow-pipeline"),
                    layout: Some(&pipeline_layout),
                    vertex: wgpu::VertexState {
                        module: &shader_meta.vertex_module,
                        entry_point: Some("main"),
                        buffers: &vertex_buffers,
                        compilation_options: Default::default(),
                    },
                    // No fragment stage -- depth-only writes.
                    fragment: None,
                    primitive: wgpu::PrimitiveState {
                        topology: wgpu::PrimitiveTopology::TriangleList,
                        front_face: wgpu::FrontFace::Ccw,
                        cull_mode: Some(wgpu::Face::Back),
                        polygon_mode: wgpu::PolygonMode::Fill,
                        ..Default::default()
                    },
                    depth_stencil: Some(wgpu::DepthStencilState {
                        format: wgpu::TextureFormat::Depth32Float,
                        depth_write_enabled: Some(true),
                        depth_compare: Some(wgpu::CompareFunction::Less),
                        stencil: wgpu::StencilState::default(),
                        bias: wgpu::DepthBiasState::default(),
                    }),
                    multisample: wgpu::MultisampleState::default(),
                    multiview_mask: None,
                    cache: None,
                });

            self.shadow_pipeline_cache.insert(key.clone(), pipeline);
        }
    }

    /// Uploads shadow uniforms and executes the depth-only shadow render pass.
    ///
    /// Called from `end_frame()` before the main render pass. Drains
    /// `shadow_draw_commands` after execution.
    pub(super) fn execute_shadow_pass(&mut self, encoder: &mut wgpu::CommandEncoder) {
        if self.shadow_draw_commands.is_empty() || self.shadow_depth_view.is_none() {
            return;
        }

        let shadow_align = self.device.limits().min_uniform_buffer_offset_alignment as usize;
        let shadow_slot_size = {
            let snap = self
                .shadow_draw_commands
                .iter()
                .map(|c| c.uniform_ring_size as usize)
                .max()
                .unwrap_or(256);
            (snap + shadow_align - 1) & !(shadow_align - 1)
        };
        let shadow_total = self.shadow_draw_commands.len() * shadow_slot_size;
        let shadow_offsets: Vec<u32> = (0..self.shadow_draw_commands.len())
            .map(|i| (i * shadow_slot_size) as u32)
            .collect();

        // Grow uniform buffers if needed.
        for cmd in &self.shadow_draw_commands {
            if let Some(meta) = self.shaders.get_mut(&cmd.shader) {
                if shadow_total > meta.uniform_buffer.size() as usize {
                    let new_size = shadow_total.next_power_of_two().max(shadow_slot_size);
                    meta.uniform_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
                        label: Some("shadow-uniforms"),
                        size: new_size as u64,
                        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                        mapped_at_creation: false,
                    });
                    meta.uniform_bind_group =
                        self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                            label: None,
                            layout: &self.uniform_bind_group_layout,
                            entries: &[wgpu::BindGroupEntry {
                                binding: 0,
                                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                                    buffer: &meta.uniform_buffer,
                                    offset: 0,
                                    size: std::num::NonZeroU64::new(shadow_slot_size as u64),
                                }),
                            }],
                        });
                }
            }
        }

        // Write shadow uniform data to GPU.
        for (i, cmd) in self.shadow_draw_commands.iter().enumerate() {
            let gpu_offset = shadow_offsets[i] as u64;
            let ring_start = cmd.uniform_ring_offset as usize;
            let ring_end = ring_start + cmd.uniform_ring_size as usize;
            if ring_end <= self.uniform_ring.len() {
                if let Some(meta) = self.shaders.get(&cmd.shader) {
                    self.queue.write_buffer(
                        &meta.uniform_buffer,
                        gpu_offset,
                        &self.uniform_ring[ring_start..ring_end],
                    );
                }
            }
        }

        let shadow_keys: Vec<PipelineKey> = self
            .shadow_draw_commands
            .iter()
            .map(|cmd| self.make_pipeline_key(cmd))
            .collect();
        self.build_missing_shadow_pipelines(&shadow_keys);

        // SAFETY: shadow_depth_view is confirmed Some above.
        let shadow_view = self.shadow_depth_view.as_ref().unwrap();

        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("shadow-pass"),
                color_attachments: &[],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: shadow_view,
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

            for (i, cmd) in self.shadow_draw_commands.iter().enumerate() {
                let key = &shadow_keys[i];
                let Some(pipeline) = self.shadow_pipeline_cache.get(key) else {
                    continue;
                };
                pass.set_pipeline(pipeline);
                for (slot, binding) in cmd.vertex_bindings.iter().enumerate() {
                    let Some(vb_meta) = self.buffers.get(&binding.buffer) else {
                        continue;
                    };
                    pass.set_vertex_buffer(slot as u32, vb_meta.buffer.slice(..));
                }
                if let Some(shader_meta) = self.shaders.get(&cmd.shader) {
                    pass.set_bind_group(0, &shader_meta.uniform_bind_group, &[shadow_offsets[i]]);
                }
                match cmd.draw_type {
                    DrawType::Arrays { first, count } => {
                        pass.draw(first..first + count, 0..1);
                    }
                    DrawType::Indexed { count, .. } | DrawType::IndexedU16 { count, .. } => {
                        let first = cmd.draw_type.first_index();
                        if let Some(ib_handle) = cmd.index_buffer {
                            if let Some(ib_meta) = self.buffers.get(&ib_handle) {
                                let format = match cmd.draw_type {
                                    DrawType::IndexedU16 { .. } => wgpu::IndexFormat::Uint16,
                                    _ => wgpu::IndexFormat::Uint32,
                                };
                                pass.set_index_buffer(ib_meta.buffer.slice(..), format);
                            }
                        }
                        pass.draw_indexed(first..first + count, 0, 0..1);
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
                        let first = cmd.draw_type.first_index();
                        pass.draw_indexed(first..first + count, 0, 0..instances);
                    }
                }
            }
        }
        self.shadow_draw_commands.clear();
    }
}
