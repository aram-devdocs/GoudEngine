//! Render pipeline building and caching.
//!
//! Creates and caches `wgpu::RenderPipeline` objects keyed on the combination
//! of GPU state that determines pipeline identity (shader, blend mode, depth
//! func, cull mode, vertex layout, etc.).

use super::{
    convert, BlendFactor, CullFace, DepthFunc, FrontFace, PipelineKey, PrimitiveTopology,
    WgpuBackend,
};
use crate::libs::graphics::backend::VertexStepMode;

impl WgpuBackend {
    /// Ensures all required pipelines in `cmd_keys` exist in the pipeline cache.
    ///
    /// For each key that is absent, this fetches the shader metadata and creates
    /// the pipeline. Keys for which no shader is found are silently skipped.
    pub(super) fn build_missing_pipelines(&mut self, cmd_keys: &[PipelineKey]) {
        for (i, key) in cmd_keys.iter().enumerate() {
            if self.pipeline_cache.contains_key(key) {
                continue;
            }
            let cmd = &self.draw_commands[i];
            let shader_meta = match self.shaders.get(&cmd.shader) {
                Some(m) => m,
                None => continue,
            };

            // Use a 3-bind-group layout when the draw command has a storage
            // buffer (GPU skinning), otherwise use the standard 2-group layout.
            let has_storage = cmd.storage_buffer.is_some();
            let pipeline_layout = if has_storage {
                self.device
                    .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                        label: None,
                        bind_group_layouts: &[
                            &self.uniform_bind_group_layout,
                            &self.texture_bind_group_layout,
                            &self.storage_bind_group_layout,
                        ],
                        immediate_size: 0,
                    })
            } else {
                self.device
                    .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                        label: None,
                        bind_group_layouts: &[
                            &self.uniform_bind_group_layout,
                            &self.texture_bind_group_layout,
                        ],
                        immediate_size: 0,
                    })
            };

            let blend_state = if key.blend_enabled {
                // SAFETY: key.blend_src is stored as u8 cast from BlendFactor (repr(u8)); the reverse transmute is always valid.
                let src = unsafe { std::mem::transmute::<u8, BlendFactor>(key.blend_src) };
                // SAFETY: key.blend_dst is stored as u8 cast from BlendFactor (repr(u8)); the reverse transmute is always valid.
                let dst = unsafe { std::mem::transmute::<u8, BlendFactor>(key.blend_dst) };
                Some(wgpu::BlendState {
                    color: wgpu::BlendComponent {
                        src_factor: convert::map_blend_factor(src),
                        dst_factor: convert::map_blend_factor(dst),
                        operation: wgpu::BlendOperation::Add,
                    },
                    alpha: wgpu::BlendComponent {
                        src_factor: convert::map_blend_factor(src),
                        dst_factor: convert::map_blend_factor(dst),
                        operation: wgpu::BlendOperation::Add,
                    },
                })
            } else {
                None
            };

            let depth_stencil = Some(if key.depth_test {
                // SAFETY: key.depth_func is stored as u8 cast from DepthFunc (repr(u8)).
                let func = unsafe { std::mem::transmute::<u8, DepthFunc>(key.depth_func) };
                wgpu::DepthStencilState {
                    format: wgpu::TextureFormat::Depth32Float,
                    depth_write_enabled: key.depth_write,
                    depth_compare: convert::map_depth_func(func),
                    stencil: wgpu::StencilState::default(),
                    bias: wgpu::DepthBiasState::default(),
                }
            } else {
                wgpu::DepthStencilState {
                    format: wgpu::TextureFormat::Depth32Float,
                    depth_write_enabled: false,
                    depth_compare: wgpu::CompareFunction::Always,
                    stencil: wgpu::StencilState::default(),
                    bias: wgpu::DepthBiasState::default(),
                }
            });

            let cull_mode = if key.cull_enabled {
                // SAFETY: key.cull_face is stored as u8 cast from CullFace (repr(u8)).
                let face = unsafe { std::mem::transmute::<u8, CullFace>(key.cull_face) };
                convert::map_cull_face(face)
            } else {
                None
            };
            // SAFETY: key.front_face is stored as u8 cast from FrontFace (repr(u8)); the reverse transmute is always valid.
            let front_face = convert::map_front_face(unsafe {
                std::mem::transmute::<u8, FrontFace>(key.front_face)
            });
            // SAFETY: key.topology is stored as u8 cast from PrimitiveTopology (repr(u8)); the reverse transmute is always valid.
            let topology = convert::map_topology(unsafe {
                std::mem::transmute::<u8, PrimitiveTopology>(key.topology)
            });
            let strip_index_format = match topology {
                wgpu::PrimitiveTopology::LineStrip | wgpu::PrimitiveTopology::TriangleStrip => {
                    Some(wgpu::IndexFormat::Uint32)
                }
                _ => None,
            };

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
                    label: None,
                    layout: Some(&pipeline_layout),
                    vertex: wgpu::VertexState {
                        module: &shader_meta.vertex_module,
                        entry_point: Some("main"),
                        buffers: &vertex_buffers,
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
