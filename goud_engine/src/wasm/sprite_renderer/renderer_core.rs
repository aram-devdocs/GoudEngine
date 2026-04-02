//! [`WgpuSpriteRenderer`] — immediate-mode sprite batcher for wgpu.
//!
//! Collects draw calls during a frame, then flushes them in a single render pass
//! at [`WgpuSpriteRenderer::flush`]. Sprites are batched by texture to minimise
//! bind-group switches.

use bytemuck::bytes_of;
use wgpu::util::DeviceExt;

use super::texture::{
    create_white_texture, generate_indices, ortho_projection, INDICES_PER_SPRITE, MAX_SPRITES,
    VERTS_PER_SPRITE,
};
use super::types::{DrawBatch, RenderStats, SpriteVertex, TextureEntry, SHADER_SRC};

/// Immediate-mode wgpu sprite renderer.
///
/// Call [`begin_frame`](WgpuSpriteRenderer::begin_frame) at the start of each tick,
/// queue sprites with [`draw_sprite`](WgpuSpriteRenderer::draw_sprite) /
/// [`draw_quad`](WgpuSpriteRenderer::draw_quad), then call
/// [`flush`](WgpuSpriteRenderer::flush) to submit the render pass.
pub struct WgpuSpriteRenderer {
    pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    uniform_buffer: wgpu::Buffer,
    uniform_bind_group: wgpu::BindGroup,
    /// Bind group layout for texture + sampler pairs.
    pub texture_bind_group_layout: wgpu::BindGroupLayout,
    /// Shared nearest-neighbour sampler.
    pub sampler: wgpu::Sampler,
    /// Pre-built bind group for the 1×1 white fallback texture.
    pub white_bind_group: wgpu::BindGroup,

    vertices: Vec<SpriteVertex>,
    batches: Vec<DrawBatch>,
    current_texture_idx: u32,
}

impl WgpuSpriteRenderer {
    /// Creates all wgpu resources needed by the sprite renderer.
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue, format: wgpu::TextureFormat) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("sprite_shader"),
            source: wgpu::ShaderSource::Wgsl(SHADER_SRC.into()),
        });

        let uniform_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("uniform_bgl"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
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

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("sprite_pipeline_layout"),
            bind_group_layouts: &[
                Some(&uniform_bind_group_layout),
                Some(&texture_bind_group_layout),
            ],
            immediate_size: 0,
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("sprite_pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[SpriteVertex::layout()],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });

        let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("sprite_vb"),
            size: (MAX_SPRITES * VERTS_PER_SPRITE * std::mem::size_of::<SpriteVertex>())
                as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let indices = generate_indices(MAX_SPRITES);
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("sprite_ib"),
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("sprite_ub"),
            size: 64, // mat4x4<f32>
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("sprite_uniform_bg"),
            layout: &uniform_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("sprite_sampler"),
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let white_bind_group =
            create_white_texture(device, queue, &texture_bind_group_layout, &sampler);

        Self {
            pipeline,
            vertex_buffer,
            index_buffer,
            uniform_buffer,
            uniform_bind_group,
            texture_bind_group_layout,
            sampler,
            white_bind_group,
            vertices: Vec::with_capacity(MAX_SPRITES * VERTS_PER_SPRITE),
            batches: Vec::with_capacity(64),
            current_texture_idx: u32::MAX,
        }
    }

    /// Clears accumulated draw calls and resets batch state for the next frame.
    pub fn begin_frame(&mut self) {
        self.vertices.clear();
        self.batches.clear();
        self.current_texture_idx = u32::MAX;
    }

    /// Queues a textured sprite.
    ///
    /// `texture_idx` is an index into the texture table managed by `WasmGame`
    /// (0 = white fallback). `(x, y)` is the sprite centre, matching the
    /// OpenGL/FFI renderer convention.
    pub fn draw_sprite(
        &mut self,
        texture_idx: u32,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        rotation: f32,
        r: f32,
        g: f32,
        b: f32,
        a: f32,
    ) {
        if self.vertices.len() / VERTS_PER_SPRITE >= MAX_SPRITES {
            return;
        }

        if texture_idx != self.current_texture_idx {
            self.flush_batch();
            self.current_texture_idx = texture_idx;
        }

        let color = [r, g, b, a];
        let hw = w * 0.5;
        let hh = h * 0.5;
        let cos = rotation.cos();
        let sin = rotation.sin();

        let corners = [
            (-hw, -hh, 0.0f32, 0.0f32),
            (hw, -hh, 1.0, 0.0),
            (hw, hh, 1.0, 1.0),
            (-hw, hh, 0.0, 1.0),
        ];

        for &(dx, dy, u, v) in &corners {
            self.vertices.push(SpriteVertex {
                position: [x + dx * cos - dy * sin, y + dx * sin + dy * cos],
                tex_coords: [u, v],
                color,
            });
        }
    }

    /// Queues an untextured coloured quad (uses the white fallback texture).
    pub fn draw_quad(&mut self, x: f32, y: f32, w: f32, h: f32, r: f32, g: f32, b: f32, a: f32) {
        self.draw_sprite(0, x, y, w, h, 0.0, r, g, b, a);
    }

    /// Queues a textured sprite using a sub-rectangle of the source texture.
    ///
    /// `(src_x, src_y, src_w, src_h)` are UV coordinates in normalised [0, 1]
    /// space. Returns `true` if the sprite was queued, `false` if the batch is
    /// full.
    pub fn draw_sprite_rect(
        &mut self,
        texture_idx: u32,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        rotation: f32,
        src_x: f32,
        src_y: f32,
        src_w: f32,
        src_h: f32,
        r: f32,
        g: f32,
        b: f32,
        a: f32,
    ) -> bool {
        if self.vertices.len() / VERTS_PER_SPRITE >= MAX_SPRITES {
            return false;
        }

        if texture_idx != self.current_texture_idx {
            self.flush_batch();
            self.current_texture_idx = texture_idx;
        }

        let color = [r, g, b, a];
        let hw = w * 0.5;
        let hh = h * 0.5;
        let cos = rotation.cos();
        let sin = rotation.sin();

        let u1 = src_x;
        let v1 = src_y;
        let u2 = src_x + src_w;
        let v2 = src_y + src_h;

        let corners = [
            (-hw, -hh, u1, v1),
            (hw, -hh, u2, v1),
            (hw, hh, u2, v2),
            (-hw, hh, u1, v2),
        ];

        for &(dx, dy, u, v) in &corners {
            self.vertices.push(SpriteVertex {
                position: [x + dx * cos - dy * sin, y + dx * sin + dy * cos],
                tex_coords: [u, v],
                color,
            });
        }

        true
    }

    /// Returns per-frame rendering statistics based on the batches queued so far.
    pub fn render_stats(&self) -> RenderStats {
        let draw_calls = self.batches.len() as u32;
        let triangles = self.batches.iter().map(|b| b.index_count / 3).sum();
        let texture_binds = draw_calls;
        RenderStats {
            draw_calls,
            triangles,
            texture_binds,
        }
    }

    /// Uploads accumulated vertices, builds a render pass, and submits to the GPU.
    pub fn flush(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        target: &wgpu::TextureView,
        textures: &[Option<TextureEntry>],
        screen_w: u32,
        screen_h: u32,
        clear_color: [f64; 4],
    ) {
        self.flush_batch();

        let projection = ortho_projection(screen_w as f32, screen_h as f32);
        queue.write_buffer(&self.uniform_buffer, 0, bytes_of(&projection));

        if !self.vertices.is_empty() {
            queue.write_buffer(&self.vertex_buffer, 0, bytemuck::cast_slice(&self.vertices));
        }

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("sprite_encoder"),
        });

        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("sprite_pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: target,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: clear_color[0],
                            g: clear_color[1],
                            b: clear_color[2],
                            a: clear_color[3],
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                ..Default::default()
            });

            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &self.uniform_bind_group, &[]);
            pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);

            for batch in &self.batches {
                let bg = if batch.texture_idx == 0 {
                    &self.white_bind_group
                } else {
                    let idx = (batch.texture_idx - 1) as usize;
                    textures
                        .get(idx)
                        .and_then(|t| t.as_ref())
                        .map(|t| &t.bind_group)
                        .unwrap_or(&self.white_bind_group)
                };
                pass.set_bind_group(1, bg, &[]);
                pass.draw_indexed(
                    batch.first_index..batch.first_index + batch.index_count,
                    0,
                    0..1,
                );
            }
        }

        queue.submit(std::iter::once(encoder.finish()));
    }

    /// Closes the current batch if it contains any sprites.
    fn flush_batch(&mut self) {
        let sprite_count = self.vertices.len() / VERTS_PER_SPRITE - self.batch_sprite_offset();
        if sprite_count == 0 {
            return;
        }
        let first_index = (self.batch_sprite_offset() * INDICES_PER_SPRITE) as u32;
        let index_count = (sprite_count * INDICES_PER_SPRITE) as u32;
        self.batches.push(DrawBatch {
            texture_idx: self.current_texture_idx,
            first_index,
            index_count,
        });
    }

    /// Returns the total number of sprites already recorded in completed batches.
    fn batch_sprite_offset(&self) -> usize {
        self.batches
            .iter()
            .map(|b| b.index_count as usize / INDICES_PER_SPRITE)
            .sum()
    }
}
