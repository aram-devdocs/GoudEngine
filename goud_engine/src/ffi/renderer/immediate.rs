//! # Immediate-Mode Rendering State
//!
//! Thread-local state management, shader sources, geometry setup, and math
//! helpers for immediate-mode quad/sprite rendering.

use crate::core::error::GoudError;
use crate::ffi::context::{GoudContextId, GOUD_INVALID_CONTEXT_ID};
use crate::ffi::window::with_window_state;
use crate::libs::graphics::backend::types::{VertexAttribute, VertexAttributeType, VertexLayout};
use crate::libs::graphics::backend::{BufferOps, RenderBackend, ShaderLanguage, ShaderOps};

// ============================================================================
// Coordinate Origin
// ============================================================================

/// Coordinate origin for immediate-mode draw calls (DrawQuad, DrawSprite).
///
/// Controls how the `(x, y)` position parameter is interpreted:
/// - `Center` (default): `(x, y)` is the center of the quad/sprite.
/// - `TopLeft`: `(x, y)` is the top-left corner of the quad/sprite.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CoordinateOrigin {
    /// `(x, y)` is the center of the shape (default, legacy behavior).
    #[default]
    Center = 0,
    /// `(x, y)` is the top-left corner of the shape.
    TopLeft = 1,
}

impl CoordinateOrigin {
    /// Converts a raw `u32` value into a `CoordinateOrigin`.
    /// Returns `None` for unknown values.
    pub fn from_u32(value: u32) -> Option<Self> {
        match value {
            0 => Some(Self::Center),
            1 => Some(Self::TopLeft),
            _ => None,
        }
    }

    /// Adjusts draw coordinates based on the origin setting.
    ///
    /// When `TopLeft`, converts top-left coordinates to center coordinates
    /// so the existing model matrix (which assumes center origin) works correctly.
    #[inline]
    pub fn adjust(self, x: f32, y: f32, width: f32, height: f32) -> (f32, f32) {
        match self {
            Self::Center => (x, y),
            Self::TopLeft => (x + width / 2.0, y + height / 2.0),
        }
    }
}

// ============================================================================
// Immediate-Mode State
// ============================================================================

// Thread-local storage for immediate-mode rendering resources per context.
// We use (index, generation) as a key to avoid needing access to private fields.
thread_local! {
    pub(super) static IMMEDIATE_STATE: std::cell::RefCell<std::collections::HashMap<(u32, u32), ImmediateRenderState>> =
        std::cell::RefCell::new(std::collections::HashMap::new());

    /// Per-context coordinate origin setting for immediate-mode draw calls.
    pub(super) static COORDINATE_ORIGIN: std::cell::RefCell<std::collections::HashMap<(u32, u32), CoordinateOrigin>> =
        std::cell::RefCell::new(std::collections::HashMap::new());
}

/// State for immediate-mode rendering.
pub(super) struct ImmediateRenderState {
    /// Shader program for sprite rendering
    pub shader: crate::libs::graphics::backend::types::ShaderHandle,
    /// Vertex buffer for quad rendering
    pub vertex_buffer: crate::libs::graphics::backend::types::BufferHandle,
    /// Index buffer for quad rendering (shared)
    pub index_buffer: crate::libs::graphics::backend::types::BufferHandle,
    /// Vertex layout for the immediate quad buffer.
    pub vertex_layout: VertexLayout,
    /// Uniform locations (cached)
    pub u_projection: i32,
    pub u_model: i32,
    pub u_color: i32,
    pub u_use_texture: i32,
    pub u_texture: i32,
    /// UV transform uniforms for sprite sheets
    pub u_uv_offset: i32,
    pub u_uv_scale: i32,
}

/// Vertex data for immediate-mode quad rendering.
#[repr(C)]
#[derive(Clone, Copy)]
pub(super) struct QuadVertex {
    pub position: [f32; 2],
    pub tex_coords: [f32; 2],
}

// SAFETY: QuadVertex is a plain data type with no padding issues
unsafe impl bytemuck::Pod for QuadVertex {}
unsafe impl bytemuck::Zeroable for QuadVertex {}

/// State data tuple type for immediate rendering.
pub(super) type ImmediateStateData = (
    crate::libs::graphics::backend::types::ShaderHandle,
    crate::libs::graphics::backend::types::BufferHandle,
    crate::libs::graphics::backend::types::BufferHandle,
    VertexLayout,
    i32, // u_projection
    i32, // u_model
    i32, // u_color
    i32, // u_use_texture
    i32, // u_texture
    i32, // u_uv_offset
    i32, // u_uv_scale
);

// ============================================================================
// Shader Sources
// ============================================================================

/// Sprite shader vertex source (GLSL 330 Core).
/// Supports UV transformation for sprite sheet animation.
pub(super) const SPRITE_VERTEX_SHADER: &str = r#"
#version 330 core

layout(location = 0) in vec2 a_position;
layout(location = 1) in vec2 a_texcoord;

uniform mat4 u_projection;
uniform mat4 u_model;
uniform vec2 u_uv_offset;  // UV offset for sprite sheets (default: 0,0)
uniform vec2 u_uv_scale;   // UV scale for sprite sheets (default: 1,1)

out vec2 v_texcoord;

void main() {
    gl_Position = u_projection * u_model * vec4(a_position, 0.0, 1.0);
    // Transform UV coordinates: apply scale then offset
    v_texcoord = a_texcoord * u_uv_scale + u_uv_offset;
}
"#;

/// Sprite shader fragment source (GLSL 330 Core).
pub(super) const SPRITE_FRAGMENT_SHADER: &str = r#"
#version 330 core

in vec2 v_texcoord;

uniform vec4 u_color;
uniform bool u_use_texture;
uniform sampler2D u_texture;

out vec4 FragColor;

void main() {
    if (u_use_texture) {
        FragColor = texture(u_texture, v_texcoord) * u_color;
    } else {
        FragColor = u_color;
    }
}
"#;

/// WGSL sprite shader (vertex + fragment combined).
///
/// Used by the wgpu backend. Matches the bind group layout in `wgpu_backend/init.rs`:
///   group(0) binding(0) = uniform buffer (Uniforms struct)
///   group(1) binding(0) = texture_2d
///   group(1) binding(1) = sampler
pub(super) const WGSL_SPRITE_VERTEX_SHADER: &str = r#"
struct Uniforms {
    u_projection: mat4x4<f32>,
    u_model: mat4x4<f32>,
    u_uv_offset: vec2<f32>,
    u_uv_scale: vec2<f32>,
    u_color: vec4<f32>,
    u_use_texture: u32,
}

@group(0) @binding(0) var<uniform> uniforms: Uniforms;

struct VertexInput {
    @location(0) a_position: vec2<f32>,
    @location(1) a_texcoord: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) v_texcoord: vec2<f32>,
}

@vertex
fn main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.position = uniforms.u_projection * uniforms.u_model * vec4<f32>(in.a_position, 0.0, 1.0);
    out.v_texcoord = in.a_texcoord * uniforms.u_uv_scale + uniforms.u_uv_offset;
    return out;
}
"#;

/// WGSL sprite fragment shader.
pub(super) const WGSL_SPRITE_FRAGMENT_SHADER: &str = r#"
struct Uniforms {
    u_projection: mat4x4<f32>,
    u_model: mat4x4<f32>,
    u_uv_offset: vec2<f32>,
    u_uv_scale: vec2<f32>,
    u_color: vec4<f32>,
    u_use_texture: u32,
}

@group(0) @binding(0) var<uniform> uniforms: Uniforms;
@group(1) @binding(0) var u_texture: texture_2d<f32>;
@group(1) @binding(1) var u_sampler: sampler;

@fragment
fn main(@location(0) v_texcoord: vec2<f32>) -> @location(0) vec4<f32> {
    if (uniforms.u_use_texture != 0u) {
        return textureSample(u_texture, u_sampler, v_texcoord) * uniforms.u_color;
    } else {
        return uniforms.u_color;
    }
}
"#;

/// Returns the backend-neutral vertex layout for `QuadVertex`.
pub(super) fn immediate_vertex_layout() -> VertexLayout {
    VertexLayout::new(std::mem::size_of::<QuadVertex>() as u32)
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
}

// ============================================================================
// Initialization
// ============================================================================

/// Initializes immediate-mode rendering resources for a context.
pub(super) fn ensure_immediate_state(context_id: GoudContextId) -> Result<(), GoudError> {
    use crate::libs::graphics::backend::types::{BufferType, BufferUsage};

    if context_id == GOUD_INVALID_CONTEXT_ID {
        return Err(GoudError::InvalidContext);
    }

    let context_key = (context_id.index(), context_id.generation());

    // Check if already initialized
    let already_initialized = IMMEDIATE_STATE.with(|cell| cell.borrow().contains_key(&context_key));

    if already_initialized {
        return Ok(());
    }

    // Initialize resources
    let state: Result<ImmediateRenderState, GoudError> =
        with_window_state(context_id, |window_state| {
            let backend = window_state.backend_mut();

            // Select shader sources based on backend type
            let (vert_src, frag_src) = match backend.shader_language() {
                ShaderLanguage::Wgsl => (WGSL_SPRITE_VERTEX_SHADER, WGSL_SPRITE_FRAGMENT_SHADER),
                ShaderLanguage::Glsl => (SPRITE_VERTEX_SHADER, SPRITE_FRAGMENT_SHADER),
            };
            let shader = backend.create_shader(vert_src, frag_src)?;

            // Get uniform locations
            let u_projection = backend
                .get_uniform_location(shader, "u_projection")
                .unwrap_or(-1);
            let u_model = backend
                .get_uniform_location(shader, "u_model")
                .unwrap_or(-1);
            let u_color = backend
                .get_uniform_location(shader, "u_color")
                .unwrap_or(-1);
            let u_use_texture = backend
                .get_uniform_location(shader, "u_use_texture")
                .unwrap_or(-1);
            let u_texture = backend
                .get_uniform_location(shader, "u_texture")
                .unwrap_or(-1);
            let u_uv_offset = backend
                .get_uniform_location(shader, "u_uv_offset")
                .unwrap_or(-1);
            let u_uv_scale = backend
                .get_uniform_location(shader, "u_uv_scale")
                .unwrap_or(-1);

            // Create quad vertices (unit quad centered at origin)
            // Position: -0.5 to 0.5, UV: 0 to 1
            let vertices: [QuadVertex; 4] = [
                QuadVertex {
                    position: [-0.5, -0.5],
                    tex_coords: [0.0, 0.0],
                }, // Bottom-left
                QuadVertex {
                    position: [0.5, -0.5],
                    tex_coords: [1.0, 0.0],
                }, // Bottom-right
                QuadVertex {
                    position: [0.5, 0.5],
                    tex_coords: [1.0, 1.0],
                }, // Top-right
                QuadVertex {
                    position: [-0.5, 0.5],
                    tex_coords: [0.0, 1.0],
                }, // Top-left
            ];

            let vertex_buffer = backend.create_buffer(
                BufferType::Vertex,
                BufferUsage::Static,
                bytemuck::cast_slice(&vertices),
            )?;

            // Create index buffer (two triangles)
            let indices: [u32; 6] = [0, 1, 2, 2, 3, 0];
            let index_buffer = backend.create_buffer(
                BufferType::Index,
                BufferUsage::Static,
                bytemuck::cast_slice(&indices),
            )?;

            Ok(ImmediateRenderState {
                shader,
                vertex_buffer,
                index_buffer,
                vertex_layout: immediate_vertex_layout(),
                u_projection,
                u_model,
                u_color,
                u_use_texture,
                u_texture,
                u_uv_offset,
                u_uv_scale,
            })
        })
        .ok_or(GoudError::InvalidContext)?;

    let state = state?;

    // Store state
    IMMEDIATE_STATE.with(|cell| {
        cell.borrow_mut().insert(context_key, state);
    });

    Ok(())
}

// ============================================================================
// Coordinate Origin Accessors
// ============================================================================

/// Returns the current coordinate origin for the given context.
/// Defaults to `CoordinateOrigin::Center` if not explicitly set.
pub(super) fn get_coordinate_origin(context_id: GoudContextId) -> CoordinateOrigin {
    let context_key = (context_id.index(), context_id.generation());
    COORDINATE_ORIGIN.with(|cell| cell.borrow().get(&context_key).copied().unwrap_or_default())
}

/// Sets the coordinate origin for the given context.
pub(super) fn set_coordinate_origin(context_id: GoudContextId, origin: CoordinateOrigin) {
    let context_key = (context_id.index(), context_id.generation());
    COORDINATE_ORIGIN.with(|cell| {
        cell.borrow_mut().insert(context_key, origin);
    });
}

// ============================================================================
// Math Helpers
// ============================================================================

/// Creates an orthographic projection matrix.
pub(super) fn ortho_matrix(left: f32, right: f32, bottom: f32, top: f32) -> [f32; 16] {
    let near = -1.0f32;
    let far = 1.0f32;

    [
        2.0 / (right - left),
        0.0,
        0.0,
        0.0,
        0.0,
        2.0 / (top - bottom),
        0.0,
        0.0,
        0.0,
        0.0,
        -2.0 / (far - near),
        0.0,
        -(right + left) / (right - left),
        -(top + bottom) / (top - bottom),
        -(far + near) / (far - near),
        1.0,
    ]
}

/// Creates a model matrix for sprite transformation.
pub(super) fn model_matrix(x: f32, y: f32, width: f32, height: f32, rotation: f32) -> [f32; 16] {
    let cos_r = rotation.cos();
    let sin_r = rotation.sin();

    // Translation * Rotation * Scale
    // Scale by width/height, rotate around center, then translate
    [
        width * cos_r,
        width * sin_r,
        0.0,
        0.0,
        -height * sin_r,
        height * cos_r,
        0.0,
        0.0,
        0.0,
        0.0,
        1.0,
        0.0,
        x,
        y,
        0.0,
        1.0,
    ]
}

#[cfg(test)]
mod tests {
    use super::CoordinateOrigin;

    #[test]
    fn test_coordinate_origin_default_is_center() {
        assert_eq!(CoordinateOrigin::default(), CoordinateOrigin::Center);
    }

    #[test]
    fn test_coordinate_origin_from_u32() {
        assert_eq!(
            CoordinateOrigin::from_u32(0),
            Some(CoordinateOrigin::Center)
        );
        assert_eq!(
            CoordinateOrigin::from_u32(1),
            Some(CoordinateOrigin::TopLeft)
        );
        assert_eq!(CoordinateOrigin::from_u32(2), None);
        assert_eq!(CoordinateOrigin::from_u32(u32::MAX), None);
    }

    #[test]
    fn test_coordinate_origin_adjust_center_is_noop() {
        let (ax, ay) = CoordinateOrigin::Center.adjust(100.0, 200.0, 50.0, 30.0);
        assert_eq!(ax, 100.0);
        assert_eq!(ay, 200.0);
    }

    #[test]
    fn test_coordinate_origin_adjust_topleft_offsets_by_half_size() {
        let (ax, ay) = CoordinateOrigin::TopLeft.adjust(100.0, 200.0, 50.0, 30.0);
        assert_eq!(ax, 125.0); // 100 + 50/2
        assert_eq!(ay, 215.0); // 200 + 30/2
    }

    #[test]
    fn test_coordinate_origin_repr_values() {
        assert_eq!(CoordinateOrigin::Center as u32, 0);
        assert_eq!(CoordinateOrigin::TopLeft as u32, 1);
    }
}
