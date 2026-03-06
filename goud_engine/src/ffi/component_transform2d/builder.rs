//! Builder pattern FFI functions for heap-allocated Transform2D construction.
//!
//! The builder pattern provides a heap-allocated mutable builder that allows
//! chaining operations without copying the full struct on each call. This is
//! useful when constructing complex transforms with many properties.
//!
//! ## Usage (C#)
//!
//! ```csharp
//! var builder = goud_transform2d_builder_new();
//! builder = goud_transform2d_builder_with_position(builder, 100.0f, 50.0f);
//! builder = goud_transform2d_builder_with_rotation(builder, 0.785f);
//! builder = goud_transform2d_builder_with_scale(builder, 2.0f, 2.0f);
//! var transform = goud_transform2d_builder_build(builder); // Consumes builder
//! ```
//!
//! ## Memory Management
//!
//! - `goud_transform2d_builder_new()` allocates a builder on the heap
//! - `goud_transform2d_builder_build()` consumes the builder and frees memory
//! - If you don't call build(), call `goud_transform2d_builder_free()` to clean up
//! - Builder functions return the same pointer for chaining

use super::factory::{goud_transform2d_default, goud_transform2d_from_position};
use crate::core::types::{FfiTransform2D, FfiTransform2DBuilder};

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
    // SAFETY: builder is non-null and was allocated by Box::into_raw in builder_new.
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
        // SAFETY: builder is non-null and was allocated by Box::into_raw in builder_new.
        drop(Box::from_raw(builder));
    }
}
