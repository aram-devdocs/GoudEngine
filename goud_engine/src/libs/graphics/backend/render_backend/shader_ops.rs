//! Shader operations sub-trait for `RenderBackend`.

use crate::libs::error::GoudResult;
use crate::libs::graphics::backend::types::ShaderHandle;

/// GPU shader program operations.
///
/// Handles compilation, linking, binding, destruction of shader programs,
/// and setting uniform values.
pub trait ShaderOps {
    /// Compiles and links a shader program from vertex and fragment shader sources.
    ///
    /// # Arguments
    /// * `vertex_src` - GLSL vertex shader source code
    /// * `fragment_src` - GLSL fragment shader source code
    ///
    /// # Returns
    /// A handle to the compiled shader program, or an error if compilation/linking failed.
    fn create_shader(&mut self, vertex_src: &str, fragment_src: &str) -> GoudResult<ShaderHandle>;

    /// Destroys a shader program and frees GPU memory.
    ///
    /// # Returns
    /// `true` if the shader was destroyed, `false` if the handle was invalid.
    fn destroy_shader(&mut self, handle: ShaderHandle) -> bool;

    /// Checks if a shader handle is valid and refers to an existing shader program.
    fn is_shader_valid(&self, handle: ShaderHandle) -> bool;

    /// Binds a shader program for use in subsequent draw calls.
    ///
    /// # Note
    /// Only one shader program can be active at a time.
    /// Binding a new shader replaces the previous one.
    fn bind_shader(&mut self, handle: ShaderHandle) -> GoudResult<()>;

    /// Unbinds the currently bound shader program.
    fn unbind_shader(&mut self);

    /// Gets the location of a uniform variable in a shader program.
    ///
    /// # Returns
    /// The uniform location, or `None` if the handle is invalid,
    /// the uniform doesn't exist, or was optimized out by the compiler.
    ///
    /// # Note
    /// The shader must be bound before setting uniform values.
    fn get_uniform_location(&self, handle: ShaderHandle, name: &str) -> Option<i32>;

    /// Sets a uniform integer value.
    fn set_uniform_int(&mut self, location: i32, value: i32);

    /// Sets a uniform float value.
    fn set_uniform_float(&mut self, location: i32, value: f32);

    /// Sets a uniform vec2 value.
    fn set_uniform_vec2(&mut self, location: i32, x: f32, y: f32);

    /// Sets a uniform vec3 value.
    fn set_uniform_vec3(&mut self, location: i32, x: f32, y: f32, z: f32);

    /// Sets a uniform vec4 value.
    fn set_uniform_vec4(&mut self, location: i32, x: f32, y: f32, z: f32, w: f32);

    /// Sets a uniform mat4 value.
    ///
    /// # Arguments
    /// * `location` - Uniform location
    /// * `matrix` - 16 floats in column-major order
    fn set_uniform_mat4(&mut self, location: i32, matrix: &[f32; 16]);
}
