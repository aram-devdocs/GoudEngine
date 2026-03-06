//! Matrix generation, point transformation, interpolation, and utility FFI functions
//! for Transform2D.

use crate::core::math::Vec2;
use crate::core::types::{FfiMat3x3, FfiTransform2D};
use crate::ecs::components::Transform2D;
use crate::ffi::types::FfiVec2;
use std::f32::consts::PI;

// =============================================================================
// Matrix Generation
// =============================================================================

/// Computes the 3x3 transformation matrix.
///
/// The matrix represents Scale * Rotation * Translation.
///
/// # Parameters
///
/// - `transform`: Pointer to the transform to read
///
/// # Returns
///
/// The transformation matrix in column-major order.
///
/// # Safety
///
/// - `transform` must be a valid, non-null pointer
#[no_mangle]
pub unsafe extern "C" fn goud_transform2d_matrix(transform: *const FfiTransform2D) -> FfiMat3x3 {
    if transform.is_null() {
        return FfiMat3x3 {
            m: [1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0],
        };
    }
    let t = &*transform;
    let transform2d: Transform2D = (*t).into();
    transform2d.matrix().into()
}

/// Computes the inverse transformation matrix.
///
/// # Parameters
///
/// - `transform`: Pointer to the transform to read
///
/// # Returns
///
/// The inverse transformation matrix in column-major order.
///
/// # Safety
///
/// - `transform` must be a valid, non-null pointer
#[no_mangle]
pub unsafe extern "C" fn goud_transform2d_matrix_inverse(
    transform: *const FfiTransform2D,
) -> FfiMat3x3 {
    if transform.is_null() {
        return FfiMat3x3 {
            m: [1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0],
        };
    }
    let t = &*transform;
    let transform2d: Transform2D = (*t).into();
    transform2d.matrix_inverse().into()
}

// =============================================================================
// Point Transformation
// =============================================================================

/// Transforms a point from local space to world space.
///
/// # Parameters
///
/// - `transform`: Pointer to the transform
/// - `point_x`: X coordinate of the point in local space
/// - `point_y`: Y coordinate of the point in local space
///
/// # Returns
///
/// The point transformed to world space.
///
/// # Safety
///
/// - `transform` must be a valid, non-null pointer
#[no_mangle]
pub unsafe extern "C" fn goud_transform2d_transform_point(
    transform: *const FfiTransform2D,
    point_x: f32,
    point_y: f32,
) -> FfiVec2 {
    if transform.is_null() {
        return FfiVec2 {
            x: point_x,
            y: point_y,
        };
    }
    let t = &*transform;
    let transform2d: Transform2D = (*t).into();
    transform2d
        .transform_point(Vec2::new(point_x, point_y))
        .into()
}

/// Transforms a direction from local space to world space.
///
/// Unlike points, directions are not affected by translation.
///
/// # Parameters
///
/// - `transform`: Pointer to the transform
/// - `dir_x`: X component of the direction in local space
/// - `dir_y`: Y component of the direction in local space
///
/// # Returns
///
/// The direction transformed to world space.
///
/// # Safety
///
/// - `transform` must be a valid, non-null pointer
#[no_mangle]
pub unsafe extern "C" fn goud_transform2d_transform_direction(
    transform: *const FfiTransform2D,
    dir_x: f32,
    dir_y: f32,
) -> FfiVec2 {
    if transform.is_null() {
        return FfiVec2 { x: dir_x, y: dir_y };
    }
    let t = &*transform;
    let transform2d: Transform2D = (*t).into();
    transform2d
        .transform_direction(Vec2::new(dir_x, dir_y))
        .into()
}

/// Transforms a point from world space to local space.
///
/// # Parameters
///
/// - `transform`: Pointer to the transform
/// - `point_x`: X coordinate of the point in world space
/// - `point_y`: Y coordinate of the point in world space
///
/// # Returns
///
/// The point transformed to local space.
///
/// # Safety
///
/// - `transform` must be a valid, non-null pointer
#[no_mangle]
pub unsafe extern "C" fn goud_transform2d_inverse_transform_point(
    transform: *const FfiTransform2D,
    point_x: f32,
    point_y: f32,
) -> FfiVec2 {
    if transform.is_null() {
        return FfiVec2 {
            x: point_x,
            y: point_y,
        };
    }
    let t = &*transform;
    let transform2d: Transform2D = (*t).into();
    transform2d
        .inverse_transform_point(Vec2::new(point_x, point_y))
        .into()
}

/// Transforms a direction from world space to local space.
///
/// # Safety
///
/// - `transform` must be a valid, non-null pointer
#[no_mangle]
pub unsafe extern "C" fn goud_transform2d_inverse_transform_direction(
    transform: *const FfiTransform2D,
    dir_x: f32,
    dir_y: f32,
) -> FfiVec2 {
    if transform.is_null() {
        return FfiVec2 { x: dir_x, y: dir_y };
    }
    let t = &*transform;
    let transform2d: Transform2D = (*t).into();
    transform2d
        .inverse_transform_direction(Vec2::new(dir_x, dir_y))
        .into()
}

// =============================================================================
// Interpolation
// =============================================================================

/// Linearly interpolates between two transforms.
///
/// Position and scale are linearly interpolated, rotation uses
/// shortest-path angle interpolation.
///
/// # Parameters
///
/// - `from`: The starting transform
/// - `to`: The ending transform
/// - `t`: Interpolation factor (0.0 = from, 1.0 = to)
///
/// # Returns
///
/// The interpolated transform.
#[no_mangle]
pub extern "C" fn goud_transform2d_lerp(
    from: FfiTransform2D,
    to: FfiTransform2D,
    t: f32,
) -> FfiTransform2D {
    let from_t: Transform2D = from.into();
    let to_t: Transform2D = to.into();
    from_t.lerp(to_t, t).into()
}

// =============================================================================
// Utility Functions
// =============================================================================

/// Normalizes an angle to the range [-PI, PI).
///
/// # Parameters
///
/// - `angle`: The angle in radians to normalize
///
/// # Returns
///
/// The normalized angle in the range [-PI, PI).
#[no_mangle]
pub extern "C" fn goud_transform2d_normalize_angle(angle: f32) -> f32 {
    let mut result = angle % (2.0 * PI);
    if result >= PI {
        result -= 2.0 * PI;
    } else if result < -PI {
        result += 2.0 * PI;
    }
    result
}
