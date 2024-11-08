// src/shader.rs

use cgmath::Matrix;
use gl::types::*;
use std::collections::HashMap;
use std::ffi::CString;
use std::fs;
use std::ptr;

use cgmath::Matrix4;
use cgmath::Vector4;

#[derive(Debug)]
pub struct ShaderProgram {
    id: GLuint,
    uniform_locations: HashMap<String, GLint>,
}

impl ShaderProgram {
    /// Creates a new Shader Program from vertex and fragment shader files.
    pub fn new(vertex_path: &str, fragment_path: &str) -> Result<ShaderProgram, String> {
        let vertex_code = fs::read_to_string(vertex_path)
            .map_err(|_| format!("Failed to read vertex shader from {}", vertex_path))?;
        let fragment_code = fs::read_to_string(fragment_path)
            .map_err(|_| format!("Failed to read fragment shader from {}", fragment_path))?;

        let vertex_shader = ShaderProgram::compile_shader(&vertex_code, gl::VERTEX_SHADER)?;
        let fragment_shader = ShaderProgram::compile_shader(&fragment_code, gl::FRAGMENT_SHADER)?;

        let id = ShaderProgram::link_program(vertex_shader, fragment_shader)?;

        // Clean up shaders as they're linked into our program now and no longer necessary
        unsafe {
            gl::DeleteShader(vertex_shader);
            gl::DeleteShader(fragment_shader);
        }

        Ok(ShaderProgram {
            id,
            uniform_locations: HashMap::new(),
        })
    }

    /// Compiles a shader from source code.
    fn compile_shader(source: &str, shader_type: GLenum) -> Result<GLuint, String> {
        let shader;
        unsafe {
            shader = gl::CreateShader(shader_type);
            let c_str = CString::new(source.as_bytes()).unwrap();
            gl::ShaderSource(shader, 1, &c_str.as_ptr(), ptr::null());
            gl::CompileShader(shader);

            // Check for compilation errors
            let mut success = gl::FALSE as GLint;
            gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut success);
            if success != gl::TRUE as GLint {
                let mut len = 0;
                gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut len);
                let mut buffer = Vec::with_capacity(len as usize);
                buffer.set_len((len as usize) - 1); // Skip null terminator
                gl::GetShaderInfoLog(
                    shader,
                    len,
                    ptr::null_mut(),
                    buffer.as_mut_ptr() as *mut GLchar,
                );
                let error = String::from_utf8_lossy(&buffer).into_owned();
                return Err(error);
            }
        }

        Ok(shader)
    }

    /// Links vertex and fragment shaders into a shader program.
    fn link_program(vertex_shader: GLuint, fragment_shader: GLuint) -> Result<GLuint, String> {
        let program;
        unsafe {
            program = gl::CreateProgram();
            gl::AttachShader(program, vertex_shader);
            gl::AttachShader(program, fragment_shader);
            gl::LinkProgram(program);

            // Check for linking errors
            let mut success = gl::FALSE as GLint;
            gl::GetProgramiv(program, gl::LINK_STATUS, &mut success);
            if success != gl::TRUE as GLint {
                let mut len = 0;
                gl::GetProgramiv(program, gl::INFO_LOG_LENGTH, &mut len);
                let mut buffer = Vec::with_capacity(len as usize);
                buffer.set_len((len as usize) - 1); // Skip null terminator
                gl::GetProgramInfoLog(
                    program,
                    len,
                    ptr::null_mut(),
                    buffer.as_mut_ptr() as *mut GLchar,
                );
                let error = String::from_utf8_lossy(&buffer).into_owned();
                return Err(error);
            }
        }
        Ok(program)
    }

    /// Activates the shader program.
    pub fn bind(&self) {
        unsafe {
            gl::UseProgram(self.id);
        }
    }

    /// Creates a uniform variable.
    pub fn create_uniform(&mut self, name: &str) -> Result<(), String> {
        let c_name = CString::new(name).unwrap();
        let location = unsafe { gl::GetUniformLocation(self.id, c_name.as_ptr()) };
        if location < 0 {
            return Err(format!("Uniform '{}' not found", name));
        }
        self.uniform_locations.insert(name.into(), location);
        Ok(())
    }

    /// Sets an integer uniform variable.
    pub fn set_uniform_int(&self, name: &str, value: GLint) -> Result<(), String> {
        if let Some(&location) = self.uniform_locations.get(name) {
            unsafe {
                gl::Uniform1i(location, value);
            }
            Ok(())
        } else {
            Err(format!("Uniform '{}' not found", name))
        }
    }

    /// Sets a 4x4 matrix uniform variable.
    pub fn set_uniform_mat4(&self, name: &str, matrix: &Matrix4<f32>) -> Result<(), String> {
        if let Some(&location) = self.uniform_locations.get(name) {
            unsafe {
                gl::UniformMatrix4fv(location, 1, gl::FALSE, matrix.as_ptr());
            }
            Ok(())
        } else {
            Err(format!("Uniform '{}' not found", name))
        }
    }

    /// Sets a vec4 uniform variable.
    pub fn set_uniform_vec4(&self, name: &str, vector: &Vector4<f32>) -> Result<(), String> {
        if let Some(&location) = self.uniform_locations.get(name) {
            unsafe {
                gl::Uniform4f(location, vector.x, vector.y, vector.z, vector.w);
            }
            Ok(())
        } else {
            Err(format!("Uniform '{}' not found", name))
        }
    }
}

impl Drop for ShaderProgram {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteProgram(self.id);
        }
    }
}
