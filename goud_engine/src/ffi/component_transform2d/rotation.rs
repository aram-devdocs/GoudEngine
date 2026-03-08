//! Rotation-related FFI functions for Transform2D.

use crate::core::error::{set_last_error, GoudError};
use crate::core::math::Vec2;
use crate::core::types::FfiTransform2D;
use crate::ecs::components::Transform2D;

/// Rotates the transform by the given angle in radians.
///
/// The rotation is normalized to [-PI, PI).
///
/// # Parameters
///
/// - `transform`: Pointer to the transform to modify
/// - `angle`: Angle to rotate by in radians (counter-clockwise)
///
/// # Safety
///
/// - `transform` must be a valid, non-null pointer
#[no_mangle]
pub unsafe extern "C" fn goud_transform2d_rotate(transform: *mut FfiTransform2D, angle: f32) {
    if transform.is_null() {
        set_last_error(GoudError::InvalidState(
            "output pointer is null".to_string(),
        ));
        return;
    }
    let t = &mut *transform;
    let mut transform2d: Transform2D = (*t).into();
    transform2d.rotate(angle);
    *t = transform2d.into();
}

/// Rotates the transform by the given angle in degrees.
///
/// # Parameters
///
/// - `transform`: Pointer to the transform to modify
/// - `degrees`: Angle to rotate by in degrees (counter-clockwise)
///
/// # Safety
///
/// - `transform` must be a valid, non-null pointer
#[no_mangle]
pub unsafe extern "C" fn goud_transform2d_rotate_degrees(
    transform: *mut FfiTransform2D,
    degrees: f32,
) {
    if transform.is_null() {
        set_last_error(GoudError::InvalidState(
            "output pointer is null".to_string(),
        ));
        return;
    }
    let t = &mut *transform;
    let mut transform2d: Transform2D = (*t).into();
    transform2d.rotate_degrees(degrees);
    *t = transform2d.into();
}

/// Sets the rotation angle in radians.
///
/// The rotation is normalized to [-PI, PI).
///
/// # Parameters
///
/// - `transform`: Pointer to the transform to modify
/// - `rotation`: New rotation angle in radians
///
/// # Safety
///
/// - `transform` must be a valid, non-null pointer
#[no_mangle]
pub unsafe extern "C" fn goud_transform2d_set_rotation(
    transform: *mut FfiTransform2D,
    rotation: f32,
) {
    if transform.is_null() {
        set_last_error(GoudError::InvalidState(
            "output pointer is null".to_string(),
        ));
        return;
    }
    let t = &mut *transform;
    let mut transform2d: Transform2D = (*t).into();
    transform2d.set_rotation(rotation);
    *t = transform2d.into();
}

/// Sets the rotation angle in degrees.
///
/// # Parameters
///
/// - `transform`: Pointer to the transform to modify
/// - `degrees`: New rotation angle in degrees
///
/// # Safety
///
/// - `transform` must be a valid, non-null pointer
#[no_mangle]
pub unsafe extern "C" fn goud_transform2d_set_rotation_degrees(
    transform: *mut FfiTransform2D,
    degrees: f32,
) {
    if transform.is_null() {
        set_last_error(GoudError::InvalidState(
            "output pointer is null".to_string(),
        ));
        return;
    }
    let t = &mut *transform;
    let mut transform2d: Transform2D = (*t).into();
    transform2d.set_rotation_degrees(degrees);
    *t = transform2d.into();
}

/// Gets the rotation angle in radians.
///
/// # Parameters
///
/// - `transform`: Pointer to the transform to read
///
/// # Returns
///
/// The rotation angle in radians.
///
/// # Safety
///
/// - `transform` must be a valid, non-null pointer
#[no_mangle]
pub unsafe extern "C" fn goud_transform2d_get_rotation(transform: *const FfiTransform2D) -> f32 {
    if transform.is_null() {
        set_last_error(GoudError::InvalidState(
            "output pointer is null".to_string(),
        ));
        return 0.0;
    }
    (*transform).rotation
}

/// Gets the rotation angle in degrees.
///
/// # Parameters
///
/// - `transform`: Pointer to the transform to read
///
/// # Returns
///
/// The rotation angle in degrees.
///
/// # Safety
///
/// - `transform` must be a valid, non-null pointer
#[no_mangle]
pub unsafe extern "C" fn goud_transform2d_get_rotation_degrees(
    transform: *const FfiTransform2D,
) -> f32 {
    if transform.is_null() {
        set_last_error(GoudError::InvalidState(
            "output pointer is null".to_string(),
        ));
        return 0.0;
    }
    (*transform).rotation.to_degrees()
}

/// Makes the transform look at a target position.
///
/// # Parameters
///
/// - `transform`: Pointer to the transform to modify
/// - `target_x`: X position of the target
/// - `target_y`: Y position of the target
///
/// # Safety
///
/// - `transform` must be a valid, non-null pointer
#[no_mangle]
pub unsafe extern "C" fn goud_transform2d_look_at_target(
    transform: *mut FfiTransform2D,
    target_x: f32,
    target_y: f32,
) {
    if transform.is_null() {
        set_last_error(GoudError::InvalidState(
            "output pointer is null".to_string(),
        ));
        return;
    }
    let t = &mut *transform;
    let mut transform2d: Transform2D = (*t).into();
    transform2d.look_at_target(Vec2::new(target_x, target_y));
    *t = transform2d.into();
}
