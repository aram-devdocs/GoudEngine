// src/shader.rs

use cgmath::Matrix;
use gl::types::*;
use std::collections::HashMap;
use std::ffi::CString;
use std::ptr;

use cgmath::Matrix4;
use cgmath::Vector3;
use cgmath::Vector4;

#[derive(Debug)]
pub struct ShaderProgram {
    program_id: GLuint,
    uniforms: HashMap<String, GLint>,
}

impl ShaderProgram {
    /// Creates a new Shader Program from vertex and fragment shader files.
    pub fn new() -> Result<ShaderProgram, String> {
        let vertex_code = include_str!("shaders/2d/vertex_shader.glsl");
        let fragment_code = include_str!("shaders/2d/fragment_shader.glsl");

        let vertex_shader = ShaderProgram::compile_shader(vertex_code, gl::VERTEX_SHADER)?;
        let fragment_shader = ShaderProgram::compile_shader(fragment_code, gl::FRAGMENT_SHADER)?;

        ShaderProgram::create_program(vertex_shader, fragment_shader)
    }

    pub fn new_3d() -> Result<ShaderProgram, String> {
        let vertex_code = include_str!("shaders/3d/vertex_shader_3d.glsl");
        let fragment_code = include_str!("shaders/3d/fragment_shader_3d.glsl");

        let vertex_shader = ShaderProgram::compile_shader(vertex_code, gl::VERTEX_SHADER)?;
        let fragment_shader = ShaderProgram::compile_shader(fragment_code, gl::FRAGMENT_SHADER)?;

        ShaderProgram::create_program(vertex_shader, fragment_shader)
    }

    pub fn new_skybox() -> Result<ShaderProgram, String> {
        println!("Loading skybox shaders...");
        let vertex_code = include_str!("shaders/3d/skybox_vertex.glsl");
        let fragment_code = include_str!("shaders/3d/skybox_fragment.glsl");

        println!("Compiling skybox vertex shader...");
        let vertex_shader = ShaderProgram::compile_shader(vertex_code, gl::VERTEX_SHADER)?;
        println!("Compiling skybox fragment shader...");
        let fragment_shader = ShaderProgram::compile_shader(fragment_code, gl::FRAGMENT_SHADER)?;

        println!("Creating skybox shader program...");
        let mut program = ShaderProgram::create_program(vertex_shader, fragment_shader)?;

        println!("Setting up skybox uniforms...");
        program.bind();
        program.create_uniform("skybox")?;
        program.set_uniform_int("skybox", 0)?;
        program.create_uniform("view")?;
        program.create_uniform("projection")?;
        println!("Skybox shader setup complete");

        Ok(program)
    }

    fn compile_shader(code: &str, shader_type: GLenum) -> Result<GLuint, String> {
        unsafe {
            let shader = gl::CreateShader(shader_type);
            let c_str = CString::new(code.as_bytes()).unwrap();
            gl::ShaderSource(shader, 1, &c_str.as_ptr(), ptr::null());
            gl::CompileShader(shader);

            let mut success = gl::FALSE as GLint;
            gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut success);
            if success != gl::TRUE as GLint {
                let mut len = 0;
                gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut len);
                let mut buffer = vec![0u8; (len as usize) - 1];
                gl::GetShaderInfoLog(
                    shader,
                    len,
                    ptr::null_mut(),
                    buffer.as_mut_ptr() as *mut GLchar,
                );
                return Err(String::from_utf8_lossy(&buffer).to_string());
            }

            Ok(shader)
        }
    }

    fn create_program(
        vertex_shader: GLuint,
        fragment_shader: GLuint,
    ) -> Result<ShaderProgram, String> {
        unsafe {
            let program = gl::CreateProgram();
            gl::AttachShader(program, vertex_shader);
            gl::AttachShader(program, fragment_shader);
            gl::LinkProgram(program);

            // Check for linking errors
            let mut success = gl::FALSE as GLint;
            gl::GetProgramiv(program, gl::LINK_STATUS, &mut success);
            if success != gl::TRUE as GLint {
                let mut len = 0;
                gl::GetProgramiv(program, gl::INFO_LOG_LENGTH, &mut len);
                let mut buffer = vec![0u8; (len as usize) - 1];
                gl::GetProgramInfoLog(
                    program,
                    len,
                    ptr::null_mut(),
                    buffer.as_mut_ptr() as *mut GLchar,
                );
                return Err(String::from_utf8_lossy(&buffer).to_string());
            }

            gl::DeleteShader(vertex_shader);
            gl::DeleteShader(fragment_shader);

            Ok(ShaderProgram {
                program_id: program,
                uniforms: HashMap::new(),
            })
        }
    }

    /// Activates the shader program.
    pub fn bind(&self) {
        unsafe {
            gl::UseProgram(self.program_id);
        }
    }

    /// Creates a uniform variable.
    pub fn create_uniform(&mut self, uniform_name: &str) -> Result<(), String> {
        let c_str = CString::new(uniform_name).unwrap();
        unsafe {
            let location = gl::GetUniformLocation(self.program_id, c_str.as_ptr());
            if location < 0 {
                return Err(format!("Could not find uniform: {}", uniform_name));
            }
            self.uniforms.insert(uniform_name.to_string(), location);
        }
        Ok(())
    }

    /// Sets an integer uniform variable.
    pub fn set_uniform_int(&self, uniform_name: &str, value: i32) -> Result<(), String> {
        let location = self.get_uniform_location(uniform_name)?;
        unsafe {
            gl::Uniform1i(location, value);
        }
        Ok(())
    }

    /// Sets a float uniform variable.
    pub fn set_uniform_float(&self, uniform_name: &str, value: f32) -> Result<(), String> {
        let location = self.get_uniform_location(uniform_name)?;
        unsafe {
            gl::Uniform1f(location, value);
        }
        Ok(())
    }

    /// Sets a vec3 uniform variable.
    pub fn set_uniform_vec3(&self, uniform_name: &str, value: &Vector3<f32>) -> Result<(), String> {
        let location = self.get_uniform_location(uniform_name)?;
        unsafe {
            gl::Uniform3f(location, value.x, value.y, value.z);
        }
        Ok(())
    }

    /// Sets a vec4 uniform variable.
    pub fn set_uniform_vec4(&self, uniform_name: &str, value: &Vector4<f32>) -> Result<(), String> {
        let location = self.get_uniform_location(uniform_name)?;
        unsafe {
            gl::Uniform4f(location, value.x, value.y, value.z, value.w);
        }
        Ok(())
    }

    /// Sets a 4x4 matrix uniform variable.
    pub fn set_uniform_mat4(&self, uniform_name: &str, value: &Matrix4<f32>) -> Result<(), String> {
        let location = self.get_uniform_location(uniform_name)?;
        unsafe {
            gl::UniformMatrix4fv(location, 1, gl::FALSE, value.as_ptr());
        }
        Ok(())
    }

    fn get_uniform_location(&self, uniform_name: &str) -> Result<GLint, String> {
        match self.uniforms.get(uniform_name) {
            Some(location) => Ok(*location),
            None => Err(format!("Could not find uniform: {}", uniform_name)),
        }
    }

    pub fn terminate(&self) {
        unsafe {
            gl::DeleteProgram(self.program_id);
        }
    }
}
