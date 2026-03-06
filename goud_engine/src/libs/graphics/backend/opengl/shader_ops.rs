//! OpenGL shader compile/link/destroy/bind operations and uniform setters.

use super::{backend::OpenGLBackend, ShaderMetadata};
use crate::core::error::{GoudError, GoudResult};
use crate::libs::graphics::backend::types::ShaderHandle;
use std::collections::HashMap;

/// Compiles and links a shader program from vertex and fragment shader sources.
pub(super) fn create_shader(
    backend: &mut OpenGLBackend,
    vertex_src: &str,
    fragment_src: &str,
) -> GoudResult<ShaderHandle> {
    if vertex_src.is_empty() {
        return Err(GoudError::ShaderCompilationFailed(
            "Vertex shader source is empty".to_string(),
        ));
    }
    if fragment_src.is_empty() {
        return Err(GoudError::ShaderCompilationFailed(
            "Fragment shader source is empty".to_string(),
        ));
    }

    // Compile vertex shader
    let vertex_shader = compile_shader(gl::VERTEX_SHADER, vertex_src)?;

    // Compile fragment shader
    let fragment_shader = match compile_shader(gl::FRAGMENT_SHADER, fragment_src) {
        Ok(shader) => shader,
        Err(e) => {
            // Clean up vertex shader on fragment compilation failure
            // SAFETY: GL context is active; vertex_shader was successfully created by compile_shader and has not been freed.
            unsafe {
                gl::DeleteShader(vertex_shader);
            }
            return Err(e);
        }
    };

    // Link shader program
    // SAFETY: GL context is active; CreateProgram returns a new program object or 0 on failure.
    let program_id = unsafe { gl::CreateProgram() };
    if program_id == 0 {
        // SAFETY: GL context is active; both shader objects were created by compile_shader and have not been freed.
        unsafe {
            gl::DeleteShader(vertex_shader);
            gl::DeleteShader(fragment_shader);
        }
        return Err(GoudError::ShaderLinkFailed(
            "Failed to create shader program".to_string(),
        ));
    }

    // SAFETY: GL context is active; program_id and both shader IDs are valid objects created above.
    unsafe {
        gl::AttachShader(program_id, vertex_shader);
        gl::AttachShader(program_id, fragment_shader);
        gl::LinkProgram(program_id);

        // Check link status
        let mut success: i32 = 0;
        gl::GetProgramiv(program_id, gl::LINK_STATUS, &mut success);

        if success == 0 {
            // Get error message
            let mut log_length: i32 = 0;
            gl::GetProgramiv(program_id, gl::INFO_LOG_LENGTH, &mut log_length);

            let mut log = vec![0u8; log_length as usize];
            gl::GetProgramInfoLog(
                program_id,
                log_length,
                std::ptr::null_mut(),
                log.as_mut_ptr() as *mut i8,
            );

            let error_msg = String::from_utf8_lossy(&log).to_string();

            // Clean up
            gl::DeleteProgram(program_id);
            gl::DeleteShader(vertex_shader);
            gl::DeleteShader(fragment_shader);

            return Err(GoudError::ShaderLinkFailed(error_msg));
        }

        // Clean up shader objects (they're now linked into the program)
        gl::DeleteShader(vertex_shader);
        gl::DeleteShader(fragment_shader);
    }

    // Allocate handle and store metadata
    let handle = backend.shader_allocator.allocate();
    let metadata = ShaderMetadata {
        gl_id: program_id,
        uniform_locations: HashMap::new(),
    };
    backend.shaders.insert(handle, metadata);

    Ok(handle)
}

/// Destroys a shader program and frees GPU memory.
pub(super) fn destroy_shader(backend: &mut OpenGLBackend, handle: ShaderHandle) -> bool {
    if let Some(metadata) = backend.shaders.remove(&handle) {
        // SAFETY: GL context is active; metadata.gl_id was allocated by CreateProgram and is being cleaned up by its owner.
        unsafe {
            gl::DeleteProgram(metadata.gl_id);
        }

        // Clear bound shader if this was it
        if backend.bound_shader == Some(metadata.gl_id) {
            backend.bound_shader = None;
        }

        backend.shader_allocator.deallocate(handle);
        true
    } else {
        false
    }
}

/// Checks if a shader handle is valid.
pub(super) fn is_shader_valid(backend: &OpenGLBackend, handle: ShaderHandle) -> bool {
    backend.shader_allocator.is_alive(handle) && backend.shaders.contains_key(&handle)
}

/// Binds a shader program for use in subsequent draw calls.
pub(super) fn bind_shader(backend: &mut OpenGLBackend, handle: ShaderHandle) -> GoudResult<()> {
    let metadata = backend
        .shaders
        .get(&handle)
        .ok_or(GoudError::InvalidHandle)?;

    let gl_id = metadata.gl_id;

    // SAFETY: GL context is active; gl_id is a valid linked program obtained from this backend's shader map.
    unsafe {
        gl::UseProgram(gl_id);
    }

    backend.bound_shader = Some(gl_id);

    Ok(())
}

/// Unbinds the currently bound shader program.
pub(super) fn unbind_shader(backend: &mut OpenGLBackend) {
    // SAFETY: GL context is active; passing 0 to UseProgram is the defined way to unbind any shader program.
    unsafe {
        gl::UseProgram(0);
    }

    backend.bound_shader = None;
}

/// Gets the location of a uniform variable in a shader program.
pub(super) fn get_uniform_location(
    backend: &OpenGLBackend,
    handle: ShaderHandle,
    name: &str,
) -> Option<i32> {
    let metadata = backend.shaders.get(&handle)?;

    // Check if we already cached this location
    if let Some(&location) = metadata.uniform_locations.get(name) {
        return if location >= 0 { Some(location) } else { None };
    }

    // Query OpenGL for the location
    let c_name = std::ffi::CString::new(name).ok()?;
    // SAFETY: GL context is active; metadata.gl_id is a valid linked program and c_name is a valid null-terminated C string.
    let location = unsafe { gl::GetUniformLocation(metadata.gl_id, c_name.as_ptr()) };

    // Note: We need mutable access to cache, but this method takes &self.
    // Caching would require interior mutability; for now we query each time.

    if location >= 0 {
        Some(location)
    } else {
        None
    }
}

/// Sets a uniform integer value.
pub(super) fn set_uniform_int(location: i32, value: i32) {
    // SAFETY: GL context is active; location was obtained from GetUniformLocation for the currently bound program.
    unsafe {
        gl::Uniform1i(location, value);
    }
}

/// Sets a uniform float value.
pub(super) fn set_uniform_float(location: i32, value: f32) {
    // SAFETY: GL context is active; location was obtained from GetUniformLocation for the currently bound program.
    unsafe {
        gl::Uniform1f(location, value);
    }
}

/// Sets a uniform vec2 value.
pub(super) fn set_uniform_vec2(location: i32, x: f32, y: f32) {
    // SAFETY: GL context is active; location was obtained from GetUniformLocation for the currently bound program.
    unsafe {
        gl::Uniform2f(location, x, y);
    }
}

/// Sets a uniform vec3 value.
pub(super) fn set_uniform_vec3(location: i32, x: f32, y: f32, z: f32) {
    // SAFETY: GL context is active; location was obtained from GetUniformLocation for the currently bound program.
    unsafe {
        gl::Uniform3f(location, x, y, z);
    }
}

/// Sets a uniform vec4 value.
pub(super) fn set_uniform_vec4(location: i32, x: f32, y: f32, z: f32, w: f32) {
    // SAFETY: GL context is active; location was obtained from GetUniformLocation for the currently bound program.
    unsafe {
        gl::Uniform4f(location, x, y, z, w);
    }
}

/// Sets a uniform mat4 value.
pub(super) fn set_uniform_mat4(location: i32, matrix: &[f32; 16]) {
    // SAFETY: GL context is active; location is valid for the current program and matrix.as_ptr() points to 16 contiguous f32 values.
    unsafe {
        gl::UniformMatrix4fv(location, 1, gl::FALSE, matrix.as_ptr());
    }
}

// ============================================================================
// Internal helpers
// ============================================================================

/// Compiles a single shader stage (vertex, fragment, etc.).
///
/// Returns the OpenGL shader ID on success, or an error with compilation message.
pub(super) fn compile_shader(shader_type: u32, source: &str) -> GoudResult<u32> {
    // SAFETY: GL context is active; shader_type is a valid GL shader stage constant (VERTEX_SHADER, FRAGMENT_SHADER, etc.).
    let shader_id = unsafe { gl::CreateShader(shader_type) };
    if shader_id == 0 {
        return Err(GoudError::ShaderCompilationFailed(
            "Failed to create shader object".to_string(),
        ));
    }

    // Compile the shader
    let c_source = std::ffi::CString::new(source).map_err(|_| {
        GoudError::ShaderCompilationFailed("Shader source contains null byte".to_string())
    })?;

    // SAFETY: GL context is active; shader_id is a valid shader object, c_source is a valid null-terminated string pointer valid for this call.
    unsafe {
        gl::ShaderSource(shader_id, 1, &c_source.as_ptr(), std::ptr::null());
        gl::CompileShader(shader_id);

        // Check compilation status
        let mut success: i32 = 0;
        gl::GetShaderiv(shader_id, gl::COMPILE_STATUS, &mut success);

        if success == 0 {
            // Get error message
            let mut log_length: i32 = 0;
            gl::GetShaderiv(shader_id, gl::INFO_LOG_LENGTH, &mut log_length);

            let mut log = vec![0u8; log_length as usize];
            gl::GetShaderInfoLog(
                shader_id,
                log_length,
                std::ptr::null_mut(),
                log.as_mut_ptr() as *mut i8,
            );

            let error_msg = String::from_utf8_lossy(&log).to_string();

            gl::DeleteShader(shader_id);

            let stage_name = match shader_type {
                gl::VERTEX_SHADER => "Vertex",
                gl::FRAGMENT_SHADER => "Fragment",
                gl::GEOMETRY_SHADER => "Geometry",
                gl::COMPUTE_SHADER => "Compute",
                _ => "Unknown",
            };

            return Err(GoudError::ShaderCompilationFailed(format!(
                "{} shader compilation failed: {}",
                stage_name, error_msg
            )));
        }
    }

    Ok(shader_id)
}
