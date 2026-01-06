//! GlobalTransform component for world-space transformations.
//!
//! The [`GlobalTransform`] component stores the computed world-space transformation
//! for entities in a hierarchy. Unlike [`Transform`] which stores local-space data
//! relative to the parent, `GlobalTransform` stores the absolute world-space result.
//!
//! # Purpose
//!
//! When entities are arranged in a parent-child hierarchy, each child's [`Transform`]
//! is relative to its parent. To render, perform physics, or do other world-space
//! operations, we need the final world-space transformation.
//!
//! For example:
//! - Parent at position (10, 0, 0)
//! - Child at local position (5, 0, 0)
//! - Child's world position is (15, 0, 0)
//!
//! The transform propagation system computes these world-space values and stores
//! them in `GlobalTransform`.
//!
//! # Usage
//!
//! `GlobalTransform` is typically:
//! 1. Added automatically when spawning entities with `Transform`
//! 2. Updated by the transform propagation system each frame
//! 3. Read by rendering systems, physics, etc.
//!
//! **Never modify `GlobalTransform` directly.** Always modify `Transform` and let
//! the propagation system compute the global value.
//!
//! ```
//! use goud_engine::ecs::components::{Transform, GlobalTransform};
//! use goud_engine::core::math::Vec3;
//!
//! // Create local transform
//! let local = Transform::from_position(Vec3::new(5.0, 0.0, 0.0));
//!
//! // GlobalTransform would be computed by the propagation system
//! // For a root entity, it equals the local transform
//! let global = GlobalTransform::from(local);
//!
//! assert_eq!(global.translation(), Vec3::new(5.0, 0.0, 0.0));
//! ```
//!
//! # Memory Layout
//!
//! GlobalTransform stores a pre-computed 4x4 affine transformation matrix (64 bytes).
//! While this uses more memory than Transform (40 bytes), it provides:
//!
//! - **Direct use**: Matrix can be sent to GPU without further computation
//! - **Composability**: Easy to combine with parent transforms
//! - **Decomposability**: Can extract position/rotation/scale when needed
//!
//! # FFI Safety
//!
//! GlobalTransform uses cgmath's `Matrix4<f32>` internally which is column-major.
//! For FFI, use the `to_cols_array` method to get a flat `[f32; 16]` array.

use crate::core::math::{Matrix4, Vec3};
use crate::ecs::components::transform::{Quat, Transform};
use crate::ecs::Component;
use cgmath::{InnerSpace, SquareMatrix};
use std::fmt;

/// A world-space transformation component.
///
/// This component caches the computed world-space transformation matrix for
/// entities in a hierarchy. It is computed by the transform propagation system
/// based on the entity's local [`Transform`] and its parent's `GlobalTransform`.
///
/// # When to Use
///
/// - Use `Transform` for setting local position/rotation/scale
/// - Use `GlobalTransform` for reading world-space values (rendering, physics)
///
/// # Do Not Modify Directly
///
/// This component is managed by the transform propagation system. Modifying it
/// directly will cause desynchronization with the entity hierarchy.
///
/// # Example
///
/// ```
/// use goud_engine::ecs::components::{Transform, GlobalTransform};
/// use goud_engine::core::math::Vec3;
///
/// // For root entities, global equals local
/// let transform = Transform::from_position(Vec3::new(10.0, 5.0, 0.0));
/// let global = GlobalTransform::from(transform);
///
/// let position = global.translation();
/// assert!((position - Vec3::new(10.0, 5.0, 0.0)).length() < 0.001);
/// ```
#[derive(Clone, Copy, PartialEq)]
pub struct GlobalTransform {
    /// The computed world-space transformation matrix.
    ///
    /// This is a column-major 4x4 affine transformation matrix.
    matrix: Matrix4<f32>,
}

impl GlobalTransform {
    /// Identity global transform (no transformation).
    pub const IDENTITY: GlobalTransform = GlobalTransform {
        matrix: Matrix4::new(
            1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
        ),
    };

    /// Creates a GlobalTransform from a 4x4 transformation matrix.
    ///
    /// # Arguments
    ///
    /// * `matrix` - The world-space transformation matrix
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::components::GlobalTransform;
    /// use cgmath::Matrix4;
    ///
    /// let matrix = Matrix4::from_translation(cgmath::Vector3::new(10.0, 0.0, 0.0));
    /// let global = GlobalTransform::from_matrix(matrix);
    /// ```
    #[inline]
    pub const fn from_matrix(matrix: Matrix4<f32>) -> Self {
        Self { matrix }
    }

    /// Creates a GlobalTransform from translation only.
    ///
    /// # Arguments
    ///
    /// * `translation` - World-space position
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::components::GlobalTransform;
    /// use goud_engine::core::math::Vec3;
    ///
    /// let global = GlobalTransform::from_translation(Vec3::new(10.0, 5.0, 0.0));
    /// ```
    #[inline]
    pub fn from_translation(translation: Vec3) -> Self {
        let matrix = Matrix4::from_translation(cgmath::Vector3::new(
            translation.x,
            translation.y,
            translation.z,
        ));
        Self { matrix }
    }

    /// Creates a GlobalTransform from translation, rotation, and scale.
    ///
    /// # Arguments
    ///
    /// * `translation` - World-space position
    /// * `rotation` - World-space rotation as quaternion
    /// * `scale` - World-space scale
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::components::GlobalTransform;
    /// use goud_engine::ecs::components::transform::Quat;
    /// use goud_engine::core::math::Vec3;
    ///
    /// let global = GlobalTransform::from_translation_rotation_scale(
    ///     Vec3::new(10.0, 0.0, 0.0),
    ///     Quat::IDENTITY,
    ///     Vec3::one(),
    /// );
    /// ```
    #[inline]
    pub fn from_translation_rotation_scale(translation: Vec3, rotation: Quat, scale: Vec3) -> Self {
        // Build transform matrix: T * R * S
        let transform = Transform::new(translation, rotation, scale);
        Self {
            matrix: transform.matrix(),
        }
    }

    /// Returns the underlying 4x4 transformation matrix.
    ///
    /// This matrix is column-major and can be used directly for rendering.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::components::GlobalTransform;
    ///
    /// let global = GlobalTransform::IDENTITY;
    /// let matrix = global.matrix();
    /// ```
    #[inline]
    pub fn matrix(&self) -> Matrix4<f32> {
        self.matrix
    }

    /// Returns a reference to the underlying matrix.
    #[inline]
    pub fn matrix_ref(&self) -> &Matrix4<f32> {
        &self.matrix
    }

    /// Returns the matrix as a flat column-major array.
    ///
    /// This is useful for FFI and sending to GPU shaders.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::components::GlobalTransform;
    ///
    /// let global = GlobalTransform::IDENTITY;
    /// let cols: [f32; 16] = global.to_cols_array();
    ///
    /// // First column (x-axis)
    /// assert_eq!(cols[0], 1.0); // m00
    /// ```
    #[inline]
    pub fn to_cols_array(&self) -> [f32; 16] {
        [
            self.matrix.x.x,
            self.matrix.x.y,
            self.matrix.x.z,
            self.matrix.x.w,
            self.matrix.y.x,
            self.matrix.y.y,
            self.matrix.y.z,
            self.matrix.y.w,
            self.matrix.z.x,
            self.matrix.z.y,
            self.matrix.z.z,
            self.matrix.z.w,
            self.matrix.w.x,
            self.matrix.w.y,
            self.matrix.w.z,
            self.matrix.w.w,
        ]
    }

    /// Returns the matrix as a flat row-major array.
    ///
    /// Some APIs expect row-major ordering.
    #[inline]
    pub fn to_rows_array(&self) -> [f32; 16] {
        [
            self.matrix.x.x,
            self.matrix.y.x,
            self.matrix.z.x,
            self.matrix.w.x,
            self.matrix.x.y,
            self.matrix.y.y,
            self.matrix.z.y,
            self.matrix.w.y,
            self.matrix.x.z,
            self.matrix.y.z,
            self.matrix.z.z,
            self.matrix.w.z,
            self.matrix.x.w,
            self.matrix.y.w,
            self.matrix.z.w,
            self.matrix.w.w,
        ]
    }

    // =========================================================================
    // Decomposition Methods
    // =========================================================================

    /// Extracts the translation (position) component.
    ///
    /// This is the fourth column of the matrix.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::components::GlobalTransform;
    /// use goud_engine::core::math::Vec3;
    ///
    /// let global = GlobalTransform::from_translation(Vec3::new(10.0, 5.0, 3.0));
    /// let pos = global.translation();
    ///
    /// assert!((pos.x - 10.0).abs() < 0.001);
    /// ```
    #[inline]
    pub fn translation(&self) -> Vec3 {
        Vec3::new(self.matrix.w.x, self.matrix.w.y, self.matrix.w.z)
    }

    /// Extracts the scale component.
    ///
    /// This is computed from the lengths of the first three column vectors.
    /// Note: This does not handle negative scales correctly.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::components::GlobalTransform;
    /// use goud_engine::ecs::components::transform::Quat;
    /// use goud_engine::core::math::Vec3;
    ///
    /// let global = GlobalTransform::from_translation_rotation_scale(
    ///     Vec3::zero(),
    ///     Quat::IDENTITY,
    ///     Vec3::new(2.0, 3.0, 4.0),
    /// );
    ///
    /// let scale = global.scale();
    /// assert!((scale.x - 2.0).abs() < 0.001);
    /// ```
    #[inline]
    pub fn scale(&self) -> Vec3 {
        let scale_x =
            cgmath::Vector3::new(self.matrix.x.x, self.matrix.x.y, self.matrix.x.z).magnitude();
        let scale_y =
            cgmath::Vector3::new(self.matrix.y.x, self.matrix.y.y, self.matrix.y.z).magnitude();
        let scale_z =
            cgmath::Vector3::new(self.matrix.z.x, self.matrix.z.y, self.matrix.z.z).magnitude();
        Vec3::new(scale_x, scale_y, scale_z)
    }

    /// Extracts the rotation component as a quaternion.
    ///
    /// This removes scale from the rotation matrix, then converts to quaternion.
    /// Note: This may have issues with non-uniform scales or negative scales.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::components::GlobalTransform;
    /// use goud_engine::ecs::components::transform::Quat;
    /// use goud_engine::core::math::Vec3;
    /// use std::f32::consts::PI;
    ///
    /// let rotation = Quat::from_axis_angle(Vec3::unit_y(), PI / 4.0);
    /// let global = GlobalTransform::from_translation_rotation_scale(
    ///     Vec3::zero(),
    ///     rotation,
    ///     Vec3::one(),
    /// );
    ///
    /// let extracted = global.rotation();
    /// // Quaternion comparison (accounting for sign flip equivalence)
    /// let dot = rotation.x * extracted.x + rotation.y * extracted.y
    ///         + rotation.z * extracted.z + rotation.w * extracted.w;
    /// assert!(dot.abs() > 0.999);
    /// ```
    #[inline]
    pub fn rotation(&self) -> Quat {
        // Extract scale
        let scale = self.scale();

        // Build rotation matrix by normalizing columns
        let m00 = if scale.x != 0.0 {
            self.matrix.x.x / scale.x
        } else {
            1.0
        };
        let m01 = if scale.y != 0.0 {
            self.matrix.y.x / scale.y
        } else {
            0.0
        };
        let m02 = if scale.z != 0.0 {
            self.matrix.z.x / scale.z
        } else {
            0.0
        };

        let m10 = if scale.x != 0.0 {
            self.matrix.x.y / scale.x
        } else {
            0.0
        };
        let m11 = if scale.y != 0.0 {
            self.matrix.y.y / scale.y
        } else {
            1.0
        };
        let m12 = if scale.z != 0.0 {
            self.matrix.z.y / scale.z
        } else {
            0.0
        };

        let m20 = if scale.x != 0.0 {
            self.matrix.x.z / scale.x
        } else {
            0.0
        };
        let m21 = if scale.y != 0.0 {
            self.matrix.y.z / scale.y
        } else {
            0.0
        };
        let m22 = if scale.z != 0.0 {
            self.matrix.z.z / scale.z
        } else {
            1.0
        };

        // Convert rotation matrix to quaternion
        let trace = m00 + m11 + m22;
        if trace > 0.0 {
            let s = (trace + 1.0).sqrt() * 2.0;
            Quat::new((m21 - m12) / s, (m02 - m20) / s, (m10 - m01) / s, 0.25 * s).normalize()
        } else if m00 > m11 && m00 > m22 {
            let s = (1.0 + m00 - m11 - m22).sqrt() * 2.0;
            Quat::new(0.25 * s, (m01 + m10) / s, (m02 + m20) / s, (m21 - m12) / s).normalize()
        } else if m11 > m22 {
            let s = (1.0 + m11 - m00 - m22).sqrt() * 2.0;
            Quat::new((m01 + m10) / s, 0.25 * s, (m12 + m21) / s, (m02 - m20) / s).normalize()
        } else {
            let s = (1.0 + m22 - m00 - m11).sqrt() * 2.0;
            Quat::new((m02 + m20) / s, (m12 + m21) / s, 0.25 * s, (m10 - m01) / s).normalize()
        }
    }

    /// Decomposes the transform into translation, rotation, and scale.
    ///
    /// Returns `(translation, rotation, scale)`.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::components::GlobalTransform;
    /// use goud_engine::ecs::components::transform::Quat;
    /// use goud_engine::core::math::Vec3;
    ///
    /// let global = GlobalTransform::from_translation_rotation_scale(
    ///     Vec3::new(10.0, 5.0, 3.0),
    ///     Quat::IDENTITY,
    ///     Vec3::new(2.0, 2.0, 2.0),
    /// );
    ///
    /// let (translation, rotation, scale) = global.decompose();
    /// assert!((translation.x - 10.0).abs() < 0.001);
    /// assert!((scale.x - 2.0).abs() < 0.001);
    /// ```
    #[inline]
    pub fn decompose(&self) -> (Vec3, Quat, Vec3) {
        (self.translation(), self.rotation(), self.scale())
    }

    /// Converts this GlobalTransform to a local Transform.
    ///
    /// This is useful for extracting a Transform that would produce this
    /// GlobalTransform when applied from the origin.
    #[inline]
    pub fn to_transform(&self) -> Transform {
        let (translation, rotation, scale) = self.decompose();
        Transform::new(translation, rotation, scale)
    }

    // =========================================================================
    // Transform Operations
    // =========================================================================

    /// Multiplies this transform with another.
    ///
    /// This combines two transformations: `self * other` applies `self` first,
    /// then `other`.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::components::GlobalTransform;
    /// use goud_engine::core::math::Vec3;
    ///
    /// let parent = GlobalTransform::from_translation(Vec3::new(10.0, 0.0, 0.0));
    /// let child_local = GlobalTransform::from_translation(Vec3::new(5.0, 0.0, 0.0));
    ///
    /// let child_global = parent.mul_transform(&child_local);
    /// let pos = child_global.translation();
    /// assert!((pos.x - 15.0).abs() < 0.001);
    /// ```
    #[inline]
    pub fn mul_transform(&self, other: &GlobalTransform) -> GlobalTransform {
        GlobalTransform {
            matrix: self.matrix * other.matrix,
        }
    }

    /// Multiplies this transform by a local Transform.
    ///
    /// This is the primary method used by transform propagation.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::components::{GlobalTransform, Transform};
    /// use goud_engine::core::math::Vec3;
    ///
    /// let parent_global = GlobalTransform::from_translation(Vec3::new(10.0, 0.0, 0.0));
    /// let child_local = Transform::from_position(Vec3::new(5.0, 0.0, 0.0));
    ///
    /// let child_global = parent_global.transform_by(&child_local);
    /// let pos = child_global.translation();
    /// assert!((pos.x - 15.0).abs() < 0.001);
    /// ```
    #[inline]
    pub fn transform_by(&self, local: &Transform) -> GlobalTransform {
        GlobalTransform {
            matrix: self.matrix * local.matrix(),
        }
    }

    /// Transforms a point from local space to world space.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::components::GlobalTransform;
    /// use goud_engine::core::math::Vec3;
    ///
    /// let global = GlobalTransform::from_translation(Vec3::new(10.0, 0.0, 0.0));
    /// let local_point = Vec3::new(5.0, 0.0, 0.0);
    /// let world_point = global.transform_point(local_point);
    ///
    /// assert!((world_point.x - 15.0).abs() < 0.001);
    /// ```
    #[inline]
    pub fn transform_point(&self, point: Vec3) -> Vec3 {
        let p = cgmath::Vector4::new(point.x, point.y, point.z, 1.0);
        let result = self.matrix * p;
        Vec3::new(result.x, result.y, result.z)
    }

    /// Transforms a direction from local space to world space.
    ///
    /// Unlike points, directions are not affected by translation.
    #[inline]
    pub fn transform_direction(&self, direction: Vec3) -> Vec3 {
        let d = cgmath::Vector4::new(direction.x, direction.y, direction.z, 0.0);
        let result = self.matrix * d;
        Vec3::new(result.x, result.y, result.z)
    }

    /// Returns the inverse of this transform.
    ///
    /// The inverse transforms from world space back to local space.
    /// Returns `None` if the matrix is not invertible (e.g., has zero scale).
    #[inline]
    pub fn inverse(&self) -> Option<GlobalTransform> {
        self.matrix.invert().map(|m| GlobalTransform { matrix: m })
    }

    // =========================================================================
    // Direction Vectors
    // =========================================================================

    /// Returns the forward direction vector (negative Z in local space).
    #[inline]
    pub fn forward(&self) -> Vec3 {
        self.transform_direction(Vec3::new(0.0, 0.0, -1.0))
            .normalize()
    }

    /// Returns the right direction vector (positive X in local space).
    #[inline]
    pub fn right(&self) -> Vec3 {
        self.transform_direction(Vec3::new(1.0, 0.0, 0.0))
            .normalize()
    }

    /// Returns the up direction vector (positive Y in local space).
    #[inline]
    pub fn up(&self) -> Vec3 {
        self.transform_direction(Vec3::new(0.0, 1.0, 0.0))
            .normalize()
    }

    /// Returns the back direction vector (positive Z in local space).
    #[inline]
    pub fn back(&self) -> Vec3 {
        self.transform_direction(Vec3::new(0.0, 0.0, 1.0))
            .normalize()
    }

    /// Returns the left direction vector (negative X in local space).
    #[inline]
    pub fn left(&self) -> Vec3 {
        self.transform_direction(Vec3::new(-1.0, 0.0, 0.0))
            .normalize()
    }

    /// Returns the down direction vector (negative Y in local space).
    #[inline]
    pub fn down(&self) -> Vec3 {
        self.transform_direction(Vec3::new(0.0, -1.0, 0.0))
            .normalize()
    }

    // =========================================================================
    // Interpolation
    // =========================================================================

    /// Linearly interpolates between two global transforms.
    ///
    /// This decomposes both transforms, interpolates components separately
    /// (slerp for rotation), then recomposes.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::components::GlobalTransform;
    /// use goud_engine::core::math::Vec3;
    ///
    /// let a = GlobalTransform::from_translation(Vec3::zero());
    /// let b = GlobalTransform::from_translation(Vec3::new(10.0, 0.0, 0.0));
    ///
    /// let mid = a.lerp(&b, 0.5);
    /// let pos = mid.translation();
    /// assert!((pos.x - 5.0).abs() < 0.001);
    /// ```
    #[inline]
    pub fn lerp(&self, other: &GlobalTransform, t: f32) -> GlobalTransform {
        let (t1, r1, s1) = self.decompose();
        let (t2, r2, s2) = other.decompose();

        GlobalTransform::from_translation_rotation_scale(
            t1.lerp(t2, t),
            r1.slerp(r2, t),
            s1.lerp(s2, t),
        )
    }
}

impl Default for GlobalTransform {
    #[inline]
    fn default() -> Self {
        Self::IDENTITY
    }
}

impl fmt::Debug for GlobalTransform {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (t, r, s) = self.decompose();
        f.debug_struct("GlobalTransform")
            .field("translation", &(t.x, t.y, t.z))
            .field(
                "rotation",
                &format!("Quat({:.3}, {:.3}, {:.3}, {:.3})", r.x, r.y, r.z, r.w),
            )
            .field("scale", &(s.x, s.y, s.z))
            .finish()
    }
}

impl From<Transform> for GlobalTransform {
    /// Converts a local Transform to a GlobalTransform.
    ///
    /// This is used for root entities where local == global.
    #[inline]
    fn from(transform: Transform) -> Self {
        GlobalTransform {
            matrix: transform.matrix(),
        }
    }
}

impl From<&Transform> for GlobalTransform {
    #[inline]
    fn from(transform: &Transform) -> Self {
        GlobalTransform {
            matrix: transform.matrix(),
        }
    }
}

impl From<Matrix4<f32>> for GlobalTransform {
    #[inline]
    fn from(matrix: Matrix4<f32>) -> Self {
        GlobalTransform { matrix }
    }
}

// Implement Component trait
impl Component for GlobalTransform {}

// Implement multiplication operators
impl std::ops::Mul for GlobalTransform {
    type Output = GlobalTransform;

    #[inline]
    fn mul(self, rhs: GlobalTransform) -> GlobalTransform {
        self.mul_transform(&rhs)
    }
}

impl std::ops::Mul<&GlobalTransform> for GlobalTransform {
    type Output = GlobalTransform;

    #[inline]
    fn mul(self, rhs: &GlobalTransform) -> GlobalTransform {
        self.mul_transform(rhs)
    }
}

impl std::ops::Mul<Transform> for GlobalTransform {
    type Output = GlobalTransform;

    #[inline]
    fn mul(self, rhs: Transform) -> GlobalTransform {
        self.transform_by(&rhs)
    }
}

impl std::ops::Mul<&Transform> for GlobalTransform {
    type Output = GlobalTransform;

    #[inline]
    fn mul(self, rhs: &Transform) -> GlobalTransform {
        self.transform_by(rhs)
    }
}

// =============================================================================
// Unit Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::FRAC_PI_2;
    use std::f32::consts::FRAC_PI_4;

    mod construction_tests {
        use super::*;

        #[test]
        fn test_identity() {
            let global = GlobalTransform::IDENTITY;
            let pos = global.translation();
            let scale = global.scale();

            assert!((pos.x).abs() < 0.0001);
            assert!((pos.y).abs() < 0.0001);
            assert!((pos.z).abs() < 0.0001);
            assert!((scale.x - 1.0).abs() < 0.0001);
            assert!((scale.y - 1.0).abs() < 0.0001);
            assert!((scale.z - 1.0).abs() < 0.0001);
        }

        #[test]
        fn test_default() {
            let global: GlobalTransform = Default::default();
            assert_eq!(global, GlobalTransform::IDENTITY);
        }

        #[test]
        fn test_from_translation() {
            let global = GlobalTransform::from_translation(Vec3::new(10.0, 20.0, 30.0));
            let pos = global.translation();

            assert!((pos.x - 10.0).abs() < 0.0001);
            assert!((pos.y - 20.0).abs() < 0.0001);
            assert!((pos.z - 30.0).abs() < 0.0001);
        }

        #[test]
        fn test_from_translation_rotation_scale() {
            let rotation = Quat::from_axis_angle(Vec3::unit_y(), FRAC_PI_4);
            let global = GlobalTransform::from_translation_rotation_scale(
                Vec3::new(5.0, 10.0, 15.0),
                rotation,
                Vec3::new(2.0, 3.0, 4.0),
            );

            let pos = global.translation();
            let scale = global.scale();

            assert!((pos.x - 5.0).abs() < 0.0001);
            assert!((pos.y - 10.0).abs() < 0.0001);
            assert!((pos.z - 15.0).abs() < 0.0001);
            assert!((scale.x - 2.0).abs() < 0.0001);
            assert!((scale.y - 3.0).abs() < 0.0001);
            assert!((scale.z - 4.0).abs() < 0.0001);
        }

        #[test]
        fn test_from_transform() {
            let transform = Transform::new(
                Vec3::new(1.0, 2.0, 3.0),
                Quat::from_axis_angle(Vec3::unit_y(), FRAC_PI_4),
                Vec3::new(2.0, 2.0, 2.0),
            );
            let global: GlobalTransform = transform.into();

            let pos = global.translation();
            assert!((pos.x - 1.0).abs() < 0.0001);
            assert!((pos.y - 2.0).abs() < 0.0001);
            assert!((pos.z - 3.0).abs() < 0.0001);
        }

        #[test]
        fn test_from_transform_ref() {
            let transform = Transform::from_position(Vec3::new(5.0, 0.0, 0.0));
            let global: GlobalTransform = (&transform).into();
            let pos = global.translation();
            assert!((pos.x - 5.0).abs() < 0.0001);
        }
    }

    mod decomposition_tests {
        use super::*;

        #[test]
        fn test_translation_extraction() {
            let global = GlobalTransform::from_translation(Vec3::new(1.0, 2.0, 3.0));
            let pos = global.translation();
            assert!((pos.x - 1.0).abs() < 0.0001);
            assert!((pos.y - 2.0).abs() < 0.0001);
            assert!((pos.z - 3.0).abs() < 0.0001);
        }

        #[test]
        fn test_scale_extraction() {
            let global = GlobalTransform::from_translation_rotation_scale(
                Vec3::zero(),
                Quat::IDENTITY,
                Vec3::new(2.0, 3.0, 4.0),
            );
            let scale = global.scale();
            assert!((scale.x - 2.0).abs() < 0.0001);
            assert!((scale.y - 3.0).abs() < 0.0001);
            assert!((scale.z - 4.0).abs() < 0.0001);
        }

        #[test]
        fn test_rotation_extraction() {
            let original = Quat::from_axis_angle(Vec3::unit_y(), FRAC_PI_4);
            let global = GlobalTransform::from_translation_rotation_scale(
                Vec3::zero(),
                original,
                Vec3::one(),
            );
            let extracted = global.rotation();

            // Compare quaternions (accounting for sign flip)
            let dot = original.x * extracted.x
                + original.y * extracted.y
                + original.z * extracted.z
                + original.w * extracted.w;
            assert!(dot.abs() > 0.999);
        }

        #[test]
        fn test_decompose() {
            let original_t = Vec3::new(10.0, 5.0, 3.0);
            let original_r = Quat::from_axis_angle(Vec3::unit_x(), FRAC_PI_4);
            let original_s = Vec3::new(2.0, 3.0, 4.0);

            let global = GlobalTransform::from_translation_rotation_scale(
                original_t, original_r, original_s,
            );
            let (t, r, s) = global.decompose();

            assert!((t - original_t).length() < 0.001);
            assert!((s - original_s).length() < 0.001);

            let dot =
                original_r.x * r.x + original_r.y * r.y + original_r.z * r.z + original_r.w * r.w;
            assert!(dot.abs() > 0.999);
        }

        #[test]
        fn test_to_transform() {
            let global = GlobalTransform::from_translation_rotation_scale(
                Vec3::new(5.0, 10.0, 15.0),
                Quat::IDENTITY,
                Vec3::new(2.0, 2.0, 2.0),
            );

            let transform = global.to_transform();
            assert!((transform.position - Vec3::new(5.0, 10.0, 15.0)).length() < 0.001);
            assert!((transform.scale - Vec3::new(2.0, 2.0, 2.0)).length() < 0.001);
        }
    }

    mod transform_tests {
        use super::*;

        #[test]
        fn test_mul_transform_translation() {
            let a = GlobalTransform::from_translation(Vec3::new(10.0, 0.0, 0.0));
            let b = GlobalTransform::from_translation(Vec3::new(5.0, 0.0, 0.0));
            let result = a.mul_transform(&b);

            let pos = result.translation();
            assert!((pos.x - 15.0).abs() < 0.0001);
        }

        #[test]
        fn test_mul_transform_scale() {
            let a = GlobalTransform::from_translation_rotation_scale(
                Vec3::zero(),
                Quat::IDENTITY,
                Vec3::new(2.0, 2.0, 2.0),
            );
            let b = GlobalTransform::from_translation(Vec3::new(5.0, 0.0, 0.0));
            let result = a.mul_transform(&b);

            let pos = result.translation();
            // Scale affects the child's translation
            assert!((pos.x - 10.0).abs() < 0.0001);
        }

        #[test]
        fn test_transform_by() {
            let parent = GlobalTransform::from_translation(Vec3::new(10.0, 0.0, 0.0));
            let child = Transform::from_position(Vec3::new(5.0, 0.0, 0.0));
            let result = parent.transform_by(&child);

            let pos = result.translation();
            assert!((pos.x - 15.0).abs() < 0.0001);
        }

        #[test]
        fn test_transform_point() {
            let global = GlobalTransform::from_translation(Vec3::new(10.0, 0.0, 0.0));
            let local_point = Vec3::new(5.0, 3.0, 0.0);
            let world_point = global.transform_point(local_point);

            assert!((world_point.x - 15.0).abs() < 0.0001);
            assert!((world_point.y - 3.0).abs() < 0.0001);
        }

        #[test]
        fn test_transform_direction() {
            let global = GlobalTransform::from_translation(Vec3::new(1000.0, 0.0, 0.0));
            let direction = Vec3::new(1.0, 0.0, 0.0);
            let world_dir = global.transform_direction(direction);

            // Direction should not be affected by translation
            assert!((world_dir.x - 1.0).abs() < 0.0001);
            assert!(world_dir.y.abs() < 0.0001);
        }

        #[test]
        fn test_inverse() {
            let global = GlobalTransform::from_translation_rotation_scale(
                Vec3::new(5.0, 10.0, 15.0),
                Quat::from_axis_angle(Vec3::unit_y(), FRAC_PI_4),
                Vec3::new(2.0, 2.0, 2.0),
            );

            let inverse = global.inverse().expect("Should be invertible");
            let identity = global.mul_transform(&inverse);

            // Should be close to identity
            let pos = identity.translation();
            assert!(pos.length() < 0.001);

            let scale = identity.scale();
            assert!((scale.x - 1.0).abs() < 0.001);
        }

        #[test]
        fn test_mul_operator() {
            let a = GlobalTransform::from_translation(Vec3::new(10.0, 0.0, 0.0));
            let b = GlobalTransform::from_translation(Vec3::new(5.0, 0.0, 0.0));
            let result = a * b;

            let pos = result.translation();
            assert!((pos.x - 15.0).abs() < 0.0001);
        }

        #[test]
        fn test_mul_operator_with_transform() {
            let parent = GlobalTransform::from_translation(Vec3::new(10.0, 0.0, 0.0));
            let child = Transform::from_position(Vec3::new(5.0, 0.0, 0.0));
            let result = parent * child;

            let pos = result.translation();
            assert!((pos.x - 15.0).abs() < 0.0001);
        }
    }

    mod direction_tests {
        use super::*;

        #[test]
        fn test_directions_identity() {
            let global = GlobalTransform::IDENTITY;

            assert!((global.forward() - Vec3::new(0.0, 0.0, -1.0)).length() < 0.0001);
            assert!((global.back() - Vec3::new(0.0, 0.0, 1.0)).length() < 0.0001);
            assert!((global.right() - Vec3::new(1.0, 0.0, 0.0)).length() < 0.0001);
            assert!((global.left() - Vec3::new(-1.0, 0.0, 0.0)).length() < 0.0001);
            assert!((global.up() - Vec3::new(0.0, 1.0, 0.0)).length() < 0.0001);
            assert!((global.down() - Vec3::new(0.0, -1.0, 0.0)).length() < 0.0001);
        }

        #[test]
        fn test_directions_rotated() {
            let global = GlobalTransform::from_translation_rotation_scale(
                Vec3::zero(),
                Quat::from_axis_angle(Vec3::unit_y(), FRAC_PI_2),
                Vec3::one(),
            );

            // After +90 degree Y rotation:
            // forward (-Z) -> -X
            let fwd = global.forward();
            assert!((fwd.x - (-1.0)).abs() < 0.0001);
            assert!(fwd.y.abs() < 0.0001);
            assert!(fwd.z.abs() < 0.0001);
        }
    }

    mod interpolation_tests {
        use super::*;

        #[test]
        fn test_lerp_translation() {
            let a = GlobalTransform::from_translation(Vec3::zero());
            let b = GlobalTransform::from_translation(Vec3::new(10.0, 0.0, 0.0));

            let mid = a.lerp(&b, 0.5);
            let pos = mid.translation();
            assert!((pos.x - 5.0).abs() < 0.0001);
        }

        #[test]
        fn test_lerp_endpoints() {
            let a = GlobalTransform::from_translation(Vec3::new(0.0, 0.0, 0.0));
            let b = GlobalTransform::from_translation(Vec3::new(10.0, 10.0, 10.0));

            let start = a.lerp(&b, 0.0);
            assert!((start.translation() - a.translation()).length() < 0.0001);

            let end = a.lerp(&b, 1.0);
            assert!((end.translation() - b.translation()).length() < 0.0001);
        }
    }

    mod array_tests {
        use super::*;

        #[test]
        fn test_to_cols_array() {
            let global = GlobalTransform::IDENTITY;
            let cols = global.to_cols_array();

            // Identity matrix
            assert_eq!(cols[0], 1.0); // m00
            assert_eq!(cols[5], 1.0); // m11
            assert_eq!(cols[10], 1.0); // m22
            assert_eq!(cols[15], 1.0); // m33
        }

        #[test]
        fn test_to_rows_array() {
            let global = GlobalTransform::from_translation(Vec3::new(10.0, 20.0, 30.0));
            let rows = global.to_rows_array();

            // Translation is in the last row for row-major
            assert!((rows[3] - 10.0).abs() < 0.0001);
            assert!((rows[7] - 20.0).abs() < 0.0001);
            assert!((rows[11] - 30.0).abs() < 0.0001);
        }
    }

    mod component_tests {
        use super::*;

        #[test]
        fn test_is_component() {
            fn assert_component<T: Component>() {}
            assert_component::<GlobalTransform>();
        }

        #[test]
        fn test_is_send() {
            fn assert_send<T: Send>() {}
            assert_send::<GlobalTransform>();
        }

        #[test]
        fn test_is_sync() {
            fn assert_sync<T: Sync>() {}
            assert_sync::<GlobalTransform>();
        }

        #[test]
        fn test_clone() {
            let global = GlobalTransform::from_translation(Vec3::new(1.0, 2.0, 3.0));
            let cloned = global.clone();
            assert_eq!(global, cloned);
        }

        #[test]
        fn test_copy() {
            let global = GlobalTransform::IDENTITY;
            let copied = global;
            assert_eq!(global, copied);
        }

        #[test]
        fn test_debug() {
            let global = GlobalTransform::from_translation(Vec3::new(10.0, 5.0, 0.0));
            let debug = format!("{:?}", global);
            assert!(debug.contains("GlobalTransform"));
            assert!(debug.contains("translation"));
        }
    }

    mod hierarchy_tests {
        use super::*;

        #[test]
        fn test_parent_child_translation() {
            // Parent at (10, 0, 0)
            let parent_global = GlobalTransform::from_translation(Vec3::new(10.0, 0.0, 0.0));

            // Child at local (5, 0, 0)
            let child_local = Transform::from_position(Vec3::new(5.0, 0.0, 0.0));

            // Child's global should be (15, 0, 0)
            let child_global = parent_global.transform_by(&child_local);
            let pos = child_global.translation();
            assert!((pos.x - 15.0).abs() < 0.0001);
        }

        #[test]
        fn test_parent_child_rotation() {
            // Parent rotated 90 degrees around Y
            let parent_global = GlobalTransform::from_translation_rotation_scale(
                Vec3::zero(),
                Quat::from_axis_angle(Vec3::unit_y(), FRAC_PI_2),
                Vec3::one(),
            );

            // Child at local (0, 0, -10) - in front of parent
            let child_local = Transform::from_position(Vec3::new(0.0, 0.0, -10.0));

            // After parent rotation, child should be at (-10, 0, 0)
            let child_global = parent_global.transform_by(&child_local);
            let pos = child_global.translation();
            assert!((pos.x - (-10.0)).abs() < 0.01);
            assert!(pos.y.abs() < 0.01);
            assert!(pos.z.abs() < 0.01);
        }

        #[test]
        fn test_parent_child_scale() {
            // Parent scaled 2x
            let parent_global = GlobalTransform::from_translation_rotation_scale(
                Vec3::zero(),
                Quat::IDENTITY,
                Vec3::new(2.0, 2.0, 2.0),
            );

            // Child at local (5, 0, 0)
            let child_local = Transform::from_position(Vec3::new(5.0, 0.0, 0.0));

            // Child's global position should be (10, 0, 0)
            let child_global = parent_global.transform_by(&child_local);
            let pos = child_global.translation();
            assert!((pos.x - 10.0).abs() < 0.0001);
        }

        #[test]
        fn test_three_level_hierarchy() {
            // Grandparent at (10, 0, 0)
            let grandparent = GlobalTransform::from_translation(Vec3::new(10.0, 0.0, 0.0));

            // Parent at local (5, 0, 0)
            let parent_local = Transform::from_position(Vec3::new(5.0, 0.0, 0.0));
            let parent_global = grandparent.transform_by(&parent_local);

            // Child at local (3, 0, 0)
            let child_local = Transform::from_position(Vec3::new(3.0, 0.0, 0.0));
            let child_global = parent_global.transform_by(&child_local);

            // Child's global should be (18, 0, 0)
            let pos = child_global.translation();
            assert!((pos.x - 18.0).abs() < 0.0001);
        }
    }
}
