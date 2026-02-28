//! Immediate-mode sprite batcher for wgpu on WebAssembly.
//!
//! Collects draw calls during a frame, then flushes them in a single
//! render pass at `end_frame()`. Sprites are batched by texture to
//! minimise bind group switches.

use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;

// ---------------------------------------------------------------------------
// Vertex format
// ---------------------------------------------------------------------------

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct SpriteVertex {
    pub position: [f32; 2],
    pub tex_coords: [f32; 2],
    pub color: [f32; 4],
}

impl SpriteVertex {
    const ATTRIBS: [wgpu::VertexAttribute; 3] = wgpu::vertex_attr_array![
        0 => Float32x2,
        1 => Float32x2,
        2 => Float32x4,
    ];

    pub fn layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<SpriteVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

// ---------------------------------------------------------------------------
// Draw batch (contiguous range of indices sharing one texture)
// ---------------------------------------------------------------------------

struct DrawBatch {
    texture_idx: u32,
    first_index: u32,
    index_count: u32,
}

// ---------------------------------------------------------------------------
// Public texture entry returned by the loader
// ---------------------------------------------------------------------------

pub struct TextureEntry {
    pub view: wgpu::TextureView,
    pub bind_group: wgpu::BindGroup,
    pub width: u32,
    pub height: u32,
}

// ---------------------------------------------------------------------------
// WGSL shader source
// ---------------------------------------------------------------------------

const SHADER_SRC: &str = r#"
struct Uniforms {
    projection: mat4x4<f32>,
}

struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) color: vec4<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) color: vec4<f32>,
}

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

@group(1) @binding(0)
var sprite_texture: texture_2d<f32>;

@group(1) @binding(1)
var sprite_sampler: sampler;

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;
    output.clip_position = uniforms.projection * vec4<f32>(input.position, 0.0, 1.0);
    output.tex_coords = input.tex_coords;
    output.color = input.color;
    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let tex_color = textureSample(sprite_texture, sprite_sampler, input.tex_coords);
    return tex_color * input.color;
}
"#;

// ---------------------------------------------------------------------------
// Sprite renderer
// ---------------------------------------------------------------------------

const MAX_SPRITES: usize = 4096;
const VERTS_PER_SPRITE: usize = 4;
const INDICES_PER_SPRITE: usize = 6;

pub struct WgpuSpriteRenderer {
    pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    uniform_buffer: wgpu::Buffer,
    uniform_bind_group: wgpu::BindGroup,
    pub texture_bind_group_layout: wgpu::BindGroupLayout,
    pub sampler: wgpu::Sampler,
    pub white_bind_group: wgpu::BindGroup,

    vertices: Vec<SpriteVertex>,
    batches: Vec<DrawBatch>,
    current_texture_idx: u32,
}

impl WgpuSpriteRenderer {
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
            bind_group_layouts: &[&uniform_bind_group_layout, &texture_bind_group_layout],
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

    pub fn begin_frame(&mut self) {
        self.vertices.clear();
        self.batches.clear();
        self.current_texture_idx = u32::MAX;
    }

    /// Queue a textured sprite. `texture_idx` is an index into the
    /// texture table managed by WasmGame (0 = white fallback).
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
        let cx = x + w * 0.5;
        let cy = y + h * 0.5;
        let cos = rotation.cos();
        let sin = rotation.sin();

        let corners = [
            (x - cx, y - cy, 0.0f32, 0.0f32),
            (x + w - cx, y - cy, 1.0, 0.0),
            (x + w - cx, y + h - cy, 1.0, 1.0),
            (x - cx, y + h - cy, 0.0, 1.0),
        ];

        for &(dx, dy, u, v) in &corners {
            self.vertices.push(SpriteVertex {
                position: [cx + dx * cos - dy * sin, cy + dx * sin + dy * cos],
                tex_coords: [u, v],
                color,
            });
        }
    }

    pub fn draw_quad(&mut self, x: f32, y: f32, w: f32, h: f32, r: f32, g: f32, b: f32, a: f32) {
        self.draw_sprite(0, x, y, w, h, 0.0, r, g, b, a);
    }

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
        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::bytes_of(&projection));

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

    fn batch_sprite_offset(&self) -> usize {
        self.batches
            .iter()
            .map(|b| b.index_count as usize / INDICES_PER_SPRITE)
            .sum()
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn generate_indices(max_sprites: usize) -> Vec<u32> {
    let mut indices = Vec::with_capacity(max_sprites * INDICES_PER_SPRITE);
    for i in 0..max_sprites as u32 {
        let base = i * VERTS_PER_SPRITE as u32;
        indices.extend_from_slice(&[base, base + 1, base + 2, base + 2, base + 3, base]);
    }
    indices
}

fn create_white_texture(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    layout: &wgpu::BindGroupLayout,
    sampler: &wgpu::Sampler,
) -> wgpu::BindGroup {
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("white_1x1"),
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
            texture: &texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        &[255u8, 255, 255, 255],
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(4),
            rows_per_image: None,
        },
        wgpu::Extent3d {
            width: 1,
            height: 1,
            depth_or_array_layers: 1,
        },
    );
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("white_bg"),
        layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(sampler),
            },
        ],
    })
}

/// 2D orthographic projection: (0,0) top-left, (w,h) bottom-right.
/// Column-major layout matching WGSL mat4x4<f32>.
#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
struct OrthoMatrix {
    data: [[f32; 4]; 4],
}

fn ortho_projection(w: f32, h: f32) -> OrthoMatrix {
    OrthoMatrix {
        data: [
            [2.0 / w, 0.0, 0.0, 0.0],
            [0.0, -2.0 / h, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [-1.0, 1.0, 0.0, 1.0],
        ],
    }
}

/// Creates a [`TextureEntry`] from raw RGBA pixels.
pub fn create_texture_entry(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    layout: &wgpu::BindGroupLayout,
    sampler: &wgpu::Sampler,
    width: u32,
    height: u32,
    rgba: &[u8],
) -> TextureEntry {
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("sprite_texture"),
        size: wgpu::Extent3d {
            width,
            height,
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
            texture: &texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        rgba,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(4 * width),
            rows_per_image: None,
        },
        wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
    );
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("sprite_tex_bg"),
        layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(sampler),
            },
        ],
    });
    TextureEntry {
        view,
        bind_group,
        width,
        height,
    }
}
