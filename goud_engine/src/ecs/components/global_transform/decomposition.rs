//! Decomposition methods for [`GlobalTransform`].
//!
//! These methods extract translation, rotation, and scale from the underlying
//! transformation matrix stored in a [`GlobalTransform`].

use crate::core::math::Vec3;
use crate::ecs::components::global_transform::core::GlobalTransform;
use crate::ecs::components::transform::{Quat, Transform};
use cgmath::InnerSpace;

impl GlobalTransform {
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
}
