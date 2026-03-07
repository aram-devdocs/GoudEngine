//! Tests for OpenGL shader operations (require GL context).

use crate::libs::error::GoudError;
use crate::libs::graphics::backend::opengl::backend::OpenGLBackend;
use crate::libs::graphics::backend::types::ShaderHandle;
use crate::libs::graphics::backend::RenderBackend;

const VERTEX_SRC: &str = r#"
    #version 330 core
    layout(location = 0) in vec3 position;
    void main() { gl_Position = vec4(position, 1.0); }
"#;

const FRAGMENT_SRC: &str = r#"
    #version 330 core
    out vec4 FragColor;
    void main() { FragColor = vec4(1.0); }
"#;

#[test]
#[ignore] // Requires OpenGL context
fn test_shader_lifecycle() {
    let mut backend = OpenGLBackend::new().unwrap();

    let fragment_src = r#"
        #version 330 core
        out vec4 FragColor;
        void main() { FragColor = vec4(1.0, 0.0, 0.0, 1.0); }
    "#;

    let handle = backend.create_shader(VERTEX_SRC, fragment_src).unwrap();

    assert!(backend.is_shader_valid(handle));
    assert!(backend.destroy_shader(handle));
    assert!(!backend.is_shader_valid(handle));
}

#[test]
#[ignore] // Requires OpenGL context
fn test_shader_empty_sources() {
    let mut backend = OpenGLBackend::new().unwrap();

    let result = backend.create_shader("", "void main() {}");
    assert!(result.is_err());

    let result = backend.create_shader("void main() {}", "");
    assert!(result.is_err());

    let result = backend.create_shader("", "");
    assert!(result.is_err());
}

#[test]
#[ignore] // Requires OpenGL context
fn test_shader_compilation_error() {
    let mut backend = OpenGLBackend::new().unwrap();

    let vertex_src = "this is not valid GLSL code";
    let fragment_src = r#"
        #version 330 core
        out vec4 FragColor;
        void main() { FragColor = vec4(1.0); }
    "#;

    let result = backend.create_shader(vertex_src, fragment_src);
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        GoudError::ShaderCompilationFailed(_)
    ));
}

#[test]
#[ignore] // Requires OpenGL context
fn test_shader_bind_unbind() {
    let mut backend = OpenGLBackend::new().unwrap();

    let handle = backend.create_shader(VERTEX_SRC, FRAGMENT_SRC).unwrap();

    backend.bind_shader(handle).unwrap();
    assert_eq!(
        backend.bound_shader,
        backend.shaders.get(&handle).map(|m| m.gl_id)
    );

    backend.unbind_shader();
    assert_eq!(backend.bound_shader, None);

    backend.destroy_shader(handle);
}

#[test]
#[ignore] // Requires OpenGL context
fn test_shader_invalid_handle() {
    let mut backend = OpenGLBackend::new().unwrap();
    let invalid_handle = ShaderHandle::INVALID;

    assert!(!backend.is_shader_valid(invalid_handle));
    assert!(backend.bind_shader(invalid_handle).is_err());
    assert!(!backend.destroy_shader(invalid_handle));
}

#[test]
#[ignore] // Requires OpenGL context
fn test_shader_uniform_location() {
    let mut backend = OpenGLBackend::new().unwrap();

    let fragment_src = r#"
        #version 330 core
        uniform vec4 color;
        out vec4 FragColor;
        void main() { FragColor = color; }
    "#;

    let handle = backend.create_shader(VERTEX_SRC, fragment_src).unwrap();

    let location = backend.get_uniform_location(handle, "color");
    assert!(location.is_some());
    assert!(location.unwrap() >= 0);

    let location = backend.get_uniform_location(handle, "nonexistent");
    assert!(location.is_none());

    backend.destroy_shader(handle);
}

#[test]
#[ignore] // Requires OpenGL context
fn test_shader_set_uniforms() {
    let mut backend = OpenGLBackend::new().unwrap();

    let fragment_src = r#"
        #version 330 core
        uniform int intVal;
        uniform float floatVal;
        uniform vec2 vec2Val;
        uniform vec3 vec3Val;
        uniform vec4 vec4Val;
        uniform mat4 matVal;
        out vec4 FragColor;
        void main() { FragColor = vec4(1.0); }
    "#;

    let handle = backend.create_shader(VERTEX_SRC, fragment_src).unwrap();
    backend.bind_shader(handle).unwrap();

    if let Some(loc) = backend.get_uniform_location(handle, "intVal") {
        backend.set_uniform_int(loc, 42);
    }
    if let Some(loc) = backend.get_uniform_location(handle, "floatVal") {
        backend.set_uniform_float(loc, 3.14);
    }
    if let Some(loc) = backend.get_uniform_location(handle, "vec2Val") {
        backend.set_uniform_vec2(loc, 1.0, 2.0);
    }
    if let Some(loc) = backend.get_uniform_location(handle, "vec3Val") {
        backend.set_uniform_vec3(loc, 1.0, 2.0, 3.0);
    }
    if let Some(loc) = backend.get_uniform_location(handle, "vec4Val") {
        backend.set_uniform_vec4(loc, 1.0, 2.0, 3.0, 4.0);
    }
    if let Some(loc) = backend.get_uniform_location(handle, "matVal") {
        let identity: [f32; 16] = [
            1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
        ];
        backend.set_uniform_mat4(loc, &identity);
    }

    backend.destroy_shader(handle);
}

#[test]
#[ignore] // Requires OpenGL context
fn test_multiple_shaders() {
    let mut backend = OpenGLBackend::new().unwrap();

    let fragment_src1 = r#"
        #version 330 core
        out vec4 FragColor;
        void main() { FragColor = vec4(1.0, 0.0, 0.0, 1.0); }
    "#;
    let fragment_src2 = r#"
        #version 330 core
        out vec4 FragColor;
        void main() { FragColor = vec4(0.0, 1.0, 0.0, 1.0); }
    "#;

    let handle1 = backend.create_shader(VERTEX_SRC, fragment_src1).unwrap();
    let handle2 = backend.create_shader(VERTEX_SRC, fragment_src2).unwrap();

    assert!(backend.is_shader_valid(handle1));
    assert!(backend.is_shader_valid(handle2));
    assert_ne!(handle1, handle2);

    backend.destroy_shader(handle1);
    backend.destroy_shader(handle2);
}

#[test]
#[ignore] // Requires OpenGL context
fn test_shader_slot_reuse() {
    let mut backend = OpenGLBackend::new().unwrap();

    let handle1 = backend.create_shader(VERTEX_SRC, FRAGMENT_SRC).unwrap();
    backend.destroy_shader(handle1);

    let handle2 = backend.create_shader(VERTEX_SRC, FRAGMENT_SRC).unwrap();

    assert_eq!(handle1.index(), handle2.index());
    assert_ne!(handle1.generation(), handle2.generation());
    assert!(!backend.is_shader_valid(handle1));
    assert!(backend.is_shader_valid(handle2));

    backend.destroy_shader(handle2);
}

#[test]
#[ignore] // Requires OpenGL context
fn test_shader_destroy_clears_bound_state() {
    let mut backend = OpenGLBackend::new().unwrap();

    let handle = backend.create_shader(VERTEX_SRC, FRAGMENT_SRC).unwrap();

    backend.bind_shader(handle).unwrap();
    assert!(backend.bound_shader.is_some());

    backend.destroy_shader(handle);

    assert!(backend.bound_shader.is_none());
}
