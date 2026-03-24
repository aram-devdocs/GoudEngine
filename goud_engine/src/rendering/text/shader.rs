use crate::libs::graphics::backend::types::ShaderHandle;
use crate::libs::graphics::backend::{RenderBackend, ShaderLanguage};

const TEXT_VERTEX_SHADER: &str = r#"#version 330 core
layout (location = 0) in vec2 a_position;
layout (location = 1) in vec2 a_tex_coord;
layout (location = 2) in vec4 a_color;

uniform vec2 u_viewport;

out vec2 v_tex_coord;
out vec4 v_color;

void main() {
    vec2 ndc;
    ndc.x = (a_position.x / u_viewport.x) * 2.0 - 1.0;
    ndc.y = 1.0 - (a_position.y / u_viewport.y) * 2.0;
    gl_Position = vec4(ndc, 0.0, 1.0);
    v_tex_coord = a_tex_coord;
    v_color = a_color;
}
"#;

const TEXT_FRAGMENT_SHADER: &str = r#"#version 330 core
in vec2 v_tex_coord;
in vec4 v_color;

uniform sampler2D u_texture;
out vec4 FragColor;

void main() {
    FragColor = texture(u_texture, v_tex_coord) * v_color;
}
"#;

const TEXT_VERTEX_SHADER_WGSL: &str = r#"
struct Uniforms {
    u_viewport: vec2<f32>,
}

@group(0) @binding(0) var<uniform> uniforms: Uniforms;

struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) tex_coord: vec2<f32>,
    @location(2) color: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coord: vec2<f32>,
    @location(1) color: vec4<f32>,
};

@vertex
fn main(input: VertexInput) -> VertexOutput {
    let safe_viewport = max(uniforms.u_viewport, vec2<f32>(1.0, 1.0));
    let ndc_x = (input.position.x / safe_viewport.x) * 2.0 - 1.0;
    let ndc_y = 1.0 - (input.position.y / safe_viewport.y) * 2.0;

    var output: VertexOutput;
    output.clip_position = vec4<f32>(ndc_x, ndc_y, 0.0, 1.0);
    output.tex_coord = input.tex_coord;
    output.color = input.color;
    return output;
}
"#;

const TEXT_FRAGMENT_SHADER_WGSL: &str = r#"
struct Uniforms {
    u_viewport: vec2<f32>,
}

@group(0) @binding(0) var<uniform> uniforms: Uniforms;
@group(1) @binding(0) var u_texture: texture_2d<f32>;
@group(1) @binding(1) var u_sampler: sampler;

@fragment
fn main(@location(0) tex_coord: vec2<f32>, @location(1) color: vec4<f32>) -> @location(0) vec4<f32> {
    return textureSample(u_texture, u_sampler, tex_coord) * color;
}
"#;

pub(crate) fn ensure_shader(
    shader_slot: &mut Option<ShaderHandle>,
    backend: &mut dyn RenderBackend,
) -> Result<ShaderHandle, String> {
    if let Some(shader) = *shader_slot {
        if backend.is_shader_valid(shader) {
            return Ok(shader);
        }
        *shader_slot = None;
    }

    let (vertex_shader, fragment_shader) = match backend.shader_language() {
        ShaderLanguage::Wgsl => (TEXT_VERTEX_SHADER_WGSL, TEXT_FRAGMENT_SHADER_WGSL),
        ShaderLanguage::Glsl => (TEXT_VERTEX_SHADER, TEXT_FRAGMENT_SHADER),
    };

    let shader = backend
        .create_shader(vertex_shader, fragment_shader)
        .map_err(|e| format!("text shader creation failed: {e}"))?;
    *shader_slot = Some(shader);
    Ok(shader)
}
