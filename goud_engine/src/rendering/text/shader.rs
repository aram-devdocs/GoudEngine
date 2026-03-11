use crate::libs::graphics::backend::types::ShaderHandle;
use crate::libs::graphics::backend::RenderBackend;

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

    let shader = backend
        .create_shader(TEXT_VERTEX_SHADER, TEXT_FRAGMENT_SHADER)
        .map_err(|e| format!("text shader creation failed: {e}"))?;
    *shader_slot = Some(shader);
    Ok(shader)
}
