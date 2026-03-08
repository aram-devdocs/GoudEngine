//! Position-related FFI functions for Transform2D.

use crate::core::error::{set_last_error, GoudError};
use crate::core::math::Vec2;
use crate::core::types::FfiTransform2D;
use crate::ecs::components::Transform2D;
use crate::ffi::types::FfiVec2;

/// Translates the transform by the given offset in world space.
///
/// # Parameters
///
/// - `transform`: Pointer to the transform to modify
/// - `dx`: X offset to translate by
/// - `dy`: Y offset to translate by
///
/// # Safety
///
/// - `transform` must be a valid, non-null pointer to a FfiTransform2D
/// - The caller must ensure exclusive access to the transform
#[no_mangle]
pub unsafe extern "C" fn goud_transform2d_translate(
    transform: *mut FfiTransform2D,
    dx: f32,
    dy: f32,
) {
    if transform.is_null() {
        set_last_error(GoudError::InvalidState(
            "output pointer is null".to_string(),
        ));
        return;
    }
    let t = &mut *transform;
    let mut transform2d: Transform2D = (*t).into();
    transform2d.translate(Vec2::new(dx, dy));
    *t = transform2d.into();
}

/// Translates the transform in local space.
///
/// The offset is rotated by the transform's rotation before being applied.
///
/// # Parameters
///
/// - `transform`: Pointer to the transform to modify
/// - `dx`: X offset in local space
/// - `dy`: Y offset in local space
///
/// # Safety
///
/// - `transform` must be a valid, non-null pointer
#[no_mangle]
pub unsafe extern "C" fn goud_transform2d_translate_local(
    transform: *mut FfiTransform2D,
    dx: f32,
    dy: f32,
) {
    if transform.is_null() {
        set_last_error(GoudError::InvalidState(
            "output pointer is null".to_string(),
        ));
        return;
    }
    let t = &mut *transform;
    let mut transform2d: Transform2D = (*t).into();
    transform2d.translate_local(Vec2::new(dx, dy));
    *t = transform2d.into();
}

/// Sets the position of the transform.
///
/// # Parameters
///
/// - `transform`: Pointer to the transform to modify
/// - `x`: New X position
/// - `y`: New Y position
///
/// # Safety
///
/// - `transform` must be a valid, non-null pointer
#[no_mangle]
pub unsafe extern "C" fn goud_transform2d_set_position(
    transform: *mut FfiTransform2D,
    x: f32,
    y: f32,
) {
    if transform.is_null() {
        set_last_error(GoudError::InvalidState(
            "output pointer is null".to_string(),
        ));
        return;
    }
    let t = &mut *transform;
    t.position_x = x;
    t.position_y = y;
}

/// Gets the position of the transform.
///
/// # Parameters
///
/// - `transform`: Pointer to the transform to read
///
/// # Returns
///
/// The position as an FfiVec2.
///
/// # Safety
///
/// - `transform` must be a valid, non-null pointer
#[no_mangle]
pub unsafe extern "C" fn goud_transform2d_get_position(
    transform: *const FfiTransform2D,
) -> FfiVec2 {
    if transform.is_null() {
        return FfiVec2 { x: 0.0, y: 0.0 };
    }
    let t = &*transform;
    FfiVec2 {
        x: t.position_x,
        y: t.position_y,
    }
}
