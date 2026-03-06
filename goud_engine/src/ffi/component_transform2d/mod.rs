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
//!
//! ## Submodules
//!
//! - `factory` — Constructor/factory functions
//! - `position` — Position get/set/translate operations
//! - `rotation` — Rotation get/set/rotate operations
//! - `scale` — Scale get/set operations
//! - `direction` — Direction vector queries (forward, right, backward, left)
//! - `matrix_ops` — Matrix generation, point transformation, interpolation, utility
//! - `builder` — Heap-allocated builder pattern

pub mod builder;
pub mod direction;
pub mod factory;
pub mod matrix_ops;
pub mod position;
pub mod rotation;
pub mod scale;

#[cfg(test)]
mod tests;

// Re-export types from core for backward compatibility
pub use crate::core::types::{FfiMat3x3, FfiTransform2D, FfiTransform2DBuilder};

// Re-export all public FFI functions so callers of `component_transform2d::*` work unchanged.
pub use builder::{
    goud_transform2d_builder_at_position, goud_transform2d_builder_build,
    goud_transform2d_builder_free, goud_transform2d_builder_looking_at,
    goud_transform2d_builder_new, goud_transform2d_builder_rotate,
    goud_transform2d_builder_scale_by, goud_transform2d_builder_translate,
    goud_transform2d_builder_with_position, goud_transform2d_builder_with_rotation,
    goud_transform2d_builder_with_rotation_degrees, goud_transform2d_builder_with_scale,
    goud_transform2d_builder_with_scale_uniform,
};
pub use direction::{
    goud_transform2d_backward, goud_transform2d_forward, goud_transform2d_left,
    goud_transform2d_right,
};
pub use factory::{
    goud_transform2d_default, goud_transform2d_from_position,
    goud_transform2d_from_position_rotation, goud_transform2d_from_rotation,
    goud_transform2d_from_rotation_degrees, goud_transform2d_from_scale,
    goud_transform2d_from_scale_uniform, goud_transform2d_look_at, goud_transform2d_new,
};
pub use matrix_ops::{
    goud_transform2d_inverse_transform_direction, goud_transform2d_inverse_transform_point,
    goud_transform2d_lerp, goud_transform2d_matrix, goud_transform2d_matrix_inverse,
    goud_transform2d_normalize_angle, goud_transform2d_transform_direction,
    goud_transform2d_transform_point,
};
pub use position::{
    goud_transform2d_get_position, goud_transform2d_set_position, goud_transform2d_translate,
    goud_transform2d_translate_local,
};
pub use rotation::{
    goud_transform2d_get_rotation, goud_transform2d_get_rotation_degrees,
    goud_transform2d_look_at_target, goud_transform2d_rotate, goud_transform2d_rotate_degrees,
    goud_transform2d_set_rotation, goud_transform2d_set_rotation_degrees,
};
pub use scale::{
    goud_transform2d_get_scale, goud_transform2d_scale_by, goud_transform2d_set_scale,
    goud_transform2d_set_scale_uniform,
};
