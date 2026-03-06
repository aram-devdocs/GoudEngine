//! Transform operations, direction helpers, and interpolation for [`GlobalTransform2D`].

use crate::core::math::Vec2;
use crate::ecs::components::transform2d::Transform2D;
use std::f32::consts::PI;

use super::types::GlobalTransform2D;

impl GlobalTransform2D {
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
pub(crate) fn lerp_angle(a: f32, b: f32, t: f32) -> f32 {
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
