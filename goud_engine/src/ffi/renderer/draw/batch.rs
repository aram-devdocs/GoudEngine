//! # Sprite Batch FFI
//!
//! Provides `goud_renderer_draw_sprite_batch` for drawing many sprites in a
//! single batched GPU call.  Sprites are sorted by z-layer then by texture to
//! minimise GPU state changes.

use std::cell::RefCell;

use crate::core::debugger;
use crate::core::error::{set_last_error, GoudError};
use crate::ffi::context::GoudContextId;
use crate::ffi::window::with_window_state;
use crate::libs::graphics::backend::types::{
    BufferHandle, BufferType, BufferUsage, PrimitiveTopology, ShaderHandle, TextureHandle,
    VertexAttribute, VertexAttributeType, VertexLayout,
};
use crate::libs::graphics::backend::{
    BlendFactor, BufferOps, DrawOps, RenderBackend, ShaderLanguage, ShaderOps, StateOps, TextureOps,
};

use super::super::immediate::get_coordinate_origin;
use super::super::texture::GoudTextureHandle;
use super::internal::pixel_to_uv;

// ============================================================================
// FfiSpriteCmd -- the struct callers fill in from C# / Python
// ============================================================================

/// A single sprite command for batch rendering.
///
/// All position values are in screen-space pixels.
/// Source-rect values (`src_x`, `src_y`, `src_w`, `src_h`) are in **pixel**
/// coordinates; the batch renderer converts them to UVs automatically.
/// When `src_w` and `src_h` are both 0 the full texture is used.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct FfiSpriteCmd {
    /// Texture handle from `goud_texture_load`.
    pub texture: GoudTextureHandle,
    /// X position in screen-space pixels.
    pub x: f32,
    /// Y position in screen-space pixels.
    pub y: f32,
    /// Width of the sprite on screen.
    pub width: f32,
    /// Height of the sprite on screen.
    pub height: f32,
    /// Rotation in radians.
    pub rotation: f32,
    /// Source rectangle X offset in pixel coordinates.
    pub src_x: f32,
    /// Source rectangle Y offset in pixel coordinates.
    pub src_y: f32,
    /// Source rectangle width in pixel coordinates (0 = full texture width).
    pub src_w: f32,
    /// Source rectangle height in pixel coordinates (0 = full texture height).
    pub src_h: f32,
    /// Red color tint (1.0 = no tint).
    pub r: f32,
    /// Green color tint (1.0 = no tint).
    pub g: f32,
    /// Blue color tint (1.0 = no tint).
    pub b: f32,
    /// Alpha (opacity, 1.0 = fully opaque).
    pub a: f32,
    /// Z-layer for depth sorting (lower values drawn first).
    pub z_layer: i32,
    /// Padding for alignment (set to 0).
    pub _padding: i32,
}

// ============================================================================
// BatchVertex -- per-corner vertex for the batch shader
// ============================================================================

#[repr(C)]
#[derive(Clone, Copy, Debug)]
struct BatchVertex {
    position: [f32; 2],
    tex_coords: [f32; 2],
    color: [f32; 4],
}

// SAFETY: BatchVertex is a plain-data #[repr(C)] struct with no padding issues.
unsafe impl bytemuck::Pod for BatchVertex {}
// SAFETY: BatchVertex contains only f32 arrays which are valid when zeroed.
unsafe impl bytemuck::Zeroable for BatchVertex {}

fn batch_vertex_layout() -> VertexLayout {
    VertexLayout::new(std::mem::size_of::<BatchVertex>() as u32)
        .with_attribute(VertexAttribute::new(
            0,
            VertexAttributeType::Float2,
            0,
            false,
        ))
        .with_attribute(VertexAttribute::new(
            1,
            VertexAttributeType::Float2,
            8,
            false,
        ))
        .with_attribute(VertexAttribute::new(
            2,
            VertexAttributeType::Float4,
            16,
            false,
        ))
}

// ============================================================================
// Batch shader sources
// ============================================================================

const BATCH_VERTEX_SHADER: &str = r#"
#version 330 core

layout(location = 0) in vec2 a_position;
layout(location = 1) in vec2 a_texcoord;
layout(location = 2) in vec4 a_color;

uniform vec2 u_viewport;

out vec2 v_texcoord;
out vec4 v_color;

void main() {
    vec2 safe_viewport = max(u_viewport, vec2(1.0, 1.0));
    vec2 ndc;
    ndc.x = (a_position.x / safe_viewport.x) * 2.0 - 1.0;
    ndc.y = 1.0 - (a_position.y / safe_viewport.y) * 2.0;
    gl_Position = vec4(ndc, 0.0, 1.0);
    v_texcoord = a_texcoord;
    v_color = a_color;
}
"#;

const BATCH_FRAGMENT_SHADER: &str = r#"
#version 330 core

in vec2 v_texcoord;
in vec4 v_color;

uniform sampler2D u_texture;

out vec4 FragColor;

void main() {
    FragColor = texture(u_texture, v_texcoord) * v_color;
}
"#;

// ============================================================================
// WGSL batch shader sources (wgpu backend)
// ============================================================================

const BATCH_VERTEX_SHADER_WGSL: &str = r#"
struct Uniforms {
    u_viewport: vec2<f32>,
}

@group(0) @binding(0) var<uniform> uniforms: Uniforms;

struct VertexInput {
    @location(0) a_position: vec2<f32>,
    @location(1) a_texcoord: vec2<f32>,
    @location(2) a_color: vec4<f32>,
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) v_texcoord: vec2<f32>,
    @location(1) v_color: vec4<f32>,
}

@vertex
fn main(in: VertexInput) -> VertexOutput {
    let safe_viewport = max(uniforms.u_viewport, vec2<f32>(1.0, 1.0));
    let ndc_x = (in.a_position.x / safe_viewport.x) * 2.0 - 1.0;
    let ndc_y = 1.0 - (in.a_position.y / safe_viewport.y) * 2.0;

    var out: VertexOutput;
    out.position = vec4<f32>(ndc_x, ndc_y, 0.0, 1.0);
    out.v_texcoord = in.a_texcoord;
    out.v_color = in.a_color;
    return out;
}
"#;

const BATCH_FRAGMENT_SHADER_WGSL: &str = r#"
struct Uniforms {
    u_viewport: vec2<f32>,
}

@group(0) @binding(0) var<uniform> uniforms: Uniforms;
@group(1) @binding(0) var u_texture: texture_2d<f32>;
@group(1) @binding(1) var u_sampler: sampler;

@fragment
fn main(@location(0) v_texcoord: vec2<f32>, @location(1) v_color: vec4<f32>) -> @location(0) vec4<f32> {
    return textureSample(u_texture, u_sampler, v_texcoord) * v_color;
}
"#;

// ============================================================================
// Thread-local batch GPU state (one per context, lazily initialized)
// ============================================================================

struct BatchRenderState {
    shader: ShaderHandle,
    vertex_buffer: BufferHandle,
    index_buffer: BufferHandle,
    vertex_layout: VertexLayout,
    u_viewport: i32,
    u_texture: i32,
    /// Current capacity of the vertex buffer (number of vertices).
    vertex_capacity: usize,
    /// Current capacity of the index buffer (number of indices).
    index_capacity: usize,
}

thread_local! {
    static BATCH_STATE: RefCell<std::collections::HashMap<(u32, u32), BatchRenderState>> =
        RefCell::new(std::collections::HashMap::new());
}

// ============================================================================
// Public FFI entry point
// ============================================================================

/// Draws a batch of sprites in a single GPU pass. Sprites are sorted by
/// `z_layer` then grouped by texture. Source-rect fields are in **pixel**
/// coordinates (`src_w = 0`, `src_h = 0` → full texture). Returns number
/// of sprites drawn (0 on error).
///
/// # Safety
///
/// `cmds` must point to `count` valid `FfiSpriteCmd` values for the call duration.
#[no_mangle]
pub unsafe extern "C" fn goud_renderer_draw_sprite_batch(
    context_id: GoudContextId,
    cmds: *const FfiSpriteCmd,
    count: u32,
) -> u32 {
    // --- null / zero guards ---------------------------------------------------
    if cmds.is_null() {
        set_last_error(GoudError::InvalidState("cmds pointer is null".into()));
        return 0;
    }
    if count == 0 {
        return 0;
    }

    // SAFETY: caller guarantees `cmds` points to `count` valid FfiSpriteCmds.
    let cmds_slice = std::slice::from_raw_parts(cmds, count as usize);

    let origin = get_coordinate_origin(context_id);

    // --- sort by z_layer then texture for optimal batching --------------------
    let mut sorted: Vec<&FfiSpriteCmd> = cmds_slice.iter().collect();
    sorted.sort_by(|a, b| {
        a.z_layer
            .cmp(&b.z_layer)
            .then_with(|| a.texture.cmp(&b.texture))
    });

    // --- build vertices -------------------------------------------------------
    let result = with_window_state(context_id, |window_state| {
        let (win_w, win_h) = window_state.get_size();
        let (fb_w, fb_h) = window_state.get_framebuffer_size();
        let viewport = [win_w as f32, win_h as f32];

        // Pre-allocate vertex + index buffers
        let sprite_count = sorted.len();
        let mut vertices: Vec<BatchVertex> = Vec::with_capacity(sprite_count * 4);
        let mut indices: Vec<u32> = Vec::with_capacity(sprite_count * 6);

        // Build a list of (texture_handle, index_start, index_count) batches
        struct Batch {
            texture: TextureHandle,
            index_start: usize,
            index_count: usize,
        }
        let mut batches: Vec<Batch> = Vec::new();
        let mut current_texture: Option<GoudTextureHandle> = None;

        let backend = window_state.backend_mut();

        for cmd in &sorted {
            let tex_index = (cmd.texture & 0xFFFFFFFF) as u32;
            let tex_generation = ((cmd.texture >> 32) & 0xFFFFFFFF) as u32;
            let tex_handle = TextureHandle::new(tex_index, tex_generation);

            // Resolve UVs from pixel source rect
            let (uv_x, uv_y, uv_w, uv_h) = if cmd.src_w == 0.0 && cmd.src_h == 0.0 {
                (0.0f32, 0.0f32, 1.0f32, 1.0f32)
            } else {
                match backend.texture_size(tex_handle) {
                    Some((tw, th)) => {
                        pixel_to_uv(cmd.src_x, cmd.src_y, cmd.src_w, cmd.src_h, tw, th)
                    }
                    None => (0.0, 0.0, 1.0, 1.0),
                }
            };

            // Coordinate origin adjustment
            let (cx, cy) = origin.adjust(cmd.x, cmd.y, cmd.width, cmd.height);

            // Build 4 corner vertices (world-space) with rotation
            let hw = cmd.width / 2.0;
            let hh = cmd.height / 2.0;
            let cos_r = cmd.rotation.cos();
            let sin_r = cmd.rotation.sin();

            let corners: [(f32, f32, f32, f32); 4] = [
                (-hw, -hh, uv_x, uv_y),             // top-left
                (hw, -hh, uv_x + uv_w, uv_y),       // top-right
                (hw, hh, uv_x + uv_w, uv_y + uv_h), // bottom-right
                (-hw, hh, uv_x, uv_y + uv_h),       // bottom-left
            ];

            let base_index = vertices.len() as u32;
            for (lx, ly, u, v) in &corners {
                let wx = cx + lx * cos_r - ly * sin_r;
                let wy = cy + lx * sin_r + ly * cos_r;
                vertices.push(BatchVertex {
                    position: [wx, wy],
                    tex_coords: [*u, *v],
                    color: [cmd.r, cmd.g, cmd.b, cmd.a],
                });
            }

            // If texture changed, start a new batch
            if current_texture != Some(cmd.texture) {
                batches.push(Batch {
                    texture: tex_handle,
                    index_start: indices.len(),
                    index_count: 0,
                });
                current_texture = Some(cmd.texture);
            }

            // Quad indices (two triangles)
            indices.extend_from_slice(&[
                base_index,
                base_index + 1,
                base_index + 2,
                base_index + 2,
                base_index + 3,
                base_index,
            ]);
            if let Some(last) = batches.last_mut() {
                last.index_count += 6;
            }
        }

        if vertices.is_empty() {
            return Ok((0u32, 0u32));
        }

        // --- ensure GPU state is initialized ----------------------------------
        let context_key = (context_id.index(), context_id.generation());
        let needs_init = BATCH_STATE.with(|cell| !cell.borrow().contains_key(&context_key));

        if needs_init {
            let (vert_src, frag_src) = match backend.shader_language() {
                ShaderLanguage::Wgsl => (BATCH_VERTEX_SHADER_WGSL, BATCH_FRAGMENT_SHADER_WGSL),
                ShaderLanguage::Glsl => (BATCH_VERTEX_SHADER, BATCH_FRAGMENT_SHADER),
            };
            let shader = backend.create_shader(vert_src, frag_src)?;
            let u_viewport = backend
                .get_uniform_location(shader, "u_viewport")
                .unwrap_or(-1);
            let u_texture = backend
                .get_uniform_location(shader, "u_texture")
                .unwrap_or(-1);

            let vb = backend.create_buffer(
                BufferType::Vertex,
                BufferUsage::Dynamic,
                bytemuck::cast_slice(&vertices),
            )?;
            let ib = backend.create_buffer(
                BufferType::Index,
                BufferUsage::Dynamic,
                bytemuck::cast_slice(&indices),
            )?;

            let state = BatchRenderState {
                shader,
                vertex_buffer: vb,
                index_buffer: ib,
                vertex_layout: batch_vertex_layout(),
                u_viewport,
                u_texture,
                vertex_capacity: vertices.len(),
                index_capacity: indices.len(),
            };
            BATCH_STATE.with(|cell| cell.borrow_mut().insert(context_key, state));
        }

        // --- upload & draw -----------------------------------------------------
        BATCH_STATE.with(|cell| {
            let mut map = cell.borrow_mut();
            let state = map.get_mut(&context_key).unwrap();

            // Resize buffers if needed (create new before destroying old to
            // avoid leaving dangling handles on allocation failure).
            if vertices.len() > state.vertex_capacity {
                let new_vb = backend.create_buffer(
                    BufferType::Vertex,
                    BufferUsage::Dynamic,
                    bytemuck::cast_slice(&vertices),
                )?;
                let _ = backend.destroy_buffer(state.vertex_buffer);
                state.vertex_buffer = new_vb;
                state.vertex_capacity = vertices.len();
            } else {
                backend.update_buffer(state.vertex_buffer, 0, bytemuck::cast_slice(&vertices))?;
            }

            if indices.len() > state.index_capacity {
                let new_ib = backend.create_buffer(
                    BufferType::Index,
                    BufferUsage::Dynamic,
                    bytemuck::cast_slice(&indices),
                )?;
                let _ = backend.destroy_buffer(state.index_buffer);
                state.index_buffer = new_ib;
                state.index_capacity = indices.len();
            } else {
                backend.update_buffer(state.index_buffer, 0, bytemuck::cast_slice(&indices))?;
            }

            // Set common GL state
            backend.set_viewport(0, 0, fb_w, fb_h);
            backend.enable_blending();
            backend.set_blend_func(BlendFactor::SrcAlpha, BlendFactor::OneMinusSrcAlpha);
            backend.bind_default_vertex_array();
            backend.bind_buffer(state.vertex_buffer)?;
            backend.bind_buffer(state.index_buffer)?;
            backend.set_vertex_attributes(&state.vertex_layout);
            backend.bind_shader(state.shader)?;
            backend.set_uniform_vec2(state.u_viewport, viewport[0], viewport[1]);
            backend.set_uniform_int(state.u_texture, 0);

            // Draw each batch
            let mut drawn = 0u32;
            let batch_count = batches.len() as u32;
            for batch in &batches {
                backend.bind_texture(batch.texture, 0)?;
                // Offset is in bytes: index_start * sizeof(u32)
                let byte_offset = batch.index_start * std::mem::size_of::<u32>();
                backend.draw_indexed(
                    PrimitiveTopology::Triangles,
                    batch.index_count as u32,
                    byte_offset,
                )?;
                drawn += (batch.index_count / 6) as u32;
            }

            Ok((drawn, batch_count))
        })
    });

    match result {
        Some(Ok((drawn, batch_count))) => {
            if drawn > 0 {
                let _ = debugger::update_render_stats_for_context(
                    context_id,
                    drawn,
                    drawn * 2,
                    batch_count,
                    batch_count,
                );
            }
            drawn
        }
        Some(Err(e)) => {
            set_last_error(e);
            0
        }
        None => {
            set_last_error(GoudError::InvalidContext);
            0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ffi_sprite_cmd_layout() {
        // u64 (8) + 13 x f32 (52) + 2 x i32 (8) = 68, padded to 72 for 8-byte alignment
        assert_eq!(
            std::mem::size_of::<FfiSpriteCmd>(),
            72,
            "FfiSpriteCmd should be 72 bytes"
        );
        assert_eq!(
            std::mem::align_of::<FfiSpriteCmd>(),
            8,
            "FfiSpriteCmd should be 8-byte aligned (due to u64 texture field)"
        );
    }

    #[test]
    fn test_batch_null_pointer_returns_zero() {
        // SAFETY: passing null pointer deliberately to test the guard
        let result = unsafe {
            goud_renderer_draw_sprite_batch(GoudContextId::from_raw(0), std::ptr::null(), 10)
        };
        assert_eq!(result, 0);
    }

    #[test]
    fn test_batch_zero_count_returns_zero() {
        let cmd = FfiSpriteCmd {
            texture: 0,
            x: 0.0,
            y: 0.0,
            width: 32.0,
            height: 32.0,
            rotation: 0.0,
            src_x: 0.0,
            src_y: 0.0,
            src_w: 0.0,
            src_h: 0.0,
            r: 1.0,
            g: 1.0,
            b: 1.0,
            a: 1.0,
            z_layer: 0,
            _padding: 0,
        };
        // SAFETY: valid pointer, count=0
        let result = unsafe {
            goud_renderer_draw_sprite_batch(
                GoudContextId::from_raw(0),
                &cmd as *const FfiSpriteCmd,
                0,
            )
        };
        assert_eq!(result, 0);
    }
}
