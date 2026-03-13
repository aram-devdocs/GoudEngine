//! OpenGL shader compile/link/destroy/bind operations and uniform setters.

use super::{backend::OpenGLBackend, gl_check_debug, ShaderMetadata};
use crate::libs::error::{GoudError, GoudResult};
use crate::libs::graphics::backend::types::ShaderHandle;
use std::collections::HashMap;
use std::sync::Mutex;

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
            // SAFETY: vertex_shader is a valid shader ID returned by compile_shader above.
            // Cleaning up on error to prevent GPU resource leak.
            unsafe {
                gl::DeleteShader(vertex_shader);
            }
            return Err(e);
        }
    };

    // SAFETY: Valid GL context is guaranteed by the renderer initialization.
    // CreateProgram returns 0 on failure, which is checked below.
    let program_id = unsafe { gl::CreateProgram() };
    if program_id == 0 {
        // SAFETY: vertex_shader and fragment_shader are valid shader IDs from compile_shader.
        // Cleaning up on error to prevent GPU resource leak.
        unsafe {
            gl::DeleteShader(vertex_shader);
            gl::DeleteShader(fragment_shader);
        }
        return Err(GoudError::ShaderLinkFailed(
            "Failed to create shader program".to_string(),
        ));
    }

    // SAFETY: program_id is a valid non-zero program ID from CreateProgram.
    // vertex_shader and fragment_shader are valid compiled shader IDs.
    // Link status is checked after LinkProgram; shaders are cleaned up in all paths.
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
        uniform_locations: Mutex::new(HashMap::new()),
    };
    backend.shaders.insert(handle, metadata);

    Ok(handle)
}

/// Destroys a shader program and frees GPU memory.
pub(super) fn destroy_shader(backend: &mut OpenGLBackend, handle: ShaderHandle) -> bool {
    if let Some(metadata) = backend.shaders.remove(&handle) {
        // SAFETY: metadata.gl_id is a valid linked program ID created by create_shader.
        // Deleting it releases GPU resources. The handle is removed from the map above.
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

    // SAFETY: gl_id is a valid linked program ID stored by `create_shader`.
    unsafe {
        gl::UseProgram(gl_id);
    }
    gl_check_debug!("UseProgram");

    backend.bound_shader = Some(gl_id);

    Ok(())
}

/// Unbinds the currently bound shader program.
pub(super) fn unbind_shader(backend: &mut OpenGLBackend) {
    // SAFETY: Passing 0 to UseProgram unbinds any currently bound program.
    unsafe {
        gl::UseProgram(0);
    }
    gl_check_debug!("UseProgram(0)");

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
    {
        let cache = metadata.uniform_locations.lock().unwrap();
        if let Some(&location) = cache.get(name) {
            return if location >= 0 { Some(location) } else { None };
        }
    } // lock dropped here

    // Query OpenGL for the location
    let c_name = std::ffi::CString::new(name).ok()?;
    // SAFETY: `metadata.gl_id` is a valid linked program ID stored by `create_shader`.
    // OpenGL contexts are single-threaded, so this call is safe from any code path
    // that holds a reference to `backend`.
    let location = unsafe { gl::GetUniformLocation(metadata.gl_id, c_name.as_ptr()) };

    // Cache the result (including negative values so we don't query again for missing uniforms)
    metadata
        .uniform_locations
        .lock()
        .unwrap()
        .insert(name.to_string(), location);

    if location >= 0 {
        Some(location)
    } else {
        None
    }
}

/// Sets a uniform integer value.
pub(super) fn set_uniform_int(location: i32, value: i32) {
    // SAFETY: location is a valid uniform location obtained from get_uniform_location;
    // a shader program must be bound before calling this.
    unsafe {
        gl::Uniform1i(location, value);
    }
    gl_check_debug!("Uniform1i");
}

/// Sets a uniform float value.
pub(super) fn set_uniform_float(location: i32, value: f32) {
    // SAFETY: location is a valid uniform location obtained from get_uniform_location;
    // a shader program must be bound before calling this.
    unsafe {
        gl::Uniform1f(location, value);
    }
    gl_check_debug!("Uniform1f");
}

/// Sets a uniform vec2 value.
pub(super) fn set_uniform_vec2(location: i32, x: f32, y: f32) {
    // SAFETY: location is a valid uniform location obtained from get_uniform_location;
    // a shader program must be bound before calling this.
    unsafe {
        gl::Uniform2f(location, x, y);
    }
    gl_check_debug!("Uniform2f");
}

/// Sets a uniform vec3 value.
pub(super) fn set_uniform_vec3(location: i32, x: f32, y: f32, z: f32) {
    // SAFETY: location is a valid uniform location obtained from get_uniform_location;
    // a shader program must be bound before calling this.
    unsafe {
        gl::Uniform3f(location, x, y, z);
    }
    gl_check_debug!("Uniform3f");
}

/// Sets a uniform vec4 value.
pub(super) fn set_uniform_vec4(location: i32, x: f32, y: f32, z: f32, w: f32) {
    // SAFETY: location is a valid uniform location obtained from get_uniform_location;
    // a shader program must be bound before calling this.
    unsafe {
        gl::Uniform4f(location, x, y, z, w);
    }
    gl_check_debug!("Uniform4f");
}

/// Sets a uniform mat4 value.
pub(super) fn set_uniform_mat4(location: i32, matrix: &[f32; 16]) {
    // SAFETY: location is a valid uniform location; matrix.as_ptr() points to 16 contiguous
    // f32 values which is exactly what UniformMatrix4fv expects for count=1.
    unsafe {
        gl::UniformMatrix4fv(location, 1, gl::FALSE, matrix.as_ptr());
    }
    gl_check_debug!("UniformMatrix4fv");
}

// ============================================================================
// Internal helpers
// ============================================================================

/// Compiles a single shader stage (vertex, fragment, etc.).
///
/// Returns the OpenGL shader ID on success, or an error with compilation message.
pub(super) fn compile_shader(shader_type: u32, source: &str) -> GoudResult<u32> {
    // SAFETY: Valid GL context is guaranteed by the renderer initialization.
    // shader_type is a valid GL shader type enum (VERTEX_SHADER, FRAGMENT_SHADER, etc.).
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

    // SAFETY: shader_id is a valid non-zero shader ID from CreateShader above.
    // c_source is a valid CString; its pointer remains valid for this block.
    // Compilation status is checked and errors reported with the info log.
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
