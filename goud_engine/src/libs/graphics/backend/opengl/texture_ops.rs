//! OpenGL texture create/update/destroy/bind operations.

use super::{
    backend::OpenGLBackend,
    conversions::{
        bytes_per_pixel, texture_filter_to_gl, texture_format_to_gl, texture_wrap_to_gl,
    },
    gl_check_debug, TextureMetadata,
};
use crate::libs::error::{GoudError, GoudResult};
use crate::libs::graphics::backend::types::{
    TextureFilter, TextureFormat, TextureHandle, TextureWrap,
};

/// Creates a GPU texture with the specified dimensions, format, and initial data.
pub(super) fn create_texture(
    backend: &mut OpenGLBackend,
    width: u32,
    height: u32,
    format: TextureFormat,
    filter: TextureFilter,
    wrap: TextureWrap,
    data: &[u8],
) -> GoudResult<TextureHandle> {
    if width == 0 || height == 0 {
        return Err(GoudError::TextureCreationFailed(
            "Texture dimensions must be greater than 0".to_string(),
        ));
    }

    let mut gl_id: u32 = 0;
    // SAFETY: Valid GL context is guaranteed by the renderer initialization.
    // gl_id is a valid stack-allocated output variable for GenTextures.
    unsafe {
        gl::GenTextures(1, &mut gl_id);
        if gl_id == 0 {
            return Err(GoudError::TextureCreationFailed(
                "Failed to generate OpenGL texture".to_string(),
            ));
        }
    }

    let (internal_format, pixel_format, pixel_type) = texture_format_to_gl(format);
    let filter_gl = texture_filter_to_gl(filter);
    let wrap_gl = texture_wrap_to_gl(wrap);

    // SAFETY: Valid GL context is guaranteed by the renderer initialization.
    // gl_id is a live texture object. Format, filter, and wrap enums are valid GL values
    // from conversion functions. data_ptr is either null (for empty data) or a valid pointer
    // to the pixel data slice.
    unsafe {
        gl::BindTexture(gl::TEXTURE_2D, gl_id);

        // Upload texture data
        let data_ptr = if data.is_empty() {
            std::ptr::null()
        } else {
            data.as_ptr() as *const std::ffi::c_void
        };

        gl::TexImage2D(
            gl::TEXTURE_2D,
            0, // mip level
            internal_format as i32,
            width as i32,
            height as i32,
            0, // border (must be 0)
            pixel_format,
            pixel_type,
            data_ptr,
        );

        // Set texture parameters
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, filter_gl as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, filter_gl as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, wrap_gl as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, wrap_gl as i32);

        // Check for errors
        let error = gl::GetError();
        if error != gl::NO_ERROR {
            gl::DeleteTextures(1, &gl_id);
            return Err(GoudError::TextureCreationFailed(format!(
                "OpenGL error during texture creation: 0x{:X}",
                error
            )));
        }

        // Unbind
        gl::BindTexture(gl::TEXTURE_2D, 0);
    }

    // Allocate handle and store metadata
    let handle = backend.texture_allocator.allocate();
    let metadata = TextureMetadata {
        gl_id,
        width,
        height,
        format,
        _filter: filter,
        _wrap: wrap,
    };
    backend.textures.insert(handle, metadata);

    // Clear all bound texture tracking (texture is now unbound)
    for unit in 0..backend.bound_textures.len() {
        if backend.bound_textures[unit] == Some(gl_id) {
            backend.bound_textures[unit] = None;
        }
    }

    Ok(handle)
}

/// Updates a region of an existing texture with new pixel data.
pub(super) fn update_texture(
    backend: &mut OpenGLBackend,
    handle: TextureHandle,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    data: &[u8],
) -> GoudResult<()> {
    let metadata = backend
        .textures
        .get(&handle)
        .ok_or(GoudError::InvalidHandle)?;

    // Validate region bounds
    if x + width > metadata.width || y + height > metadata.height {
        return Err(GoudError::TextureCreationFailed(format!(
            "Update region ({}x{} at {},{}) exceeds texture bounds ({}x{})",
            width, height, x, y, metadata.width, metadata.height
        )));
    }

    // Validate data size
    let expected_size = (width * height) as usize * bytes_per_pixel(metadata.format);
    if data.len() != expected_size {
        return Err(GoudError::TextureCreationFailed(format!(
            "Data size mismatch: expected {} bytes, got {}",
            expected_size,
            data.len()
        )));
    }

    let (_, pixel_format, pixel_type) = texture_format_to_gl(metadata.format);
    let gl_id = metadata.gl_id;

    // SAFETY: Valid GL context is guaranteed by the renderer initialization.
    // gl_id is a live texture object from the backend. Region bounds and data size
    // are validated above. Format enums come from the stored metadata.
    unsafe {
        // Bind texture
        gl::BindTexture(gl::TEXTURE_2D, gl_id);

        // Upload sub-image
        gl::TexSubImage2D(
            gl::TEXTURE_2D,
            0, // mip level
            x as i32,
            y as i32,
            width as i32,
            height as i32,
            pixel_format,
            pixel_type,
            data.as_ptr() as *const std::ffi::c_void,
        );

        // Check for errors
        let error = gl::GetError();
        if error != gl::NO_ERROR {
            return Err(GoudError::TextureCreationFailed(format!(
                "OpenGL error during texture update: 0x{:X}",
                error
            )));
        }

        // Unbind
        gl::BindTexture(gl::TEXTURE_2D, 0);
    }

    Ok(())
}

/// Destroys a texture and frees GPU memory.
pub(super) fn destroy_texture(backend: &mut OpenGLBackend, handle: TextureHandle) -> bool {
    if let Some(metadata) = backend.textures.remove(&handle) {
        // SAFETY: metadata.gl_id is a valid GL texture name created by create_texture.
        // Deleting it releases GPU memory. The handle is removed from the map above.
        unsafe {
            gl::DeleteTextures(1, &metadata.gl_id);
        }

        // Clear from bound texture tracking
        for unit in 0..backend.bound_textures.len() {
            if backend.bound_textures[unit] == Some(metadata.gl_id) {
                backend.bound_textures[unit] = None;
            }
        }

        backend.texture_allocator.deallocate(handle);
        true
    } else {
        false
    }
}

/// Checks if a texture handle is valid.
pub(super) fn is_texture_valid(backend: &OpenGLBackend, handle: TextureHandle) -> bool {
    backend.texture_allocator.is_alive(handle) && backend.textures.contains_key(&handle)
}

/// Returns the dimensions of a texture.
pub(super) fn texture_size(backend: &OpenGLBackend, handle: TextureHandle) -> Option<(u32, u32)> {
    backend
        .textures
        .get(&handle)
        .map(|meta| (meta.width, meta.height))
}

/// Binds a texture to a texture unit for use in subsequent draw calls.
pub(super) fn bind_texture(
    backend: &mut OpenGLBackend,
    handle: TextureHandle,
    unit: u32,
) -> GoudResult<()> {
    let metadata = backend
        .textures
        .get(&handle)
        .ok_or(GoudError::InvalidHandle)?;

    // Validate texture unit
    if unit >= backend.bound_textures.len() as u32 {
        return Err(GoudError::TextureCreationFailed(format!(
            "Texture unit {} exceeds maximum supported units ({})",
            unit,
            backend.bound_textures.len()
        )));
    }

    let gl_id = metadata.gl_id;

    // SAFETY: unit is validated to be within bounds above; TEXTURE0 + unit selects a valid texture unit.
    // gl_id is a live GL texture object. TEXTURE_2D is a valid texture target.
    unsafe {
        gl::ActiveTexture(gl::TEXTURE0 + unit);
        gl::BindTexture(gl::TEXTURE_2D, gl_id);
    }
    gl_check_debug!("bind_texture");

    set_bound_texture(backend, unit, Some(gl_id));

    Ok(())
}

/// Unbinds any texture from the specified texture unit.
pub(super) fn unbind_texture(backend: &mut OpenGLBackend, unit: u32) {
    if unit < backend.bound_textures.len() as u32 {
        // SAFETY: unit is validated to be within bounds; TEXTURE0 + unit selects a valid texture unit.
        // Passing 0 unbinds any texture on the TEXTURE_2D target for that unit.
        unsafe {
            gl::ActiveTexture(gl::TEXTURE0 + unit);
            gl::BindTexture(gl::TEXTURE_2D, 0);
        }
        gl_check_debug!("unbind_texture");

        set_bound_texture(backend, unit, None);
    }
}

// ============================================================================
// Internal bound-texture tracking helpers
// ============================================================================

fn set_bound_texture(backend: &mut OpenGLBackend, unit: u32, gl_id: Option<u32>) {
    if let Some(slot) = backend.bound_textures.get_mut(unit as usize) {
        *slot = gl_id;
    }
}
