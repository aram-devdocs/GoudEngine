//! FFI-safe Transform2D and matrix types.

// =============================================================================
// Transform2D Types
// =============================================================================

/// FFI-safe Transform2D representation.
///
/// This matches the memory layout of `Transform2D` exactly and is used
/// for passing transforms across FFI boundaries.
///
/// The `From<Transform2D>` and `Into<Transform2D>` impls live in the
/// `ecs::components::transform2d` module (higher layer) to avoid a
/// Foundation→Services dependency.
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

/// FFI-safe Mat3x3 representation (9 floats in column-major order).
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct FfiMat3x3 {
    /// Matrix elements in column-major order.
    pub m: [f32; 9],
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
