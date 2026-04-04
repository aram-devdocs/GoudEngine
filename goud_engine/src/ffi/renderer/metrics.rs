//! Render metrics FFI functions.
//!
//! Provides a C-compatible function to retrieve per-frame render metrics
//! from the debugger snapshot for a given context.

use crate::core::error::{set_last_error, GoudError};
use crate::ffi::context::{GoudContextId, GOUD_INVALID_CONTEXT_ID};
use crate::ffi::types::{FfiFramePhaseTimings, FfiRenderMetrics};

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

/// Retrieves per-frame phase timings for performance diagnosis.
///
/// Reads the latest frame phase timings from the thread-local cache.
/// These timings are always available (no debugger required).
///
/// # Safety
///
/// `out_timings` must point to writable storage for one [`FfiFramePhaseTimings`].
#[no_mangle]
pub unsafe extern "C" fn goud_renderer_get_frame_phase_timings(
    out_timings: *mut FfiFramePhaseTimings,
) -> i32 {
    if out_timings.is_null() {
        set_last_error(GoudError::InvalidState(
            "out_timings pointer is null".to_string(),
        ));
        return -1;
    }

    let timings = crate::libs::graphics::frame_timing::latest_timings();
    // SAFETY: out_timings is non-null and points to writable storage for one FfiFramePhaseTimings.
    *out_timings = FfiFramePhaseTimings {
        surface_acquire_us: timings.surface_acquire_us,
        shadow_pass_us: timings.shadow_pass_us,
        shadow_build_us: timings.shadow_build_us,
        render3d_scene_us: timings.render3d_scene_us,
        uniform_upload_us: timings.uniform_upload_us,
        render_pass_us: timings.render_pass_us,
        gpu_submit_us: timings.gpu_submit_us,
        readback_stall_us: timings.readback_stall_us,
        surface_present_us: timings.surface_present_us,
        anim_eval_us: timings.anim_eval_us,
        bone_pack_us: timings.bone_pack_us,
        bone_upload_us: timings.bone_upload_us,
    };
    0
}
