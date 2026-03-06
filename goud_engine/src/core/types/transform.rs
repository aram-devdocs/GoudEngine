//! FFI-safe Transform2D and matrix types.

use crate::core::math::Vec2;
use crate::ecs::components::transform2d::Mat3x3;
use crate::ecs::components::Transform2D;

// =============================================================================
// Transform2D Types
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

/// Heap-allocated transform builder for FFI use.
///
/// This builder allows constructing a transform by setting properties one
/// at a time without copying the entire struct on each modification.
#[repr(C)]
pub struct FfiTransform2DBuilder {
    /// The transform being built.
    pub transform: FfiTransform2D,
}
