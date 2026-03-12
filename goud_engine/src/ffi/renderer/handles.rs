//! # Renderer Handles and Statistics FFI
//!
//! Opaque handle types for shaders and buffers, rendering statistics.

use crate::core::error::{set_last_error, GoudError};
use crate::core::debugger;
use crate::ffi::context::{GoudContextId, GOUD_INVALID_CONTEXT_ID};

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
/// `out_stats` must be a valid non-null pointer to a `GoudRenderStats`.
#[no_mangle]
pub unsafe extern "C" fn goud_renderer_get_stats(
    context_id: GoudContextId,
    out_stats: *mut GoudRenderStats,
) -> bool {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return false;
    }
    if out_stats.is_null() {
        set_last_error(GoudError::InvalidState(
            "output pointer is null".to_string(),
        ));
        return false;
    }

    let stats = debugger::snapshot_for_context(context_id)
        .map(|snapshot| GoudRenderStats {
            draw_calls: snapshot.stats.render.draw_calls,
            triangles: snapshot.stats.render.triangles,
            texture_binds: snapshot.stats.render.texture_binds,
            shader_binds: snapshot.stats.render.shader_binds,
        })
        .unwrap_or_default();

    // SAFETY: caller guarantees out_stats is a valid non-null pointer.
    *out_stats = stats;
    true
}
