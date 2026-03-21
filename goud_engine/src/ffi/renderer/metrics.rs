//! Render metrics FFI functions.
//!
//! Provides a C-compatible function to retrieve per-frame render metrics
//! from the debugger snapshot for a given context.

use crate::core::error::{set_last_error, GoudError};
use crate::ffi::context::{GoudContextId, GOUD_INVALID_CONTEXT_ID};

/// FFI-safe per-frame render metrics.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct FfiRenderMetrics {
    /// Total draw calls across all render subsystems.
    pub draw_call_count: u32,
    /// Total sprites submitted before culling.
    pub sprites_submitted: u32,
    /// Sprites that passed culling and were drawn.
    pub sprites_drawn: u32,
    /// Sprites rejected by frustum culling.
    pub sprites_culled: u32,
    /// Number of sprite batches submitted.
    pub batches_submitted: u32,
    /// Average sprites per batch (batch efficiency).
    pub avg_sprites_per_batch: f32,
    /// Time spent rendering sprites (ms).
    pub sprite_render_ms: f32,
    /// Time spent rendering text (ms).
    pub text_render_ms: f32,
    /// Time spent rendering UI (ms).
    pub ui_render_ms: f32,
    /// Total render phase time (ms).
    pub total_render_ms: f32,
    /// Draw calls from text rendering.
    pub text_draw_calls: u32,
    /// Glyphs rendered this frame.
    pub text_glyph_count: u32,
    /// Draw calls from UI rendering.
    pub ui_draw_calls: u32,
}

/// Retrieves per-frame render metrics for a context.
///
/// Reads the render metrics from the debugger snapshot for the given context.
/// If no render metrics are available (e.g., debugger not attached or no
/// frames rendered), a zeroed struct is returned.
///
/// # Arguments
///
/// * `context_id` - The context to query.
/// * `out_metrics` - Pointer to caller-allocated storage for one `FfiRenderMetrics`.
///
/// # Returns
///
/// 0 on success, negative error code on failure.
///
/// # Safety
///
/// `out_metrics` must point to writable storage for one [`FfiRenderMetrics`].
#[no_mangle]
pub unsafe extern "C" fn goud_renderer_get_frame_metrics(
    context_id: GoudContextId,
    out_metrics: *mut FfiRenderMetrics,
) -> i32 {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return -1;
    }

    if out_metrics.is_null() {
        set_last_error(GoudError::InvalidState(
            "out_metrics pointer is null".to_string(),
        ));
        return -2;
    }

    // Try to read render metrics from the debugger snapshot.
    let route_id = crate::core::debugger::route_for_context(context_id);
    let metrics = route_id.and_then(|rid| {
        crate::core::debugger::snapshot_for_route(&rid)
            .map(|snapshot| snapshot.stats.render_metrics)
    });

    let ffi_metrics = match metrics {
        Some(rm) => FfiRenderMetrics {
            draw_call_count: rm.draw_call_count,
            sprites_submitted: rm.sprites_submitted,
            sprites_drawn: rm.sprites_drawn,
            sprites_culled: rm.sprites_culled,
            batches_submitted: rm.batches_submitted,
            avg_sprites_per_batch: rm.avg_sprites_per_batch,
            sprite_render_ms: rm.sprite_render_ms,
            text_render_ms: rm.text_render_ms,
            ui_render_ms: rm.ui_render_ms,
            total_render_ms: rm.total_render_ms,
            text_draw_calls: rm.text_draw_calls,
            text_glyph_count: rm.text_glyph_count,
            ui_draw_calls: rm.ui_draw_calls,
        },
        None => FfiRenderMetrics::default(),
    };

    // SAFETY: out_metrics is non-null and points to writable storage for one FfiRenderMetrics.
    *out_metrics = ffi_metrics;
    0
}
