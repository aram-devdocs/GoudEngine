//! Vertex format, batch bookkeeping, and texture entry types for the wgpu sprite renderer.

use bytemuck::{Pod, Zeroable};

// ---------------------------------------------------------------------------
// Vertex format
// ---------------------------------------------------------------------------

/// A single vertex in the sprite batch, carrying position, UV coordinates,
/// and a per-vertex tint colour.
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

    /// Returns the [`wgpu::VertexBufferLayout`] for this vertex type.
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

/// A contiguous range of indexed draw calls that share a single texture bind group.
pub(super) struct DrawBatch {
    pub texture_idx: u32,
    pub first_index: u32,
    pub index_count: u32,
}

// ---------------------------------------------------------------------------
// Render statistics
// ---------------------------------------------------------------------------

/// Per-frame rendering statistics collected by [`WgpuSpriteRenderer`].
pub struct RenderStats {
    /// Number of render-pass draw calls issued this frame.
    pub draw_calls: u32,
    /// Total triangles submitted this frame (2 per sprite).
    pub triangles: u32,
    /// Number of texture bind-group switches this frame.
    pub texture_binds: u32,
}

// ---------------------------------------------------------------------------
// Public texture entry returned by the loader
// ---------------------------------------------------------------------------

/// A fully-uploaded wgpu texture with its associated bind group, ready to render.
pub struct TextureEntry {
    pub view: wgpu::TextureView,
    pub bind_group: wgpu::BindGroup,
    pub width: u32,
    pub height: u32,
}

// ---------------------------------------------------------------------------
// WGSL shader source
// ---------------------------------------------------------------------------

/// WGSL shader used by the sprite pipeline.
pub(super) const SHADER_SRC: &str = r#"
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
