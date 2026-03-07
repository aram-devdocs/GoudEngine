//! Core type definition and constructor methods for [`GlobalTransform2D`].

use crate::core::math::Vec2;
use crate::ecs::components::transform2d::{Mat3x3, Transform2D};

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
#[derive(Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct GlobalTransform2D {
    /// The computed world-space 3x3 transformation matrix.
    ///
    /// This is a column-major affine transformation matrix.
    pub(super) matrix: Mat3x3,
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
        self.rotation() * 180.0 / std::f32::consts::PI
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
}
