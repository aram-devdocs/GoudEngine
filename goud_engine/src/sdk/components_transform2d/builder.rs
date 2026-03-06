//! Builder pattern for constructing Transform2D via heap allocation.

use crate::core::math::Vec2;
use crate::core::types::{FfiTransform2D, FfiTransform2DBuilder};
use crate::ecs::components::Transform2D;

/// Zero-sized type for Transform2D builder operations.
pub struct Transform2DBuilderOps;

// NOTE: FFI wrappers are hand-written in ffi/component_transform2d.rs. The `#[goud_api]`
// attribute is omitted here to avoid duplicate `#[no_mangle]` symbol conflicts.
impl Transform2DBuilderOps {
    /// Creates a new transform builder with default values.
    pub fn builder_new() -> *mut FfiTransform2DBuilder {
        let builder = FfiTransform2DBuilder {
            transform: Transform2D::default().into(),
        };
        Box::into_raw(Box::new(builder))
    }

    /// Creates a new transform builder at a specific position.
    pub fn builder_at_position(x: f32, y: f32) -> *mut FfiTransform2DBuilder {
        let builder = FfiTransform2DBuilder {
            transform: Transform2D::from_position(Vec2::new(x, y)).into(),
        };
        Box::into_raw(Box::new(builder))
    }

    /// Sets the position on the builder.
    pub fn builder_with_position(
        builder: *mut FfiTransform2DBuilder,
        x: f32,
        y: f32,
    ) -> *mut FfiTransform2DBuilder {
        if builder.is_null() {
            return builder;
        }
        // SAFETY: Caller guarantees pointer from builder_new.
        let t = unsafe { &mut (*builder).transform };
        t.position_x = x;
        t.position_y = y;
        builder
    }

    /// Sets the rotation (radians) on the builder.
    pub fn builder_with_rotation(
        builder: *mut FfiTransform2DBuilder,
        rotation: f32,
    ) -> *mut FfiTransform2DBuilder {
        if builder.is_null() {
            return builder;
        }
        // SAFETY: Pointer checked non-null above; allocated by builder_new via Box::into_raw.
        unsafe { (*builder).transform.rotation = rotation };
        builder
    }

    /// Sets the rotation (degrees) on the builder.
    pub fn builder_with_rotation_degrees(
        builder: *mut FfiTransform2DBuilder,
        degrees: f32,
    ) -> *mut FfiTransform2DBuilder {
        if builder.is_null() {
            return builder;
        }
        // SAFETY: Pointer checked non-null above; allocated by builder_new via Box::into_raw.
        unsafe { (*builder).transform.rotation = degrees.to_radians() };
        builder
    }

    /// Sets the scale on the builder.
    pub fn builder_with_scale(
        builder: *mut FfiTransform2DBuilder,
        scale_x: f32,
        scale_y: f32,
    ) -> *mut FfiTransform2DBuilder {
        if builder.is_null() {
            return builder;
        }
        // SAFETY: Pointer checked non-null above; allocated by builder_new via Box::into_raw.
        let t = unsafe { &mut (*builder).transform };
        t.scale_x = scale_x;
        t.scale_y = scale_y;
        builder
    }

    /// Sets uniform scale on the builder.
    pub fn builder_with_scale_uniform(
        builder: *mut FfiTransform2DBuilder,
        scale: f32,
    ) -> *mut FfiTransform2DBuilder {
        if builder.is_null() {
            return builder;
        }
        // SAFETY: Pointer checked non-null above; allocated by builder_new via Box::into_raw.
        let t = unsafe { &mut (*builder).transform };
        t.scale_x = scale;
        t.scale_y = scale;
        builder
    }

    /// Makes the builder's transform look at a target.
    pub fn builder_looking_at(
        builder: *mut FfiTransform2DBuilder,
        target_x: f32,
        target_y: f32,
    ) -> *mut FfiTransform2DBuilder {
        if builder.is_null() {
            return builder;
        }
        // SAFETY: Pointer checked non-null above; allocated by builder_new via Box::into_raw.
        let t = unsafe { &mut (*builder).transform };
        let dx = target_x - t.position_x;
        let dy = target_y - t.position_y;
        t.rotation = dy.atan2(dx);
        builder
    }

    /// Translates the builder's position.
    pub fn builder_translate(
        builder: *mut FfiTransform2DBuilder,
        dx: f32,
        dy: f32,
    ) -> *mut FfiTransform2DBuilder {
        if builder.is_null() {
            return builder;
        }
        // SAFETY: Pointer checked non-null above; allocated by builder_new via Box::into_raw.
        let t = unsafe { &mut (*builder).transform };
        t.position_x += dx;
        t.position_y += dy;
        builder
    }

    /// Rotates the builder's transform.
    pub fn builder_rotate(
        builder: *mut FfiTransform2DBuilder,
        angle: f32,
    ) -> *mut FfiTransform2DBuilder {
        if builder.is_null() {
            return builder;
        }
        // SAFETY: Pointer checked non-null above; allocated by builder_new via Box::into_raw.
        unsafe { (*builder).transform.rotation += angle };
        builder
    }

    /// Multiplies the builder's scale.
    pub fn builder_scale_by(
        builder: *mut FfiTransform2DBuilder,
        factor_x: f32,
        factor_y: f32,
    ) -> *mut FfiTransform2DBuilder {
        if builder.is_null() {
            return builder;
        }
        // SAFETY: Pointer checked non-null above; allocated by builder_new via Box::into_raw.
        let t = unsafe { &mut (*builder).transform };
        t.scale_x *= factor_x;
        t.scale_y *= factor_y;
        builder
    }

    /// Builds the transform, consuming and freeing the builder.
    pub fn builder_build(builder: *mut FfiTransform2DBuilder) -> FfiTransform2D {
        if builder.is_null() {
            return Transform2D::default().into();
        }
        // SAFETY: Takes ownership from builder_new allocation.
        let boxed = unsafe { Box::from_raw(builder) };
        boxed.transform
    }

    /// Frees a transform builder without building.
    pub fn builder_free(builder: *mut FfiTransform2DBuilder) {
        if !builder.is_null() {
            // SAFETY: Takes ownership and drops.
            drop(unsafe { Box::from_raw(builder) });
        }
    }
}
