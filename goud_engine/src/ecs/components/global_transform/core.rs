//! Core [`GlobalTransform`] struct definition and constructor methods.

use crate::core::math::{Matrix4, Vec3};
use crate::ecs::components::transform::{Quat, Transform};

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
    pub(super) matrix: Matrix4<f32>,
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
}
