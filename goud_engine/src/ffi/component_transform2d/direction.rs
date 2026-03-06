//! Direction vector FFI functions for Transform2D.

use crate::core::types::FfiTransform2D;
use crate::ecs::components::Transform2D;
use crate::ffi::types::FfiVec2;

/// Returns the forward direction vector (positive X axis after rotation).
///
/// # Parameters
///
/// - `transform`: Pointer to the transform to read
///
/// # Returns
///
/// The forward direction as a unit vector.
///
/// # Safety
///
/// - `transform` must be a valid, non-null pointer
#[no_mangle]
pub unsafe extern "C" fn goud_transform2d_forward(transform: *const FfiTransform2D) -> FfiVec2 {
    if transform.is_null() {
        return FfiVec2 { x: 1.0, y: 0.0 };
    }
    let t = &*transform;
    let transform2d: Transform2D = (*t).into();
    transform2d.forward().into()
}

/// Returns the right direction vector (positive Y axis after rotation).
///
/// This is perpendicular to forward, rotated 90 degrees counter-clockwise.
///
/// # Parameters
///
/// - `transform`: Pointer to the transform to read
///
/// # Returns
///
/// The right direction as a unit vector.
///
/// # Safety
///
/// - `transform` must be a valid, non-null pointer
#[no_mangle]
pub unsafe extern "C" fn goud_transform2d_right(transform: *const FfiTransform2D) -> FfiVec2 {
    if transform.is_null() {
        return FfiVec2 { x: 0.0, y: 1.0 };
    }
    let t = &*transform;
    let transform2d: Transform2D = (*t).into();
    transform2d.right().into()
}

/// Returns the backward direction vector (negative X axis after rotation).
///
/// # Safety
///
/// - `transform` must be a valid, non-null pointer
#[no_mangle]
pub unsafe extern "C" fn goud_transform2d_backward(transform: *const FfiTransform2D) -> FfiVec2 {
    if transform.is_null() {
        return FfiVec2 { x: -1.0, y: 0.0 };
    }
    let t = &*transform;
    let transform2d: Transform2D = (*t).into();
    transform2d.backward().into()
}

/// Returns the left direction vector (negative Y axis after rotation).
///
/// # Safety
///
/// - `transform` must be a valid, non-null pointer
#[no_mangle]
pub unsafe extern "C" fn goud_transform2d_left(transform: *const FfiTransform2D) -> FfiVec2 {
    if transform.is_null() {
        return FfiVec2 { x: 0.0, y: -1.0 };
    }
    let t = &*transform;
    let transform2d: Transform2D = (*t).into();
    transform2d.left().into()
}
