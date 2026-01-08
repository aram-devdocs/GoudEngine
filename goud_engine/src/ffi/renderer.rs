//! # FFI Renderer Module
//!
//! This module provides C-compatible functions for rendering operations.
//! It integrates with the window FFI to provide basic 2D rendering capabilities.
//!
//! ## Design
//!
//! The renderer FFI provides two modes of operation:
//!
//! 1. **Immediate mode**: Draw individual sprites/quads with explicit parameters
//! 2. **ECS mode**: Automatically render all entities with Sprite + Transform2D components
//!
//! ## Example Usage (C#)
//!
//! ```csharp
//! // In game loop
//! while (!goud_window_should_close(contextId)) {
//!     float deltaTime = goud_window_poll_events(contextId);
//!     
//!     // Clear screen
//!     goud_window_clear(contextId, 0.1f, 0.1f, 0.2f, 1.0f);
//!     
//!     // Begin rendering
//!     goud_renderer_begin(contextId);
//!     
//!     // Draw sprites (immediate mode)
//!     goud_renderer_draw_quad(contextId, x, y, width, height, r, g, b, a);
//!     
//!     // Or draw all ECS sprites
//!     goud_renderer_draw_ecs_sprites(contextId);
//!     
//!     // End rendering
//!     goud_renderer_end(contextId);
//!     
//!     goud_window_swap_buffers(contextId);
//! }
//! ```

use crate::core::error::{set_last_error, GoudError};
use crate::ffi::context::{GoudContextId, GOUD_INVALID_CONTEXT_ID};
use crate::ffi::window::with_window_state;
use crate::libs::graphics::backend::RenderBackend;

// ============================================================================
// Renderer State
// ============================================================================

// Tracks whether we're currently in a rendering frame.
thread_local! {
    static RENDER_ACTIVE: std::cell::RefCell<bool> = const { std::cell::RefCell::new(false) };
}

// ============================================================================
// FFI Functions
// ============================================================================

/// Begins a new rendering frame.
///
/// This must be called before any drawing operations and before `goud_renderer_end`.
///
/// # Arguments
///
/// * `context_id` - The windowed context
///
/// # Returns
///
/// `true` on success, `false` on error.
#[no_mangle]
pub extern "C" fn goud_renderer_begin(context_id: GoudContextId) -> bool {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return false;
    }

    // Mark rendering as active
    RENDER_ACTIVE.with(|cell| {
        *cell.borrow_mut() = true;
    });

    // Begin frame on the backend
    with_window_state(context_id, |state| {
        if let Err(e) = state.backend_mut().begin_frame() {
            set_last_error(e);
            return false;
        }
        true
    })
    .unwrap_or_else(|| {
        set_last_error(GoudError::InvalidContext);
        false
    })
}

/// Ends the current rendering frame.
///
/// This must be called after all drawing operations and before `goud_window_swap_buffers`.
///
/// # Arguments
///
/// * `context_id` - The windowed context
///
/// # Returns
///
/// `true` on success, `false` on error.
#[no_mangle]
pub extern "C" fn goud_renderer_end(context_id: GoudContextId) -> bool {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return false;
    }

    // Mark rendering as inactive
    RENDER_ACTIVE.with(|cell| {
        *cell.borrow_mut() = false;
    });

    // End frame on the backend
    with_window_state(context_id, |state| {
        if let Err(e) = state.backend_mut().end_frame() {
            set_last_error(e);
            return false;
        }
        true
    })
    .unwrap_or_else(|| {
        set_last_error(GoudError::InvalidContext);
        false
    })
}

/// Sets the viewport for rendering.
///
/// # Arguments
///
/// * `context_id` - The windowed context
/// * `x` - Viewport X position
/// * `y` - Viewport Y position
/// * `width` - Viewport width
/// * `height` - Viewport height
#[no_mangle]
pub extern "C" fn goud_renderer_set_viewport(
    context_id: GoudContextId,
    x: i32,
    y: i32,
    width: u32,
    height: u32,
) {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        return;
    }

    with_window_state(context_id, |state| {
        state.backend_mut().set_viewport(x, y, width, height);
    });
}

/// Enables alpha blending for transparent sprites.
///
/// # Arguments
///
/// * `context_id` - The windowed context
#[no_mangle]
pub extern "C" fn goud_renderer_enable_blending(context_id: GoudContextId) {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        return;
    }

    with_window_state(context_id, |state| {
        state.backend_mut().enable_blending();
    });
}

/// Disables alpha blending.
///
/// # Arguments
///
/// * `context_id` - The windowed context
#[no_mangle]
pub extern "C" fn goud_renderer_disable_blending(context_id: GoudContextId) {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        return;
    }

    with_window_state(context_id, |state| {
        state.backend_mut().disable_blending();
    });
}

/// Enables depth testing.
///
/// # Arguments
///
/// * `context_id` - The windowed context
#[no_mangle]
pub extern "C" fn goud_renderer_enable_depth_test(context_id: GoudContextId) {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        return;
    }

    with_window_state(context_id, |state| {
        state.backend_mut().enable_depth_test();
    });
}

/// Disables depth testing.
///
/// # Arguments
///
/// * `context_id` - The windowed context
#[no_mangle]
pub extern "C" fn goud_renderer_disable_depth_test(context_id: GoudContextId) {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        return;
    }

    with_window_state(context_id, |state| {
        state.backend_mut().disable_depth_test();
    });
}

/// Clears the depth buffer.
///
/// # Arguments
///
/// * `context_id` - The windowed context
#[no_mangle]
pub extern "C" fn goud_renderer_clear_depth(context_id: GoudContextId) {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        return;
    }

    with_window_state(context_id, |state| {
        state.backend_mut().clear_depth();
    });
}

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

    result.unwrap_or(GOUD_INVALID_TEXTURE)
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
    if context_id == GOUD_INVALID_CONTEXT_ID || texture == GOUD_INVALID_TEXTURE {
        return false;
    }

    with_window_state(context_id, |state| {
        use crate::libs::graphics::backend::types::TextureHandle;

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

// ============================================================================
// Shader Operations (placeholder for future expansion)
// ============================================================================

/// Opaque shader handle for FFI.
pub type GoudShaderHandle = u64;

/// Invalid shader handle constant.
pub const GOUD_INVALID_SHADER: GoudShaderHandle = u64::MAX;

// ============================================================================
// Buffer Operations (placeholder for future expansion)
// ============================================================================

/// Opaque buffer handle for FFI.
pub type GoudBufferHandle = u64;

/// Invalid buffer handle constant.
pub const GOUD_INVALID_BUFFER: GoudBufferHandle = u64::MAX;

// ============================================================================
// Rendering Statistics
// ============================================================================

/// FFI-safe rendering statistics.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct GoudRenderStats {
    /// Number of draw calls this frame
    pub draw_calls: u32,
    /// Number of triangles rendered
    pub triangles: u32,
    /// Number of texture binds
    pub texture_binds: u32,
    /// Number of shader binds  
    pub shader_binds: u32,
}

/// Gets rendering statistics for the current frame.
///
/// # Arguments
///
/// * `context_id` - The windowed context
/// * `out_stats` - Pointer to store the statistics
///
/// # Returns
///
/// `true` on success, `false` on error.
///
/// # Safety
///
/// `out_stats` must be a valid pointer.
#[no_mangle]
pub unsafe extern "C" fn goud_renderer_get_stats(
    context_id: GoudContextId,
    out_stats: *mut GoudRenderStats,
) -> bool {
    if context_id == GOUD_INVALID_CONTEXT_ID || out_stats.is_null() {
        return false;
    }

    // For now, return empty stats (will be populated when we have a proper stats tracking)
    *out_stats = GoudRenderStats::default();
    true
}
