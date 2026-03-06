//! Pointer-based mutation and query operations on Transform2D.

use crate::core::math::Vec2;
use crate::core::types::{FfiMat3x3, FfiTransform2D, FfiVec2};
use crate::ecs::components::Transform2D;

/// Zero-sized type for Transform2D pointer operations.
pub struct Transform2DPtrOps;

// NOTE: FFI wrappers are hand-written in ffi/component_transform2d.rs. The `#[goud_api]`
// attribute is omitted here to avoid duplicate `#[no_mangle]` symbol conflicts.
impl Transform2DPtrOps {
    /// Translates the transform by the given offset.
    pub fn translate(transform: *mut FfiTransform2D, dx: f32, dy: f32) {
        if transform.is_null() {
            return;
        }
        // SAFETY: Caller guarantees pointer is valid.
        let t = unsafe { &mut *transform };
        let mut t2d: Transform2D = (*t).into();
        t2d.translate(Vec2::new(dx, dy));
        *t = t2d.into();
    }

    /// Translates the transform in local space.
    pub fn translate_local(transform: *mut FfiTransform2D, dx: f32, dy: f32) {
        if transform.is_null() {
            return;
        }
        // SAFETY: Pointer checked non-null above; allocated by the corresponding _create function.
        let t = unsafe { &mut *transform };
        let mut t2d: Transform2D = (*t).into();
        t2d.translate_local(Vec2::new(dx, dy));
        *t = t2d.into();
    }

    /// Sets the position of the transform.
    pub fn set_position(transform: *mut FfiTransform2D, x: f32, y: f32) {
        if transform.is_null() {
            return;
        }
        // SAFETY: Pointer checked non-null above; allocated by the corresponding _create function.
        let t = unsafe { &mut *transform };
        t.position_x = x;
        t.position_y = y;
    }

    /// Gets the position of the transform.
    pub fn get_position(transform: *const FfiTransform2D) -> FfiVec2 {
        if transform.is_null() {
            return FfiVec2 { x: 0.0, y: 0.0 };
        }
        // SAFETY: Pointer checked non-null above; allocated by the corresponding _create function.
        let t = unsafe { &*transform };
        FfiVec2 {
            x: t.position_x,
            y: t.position_y,
        }
    }

    /// Rotates the transform by the given angle in radians.
    pub fn rotate(transform: *mut FfiTransform2D, angle: f32) {
        if transform.is_null() {
            return;
        }
        // SAFETY: Pointer checked non-null above; allocated by the corresponding _create function.
        let t = unsafe { &mut *transform };
        let mut t2d: Transform2D = (*t).into();
        t2d.rotate(angle);
        *t = t2d.into();
    }

    /// Rotates the transform by the given angle in degrees.
    pub fn rotate_degrees(transform: *mut FfiTransform2D, degrees: f32) {
        if transform.is_null() {
            return;
        }
        // SAFETY: Pointer checked non-null above; allocated by the corresponding _create function.
        let t = unsafe { &mut *transform };
        let mut t2d: Transform2D = (*t).into();
        t2d.rotate_degrees(degrees);
        *t = t2d.into();
    }

    /// Sets the rotation angle in radians.
    pub fn set_rotation(transform: *mut FfiTransform2D, rotation: f32) {
        if transform.is_null() {
            return;
        }
        // SAFETY: Pointer checked non-null above; allocated by the corresponding _create function.
        let t = unsafe { &mut *transform };
        let mut t2d: Transform2D = (*t).into();
        t2d.set_rotation(rotation);
        *t = t2d.into();
    }

    /// Sets the rotation angle in degrees.
    pub fn set_rotation_degrees(transform: *mut FfiTransform2D, degrees: f32) {
        if transform.is_null() {
            return;
        }
        // SAFETY: Pointer checked non-null above; allocated by the corresponding _create function.
        let t = unsafe { &mut *transform };
        let mut t2d: Transform2D = (*t).into();
        t2d.set_rotation_degrees(degrees);
        *t = t2d.into();
    }

    /// Gets the rotation angle in radians.
    pub fn get_rotation(transform: *const FfiTransform2D) -> f32 {
        if transform.is_null() {
            return 0.0;
        }
        // SAFETY: Pointer checked non-null above; allocated by the corresponding _create function.
        unsafe { (*transform).rotation }
    }

    /// Gets the rotation angle in degrees.
    pub fn get_rotation_degrees(transform: *const FfiTransform2D) -> f32 {
        if transform.is_null() {
            return 0.0;
        }
        // SAFETY: Pointer checked non-null above; allocated by the corresponding _create function.
        unsafe { (*transform).rotation.to_degrees() }
    }

    /// Makes the transform look at a target position.
    pub fn look_at_target(transform: *mut FfiTransform2D, target_x: f32, target_y: f32) {
        if transform.is_null() {
            return;
        }
        // SAFETY: Pointer checked non-null above; allocated by the corresponding _create function.
        let t = unsafe { &mut *transform };
        let mut t2d: Transform2D = (*t).into();
        t2d.look_at_target(Vec2::new(target_x, target_y));
        *t = t2d.into();
    }

    /// Sets the scale of the transform.
    pub fn set_scale(transform: *mut FfiTransform2D, scale_x: f32, scale_y: f32) {
        if transform.is_null() {
            return;
        }
        // SAFETY: Pointer checked non-null above; allocated by the corresponding _create function.
        let t = unsafe { &mut *transform };
        t.scale_x = scale_x;
        t.scale_y = scale_y;
    }

    /// Sets uniform scale on both axes.
    pub fn set_scale_uniform(transform: *mut FfiTransform2D, scale: f32) {
        if transform.is_null() {
            return;
        }
        // SAFETY: Pointer checked non-null above; allocated by the corresponding _create function.
        let t = unsafe { &mut *transform };
        t.scale_x = scale;
        t.scale_y = scale;
    }

    /// Gets the scale of the transform.
    pub fn get_scale(transform: *const FfiTransform2D) -> FfiVec2 {
        if transform.is_null() {
            return FfiVec2 { x: 1.0, y: 1.0 };
        }
        // SAFETY: Pointer checked non-null above; allocated by the corresponding _create function.
        let t = unsafe { &*transform };
        FfiVec2 {
            x: t.scale_x,
            y: t.scale_y,
        }
    }

    /// Multiplies the current scale by the given factors.
    pub fn scale_by(transform: *mut FfiTransform2D, factor_x: f32, factor_y: f32) {
        if transform.is_null() {
            return;
        }
        // SAFETY: Pointer checked non-null above; allocated by the corresponding _create function.
        let t = unsafe { &mut *transform };
        t.scale_x *= factor_x;
        t.scale_y *= factor_y;
    }

    /// Returns the forward direction vector.
    pub fn forward(transform: *const FfiTransform2D) -> FfiVec2 {
        if transform.is_null() {
            return FfiVec2 { x: 1.0, y: 0.0 };
        }
        // SAFETY: Pointer checked non-null above; allocated by the corresponding _create function.
        let t = unsafe { &*transform };
        let t2d: Transform2D = (*t).into();
        t2d.forward().into()
    }

    /// Returns the right direction vector.
    pub fn right(transform: *const FfiTransform2D) -> FfiVec2 {
        if transform.is_null() {
            return FfiVec2 { x: 0.0, y: 1.0 };
        }
        // SAFETY: Pointer checked non-null above; allocated by the corresponding _create function.
        let t = unsafe { &*transform };
        let t2d: Transform2D = (*t).into();
        t2d.right().into()
    }

    /// Returns the backward direction vector.
    pub fn backward(transform: *const FfiTransform2D) -> FfiVec2 {
        if transform.is_null() {
            return FfiVec2 { x: -1.0, y: 0.0 };
        }
        // SAFETY: Pointer checked non-null above; allocated by the corresponding _create function.
        let t = unsafe { &*transform };
        let t2d: Transform2D = (*t).into();
        t2d.backward().into()
    }

    /// Returns the left direction vector.
    pub fn left(transform: *const FfiTransform2D) -> FfiVec2 {
        if transform.is_null() {
            return FfiVec2 { x: 0.0, y: -1.0 };
        }
        // SAFETY: Pointer checked non-null above; allocated by the corresponding _create function.
        let t = unsafe { &*transform };
        let t2d: Transform2D = (*t).into();
        t2d.left().into()
    }

    /// Computes the 3x3 transformation matrix.
    pub fn matrix(transform: *const FfiTransform2D) -> FfiMat3x3 {
        if transform.is_null() {
            return FfiMat3x3 {
                m: [1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0],
            };
        }
        // SAFETY: Pointer checked non-null above; allocated by the corresponding _create function.
        let t = unsafe { &*transform };
        let t2d: Transform2D = (*t).into();
        t2d.matrix().into()
    }

    /// Computes the inverse transformation matrix.
    pub fn matrix_inverse(transform: *const FfiTransform2D) -> FfiMat3x3 {
        if transform.is_null() {
            return FfiMat3x3 {
                m: [1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0],
            };
        }
        // SAFETY: Pointer checked non-null above; allocated by the corresponding _create function.
        let t = unsafe { &*transform };
        let t2d: Transform2D = (*t).into();
        t2d.matrix_inverse().into()
    }

    /// Transforms a point from local space to world space.
    pub fn transform_point(
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
        // SAFETY: Pointer checked non-null above; allocated by the corresponding _create function.
        let t = unsafe { &*transform };
        let t2d: Transform2D = (*t).into();
        t2d.transform_point(Vec2::new(point_x, point_y)).into()
    }

    /// Transforms a direction from local space to world space.
    pub fn transform_direction(
        transform: *const FfiTransform2D,
        dir_x: f32,
        dir_y: f32,
    ) -> FfiVec2 {
        if transform.is_null() {
            return FfiVec2 { x: dir_x, y: dir_y };
        }
        // SAFETY: Pointer checked non-null above; allocated by the corresponding _create function.
        let t = unsafe { &*transform };
        let t2d: Transform2D = (*t).into();
        t2d.transform_direction(Vec2::new(dir_x, dir_y)).into()
    }

    /// Transforms a point from world space to local space.
    pub fn inverse_transform_point(
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
        // SAFETY: Pointer checked non-null above; allocated by the corresponding _create function.
        let t = unsafe { &*transform };
        let t2d: Transform2D = (*t).into();
        t2d.inverse_transform_point(Vec2::new(point_x, point_y))
            .into()
    }

    /// Transforms a direction from world space to local space.
    pub fn inverse_transform_direction(
        transform: *const FfiTransform2D,
        dir_x: f32,
        dir_y: f32,
    ) -> FfiVec2 {
        if transform.is_null() {
            return FfiVec2 { x: dir_x, y: dir_y };
        }
        // SAFETY: Pointer checked non-null above; allocated by the corresponding _create function.
        let t = unsafe { &*transform };
        let t2d: Transform2D = (*t).into();
        t2d.inverse_transform_direction(Vec2::new(dir_x, dir_y))
            .into()
    }
}
