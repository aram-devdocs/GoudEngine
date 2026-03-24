//! Shader source constants for the sprite batch renderer.
//!
//! Contains GLSL 330 core shaders for the OpenGL backend and WGSL shaders
//! for the wgpu backend. Both perform the same viewport-to-NDC transform
//! with texture sampling and vertex color multiplication.

// ============================================================================
// GLSL batch shader sources (OpenGL backend)
// ============================================================================

pub(super) const BATCH_VERTEX_SHADER: &str = r#"
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

pub(super) const BATCH_FRAGMENT_SHADER: &str = r#"
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

pub(super) const BATCH_VERTEX_SHADER_WGSL: &str = r#"
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

pub(super) const BATCH_FRAGMENT_SHADER_WGSL: &str = r#"
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
