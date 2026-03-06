//! Transform operations, direction helpers, interpolation, and trait
//! implementations for [`GlobalTransform`].

use crate::core::math::{Matrix4, Vec3};
use crate::ecs::components::global_transform::core::GlobalTransform;
use crate::ecs::components::transform::Transform;
use crate::ecs::Component;
use cgmath::SquareMatrix;
use std::fmt;

impl GlobalTransform {
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

impl Component for GlobalTransform {}

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
