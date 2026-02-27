//! # FFI Functions for Transform2D Component
//!
//! This module provides C-compatible functions for manipulating Transform2D components.
//! These functions allow language bindings to perform transform operations without
//! duplicating logic across SDKs.
//!
//! ## Design Philosophy
//!
//! All transform logic lives in Rust. SDKs (C#, Python, etc.) are thin wrappers
//! that marshal data and call these FFI functions. This ensures:
//! - Single source of truth for transform operations
//! - Consistent behavior across all language bindings
//! - No logic duplication in SDK code
//!
//! ## Usage (C#)
//!
//! ```csharp
//! // Create a transform at position (100, 50)
//! var transform = goud_transform2d_from_position(100.0f, 50.0f);
//!
//! // Translate by (10, 20)
//! goud_transform2d_translate(ref transform, 10.0f, 20.0f);
//!
//! // Rotate by 45 degrees (in radians)
//! goud_transform2d_rotate(ref transform, 0.785f);
//!
//! // Get the forward direction vector
//! var forward = goud_transform2d_forward(ref transform);
//! ```
//!
//! ## Thread Safety
//!
//! These functions operate on raw pointers and are not thread-safe. The caller
//! must ensure exclusive access to the Transform2D during mutation.

use crate::core::math::Vec2;
use crate::ecs::components::transform2d::Mat3x3;
use crate::ecs::components::Transform2D;
use crate::ffi::types::FfiVec2;
use std::f32::consts::PI;

// =============================================================================
// FFI-Safe Transform2D Type
// =============================================================================

/// FFI-safe Transform2D representation.
///
/// This matches the memory layout of `Transform2D` exactly and is used
/// for passing transforms across FFI boundaries.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FfiTransform2D {
    /// Position X in world space.
    pub position_x: f32,
    /// Position Y in world space.
    pub position_y: f32,
    /// Rotation angle in radians.
    pub rotation: f32,
    /// Scale along X axis.
    pub scale_x: f32,
    /// Scale along Y axis.
    pub scale_y: f32,
}

impl From<Transform2D> for FfiTransform2D {
    fn from(t: Transform2D) -> Self {
        Self {
            position_x: t.position.x,
            position_y: t.position.y,
            rotation: t.rotation,
            scale_x: t.scale.x,
            scale_y: t.scale.y,
        }
    }
}

impl From<FfiTransform2D> for Transform2D {
    fn from(t: FfiTransform2D) -> Self {
        Self {
            position: Vec2::new(t.position_x, t.position_y),
            rotation: t.rotation,
            scale: Vec2::new(t.scale_x, t.scale_y),
        }
    }
}

/// FFI-safe Mat3x3 representation (9 floats in column-major order).
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct FfiMat3x3 {
    /// Matrix elements in column-major order.
    pub m: [f32; 9],
}

impl From<Mat3x3> for FfiMat3x3 {
    fn from(m: Mat3x3) -> Self {
        Self { m: m.m }
    }
}

// =============================================================================
// Factory Functions
// =============================================================================

/// Creates a default Transform2D at the origin with no rotation and unit scale.
///
/// # Returns
///
/// A Transform2D with position (0, 0), rotation 0, and scale (1, 1).
#[no_mangle]
pub extern "C" fn goud_transform2d_default() -> FfiTransform2D {
    Transform2D::default().into()
}

/// Creates a Transform2D at the specified position with default rotation and scale.
///
/// # Parameters
///
/// - `x`: X position in world space
/// - `y`: Y position in world space
///
/// # Returns
///
/// A Transform2D with the specified position, rotation 0, and scale (1, 1).
#[no_mangle]
pub extern "C" fn goud_transform2d_from_position(x: f32, y: f32) -> FfiTransform2D {
    Transform2D::from_position(Vec2::new(x, y)).into()
}

/// Creates a Transform2D at the origin with the specified rotation.
///
/// # Parameters
///
/// - `rotation`: Rotation angle in radians (counter-clockwise)
///
/// # Returns
///
/// A Transform2D with position (0, 0), the specified rotation, and scale (1, 1).
#[no_mangle]
pub extern "C" fn goud_transform2d_from_rotation(rotation: f32) -> FfiTransform2D {
    Transform2D::from_rotation(rotation).into()
}

/// Creates a Transform2D at the origin with the specified rotation in degrees.
///
/// # Parameters
///
/// - `degrees`: Rotation angle in degrees (counter-clockwise)
///
/// # Returns
///
/// A Transform2D with position (0, 0), the specified rotation, and scale (1, 1).
#[no_mangle]
pub extern "C" fn goud_transform2d_from_rotation_degrees(degrees: f32) -> FfiTransform2D {
    Transform2D::from_rotation_degrees(degrees).into()
}

/// Creates a Transform2D at the origin with the specified scale.
///
/// # Parameters
///
/// - `scale_x`: Scale along X axis
/// - `scale_y`: Scale along Y axis
///
/// # Returns
///
/// A Transform2D with position (0, 0), rotation 0, and the specified scale.
#[no_mangle]
pub extern "C" fn goud_transform2d_from_scale(scale_x: f32, scale_y: f32) -> FfiTransform2D {
    Transform2D::from_scale(Vec2::new(scale_x, scale_y)).into()
}

/// Creates a Transform2D with uniform scale.
///
/// # Parameters
///
/// - `scale`: Uniform scale factor for both axes
///
/// # Returns
///
/// A Transform2D with position (0, 0), rotation 0, and uniform scale.
#[no_mangle]
pub extern "C" fn goud_transform2d_from_scale_uniform(scale: f32) -> FfiTransform2D {
    Transform2D::from_scale_uniform(scale).into()
}

/// Creates a Transform2D with position and rotation.
///
/// # Parameters
///
/// - `x`: X position in world space
/// - `y`: Y position in world space
/// - `rotation`: Rotation angle in radians
///
/// # Returns
///
/// A Transform2D with the specified position and rotation, with scale (1, 1).
#[no_mangle]
pub extern "C" fn goud_transform2d_from_position_rotation(
    x: f32,
    y: f32,
    rotation: f32,
) -> FfiTransform2D {
    Transform2D::from_position_rotation(Vec2::new(x, y), rotation).into()
}

/// Creates a Transform2D with all components specified.
///
/// # Parameters
///
/// - `pos_x`: X position in world space
/// - `pos_y`: Y position in world space
/// - `rotation`: Rotation angle in radians
/// - `scale_x`: Scale along X axis
/// - `scale_y`: Scale along Y axis
///
/// # Returns
///
/// A fully specified Transform2D.
#[no_mangle]
pub extern "C" fn goud_transform2d_new(
    pos_x: f32,
    pos_y: f32,
    rotation: f32,
    scale_x: f32,
    scale_y: f32,
) -> FfiTransform2D {
    Transform2D::new(
        Vec2::new(pos_x, pos_y),
        rotation,
        Vec2::new(scale_x, scale_y),
    )
    .into()
}

/// Creates a Transform2D looking at a target position.
///
/// The transform's forward direction (positive X after rotation)
/// will point towards the target.
///
/// # Parameters
///
/// - `pos_x`: X position of the transform
/// - `pos_y`: Y position of the transform
/// - `target_x`: X position of the target to look at
/// - `target_y`: Y position of the target to look at
///
/// # Returns
///
/// A Transform2D positioned at (pos_x, pos_y) looking towards (target_x, target_y).
#[no_mangle]
pub extern "C" fn goud_transform2d_look_at(
    pos_x: f32,
    pos_y: f32,
    target_x: f32,
    target_y: f32,
) -> FfiTransform2D {
    Transform2D::look_at(Vec2::new(pos_x, pos_y), Vec2::new(target_x, target_y)).into()
}

// =============================================================================
// Position Methods
// =============================================================================

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

// =============================================================================
// Rotation Methods
// =============================================================================

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
        return;
    }
    let t = &mut *transform;
    let mut transform2d: Transform2D = (*t).into();
    transform2d.look_at_target(Vec2::new(target_x, target_y));
    *t = transform2d.into();
}

// =============================================================================
// Scale Methods
// =============================================================================

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

// =============================================================================
// Direction Vectors
// =============================================================================

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

// =============================================================================
// Builder Pattern (Heap-Allocated)
// =============================================================================
//
// The builder pattern provides a heap-allocated mutable builder that allows
// chaining operations without copying the full struct on each call. This is
// useful when constructing complex transforms with many properties.
//
// ## Usage (C#)
//
// ```csharp
// var builder = goud_transform2d_builder_new();
// builder = goud_transform2d_builder_with_position(builder, 100.0f, 50.0f);
// builder = goud_transform2d_builder_with_rotation(builder, 0.785f);
// builder = goud_transform2d_builder_with_scale(builder, 2.0f, 2.0f);
// var transform = goud_transform2d_builder_build(builder); // Consumes builder
// ```
//
// ## Memory Management
//
// - `goud_transform2d_builder_new()` allocates a builder on the heap
// - `goud_transform2d_builder_build()` consumes the builder and frees memory
// - If you don't call build(), call `goud_transform2d_builder_free()` to clean up
// - Builder functions return the same pointer for chaining

/// Heap-allocated transform builder for FFI use.
///
/// This builder allows constructing a transform by setting properties one at a time
/// without copying the entire struct on each modification.
#[repr(C)]
pub struct FfiTransform2DBuilder {
    transform: FfiTransform2D,
}

/// Creates a new transform builder with default values.
///
/// The builder is heap-allocated and must be either:
/// - Consumed with `goud_transform2d_builder_build()`, or
/// - Freed with `goud_transform2d_builder_free()`
///
/// # Returns
///
/// A pointer to a new FfiTransform2DBuilder, or null if allocation fails.
///
/// # Safety
///
/// The returned pointer must be managed by the caller.
#[no_mangle]
pub extern "C" fn goud_transform2d_builder_new() -> *mut FfiTransform2DBuilder {
    let builder = FfiTransform2DBuilder {
        transform: goud_transform2d_default(),
    };
    Box::into_raw(Box::new(builder))
}

/// Creates a new transform builder at a specific position.
///
/// # Parameters
///
/// - `x`: X position in world space
/// - `y`: Y position in world space
///
/// # Returns
///
/// A pointer to a new FfiTransform2DBuilder.
#[no_mangle]
pub extern "C" fn goud_transform2d_builder_at_position(
    x: f32,
    y: f32,
) -> *mut FfiTransform2DBuilder {
    let builder = FfiTransform2DBuilder {
        transform: goud_transform2d_from_position(x, y),
    };
    Box::into_raw(Box::new(builder))
}

/// Sets the position on the builder.
///
/// # Parameters
///
/// - `builder`: Pointer to the builder
/// - `x`: X position in world space
/// - `y`: Y position in world space
///
/// # Returns
///
/// The same builder pointer for chaining.
///
/// # Safety
///
/// - `builder` must be a valid pointer from `goud_transform2d_builder_new()`
#[no_mangle]
pub unsafe extern "C" fn goud_transform2d_builder_with_position(
    builder: *mut FfiTransform2DBuilder,
    x: f32,
    y: f32,
) -> *mut FfiTransform2DBuilder {
    if builder.is_null() {
        return builder;
    }
    let t = &mut (*builder).transform;
    t.position_x = x;
    t.position_y = y;
    builder
}

/// Sets the rotation (in radians) on the builder.
///
/// # Parameters
///
/// - `builder`: Pointer to the builder
/// - `rotation`: Rotation angle in radians (counter-clockwise)
///
/// # Returns
///
/// The same builder pointer for chaining.
///
/// # Safety
///
/// - `builder` must be a valid pointer from `goud_transform2d_builder_new()`
#[no_mangle]
pub unsafe extern "C" fn goud_transform2d_builder_with_rotation(
    builder: *mut FfiTransform2DBuilder,
    rotation: f32,
) -> *mut FfiTransform2DBuilder {
    if builder.is_null() {
        return builder;
    }
    (*builder).transform.rotation = rotation;
    builder
}

/// Sets the rotation (in degrees) on the builder.
///
/// # Parameters
///
/// - `builder`: Pointer to the builder
/// - `degrees`: Rotation angle in degrees (counter-clockwise)
///
/// # Returns
///
/// The same builder pointer for chaining.
///
/// # Safety
///
/// - `builder` must be a valid pointer from `goud_transform2d_builder_new()` or null
#[no_mangle]
pub unsafe extern "C" fn goud_transform2d_builder_with_rotation_degrees(
    builder: *mut FfiTransform2DBuilder,
    degrees: f32,
) -> *mut FfiTransform2DBuilder {
    if builder.is_null() {
        return builder;
    }
    (*builder).transform.rotation = degrees.to_radians();
    builder
}

/// Sets the scale on the builder.
///
/// # Parameters
///
/// - `builder`: Pointer to the builder
/// - `scale_x`: Scale along X axis
/// - `scale_y`: Scale along Y axis
///
/// # Returns
///
/// The same builder pointer for chaining.
///
/// # Safety
///
/// - `builder` must be a valid pointer from `goud_transform2d_builder_new()` or null
#[no_mangle]
pub unsafe extern "C" fn goud_transform2d_builder_with_scale(
    builder: *mut FfiTransform2DBuilder,
    scale_x: f32,
    scale_y: f32,
) -> *mut FfiTransform2DBuilder {
    if builder.is_null() {
        return builder;
    }
    let t = &mut (*builder).transform;
    t.scale_x = scale_x;
    t.scale_y = scale_y;
    builder
}

/// Sets uniform scale on the builder.
///
/// # Parameters
///
/// - `builder`: Pointer to the builder
/// - `scale`: Uniform scale factor for both axes
///
/// # Returns
///
/// The same builder pointer for chaining.
///
/// # Safety
///
/// - `builder` must be a valid pointer from `goud_transform2d_builder_new()` or null
#[no_mangle]
pub unsafe extern "C" fn goud_transform2d_builder_with_scale_uniform(
    builder: *mut FfiTransform2DBuilder,
    scale: f32,
) -> *mut FfiTransform2DBuilder {
    if builder.is_null() {
        return builder;
    }
    let t = &mut (*builder).transform;
    t.scale_x = scale;
    t.scale_y = scale;
    builder
}

/// Makes the builder's transform look at a target position.
///
/// Sets the rotation so the forward direction points at the target.
///
/// # Parameters
///
/// - `builder`: Pointer to the builder
/// - `target_x`: X position of the target
/// - `target_y`: Y position of the target
///
/// # Returns
///
/// The same builder pointer for chaining.
///
/// # Safety
///
/// - `builder` must be a valid pointer from `goud_transform2d_builder_new()` or null
#[no_mangle]
pub unsafe extern "C" fn goud_transform2d_builder_looking_at(
    builder: *mut FfiTransform2DBuilder,
    target_x: f32,
    target_y: f32,
) -> *mut FfiTransform2DBuilder {
    if builder.is_null() {
        return builder;
    }
    let t = &mut (*builder).transform;
    let dx = target_x - t.position_x;
    let dy = target_y - t.position_y;
    t.rotation = dy.atan2(dx);
    builder
}

/// Translates the builder's position.
///
/// Adds the offset to the current position.
///
/// # Parameters
///
/// - `builder`: Pointer to the builder
/// - `dx`: X offset
/// - `dy`: Y offset
///
/// # Returns
///
/// The same builder pointer for chaining.
///
/// # Safety
///
/// - `builder` must be a valid pointer from `goud_transform2d_builder_new()` or null
#[no_mangle]
pub unsafe extern "C" fn goud_transform2d_builder_translate(
    builder: *mut FfiTransform2DBuilder,
    dx: f32,
    dy: f32,
) -> *mut FfiTransform2DBuilder {
    if builder.is_null() {
        return builder;
    }
    let t = &mut (*builder).transform;
    t.position_x += dx;
    t.position_y += dy;
    builder
}

/// Rotates the builder's transform.
///
/// Adds the angle to the current rotation.
///
/// # Parameters
///
/// - `builder`: Pointer to the builder
/// - `angle`: Angle to rotate by in radians
///
/// # Returns
///
/// The same builder pointer for chaining.
///
/// # Safety
///
/// - `builder` must be a valid pointer from `goud_transform2d_builder_new()` or null
#[no_mangle]
pub unsafe extern "C" fn goud_transform2d_builder_rotate(
    builder: *mut FfiTransform2DBuilder,
    angle: f32,
) -> *mut FfiTransform2DBuilder {
    if builder.is_null() {
        return builder;
    }
    (*builder).transform.rotation += angle;
    builder
}

/// Multiplies the builder's scale.
///
/// # Parameters
///
/// - `builder`: Pointer to the builder
/// - `factor_x`: X scale multiplier
/// - `factor_y`: Y scale multiplier
///
/// # Returns
///
/// The same builder pointer for chaining.
///
/// # Safety
///
/// - `builder` must be a valid pointer from `goud_transform2d_builder_new()` or null
#[no_mangle]
pub unsafe extern "C" fn goud_transform2d_builder_scale_by(
    builder: *mut FfiTransform2DBuilder,
    factor_x: f32,
    factor_y: f32,
) -> *mut FfiTransform2DBuilder {
    if builder.is_null() {
        return builder;
    }
    let t = &mut (*builder).transform;
    t.scale_x *= factor_x;
    t.scale_y *= factor_y;
    builder
}

/// Builds and consumes the transform builder, returning the final transform.
///
/// This function:
/// 1. Extracts the transform from the builder
/// 2. Frees the builder memory
/// 3. Returns the transform by value
///
/// After calling this, the builder pointer is invalid.
///
/// # Parameters
///
/// - `builder`: Pointer to the builder to consume
///
/// # Returns
///
/// The constructed FfiTransform2D. If builder is null, returns a default transform.
///
/// # Safety
///
/// - `builder` must be a valid pointer from `goud_transform2d_builder_new()`
/// - After this call, the builder pointer must not be used
#[no_mangle]
pub unsafe extern "C" fn goud_transform2d_builder_build(
    builder: *mut FfiTransform2DBuilder,
) -> FfiTransform2D {
    if builder.is_null() {
        return goud_transform2d_default();
    }
    // Take ownership of the box and extract the transform
    let boxed = Box::from_raw(builder);
    boxed.transform
}

/// Frees a transform builder without building.
///
/// Use this if you need to abort transform construction and clean up memory.
///
/// # Parameters
///
/// - `builder`: Pointer to the builder to free
///
/// # Safety
///
/// - `builder` must be a valid pointer from `goud_transform2d_builder_new()` or null
/// - After this call, the builder pointer must not be used
#[no_mangle]
pub unsafe extern "C" fn goud_transform2d_builder_free(builder: *mut FfiTransform2DBuilder) {
    if !builder.is_null() {
        drop(Box::from_raw(builder));
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::{FRAC_PI_2, FRAC_PI_4};

    #[test]
    fn test_ffi_transform2d_default() {
        let t = goud_transform2d_default();
        assert_eq!(t.position_x, 0.0);
        assert_eq!(t.position_y, 0.0);
        assert_eq!(t.rotation, 0.0);
        assert_eq!(t.scale_x, 1.0);
        assert_eq!(t.scale_y, 1.0);
    }

    #[test]
    fn test_ffi_transform2d_from_position() {
        let t = goud_transform2d_from_position(10.0, 20.0);
        assert_eq!(t.position_x, 10.0);
        assert_eq!(t.position_y, 20.0);
        assert_eq!(t.rotation, 0.0);
        assert_eq!(t.scale_x, 1.0);
        assert_eq!(t.scale_y, 1.0);
    }

    #[test]
    fn test_ffi_transform2d_translate() {
        let mut t = goud_transform2d_from_position(10.0, 20.0);
        unsafe {
            goud_transform2d_translate(&mut t, 5.0, 10.0);
        }
        assert_eq!(t.position_x, 15.0);
        assert_eq!(t.position_y, 30.0);
    }

    #[test]
    fn test_ffi_transform2d_rotate() {
        let mut t = goud_transform2d_default();
        unsafe {
            goud_transform2d_rotate(&mut t, FRAC_PI_4);
        }
        assert!((t.rotation - FRAC_PI_4).abs() < 0.0001);
    }

    #[test]
    fn test_ffi_transform2d_forward() {
        let t = goud_transform2d_from_rotation(FRAC_PI_2);
        let forward = unsafe { goud_transform2d_forward(&t) };
        // 90 degree rotation: forward (1, 0) -> (0, 1)
        assert!(forward.x.abs() < 0.0001);
        assert!((forward.y - 1.0).abs() < 0.0001);
    }

    #[test]
    fn test_ffi_transform2d_lerp() {
        let a = goud_transform2d_from_position(0.0, 0.0);
        let b = goud_transform2d_from_position(10.0, 20.0);
        let mid = goud_transform2d_lerp(a, b, 0.5);
        assert_eq!(mid.position_x, 5.0);
        assert_eq!(mid.position_y, 10.0);
    }

    #[test]
    fn test_ffi_transform2d_null_safety() {
        // Test that null pointer functions don't crash
        unsafe {
            goud_transform2d_translate(std::ptr::null_mut(), 1.0, 2.0);
            goud_transform2d_rotate(std::ptr::null_mut(), 1.0);
            goud_transform2d_set_position(std::ptr::null_mut(), 1.0, 2.0);
            goud_transform2d_set_scale(std::ptr::null_mut(), 1.0, 2.0);

            let pos = goud_transform2d_get_position(std::ptr::null());
            assert_eq!(pos.x, 0.0);
            assert_eq!(pos.y, 0.0);

            let rot = goud_transform2d_get_rotation(std::ptr::null());
            assert_eq!(rot, 0.0);

            let fwd = goud_transform2d_forward(std::ptr::null());
            assert_eq!(fwd.x, 1.0);
            assert_eq!(fwd.y, 0.0);
        }
    }

    #[test]
    fn test_ffi_transform2d_size() {
        // Verify FFI type has expected size
        assert_eq!(std::mem::size_of::<FfiTransform2D>(), 20);
        assert_eq!(std::mem::size_of::<FfiVec2>(), 8);
        assert_eq!(std::mem::size_of::<FfiMat3x3>(), 36);
    }

    // =========================================================================
    // Builder Pattern Tests
    // =========================================================================

    #[test]
    fn test_builder_new_and_build() {
        let builder = goud_transform2d_builder_new();
        assert!(!builder.is_null());

        let transform = unsafe { goud_transform2d_builder_build(builder) };
        assert_eq!(transform.position_x, 0.0);
        assert_eq!(transform.position_y, 0.0);
        assert_eq!(transform.rotation, 0.0);
        assert_eq!(transform.scale_x, 1.0);
        assert_eq!(transform.scale_y, 1.0);
    }

    #[test]
    fn test_builder_at_position() {
        let builder = goud_transform2d_builder_at_position(100.0, 50.0);
        let transform = unsafe { goud_transform2d_builder_build(builder) };

        assert_eq!(transform.position_x, 100.0);
        assert_eq!(transform.position_y, 50.0);
    }

    #[test]
    fn test_builder_with_position() {
        let builder = goud_transform2d_builder_new();
        let builder = unsafe { goud_transform2d_builder_with_position(builder, 200.0, 150.0) };
        let transform = unsafe { goud_transform2d_builder_build(builder) };

        assert_eq!(transform.position_x, 200.0);
        assert_eq!(transform.position_y, 150.0);
    }

    #[test]
    fn test_builder_with_rotation() {
        let builder = goud_transform2d_builder_new();
        let builder = unsafe { goud_transform2d_builder_with_rotation(builder, FRAC_PI_4) };
        let transform = unsafe { goud_transform2d_builder_build(builder) };

        assert!((transform.rotation - FRAC_PI_4).abs() < 0.0001);
    }

    #[test]
    fn test_builder_with_rotation_degrees() {
        let builder = goud_transform2d_builder_new();
        let builder = unsafe { goud_transform2d_builder_with_rotation_degrees(builder, 90.0) };
        let transform = unsafe { goud_transform2d_builder_build(builder) };

        assert!((transform.rotation - FRAC_PI_2).abs() < 0.0001);
    }

    #[test]
    fn test_builder_with_scale() {
        let builder = goud_transform2d_builder_new();
        let builder = unsafe { goud_transform2d_builder_with_scale(builder, 2.0, 3.0) };
        let transform = unsafe { goud_transform2d_builder_build(builder) };

        assert_eq!(transform.scale_x, 2.0);
        assert_eq!(transform.scale_y, 3.0);
    }

    #[test]
    fn test_builder_with_scale_uniform() {
        let builder = goud_transform2d_builder_new();
        let builder = unsafe { goud_transform2d_builder_with_scale_uniform(builder, 5.0) };
        let transform = unsafe { goud_transform2d_builder_build(builder) };

        assert_eq!(transform.scale_x, 5.0);
        assert_eq!(transform.scale_y, 5.0);
    }

    #[test]
    fn test_builder_looking_at() {
        let builder = goud_transform2d_builder_at_position(0.0, 0.0);
        let builder = unsafe { goud_transform2d_builder_looking_at(builder, 0.0, 10.0) };
        let transform = unsafe { goud_transform2d_builder_build(builder) };

        // Looking straight up (0, 10) from origin should be 90 degrees
        assert!((transform.rotation - FRAC_PI_2).abs() < 0.0001);
    }

    #[test]
    fn test_builder_translate() {
        let builder = goud_transform2d_builder_at_position(10.0, 20.0);
        let builder = unsafe { goud_transform2d_builder_translate(builder, 5.0, 10.0) };
        let transform = unsafe { goud_transform2d_builder_build(builder) };

        assert_eq!(transform.position_x, 15.0);
        assert_eq!(transform.position_y, 30.0);
    }

    #[test]
    fn test_builder_rotate() {
        let builder = goud_transform2d_builder_new();
        let builder = unsafe { goud_transform2d_builder_with_rotation(builder, FRAC_PI_4) };
        let builder = unsafe { goud_transform2d_builder_rotate(builder, FRAC_PI_4) };
        let transform = unsafe { goud_transform2d_builder_build(builder) };

        assert!((transform.rotation - FRAC_PI_2).abs() < 0.0001);
    }

    #[test]
    fn test_builder_scale_by() {
        let builder = goud_transform2d_builder_new();
        let builder = unsafe { goud_transform2d_builder_with_scale(builder, 2.0, 3.0) };
        let builder = unsafe { goud_transform2d_builder_scale_by(builder, 2.0, 2.0) };
        let transform = unsafe { goud_transform2d_builder_build(builder) };

        assert_eq!(transform.scale_x, 4.0);
        assert_eq!(transform.scale_y, 6.0);
    }

    #[test]
    fn test_builder_chain() {
        let builder = goud_transform2d_builder_new();
        let builder = unsafe { goud_transform2d_builder_with_position(builder, 100.0, 50.0) };
        let builder = unsafe { goud_transform2d_builder_with_rotation(builder, FRAC_PI_4) };
        let builder = unsafe { goud_transform2d_builder_with_scale(builder, 2.0, 2.0) };
        let builder = unsafe { goud_transform2d_builder_translate(builder, 10.0, 10.0) };
        let transform = unsafe { goud_transform2d_builder_build(builder) };

        assert_eq!(transform.position_x, 110.0);
        assert_eq!(transform.position_y, 60.0);
        assert!((transform.rotation - FRAC_PI_4).abs() < 0.0001);
        assert_eq!(transform.scale_x, 2.0);
        assert_eq!(transform.scale_y, 2.0);
    }

    #[test]
    fn test_builder_free() {
        let builder = goud_transform2d_builder_new();
        unsafe { goud_transform2d_builder_free(builder) };
        // Should not crash - just testing memory is freed properly
    }

    #[test]
    fn test_builder_null_safety() {
        // All builder functions should handle null safely
        unsafe {
            let null_builder: *mut FfiTransform2DBuilder = std::ptr::null_mut();

            assert!(goud_transform2d_builder_with_position(null_builder, 1.0, 2.0).is_null());
            assert!(goud_transform2d_builder_with_rotation(null_builder, 1.0).is_null());
            assert!(goud_transform2d_builder_with_rotation_degrees(null_builder, 90.0).is_null());
            assert!(goud_transform2d_builder_with_scale(null_builder, 2.0, 2.0).is_null());
            assert!(goud_transform2d_builder_with_scale_uniform(null_builder, 2.0).is_null());
            assert!(goud_transform2d_builder_looking_at(null_builder, 0.0, 10.0).is_null());
            assert!(goud_transform2d_builder_translate(null_builder, 1.0, 2.0).is_null());
            assert!(goud_transform2d_builder_rotate(null_builder, 1.0).is_null());
            assert!(goud_transform2d_builder_scale_by(null_builder, 2.0, 2.0).is_null());

            // Build with null should return default
            let transform = goud_transform2d_builder_build(null_builder);
            assert_eq!(transform.position_x, 0.0);
            assert_eq!(transform.position_y, 0.0);
            assert_eq!(transform.rotation, 0.0);

            // Free null should not crash
            goud_transform2d_builder_free(null_builder);
        }
    }
}
