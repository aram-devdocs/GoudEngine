use crate::libs::graphics::backend::opengl::{gl_check_debug, OpenGLBackend};

pub(super) fn read_default_framebuffer_rgba8(
    _backend: &mut OpenGLBackend,
    width: u32,
    height: u32,
) -> Result<Vec<u8>, String> {
    read_default_framebuffer_rgba8_standalone(width, height)
}

/// Reads the default framebuffer as RGBA8 bytes without needing a backend reference.
///
/// Requires a current OpenGL context on the calling thread.
pub(super) fn read_default_framebuffer_rgba8_standalone(
    width: u32,
    height: u32,
) -> Result<Vec<u8>, String> {
    if width == 0 || height == 0 {
        return Ok(Vec::new());
    }
    if width > i32::MAX as u32 || height > i32::MAX as u32 {
        return Err("framebuffer dimensions exceed OpenGL i32 limits".to_string());
    }

    let row_bytes = (width as usize)
        .checked_mul(4)
        .ok_or_else(|| "framebuffer row byte size overflow".to_string())?;
    let total_bytes = row_bytes
        .checked_mul(height as usize)
        .ok_or_else(|| "framebuffer byte size overflow".to_string())?;
    let mut pixels = vec![0u8; total_bytes];

    let mut previous_read_framebuffer = 0i32;
    let mut previous_read_buffer = 0i32;
    let mut previous_pack_alignment = 0i32;

    // SAFETY: The caller owns the active OpenGL context for this backend.
    // Buffer pointers are valid for `total_bytes` and OpenGL state restored
    // before returning.
    unsafe {
        gl::GetIntegerv(gl::READ_FRAMEBUFFER_BINDING, &mut previous_read_framebuffer);
        gl::GetIntegerv(gl::READ_BUFFER, &mut previous_read_buffer);
        gl::GetIntegerv(gl::PACK_ALIGNMENT, &mut previous_pack_alignment);

        gl::BindFramebuffer(gl::READ_FRAMEBUFFER, 0);
        gl::ReadBuffer(gl::BACK);
        gl::PixelStorei(gl::PACK_ALIGNMENT, 1);
        gl::ReadPixels(
            0,
            0,
            width as i32,
            height as i32,
            gl::RGBA,
            gl::UNSIGNED_BYTE,
            pixels.as_mut_ptr() as *mut std::ffi::c_void,
        );

        gl::PixelStorei(gl::PACK_ALIGNMENT, previous_pack_alignment);
        gl::ReadBuffer(previous_read_buffer as u32);
        gl::BindFramebuffer(gl::READ_FRAMEBUFFER, previous_read_framebuffer as u32);
    }
    gl_check_debug!("read_default_framebuffer_rgba8");

    // OpenGL returns bottom-left origin; flip rows for top-left origin.
    let mut scratch_row = vec![0u8; row_bytes];
    for y in 0..(height as usize / 2) {
        let top_start = y * row_bytes;
        let bottom_start = (height as usize - 1 - y) * row_bytes;

        scratch_row.copy_from_slice(&pixels[top_start..top_start + row_bytes]);
        pixels.copy_within(bottom_start..bottom_start + row_bytes, top_start);
        pixels[bottom_start..bottom_start + row_bytes].copy_from_slice(&scratch_row);
    }

    Ok(pixels)
}
