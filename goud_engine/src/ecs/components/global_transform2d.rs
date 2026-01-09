//! GlobalTransform2D component for 2D world-space transformations.
//!
//! The [`GlobalTransform2D`] component stores the computed world-space transformation
//! for 2D entities in a hierarchy. Unlike [`Transform2D`] which stores local-space data
//! relative to the parent, `GlobalTransform2D` stores the absolute world-space result.
//!
//! # Purpose
//!
//! When entities are arranged in a parent-child hierarchy, each child's [`Transform2D`]
//! is relative to its parent. To render, perform physics, or do other world-space
//! operations, we need the final world-space transformation.
//!
//! For example:
//! - Parent at position (100, 0)
//! - Child at local position (50, 0)
//! - Child's world position is (150, 0)
//!
//! The 2D transform propagation system computes these world-space values and stores
//! them in `GlobalTransform2D`.
//!
//! # Usage
//!
//! `GlobalTransform2D` is typically:
//! 1. Added automatically when spawning entities with `Transform2D`
//! 2. Updated by the 2D transform propagation system each frame
//! 3. Read by rendering systems, physics, etc.
//!
//! **Never modify `GlobalTransform2D` directly.** Always modify `Transform2D` and let
//! the propagation system compute the global value.
//!
//! ```
//! use goud_engine::ecs::components::{Transform2D, GlobalTransform2D};
//! use goud_engine::core::math::Vec2;
//!
//! // Create local transform
//! let local = Transform2D::from_position(Vec2::new(50.0, 0.0));
//!
//! // GlobalTransform2D would be computed by the propagation system
//! // For a root entity, it equals the local transform
//! let global = GlobalTransform2D::from(local);
//!
//! assert!((global.translation() - Vec2::new(50.0, 0.0)).length() < 0.001);
//! ```
//!
//! # Memory Layout
//!
//! GlobalTransform2D stores a pre-computed 3x3 affine transformation matrix (36 bytes).
//! While this uses more memory than Transform2D (20 bytes), it provides:
//!
//! - **Direct use**: Matrix can be sent to GPU without further computation
//! - **Composability**: Easy to combine with parent transforms
//! - **Decomposability**: Can extract position/rotation/scale when needed
//!
//! # FFI Safety
//!
//! GlobalTransform2D is `#[repr(C)]` and can be safely passed across FFI boundaries.

use crate::core::math::Vec2;
use crate::ecs::components::transform2d::{Mat3x3, Transform2D};
use crate::ecs::Component;
use std::f32::consts::PI;
use std::fmt;

/// A 2D world-space transformation component.
///
/// This component caches the computed world-space transformation matrix for
/// 2D entities in a hierarchy. It is computed by the 2D transform propagation system
/// based on the entity's local [`Transform2D`] and its parent's `GlobalTransform2D`.
///
/// # When to Use
///
/// - Use `Transform2D` for setting local position/rotation/scale
/// - Use `GlobalTransform2D` for reading world-space values (rendering, physics)
///
/// # Do Not Modify Directly
///
/// This component is managed by the transform propagation system. Modifying it
/// directly will cause desynchronization with the entity hierarchy.
///
/// # Example
///
/// ```
/// use goud_engine::ecs::components::{Transform2D, GlobalTransform2D};
/// use goud_engine::core::math::Vec2;
///
/// // For root entities, global equals local
/// let transform = Transform2D::from_position(Vec2::new(100.0, 50.0));
/// let global = GlobalTransform2D::from(transform);
///
/// let position = global.translation();
/// assert!((position - Vec2::new(100.0, 50.0)).length() < 0.001);
/// ```
#[repr(C)]
#[derive(Clone, Copy, PartialEq)]
pub struct GlobalTransform2D {
    /// The computed world-space 3x3 transformation matrix.
    ///
    /// This is a column-major affine transformation matrix.
    matrix: Mat3x3,
}

impl GlobalTransform2D {
    /// Identity global transform (no transformation).
    pub const IDENTITY: GlobalTransform2D = GlobalTransform2D {
        matrix: Mat3x3::IDENTITY,
    };

    /// Creates a GlobalTransform2D from a 3x3 transformation matrix.
    ///
    /// # Arguments
    ///
    /// * `matrix` - The world-space transformation matrix
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::components::GlobalTransform2D;
    /// use goud_engine::ecs::components::Mat3x3;
    ///
    /// let matrix = Mat3x3::translation(100.0, 50.0);
    /// let global = GlobalTransform2D::from_matrix(matrix);
    /// ```
    #[inline]
    pub const fn from_matrix(matrix: Mat3x3) -> Self {
        Self { matrix }
    }

    /// Creates a GlobalTransform2D from translation only.
    ///
    /// # Arguments
    ///
    /// * `translation` - World-space position
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::components::GlobalTransform2D;
    /// use goud_engine::core::math::Vec2;
    ///
    /// let global = GlobalTransform2D::from_translation(Vec2::new(100.0, 50.0));
    /// ```
    #[inline]
    pub fn from_translation(translation: Vec2) -> Self {
        Self {
            matrix: Mat3x3::translation(translation.x, translation.y),
        }
    }

    /// Creates a GlobalTransform2D from translation, rotation, and scale.
    ///
    /// # Arguments
    ///
    /// * `translation` - World-space position
    /// * `rotation` - World-space rotation angle in radians
    /// * `scale` - World-space scale
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::components::GlobalTransform2D;
    /// use goud_engine::core::math::Vec2;
    ///
    /// let global = GlobalTransform2D::from_translation_rotation_scale(
    ///     Vec2::new(100.0, 0.0),
    ///     0.0,
    ///     Vec2::one(),
    /// );
    /// ```
    #[inline]
    pub fn from_translation_rotation_scale(translation: Vec2, rotation: f32, scale: Vec2) -> Self {
        // Build transform matrix: T * R * S
        let transform = Transform2D::new(translation, rotation, scale);
        Self {
            matrix: transform.matrix(),
        }
    }

    /// Returns the underlying 3x3 transformation matrix.
    ///
    /// This matrix is column-major and can be used directly for rendering.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::components::GlobalTransform2D;
    ///
    /// let global = GlobalTransform2D::IDENTITY;
    /// let matrix = global.matrix();
    /// ```
    #[inline]
    pub fn matrix(&self) -> Mat3x3 {
        self.matrix
    }

    /// Returns a reference to the underlying matrix.
    #[inline]
    pub fn matrix_ref(&self) -> &Mat3x3 {
        &self.matrix
    }

    /// Returns the matrix as a flat column-major array.
    ///
    /// This is useful for FFI and sending to GPU shaders.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::components::GlobalTransform2D;
    ///
    /// let global = GlobalTransform2D::IDENTITY;
    /// let cols: [f32; 9] = global.to_cols_array();
    ///
    /// // First column (x-axis)
    /// assert_eq!(cols[0], 1.0); // m00
    /// ```
    #[inline]
    pub fn to_cols_array(&self) -> [f32; 9] {
        self.matrix.m
    }

    /// Returns the 2D matrix as a 4x4 matrix array for 3D rendering APIs.
    ///
    /// The Z components are set to identity (z=0, w=1).
    #[inline]
    pub fn to_mat4_array(&self) -> [f32; 16] {
        let m = &self.matrix.m;
        [
            m[0], m[1], 0.0, 0.0, m[3], m[4], 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, m[6], m[7], 0.0, 1.0,
        ]
    }

    // =========================================================================
    // Decomposition Methods
    // =========================================================================

    /// Extracts the translation (position) component.
    ///
    /// This is from the third column of the matrix.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::components::GlobalTransform2D;
    /// use goud_engine::core::math::Vec2;
    ///
    /// let global = GlobalTransform2D::from_translation(Vec2::new(100.0, 50.0));
    /// let pos = global.translation();
    ///
    /// assert!((pos.x - 100.0).abs() < 0.001);
    /// ```
    #[inline]
    pub fn translation(&self) -> Vec2 {
        self.matrix.get_translation()
    }

    /// Extracts the scale component.
    ///
    /// This is computed from the lengths of the first two column vectors.
    /// Note: This does not handle negative scales correctly.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::components::GlobalTransform2D;
    /// use goud_engine::core::math::Vec2;
    ///
    /// let global = GlobalTransform2D::from_translation_rotation_scale(
    ///     Vec2::zero(),
    ///     0.0,
    ///     Vec2::new(2.0, 3.0),
    /// );
    ///
    /// let scale = global.scale();
    /// assert!((scale.x - 2.0).abs() < 0.001);
    /// ```
    #[inline]
    pub fn scale(&self) -> Vec2 {
        let m = &self.matrix.m;
        let scale_x = (m[0] * m[0] + m[1] * m[1]).sqrt();
        let scale_y = (m[3] * m[3] + m[4] * m[4]).sqrt();
        Vec2::new(scale_x, scale_y)
    }

    /// Extracts the rotation component as an angle in radians.
    ///
    /// This removes scale from the rotation matrix, then extracts the angle.
    /// Note: This may have issues with negative scales.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::components::GlobalTransform2D;
    /// use goud_engine::core::math::Vec2;
    /// use std::f32::consts::PI;
    ///
    /// let global = GlobalTransform2D::from_translation_rotation_scale(
    ///     Vec2::zero(),
    ///     PI / 4.0,
    ///     Vec2::one(),
    /// );
    ///
    /// let rotation = global.rotation();
    /// assert!((rotation - PI / 4.0).abs() < 0.001);
    /// ```
    #[inline]
    pub fn rotation(&self) -> f32 {
        let scale = self.scale();
        let m = &self.matrix.m;

        // Get normalized rotation column
        let cos_r = if scale.x != 0.0 { m[0] / scale.x } else { 1.0 };
        let sin_r = if scale.x != 0.0 { m[1] / scale.x } else { 0.0 };

        sin_r.atan2(cos_r)
    }

    /// Extracts the rotation component as degrees.
    #[inline]
    pub fn rotation_degrees(&self) -> f32 {
        self.rotation() * 180.0 / PI
    }

    /// Decomposes the transform into translation, rotation, and scale.
    ///
    /// Returns `(translation, rotation, scale)`.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::components::GlobalTransform2D;
    /// use goud_engine::core::math::Vec2;
    ///
    /// let global = GlobalTransform2D::from_translation_rotation_scale(
    ///     Vec2::new(100.0, 50.0),
    ///     0.0,
    ///     Vec2::new(2.0, 2.0),
    /// );
    ///
    /// let (translation, rotation, scale) = global.decompose();
    /// assert!((translation.x - 100.0).abs() < 0.001);
    /// assert!((scale.x - 2.0).abs() < 0.001);
    /// ```
    #[inline]
    pub fn decompose(&self) -> (Vec2, f32, Vec2) {
        (self.translation(), self.rotation(), self.scale())
    }

    /// Converts this GlobalTransform2D to a local Transform2D.
    ///
    /// This is useful for extracting a Transform2D that would produce this
    /// GlobalTransform2D when applied from the origin.
    #[inline]
    pub fn to_transform(&self) -> Transform2D {
        let (translation, rotation, scale) = self.decompose();
        Transform2D::new(translation, rotation, scale)
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
    /// use goud_engine::ecs::components::GlobalTransform2D;
    /// use goud_engine::core::math::Vec2;
    ///
    /// let parent = GlobalTransform2D::from_translation(Vec2::new(100.0, 0.0));
    /// let child_local = GlobalTransform2D::from_translation(Vec2::new(50.0, 0.0));
    ///
    /// let child_global = parent.mul_transform(&child_local);
    /// let pos = child_global.translation();
    /// assert!((pos.x - 150.0).abs() < 0.001);
    /// ```
    #[inline]
    pub fn mul_transform(&self, other: &GlobalTransform2D) -> GlobalTransform2D {
        GlobalTransform2D {
            matrix: self.matrix.multiply(&other.matrix),
        }
    }

    /// Multiplies this transform by a local Transform2D.
    ///
    /// This is the primary method used by 2D transform propagation.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::components::{GlobalTransform2D, Transform2D};
    /// use goud_engine::core::math::Vec2;
    ///
    /// let parent_global = GlobalTransform2D::from_translation(Vec2::new(100.0, 0.0));
    /// let child_local = Transform2D::from_position(Vec2::new(50.0, 0.0));
    ///
    /// let child_global = parent_global.transform_by(&child_local);
    /// let pos = child_global.translation();
    /// assert!((pos.x - 150.0).abs() < 0.001);
    /// ```
    #[inline]
    pub fn transform_by(&self, local: &Transform2D) -> GlobalTransform2D {
        GlobalTransform2D {
            matrix: self.matrix.multiply(&local.matrix()),
        }
    }

    /// Transforms a point from local space to world space.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::components::GlobalTransform2D;
    /// use goud_engine::core::math::Vec2;
    ///
    /// let global = GlobalTransform2D::from_translation(Vec2::new(100.0, 0.0));
    /// let local_point = Vec2::new(50.0, 0.0);
    /// let world_point = global.transform_point(local_point);
    ///
    /// assert!((world_point.x - 150.0).abs() < 0.001);
    /// ```
    #[inline]
    pub fn transform_point(&self, point: Vec2) -> Vec2 {
        self.matrix.transform_point(point)
    }

    /// Transforms a direction from local space to world space.
    ///
    /// Unlike points, directions are not affected by translation.
    #[inline]
    pub fn transform_direction(&self, direction: Vec2) -> Vec2 {
        self.matrix.transform_direction(direction)
    }

    /// Returns the inverse of this transform.
    ///
    /// The inverse transforms from world space back to local space.
    /// Returns `None` if the matrix is not invertible (e.g., has zero scale).
    #[inline]
    pub fn inverse(&self) -> Option<GlobalTransform2D> {
        self.matrix
            .inverse()
            .map(|m| GlobalTransform2D { matrix: m })
    }

    // =========================================================================
    // Direction Vectors
    // =========================================================================

    /// Returns the forward direction vector (positive Y in local space for 2D).
    ///
    /// Note: In 2D, "forward" is typically the positive Y direction.
    #[inline]
    pub fn forward(&self) -> Vec2 {
        self.transform_direction(Vec2::new(0.0, 1.0)).normalize()
    }

    /// Returns the right direction vector (positive X in local space).
    #[inline]
    pub fn right(&self) -> Vec2 {
        self.transform_direction(Vec2::new(1.0, 0.0)).normalize()
    }

    /// Returns the backward direction vector (negative Y in local space).
    #[inline]
    pub fn backward(&self) -> Vec2 {
        self.transform_direction(Vec2::new(0.0, -1.0)).normalize()
    }

    /// Returns the left direction vector (negative X in local space).
    #[inline]
    pub fn left(&self) -> Vec2 {
        self.transform_direction(Vec2::new(-1.0, 0.0)).normalize()
    }

    // =========================================================================
    // Interpolation
    // =========================================================================

    /// Linearly interpolates between two global transforms.
    ///
    /// This decomposes both transforms, interpolates components separately
    /// (lerp for angle), then recomposes.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::components::GlobalTransform2D;
    /// use goud_engine::core::math::Vec2;
    ///
    /// let a = GlobalTransform2D::from_translation(Vec2::zero());
    /// let b = GlobalTransform2D::from_translation(Vec2::new(100.0, 0.0));
    ///
    /// let mid = a.lerp(&b, 0.5);
    /// let pos = mid.translation();
    /// assert!((pos.x - 50.0).abs() < 0.001);
    /// ```
    #[inline]
    pub fn lerp(&self, other: &GlobalTransform2D, t: f32) -> GlobalTransform2D {
        let (t1, r1, s1) = self.decompose();
        let (t2, r2, s2) = other.decompose();

        GlobalTransform2D::from_translation_rotation_scale(
            t1.lerp(t2, t),
            lerp_angle(r1, r2, t),
            s1.lerp(s2, t),
        )
    }
}

/// Linearly interpolates between two angles, taking the shortest path.
#[inline]
fn lerp_angle(a: f32, b: f32, t: f32) -> f32 {
    let mut diff = b - a;

    // Normalize to [-PI, PI]
    while diff > PI {
        diff -= 2.0 * PI;
    }
    while diff < -PI {
        diff += 2.0 * PI;
    }

    a + diff * t
}

impl Default for GlobalTransform2D {
    #[inline]
    fn default() -> Self {
        Self::IDENTITY
    }
}

impl fmt::Debug for GlobalTransform2D {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (t, r, s) = self.decompose();
        f.debug_struct("GlobalTransform2D")
            .field("translation", &(t.x, t.y))
            .field("rotation", &format!("{:.3} rad", r))
            .field("scale", &(s.x, s.y))
            .finish()
    }
}

impl From<Transform2D> for GlobalTransform2D {
    /// Converts a local Transform2D to a GlobalTransform2D.
    ///
    /// This is used for root entities where local == global.
    #[inline]
    fn from(transform: Transform2D) -> Self {
        GlobalTransform2D {
            matrix: transform.matrix(),
        }
    }
}

impl From<&Transform2D> for GlobalTransform2D {
    #[inline]
    fn from(transform: &Transform2D) -> Self {
        GlobalTransform2D {
            matrix: transform.matrix(),
        }
    }
}

impl From<Mat3x3> for GlobalTransform2D {
    #[inline]
    fn from(matrix: Mat3x3) -> Self {
        GlobalTransform2D { matrix }
    }
}

// Implement Component trait
impl Component for GlobalTransform2D {}

// Implement multiplication operators
impl std::ops::Mul for GlobalTransform2D {
    type Output = GlobalTransform2D;

    #[inline]
    fn mul(self, rhs: GlobalTransform2D) -> GlobalTransform2D {
        self.mul_transform(&rhs)
    }
}

impl std::ops::Mul<&GlobalTransform2D> for GlobalTransform2D {
    type Output = GlobalTransform2D;

    #[inline]
    fn mul(self, rhs: &GlobalTransform2D) -> GlobalTransform2D {
        self.mul_transform(rhs)
    }
}

impl std::ops::Mul<Transform2D> for GlobalTransform2D {
    type Output = GlobalTransform2D;

    #[inline]
    fn mul(self, rhs: Transform2D) -> GlobalTransform2D {
        self.transform_by(&rhs)
    }
}

impl std::ops::Mul<&Transform2D> for GlobalTransform2D {
    type Output = GlobalTransform2D;

    #[inline]
    fn mul(self, rhs: &Transform2D) -> GlobalTransform2D {
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
            let global = GlobalTransform2D::IDENTITY;
            let pos = global.translation();
            let scale = global.scale();

            assert!((pos.x).abs() < 0.0001);
            assert!((pos.y).abs() < 0.0001);
            assert!((scale.x - 1.0).abs() < 0.0001);
            assert!((scale.y - 1.0).abs() < 0.0001);
        }

        #[test]
        fn test_default() {
            let global: GlobalTransform2D = Default::default();
            assert_eq!(global, GlobalTransform2D::IDENTITY);
        }

        #[test]
        fn test_from_translation() {
            let global = GlobalTransform2D::from_translation(Vec2::new(100.0, 200.0));
            let pos = global.translation();

            assert!((pos.x - 100.0).abs() < 0.0001);
            assert!((pos.y - 200.0).abs() < 0.0001);
        }

        #[test]
        fn test_from_translation_rotation_scale() {
            let global = GlobalTransform2D::from_translation_rotation_scale(
                Vec2::new(50.0, 100.0),
                FRAC_PI_4,
                Vec2::new(2.0, 3.0),
            );

            let pos = global.translation();
            let scale = global.scale();

            assert!((pos.x - 50.0).abs() < 0.0001);
            assert!((pos.y - 100.0).abs() < 0.0001);
            assert!((scale.x - 2.0).abs() < 0.0001);
            assert!((scale.y - 3.0).abs() < 0.0001);
        }

        #[test]
        fn test_from_transform() {
            let transform = Transform2D::new(Vec2::new(10.0, 20.0), FRAC_PI_4, Vec2::new(2.0, 2.0));
            let global: GlobalTransform2D = transform.into();

            let pos = global.translation();
            assert!((pos.x - 10.0).abs() < 0.0001);
            assert!((pos.y - 20.0).abs() < 0.0001);
        }

        #[test]
        fn test_from_transform_ref() {
            let transform = Transform2D::from_position(Vec2::new(50.0, 0.0));
            let global: GlobalTransform2D = (&transform).into();
            let pos = global.translation();
            assert!((pos.x - 50.0).abs() < 0.0001);
        }
    }

    mod decomposition_tests {
        use super::*;

        #[test]
        fn test_translation_extraction() {
            let global = GlobalTransform2D::from_translation(Vec2::new(10.0, 20.0));
            let pos = global.translation();
            assert!((pos.x - 10.0).abs() < 0.0001);
            assert!((pos.y - 20.0).abs() < 0.0001);
        }

        #[test]
        fn test_scale_extraction() {
            let global = GlobalTransform2D::from_translation_rotation_scale(
                Vec2::zero(),
                0.0,
                Vec2::new(2.0, 3.0),
            );
            let scale = global.scale();
            assert!((scale.x - 2.0).abs() < 0.0001);
            assert!((scale.y - 3.0).abs() < 0.0001);
        }

        #[test]
        fn test_rotation_extraction() {
            let original = FRAC_PI_4;
            let global = GlobalTransform2D::from_translation_rotation_scale(
                Vec2::zero(),
                original,
                Vec2::one(),
            );
            let extracted = global.rotation();

            assert!((extracted - original).abs() < 0.001);
        }

        #[test]
        fn test_rotation_degrees() {
            let global = GlobalTransform2D::from_translation_rotation_scale(
                Vec2::zero(),
                FRAC_PI_2,
                Vec2::one(),
            );
            let degrees = global.rotation_degrees();
            assert!((degrees - 90.0).abs() < 0.1);
        }

        #[test]
        fn test_decompose() {
            let original_t = Vec2::new(100.0, 50.0);
            let original_r = FRAC_PI_4;
            let original_s = Vec2::new(2.0, 3.0);

            let global = GlobalTransform2D::from_translation_rotation_scale(
                original_t, original_r, original_s,
            );
            let (t, r, s) = global.decompose();

            assert!((t - original_t).length() < 0.001);
            assert!((r - original_r).abs() < 0.001);
            assert!((s - original_s).length() < 0.001);
        }

        #[test]
        fn test_to_transform() {
            let global = GlobalTransform2D::from_translation_rotation_scale(
                Vec2::new(50.0, 100.0),
                0.0,
                Vec2::new(2.0, 2.0),
            );

            let transform = global.to_transform();
            assert!((transform.position - Vec2::new(50.0, 100.0)).length() < 0.001);
            assert!((transform.scale - Vec2::new(2.0, 2.0)).length() < 0.001);
        }
    }

    mod transform_tests {
        use super::*;

        #[test]
        fn test_mul_transform_translation() {
            let a = GlobalTransform2D::from_translation(Vec2::new(100.0, 0.0));
            let b = GlobalTransform2D::from_translation(Vec2::new(50.0, 0.0));
            let result = a.mul_transform(&b);

            let pos = result.translation();
            assert!((pos.x - 150.0).abs() < 0.0001);
        }

        #[test]
        fn test_mul_transform_scale() {
            let a = GlobalTransform2D::from_translation_rotation_scale(
                Vec2::zero(),
                0.0,
                Vec2::new(2.0, 2.0),
            );
            let b = GlobalTransform2D::from_translation(Vec2::new(50.0, 0.0));
            let result = a.mul_transform(&b);

            let pos = result.translation();
            // Scale affects the child's translation
            assert!((pos.x - 100.0).abs() < 0.0001);
        }

        #[test]
        fn test_transform_by() {
            let parent = GlobalTransform2D::from_translation(Vec2::new(100.0, 0.0));
            let child = Transform2D::from_position(Vec2::new(50.0, 0.0));
            let result = parent.transform_by(&child);

            let pos = result.translation();
            assert!((pos.x - 150.0).abs() < 0.0001);
        }

        #[test]
        fn test_transform_point() {
            let global = GlobalTransform2D::from_translation(Vec2::new(100.0, 0.0));
            let local_point = Vec2::new(50.0, 30.0);
            let world_point = global.transform_point(local_point);

            assert!((world_point.x - 150.0).abs() < 0.0001);
            assert!((world_point.y - 30.0).abs() < 0.0001);
        }

        #[test]
        fn test_transform_direction() {
            let global = GlobalTransform2D::from_translation(Vec2::new(1000.0, 0.0));
            let direction = Vec2::new(1.0, 0.0);
            let world_dir = global.transform_direction(direction);

            // Direction should not be affected by translation
            assert!((world_dir.x - 1.0).abs() < 0.0001);
            assert!(world_dir.y.abs() < 0.0001);
        }

        #[test]
        fn test_inverse() {
            let global = GlobalTransform2D::from_translation_rotation_scale(
                Vec2::new(50.0, 100.0),
                FRAC_PI_4,
                Vec2::new(2.0, 2.0),
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
            let a = GlobalTransform2D::from_translation(Vec2::new(100.0, 0.0));
            let b = GlobalTransform2D::from_translation(Vec2::new(50.0, 0.0));
            let result = a * b;

            let pos = result.translation();
            assert!((pos.x - 150.0).abs() < 0.0001);
        }

        #[test]
        fn test_mul_operator_with_transform() {
            let parent = GlobalTransform2D::from_translation(Vec2::new(100.0, 0.0));
            let child = Transform2D::from_position(Vec2::new(50.0, 0.0));
            let result = parent * child;

            let pos = result.translation();
            assert!((pos.x - 150.0).abs() < 0.0001);
        }
    }

    mod direction_tests {
        use super::*;

        #[test]
        fn test_directions_identity() {
            let global = GlobalTransform2D::IDENTITY;

            assert!((global.forward() - Vec2::new(0.0, 1.0)).length() < 0.0001);
            assert!((global.backward() - Vec2::new(0.0, -1.0)).length() < 0.0001);
            assert!((global.right() - Vec2::new(1.0, 0.0)).length() < 0.0001);
            assert!((global.left() - Vec2::new(-1.0, 0.0)).length() < 0.0001);
        }

        #[test]
        fn test_directions_rotated() {
            let global = GlobalTransform2D::from_translation_rotation_scale(
                Vec2::zero(),
                FRAC_PI_2, // 90 degrees
                Vec2::one(),
            );

            // After 90 degree rotation:
            // forward (0, 1) -> (-1, 0)
            let fwd = global.forward();
            assert!((fwd.x - (-1.0)).abs() < 0.0001);
            assert!(fwd.y.abs() < 0.0001);
        }
    }

    mod interpolation_tests {
        use super::*;

        #[test]
        fn test_lerp_translation() {
            let a = GlobalTransform2D::from_translation(Vec2::zero());
            let b = GlobalTransform2D::from_translation(Vec2::new(100.0, 0.0));

            let mid = a.lerp(&b, 0.5);
            let pos = mid.translation();
            assert!((pos.x - 50.0).abs() < 0.0001);
        }

        #[test]
        fn test_lerp_endpoints() {
            let a = GlobalTransform2D::from_translation(Vec2::new(0.0, 0.0));
            let b = GlobalTransform2D::from_translation(Vec2::new(100.0, 100.0));

            let start = a.lerp(&b, 0.0);
            assert!((start.translation() - a.translation()).length() < 0.0001);

            let end = a.lerp(&b, 1.0);
            assert!((end.translation() - b.translation()).length() < 0.0001);
        }

        #[test]
        fn test_lerp_angle() {
            // Test shortest path angle interpolation
            let result = lerp_angle(0.0, PI, 0.5);
            assert!((result - FRAC_PI_2).abs() < 0.0001);

            // Test wrapping around
            let result = lerp_angle(0.1, -0.1, 0.5);
            assert!(result.abs() < 0.0001);
        }
    }

    mod array_tests {
        use super::*;

        #[test]
        fn test_to_cols_array() {
            let global = GlobalTransform2D::IDENTITY;
            let cols = global.to_cols_array();

            // Identity matrix
            assert_eq!(cols[0], 1.0); // m00
            assert_eq!(cols[4], 1.0); // m11
            assert_eq!(cols[8], 1.0); // m22
        }

        #[test]
        fn test_to_mat4_array() {
            let global = GlobalTransform2D::from_translation(Vec2::new(100.0, 200.0));
            let mat4 = global.to_mat4_array();

            // Translation is in column 4
            assert!((mat4[12] - 100.0).abs() < 0.0001);
            assert!((mat4[13] - 200.0).abs() < 0.0001);
            // Z row/column should be identity-like
            assert_eq!(mat4[10], 1.0);
            assert_eq!(mat4[15], 1.0);
        }
    }

    mod component_tests {
        use super::*;

        #[test]
        fn test_is_component() {
            fn assert_component<T: Component>() {}
            assert_component::<GlobalTransform2D>();
        }

        #[test]
        fn test_is_send() {
            fn assert_send<T: Send>() {}
            assert_send::<GlobalTransform2D>();
        }

        #[test]
        fn test_is_sync() {
            fn assert_sync<T: Sync>() {}
            assert_sync::<GlobalTransform2D>();
        }

        #[test]
        fn test_clone() {
            let global = GlobalTransform2D::from_translation(Vec2::new(10.0, 20.0));
            let cloned = global.clone();
            assert_eq!(global, cloned);
        }

        #[test]
        fn test_copy() {
            let global = GlobalTransform2D::IDENTITY;
            let copied = global;
            assert_eq!(global, copied);
        }

        #[test]
        fn test_debug() {
            let global = GlobalTransform2D::from_translation(Vec2::new(100.0, 50.0));
            let debug = format!("{:?}", global);
            assert!(debug.contains("GlobalTransform2D"));
            assert!(debug.contains("translation"));
        }
    }

    mod hierarchy_tests {
        use super::*;

        #[test]
        fn test_parent_child_translation() {
            // Parent at (100, 0)
            let parent_global = GlobalTransform2D::from_translation(Vec2::new(100.0, 0.0));

            // Child at local (50, 0)
            let child_local = Transform2D::from_position(Vec2::new(50.0, 0.0));

            // Child's global should be (150, 0)
            let child_global = parent_global.transform_by(&child_local);
            let pos = child_global.translation();
            assert!((pos.x - 150.0).abs() < 0.0001);
        }

        #[test]
        fn test_parent_child_rotation() {
            // Parent rotated 90 degrees
            let parent_global = GlobalTransform2D::from_translation_rotation_scale(
                Vec2::zero(),
                FRAC_PI_2,
                Vec2::one(),
            );

            // Child at local (0, 100) - above parent in local space
            let child_local = Transform2D::from_position(Vec2::new(0.0, 100.0));

            // After parent rotation, child should be at (-100, 0)
            let child_global = parent_global.transform_by(&child_local);
            let pos = child_global.translation();
            assert!((pos.x - (-100.0)).abs() < 0.01);
            assert!(pos.y.abs() < 0.01);
        }

        #[test]
        fn test_parent_child_scale() {
            // Parent scaled 2x
            let parent_global = GlobalTransform2D::from_translation_rotation_scale(
                Vec2::zero(),
                0.0,
                Vec2::new(2.0, 2.0),
            );

            // Child at local (50, 0)
            let child_local = Transform2D::from_position(Vec2::new(50.0, 0.0));

            // Child's global position should be (100, 0)
            let child_global = parent_global.transform_by(&child_local);
            let pos = child_global.translation();
            assert!((pos.x - 100.0).abs() < 0.0001);
        }

        #[test]
        fn test_three_level_hierarchy() {
            // Grandparent at (100, 0)
            let grandparent = GlobalTransform2D::from_translation(Vec2::new(100.0, 0.0));

            // Parent at local (50, 0)
            let parent_local = Transform2D::from_position(Vec2::new(50.0, 0.0));
            let parent_global = grandparent.transform_by(&parent_local);

            // Child at local (30, 0)
            let child_local = Transform2D::from_position(Vec2::new(30.0, 0.0));
            let child_global = parent_global.transform_by(&child_local);

            // Child's global should be (180, 0)
            let pos = child_global.translation();
            assert!((pos.x - 180.0).abs() < 0.0001);
        }
    }

    mod ffi_tests {
        use super::*;
        use std::mem::{align_of, size_of};

        #[test]
        fn test_size() {
            // Mat3x3 is 9 * 4 = 36 bytes
            assert_eq!(size_of::<GlobalTransform2D>(), 36);
        }

        #[test]
        fn test_align() {
            assert_eq!(align_of::<GlobalTransform2D>(), 4);
        }
    }
}
