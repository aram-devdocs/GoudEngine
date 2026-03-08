//! # Renderer Texture FFI
//!
//! Texture loading and destruction operations.

use crate::core::error::{set_last_error, GoudError};
use crate::ffi::context::{GoudContextId, GOUD_INVALID_CONTEXT_ID};
use crate::ffi::window::with_window_state;

// ============================================================================
// Texture Operations
// ============================================================================

/// Opaque texture handle for FFI.
pub type GoudTextureHandle = u64;

/// Invalid texture handle constant.
pub const GOUD_INVALID_TEXTURE: GoudTextureHandle = u64::MAX;

/// Loads a texture from an image file.
///
/// # Arguments
///
/// * `context_id` - The windowed context
/// * `path` - Path to the image file (null-terminated C string)
///
/// # Returns
///
/// A texture handle on success, or `GOUD_INVALID_TEXTURE` on error.
///
/// # Safety
///
/// The `path` pointer must be a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn goud_texture_load(
    context_id: GoudContextId,
    path: *const std::os::raw::c_char,
) -> GoudTextureHandle {
    use std::ffi::CStr;

    if context_id == GOUD_INVALID_CONTEXT_ID || path.is_null() {
        set_last_error(GoudError::InvalidContext);
        return GOUD_INVALID_TEXTURE;
    }

    // SAFETY: caller guarantees path is a valid null-terminated C string
    let path_str = match CStr::from_ptr(path).to_str() {
        Ok(s) => s,
        Err(_) => {
            set_last_error(GoudError::InternalError(
                "Invalid UTF-8 in path".to_string(),
            ));
            return GOUD_INVALID_TEXTURE;
        }
    };

    // Load image data
    let img = match image::open(path_str) {
        Ok(i) => i.to_rgba8(),
        Err(e) => {
            set_last_error(GoudError::ResourceLoadFailed(format!(
                "Failed to load image '{}': {}",
                path_str, e
            )));
            return GOUD_INVALID_TEXTURE;
        }
    };

    let width = img.width();
    let height = img.height();
    let data = img.into_raw();

    // Create GPU texture
    let result = with_window_state(context_id, |state| {
        use crate::libs::graphics::backend::types::{TextureFilter, TextureFormat, TextureWrap};
        use crate::libs::graphics::backend::TextureOps;

        match state.backend_mut().create_texture(
            width,
            height,
            TextureFormat::RGBA8,
            TextureFilter::Linear,
            TextureWrap::ClampToEdge,
            &data,
        ) {
            Ok(handle) => {
                // Pack index and generation into a u64 handle
                ((handle.generation() as u64) << 32) | (handle.index() as u64)
            }
            Err(e) => {
                set_last_error(e);
                GOUD_INVALID_TEXTURE
            }
        }
    });

    result.unwrap_or_else(|| {
        set_last_error(GoudError::InvalidContext);
        GOUD_INVALID_TEXTURE
    })
}

/// Destroys a texture and releases its GPU resources.
///
/// # Arguments
///
/// * `context_id` - The windowed context
/// * `texture` - The texture handle to destroy
///
/// # Returns
///
/// `true` on success, `false` on error.
#[no_mangle]
pub extern "C" fn goud_texture_destroy(
    context_id: GoudContextId,
    texture: GoudTextureHandle,
) -> bool {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return false;
    }
    if texture == GOUD_INVALID_TEXTURE {
        set_last_error(GoudError::InvalidHandle);
        return false;
    }

    with_window_state(context_id, |state| {
        use crate::libs::graphics::backend::types::TextureHandle;
        use crate::libs::graphics::backend::TextureOps;

        // Unpack index and generation from the u64 handle
        let index = (texture & 0xFFFFFFFF) as u32;
        let generation = ((texture >> 32) & 0xFFFFFFFF) as u32;
        let handle = TextureHandle::new(index, generation);

        if state.backend_mut().destroy_texture(handle) {
            true
        } else {
            set_last_error(GoudError::InvalidHandle);
            false
        }
    })
    .unwrap_or(false)
}
