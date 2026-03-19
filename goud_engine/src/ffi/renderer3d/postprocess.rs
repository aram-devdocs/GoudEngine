//! FFI functions for the post-processing pipeline.

use super::state::{ensure_renderer3d_state, with_renderer};
use crate::core::error::{set_last_error, GoudError};
use crate::ffi::context::{GoudContextId, GOUD_INVALID_CONTEXT_ID};
use crate::libs::graphics::renderer3d::{BloomPass, ColorGradePass, GaussianBlurPass};

/// Adds a bloom pass to the post-processing pipeline. Returns the pass index.
#[no_mangle]
pub extern "C" fn goud_renderer3d_add_bloom_pass(
    context_id: GoudContextId,
    threshold: f32,
    intensity: f32,
) -> i32 {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return -1;
    }

    if let Err(e) = ensure_renderer3d_state(context_id) {
        set_last_error(e);
        return -1;
    }

    with_renderer(context_id, |renderer| {
        let pipeline = renderer.postprocess_pipeline_mut();
        let index = pipeline.pass_count() as i32;
        pipeline.add_pass(Box::new(BloomPass {
            threshold,
            intensity,
            enabled: true,
        }));
        index
    })
    .unwrap_or(-1)
}

/// Adds a Gaussian blur pass to the post-processing pipeline. Returns the pass index.
#[no_mangle]
pub extern "C" fn goud_renderer3d_add_blur_pass(context_id: GoudContextId, radius: u32) -> i32 {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return -1;
    }

    if let Err(e) = ensure_renderer3d_state(context_id) {
        set_last_error(e);
        return -1;
    }

    with_renderer(context_id, |renderer| {
        let pipeline = renderer.postprocess_pipeline_mut();
        let index = pipeline.pass_count() as i32;
        pipeline.add_pass(Box::new(GaussianBlurPass {
            radius,
            enabled: true,
        }));
        index
    })
    .unwrap_or(-1)
}

/// Adds a color grading pass to the post-processing pipeline. Returns the pass index.
#[no_mangle]
pub extern "C" fn goud_renderer3d_add_color_grade_pass(
    context_id: GoudContextId,
    exposure: f32,
    contrast: f32,
    saturation: f32,
) -> i32 {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return -1;
    }

    if let Err(e) = ensure_renderer3d_state(context_id) {
        set_last_error(e);
        return -1;
    }

    with_renderer(context_id, |renderer| {
        let pipeline = renderer.postprocess_pipeline_mut();
        let index = pipeline.pass_count() as i32;
        pipeline.add_pass(Box::new(ColorGradePass {
            exposure,
            contrast,
            saturation,
            enabled: true,
        }));
        index
    })
    .unwrap_or(-1)
}

/// Removes a post-processing pass by index.
#[no_mangle]
pub extern "C" fn goud_renderer3d_remove_postprocess_pass(
    context_id: GoudContextId,
    index: u32,
) -> bool {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return false;
    }

    with_renderer(context_id, |renderer| {
        renderer
            .postprocess_pipeline_mut()
            .remove_pass(index as usize)
    })
    .unwrap_or(false)
}

/// Returns the number of post-processing passes.
#[no_mangle]
pub extern "C" fn goud_renderer3d_postprocess_pass_count(context_id: GoudContextId) -> u32 {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        return 0;
    }

    with_renderer(context_id, |renderer| {
        renderer.postprocess_pipeline().pass_count() as u32
    })
    .unwrap_or(0)
}
