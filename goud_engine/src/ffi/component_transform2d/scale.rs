//! Scale-related FFI functions for Transform2D.

use crate::core::types::FfiTransform2D;
use crate::ffi::types::FfiVec2;

/// Sets the scale of the transform.
///
/// # Parameters
///
/// - `transform`: Pointer to the transform to modify
/// - `scale_x`: New X scale
/// - `scale_y`: New Y scale
///
/// # Safety
///
/// - `transform` must be a valid, non-null pointer
#[no_mangle]
pub unsafe extern "C" fn goud_transform2d_set_scale(
    transform: *mut FfiTransform2D,
    scale_x: f32,
    scale_y: f32,
) {
    if transform.is_null() {
        return;
    }
    let t = &mut *transform;
    t.scale_x = scale_x;
    t.scale_y = scale_y;
}

/// Sets uniform scale on both axes.
///
/// # Parameters
///
/// - `transform`: Pointer to the transform to modify
/// - `scale`: Uniform scale factor
///
/// # Safety
///
/// - `transform` must be a valid, non-null pointer
#[no_mangle]
pub unsafe extern "C" fn goud_transform2d_set_scale_uniform(
    transform: *mut FfiTransform2D,
    scale: f32,
) {
    if transform.is_null() {
        return;
    }
    let t = &mut *transform;
    t.scale_x = scale;
    t.scale_y = scale;
}

/// Gets the scale of the transform.
///
/// # Parameters
///
/// - `transform`: Pointer to the transform to read
///
/// # Returns
///
/// The scale as an FfiVec2.
///
/// # Safety
///
/// - `transform` must be a valid, non-null pointer
#[no_mangle]
pub unsafe extern "C" fn goud_transform2d_get_scale(transform: *const FfiTransform2D) -> FfiVec2 {
    if transform.is_null() {
        return FfiVec2 { x: 1.0, y: 1.0 };
    }
    let t = &*transform;
    FfiVec2 {
        x: t.scale_x,
        y: t.scale_y,
    }
}

/// Multiplies the current scale by the given factors.
///
/// # Parameters
///
/// - `transform`: Pointer to the transform to modify
/// - `factor_x`: X scale multiplier
/// - `factor_y`: Y scale multiplier
///
/// # Safety
///
/// - `transform` must be a valid, non-null pointer
#[no_mangle]
pub unsafe extern "C" fn goud_transform2d_scale_by(
    transform: *mut FfiTransform2D,
    factor_x: f32,
    factor_y: f32,
) {
    if transform.is_null() {
        return;
    }
    let t = &mut *transform;
    t.scale_x *= factor_x;
    t.scale_y *= factor_y;
}
