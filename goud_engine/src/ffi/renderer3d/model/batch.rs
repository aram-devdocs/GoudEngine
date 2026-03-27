//! Batch FFI helpers for bulk model operations.

use super::super::state::with_renderer;
use super::GOUD_INVALID_MODEL;
use crate::core::error::{set_last_error, GoudError};
use crate::ffi::context::{GoudContextId, GOUD_INVALID_CONTEXT_ID};

/// Creates multiple independent instances of a source model in one call.
///
/// The caller must provide a buffer `out_ids` of at least `count` elements
/// where the new instance handles will be written.
///
/// # Returns
/// The number of successfully created instances (equal to `count` on full
/// success), or `-1` on error (null pointer or invalid context).
///
/// # Safety
/// * `out_ids` must point to a caller-allocated buffer of at least `count`
///   `u32` elements.
#[no_mangle]
pub unsafe extern "C" fn goud_renderer3d_instantiate_model_batch(
    context_id: GoudContextId,
    source_model_id: u32,
    count: u32,
    out_ids: *mut u32,
) -> i32 {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return -1;
    }

    if out_ids.is_null() {
        set_last_error(GoudError::InvalidHandle);
        return -1;
    }

    if count == 0 {
        return 0;
    }

    // SAFETY: caller guarantees `out_ids` points to a buffer of at least
    // `count` u32 elements.
    let out_slice = std::slice::from_raw_parts_mut(out_ids, count as usize);

    with_renderer(context_id, |renderer| {
        let mut created: i32 = 0;
        for slot in out_slice.iter_mut() {
            let id = renderer
                .instantiate_model(source_model_id)
                .unwrap_or(GOUD_INVALID_MODEL);
            *slot = id;
            if id != GOUD_INVALID_MODEL {
                created += 1;
            }
        }
        created
    })
    .unwrap_or(-1)
}

/// Sets positions for multiple models/instances in one call.
///
/// `positions` is a flat `f32` array laid out as
/// `[x0, y0, z0, x1, y1, z1, ...]` with `count * 3` elements total.
///
/// # Returns
/// The number of models whose position was successfully set, or `-1` on
/// error (null pointer or invalid context).
///
/// # Safety
/// * `model_ids` must point to at least `count` `u32` elements.
/// * `positions` must point to at least `count * 3` `f32` elements.
#[no_mangle]
pub unsafe extern "C" fn goud_renderer3d_set_model_positions_batch(
    context_id: GoudContextId,
    model_ids: *const u32,
    positions: *const f32,
    count: u32,
) -> i32 {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return -1;
    }

    if model_ids.is_null() || positions.is_null() {
        set_last_error(GoudError::InvalidHandle);
        return -1;
    }

    if count == 0 {
        return 0;
    }

    // Guard against integer overflow: count * 3 could exceed usize on 32-bit.
    let total_floats = match (count as usize).checked_mul(3) {
        Some(n) => n,
        None => {
            set_last_error(GoudError::InvalidHandle);
            return -1;
        }
    };

    // SAFETY: caller guarantees `model_ids` has at least `count` elements and
    // `positions` has at least `count * 3` elements.
    let ids = std::slice::from_raw_parts(model_ids, count as usize);
    let pos = std::slice::from_raw_parts(positions, total_floats);

    with_renderer(context_id, |renderer| {
        let mut updated: i32 = 0;
        for (i, &id) in ids.iter().enumerate() {
            let base = i * 3;
            if renderer.set_model_position(id, pos[base], pos[base + 1], pos[base + 2]) {
                updated += 1;
            }
        }
        updated
    })
    .unwrap_or(-1)
}

/// Adds multiple models/instances to a scene in one call.
///
/// # Returns
/// The number of models successfully added to the scene, or `-1` on error
/// (null pointer or invalid context).
///
/// # Safety
/// * `model_ids` must point to at least `count` `u32` elements.
#[no_mangle]
pub unsafe extern "C" fn goud_renderer3d_add_models_to_scene_batch(
    context_id: GoudContextId,
    scene_id: u32,
    model_ids: *const u32,
    count: u32,
) -> i32 {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return -1;
    }

    if model_ids.is_null() {
        set_last_error(GoudError::InvalidHandle);
        return -1;
    }

    if count == 0 {
        return 0;
    }

    // SAFETY: caller guarantees `model_ids` points to at least `count` u32
    // elements.
    let ids = std::slice::from_raw_parts(model_ids, count as usize);

    with_renderer(context_id, |renderer| {
        let mut added: i32 = 0;
        for &id in ids {
            if renderer.add_model_to_scene(scene_id, id) {
                added += 1;
            }
        }
        added
    })
    .unwrap_or(-1)
}
